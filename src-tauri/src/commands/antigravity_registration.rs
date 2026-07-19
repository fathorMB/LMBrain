//! Registers the repository-scoped `lmbrain-mcp` server with the Antigravity IDE.
//!
//! Antigravity has no project-level MCP configuration: servers live in a
//! user-global `mcp_config.json` (`~/.gemini/antigravity/` for the 1.x IDE,
//! `~/.gemini/config/` for the 2.0 unified CLI+IDE layout). LMBrain therefore
//! maintains a single `mcpServers.lmbrain` entry pointing at the most recently
//! opened workspace and only writes where an Antigravity installation is
//! already detectable, preserving every unrelated server and key.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::errors::AppError;

/// Build the Antigravity `mcp_config.json` content that registers the
/// `lmbrain` server, merging into any existing configuration and preserving
/// unrelated keys and other servers.
pub fn build_antigravity_config(
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

/// Write/refresh the user-global Antigravity MCP configuration for this
/// workspace. Both known layouts are updated when present so the 1.x IDE and
/// the 2.0 CLI stay consistent. When no Antigravity installation is
/// detectable, nothing is written.
pub fn register_antigravity_mcp_server(
    root: &Path,
    command: &str,
) -> Result<Vec<PathBuf>, AppError> {
    register_antigravity_mcp_server_in(&antigravity_home()?, root, command)
}

fn register_antigravity_mcp_server_in(
    home: &Path,
    root: &Path,
    command: &str,
) -> Result<Vec<PathBuf>, AppError> {
    let mut written = Vec::new();
    for target in antigravity_config_targets(home) {
        let existing = std::fs::read_to_string(&target).ok();
        let content =
            build_antigravity_config(existing.as_deref(), command, &root.to_string_lossy())?;
        write_if_changed(&target, existing.as_deref(), &content)?;
        written.push(target);
    }
    Ok(written)
}

/// Candidate user-global config files. A location qualifies only when its
/// config file or its parent directory already exists: the write must never
/// create `~/.gemini` for users without Antigravity.
fn antigravity_config_targets(home: &Path) -> Vec<PathBuf> {
    [
        home.join(".gemini").join("antigravity"),
        home.join(".gemini").join("config"),
    ]
    .into_iter()
    .filter_map(|dir| {
        let file = dir.join("mcp_config.json");
        (file.is_file() || dir.is_dir()).then_some(file)
    })
    .collect()
}

fn antigravity_home() -> Result<PathBuf, AppError> {
    if let Ok(home) = std::env::var("LMBRAIN_ANTIGRAVITY_HOME") {
        if !home.trim().is_empty() {
            return Ok(PathBuf::from(home));
        }
    }

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| AppError::WorkspaceNotFound("Could not resolve home directory".into()))?;
    Ok(PathBuf::from(home))
}

fn write_if_changed(path: &Path, existing: Option<&str>, content: &str) -> Result<(), AppError> {
    if existing == Some(content) {
        return Ok(());
    }
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, content)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_antigravity_config, register_antigravity_mcp_server_in};
    use serde_json::Value;
    use std::path::Path;

    fn read_config(path: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn creates_config_when_absent() {
        let out = build_antigravity_config(None, "/bin/lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "/bin/lmbrain-mcp");
        assert_eq!(value["mcpServers"]["lmbrain"]["args"][0], "--root");
        assert_eq!(value["mcpServers"]["lmbrain"]["args"][1], "/ws");
    }

    #[test]
    fn preserves_other_servers_and_keys() {
        let existing = r#"{
            "otherKey": 1,
            "mcpServers": {
                "github": { "serverUrl": "https://api.githubcopilot.com/mcp/", "disabled": true },
                "local": { "command": "local-mcp", "env": { "X": "1" } }
            }
        }"#;
        let out = build_antigravity_config(Some(existing), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["otherKey"], 1);
        assert_eq!(
            value["mcpServers"]["github"]["serverUrl"],
            "https://api.githubcopilot.com/mcp/"
        );
        assert_eq!(value["mcpServers"]["github"]["disabled"], true);
        assert_eq!(value["mcpServers"]["local"]["env"]["X"], "1");
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
    }

    #[test]
    fn is_idempotent() {
        let first = build_antigravity_config(None, "lmbrain-mcp", "/ws").unwrap();
        let second = build_antigravity_config(Some(&first), "lmbrain-mcp", "/ws").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn replaces_a_non_object_root() {
        let out = build_antigravity_config(Some("[]"), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
    }

    #[test]
    fn skips_when_no_antigravity_installation_is_detectable() {
        let home = tempfile::tempdir().unwrap();
        let ws = tempfile::tempdir().unwrap();

        let written =
            register_antigravity_mcp_server_in(home.path(), ws.path(), "lmbrain-mcp").unwrap();

        assert!(written.is_empty());
        assert!(!home.path().join(".gemini").exists());
    }

    #[test]
    fn creates_config_inside_existing_ide_directory() {
        let home = tempfile::tempdir().unwrap();
        let ws = tempfile::tempdir().unwrap();
        let ide_dir = home.path().join(".gemini").join("antigravity");
        std::fs::create_dir_all(&ide_dir).unwrap();

        let written =
            register_antigravity_mcp_server_in(home.path(), ws.path(), "lmbrain-mcp").unwrap();

        assert_eq!(written, vec![ide_dir.join("mcp_config.json")]);
        let value = read_config(&written[0]);
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
    }

    #[test]
    fn updates_both_layouts_when_both_exist() {
        let home = tempfile::tempdir().unwrap();
        let ws = tempfile::tempdir().unwrap();
        for dir in ["antigravity", "config"] {
            let dir = home.path().join(".gemini").join(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("mcp_config.json"),
                r#"{"mcpServers":{"keep":{"command":"keep-mcp"}}}"#,
            )
            .unwrap();
        }

        let written =
            register_antigravity_mcp_server_in(home.path(), ws.path(), "lmbrain-mcp").unwrap();

        assert_eq!(written.len(), 2);
        for path in &written {
            let value = read_config(path);
            assert_eq!(value["mcpServers"]["keep"]["command"], "keep-mcp");
            assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");
        }
    }

    #[test]
    fn reregistration_points_at_the_last_opened_workspace() {
        let home = tempfile::tempdir().unwrap();
        let first_ws = tempfile::tempdir().unwrap();
        let second_ws = tempfile::tempdir().unwrap();
        let dir = home.path().join(".gemini").join("antigravity");
        std::fs::create_dir_all(&dir).unwrap();

        register_antigravity_mcp_server_in(home.path(), first_ws.path(), "lmbrain-mcp").unwrap();
        let written =
            register_antigravity_mcp_server_in(home.path(), second_ws.path(), "lmbrain-mcp")
                .unwrap();

        let value = read_config(&written[0]);
        assert_eq!(
            value["mcpServers"]["lmbrain"]["args"][1],
            second_ws.path().to_string_lossy().as_ref()
        );
    }
}
