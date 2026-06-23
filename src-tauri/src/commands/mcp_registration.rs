//! Registers the repository-scoped `lmbrain-mcp` server with the agent host by
//! writing a host-readable `.mcp.json` at the workspace root, so agents receive
//! the controlled-mutation tools instead of editing Markdown by hand.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::errors::AppError;

/// Build the `.mcp.json` content that registers the `lmbrain` server, merging into
/// any existing configuration and preserving unrelated keys and other servers.
pub fn build_mcp_config(existing: Option<&str>, command: &str, root: &str) -> Result<String, AppError> {
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
    servers.as_object_mut().expect("mcpServers is an object").insert(
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

/// Resolve the value to place in `.mcp.json`'s `command`: an explicit
/// `LMBRAIN_MCP_BIN` override first, then the binary sitting next to the running
/// executable (the dev/workspace build and, if bundled, the installed app), then
/// the bare name resolved through `PATH`.
pub fn resolve_mcp_command() -> String {
    if let Ok(value) = std::env::var("LMBRAIN_MCP_BIN") {
        if !value.trim().is_empty() {
            return value;
        }
    }
    let binary = if cfg!(windows) { "lmbrain-mcp.exe" } else { "lmbrain-mcp" };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(candidate) = exe.parent().map(|dir| dir.join(binary)) {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    "lmbrain-mcp".to_string()
}

#[cfg(test)]
mod tests {
    use super::build_mcp_config;
    use serde_json::Value;

    #[test]
    fn creates_config_when_absent() {
        let out = build_mcp_config(None, "/bin/lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "/bin/lmbrain-mcp");
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
}
