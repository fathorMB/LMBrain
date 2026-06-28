//! Registers the repository-scoped `lmbrain-mcp` server with the agent host by
//! writing a host-readable `.mcp.json` at the workspace root, so agents receive
//! the controlled-mutation tools instead of editing Markdown by hand.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::errors::AppError;

/// Build the `.mcp.json` content that registers the `lmbrain` server, merging into
/// any existing configuration and preserving unrelated keys and other servers.
pub fn build_mcp_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, AppError> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => serde_json::from_str(text)?,
        _ => json!({}),
    };
    if !value.is_object() {
        value = json!({});
    }
    let object = value.as_object_mut().expect("value is an object");
    let servers = object.entry("mcpServers").or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers
        .as_object_mut()
        .expect("mcpServers is an object")
        .insert(
            "lmbrain".to_string(),
            json!({ "command": command, "args": ["--root", root] }),
        );
    Ok(serde_json::to_string_pretty(&value)?)
}

/// Write/refresh `.mcp.json` at the workspace root. Idempotent: it rewrites only
/// when the resulting content differs from what is already on disk.
pub fn register_mcp_server(root: &Path, command: &str) -> Result<PathBuf, AppError> {
    let config_path = root.join(".mcp.json");
    let existing = std::fs::read_to_string(&config_path).ok();
    let content = build_mcp_config(existing.as_deref(), command, &root.to_string_lossy())?;
    if existing.as_deref() != Some(content.as_str()) {
        let temp = root.join(".mcp.json.tmp");
        std::fs::write(&temp, &content)?;
        std::fs::rename(&temp, &config_path)?;
    }
    Ok(config_path)
}

/// Resolve the command to register for a concrete workspace. If automatic
/// discovery only finds the bare fallback, preserve an existing generated
/// absolute command that still points to a real binary.
pub fn resolve_mcp_command_for_root(root: &Path) -> String {
    let resolved = resolve_mcp_command();
    resolve_mcp_command_for_root_from(root, resolved)
}

fn resolve_mcp_command_for_root_from(root: &Path, resolved: String) -> String {
    if resolved != "lmbrain-mcp" {
        return resolved;
    }

    existing_registered_mcp_command(root)
        .filter(|command| concrete_command_exists(command))
        .unwrap_or(resolved)
}

/// Resolve the value to place in `.mcp.json`'s `command`: an explicit
/// `LMBRAIN_MCP_BIN` override first, then the binary sitting next to the running
/// executable (the dev/workspace build and, if bundled, the installed app), then
/// Cargo workspace build outputs, then the bare name resolved through `PATH`.
pub fn resolve_mcp_command() -> String {
    resolve_mcp_command_from(
        std::env::var("LMBRAIN_MCP_BIN").ok(),
        std::env::current_exe().ok(),
        cargo_workspace_mcp_candidates(),
    )
}

fn resolve_mcp_command_from(
    env_override: Option<String>,
    current_exe: Option<PathBuf>,
    extra_candidates: Vec<PathBuf>,
) -> String {
    if let Some(value) = env_override.map(|value| value.trim().to_string()) {
        if !value.is_empty() {
            return value;
        }
    }

    let binary = mcp_binary_name();
    if let Some(candidate) = current_exe
        .as_deref()
        .and_then(Path::parent)
        .map(|dir| dir.join(binary))
    {
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    for candidate in extra_candidates {
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    "lmbrain-mcp".to_string()
}

fn cargo_workspace_mcp_candidates() -> Vec<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.to_path_buf());
    let binary = mcp_binary_name();

    ["debug", "release"]
        .into_iter()
        .map(|profile| workspace_root.join("target").join(profile).join(binary))
        .collect()
}

fn mcp_binary_name() -> &'static str {
    if cfg!(windows) {
        "lmbrain-mcp.exe"
    } else {
        "lmbrain-mcp"
    }
}

fn existing_registered_mcp_command(root: &Path) -> Option<String> {
    let config = std::fs::read_to_string(root.join(".mcp.json")).ok()?;
    let value: Value = serde_json::from_str(&config).ok()?;
    value
        .get("mcpServers")?
        .get("lmbrain")?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

fn concrete_command_exists(command: &str) -> bool {
    let trimmed = command.trim();
    !trimmed.is_empty() && Path::new(trimmed).is_file()
}

#[cfg(test)]
mod tests {
    use super::{
        build_mcp_config, concrete_command_exists, mcp_binary_name,
        resolve_mcp_command_for_root_from, resolve_mcp_command_from,
    };
    use serde_json::Value;

    #[test]
    fn creates_config_when_absent() {
        let out = build_mcp_config(None, "/bin/lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            value["mcpServers"]["lmbrain"]["command"],
            "/bin/lmbrain-mcp"
        );
        assert_eq!(value["mcpServers"]["lmbrain"]["args"][0], "--root");
        assert_eq!(value["mcpServers"]["lmbrain"]["args"][1], "/ws");
    }

    #[test]
    fn preserves_other_servers_and_keys() {
        let existing = r#"{"otherKey":1,"mcpServers":{"github":{"command":"gh-mcp"}}}"#;
        let out = build_mcp_config(Some(existing), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["otherKey"], 1);
        assert_eq!(value["mcpServers"]["github"]["command"], "gh-mcp");
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
    }

    #[test]
    fn is_idempotent() {
        let first = build_mcp_config(None, "lmbrain-mcp", "/ws").unwrap();
        let second = build_mcp_config(Some(&first), "lmbrain-mcp", "/ws").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn replaces_a_non_object_root() {
        let out = build_mcp_config(Some("[]"), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
    }

    #[test]
    fn resolver_honors_env_override_first() {
        let dir = tempfile::tempdir().unwrap();
        let sibling = dir.path().join(mcp_binary_name());
        std::fs::write(&sibling, "").unwrap();

        let resolved = resolve_mcp_command_from(
            Some("/custom/lmbrain-mcp".into()),
            Some(dir.path().join("lmbrain")),
            Vec::new(),
        );

        assert_eq!(resolved, "/custom/lmbrain-mcp");
    }

    #[test]
    fn resolver_prefers_sibling_binary_before_extra_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let sibling = dir.path().join(mcp_binary_name());
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join(mcp_binary_name());
        std::fs::write(&sibling, "").unwrap();
        std::fs::write(&target, "").unwrap();

        let resolved =
            resolve_mcp_command_from(None, Some(dir.path().join("lmbrain")), vec![target]);

        assert_eq!(resolved, sibling.to_string_lossy());
    }

    #[test]
    fn resolver_uses_extra_candidate_when_not_next_to_app() {
        let app_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join(mcp_binary_name());
        std::fs::write(&target, "").unwrap();

        let resolved = resolve_mcp_command_from(
            None,
            Some(app_dir.path().join("lmbrain")),
            vec![target.clone()],
        );

        assert_eq!(resolved, target.to_string_lossy());
    }

    #[test]
    fn resolver_falls_back_to_bare_command_when_no_candidate_exists() {
        let dir = tempfile::tempdir().unwrap();

        let resolved = resolve_mcp_command_from(
            Some("   ".into()),
            Some(dir.path().join("lmbrain")),
            vec![dir.path().join("missing").join(mcp_binary_name())],
        );

        assert_eq!(resolved, "lmbrain-mcp");
    }

    #[test]
    fn workspace_resolver_preserves_existing_concrete_command_when_auto_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join(mcp_binary_name());
        std::fs::write(&binary, "").unwrap();
        let config = build_mcp_config(None, &binary.to_string_lossy(), "/ws").unwrap();
        std::fs::write(dir.path().join(".mcp.json"), config).unwrap();

        let resolved = resolve_mcp_command_for_root_from(dir.path(), "lmbrain-mcp".to_string());

        assert_eq!(resolved, binary.to_string_lossy());
    }

    #[test]
    fn concrete_command_requires_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join(mcp_binary_name());

        assert!(!concrete_command_exists(&binary.to_string_lossy()));
        std::fs::write(&binary, "").unwrap();
        assert!(concrete_command_exists(&binary.to_string_lossy()));
        assert!(!concrete_command_exists("lmbrain-mcp"));
    }
}
