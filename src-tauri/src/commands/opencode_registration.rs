//! Registers LMBrain's repository-scoped MCP server in OpenCode's project
//! configuration. OpenCode supports local MCP servers natively, so this never
//! installs a project package or mutates user-level configuration.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::errors::AppError;

pub fn build_opencode_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, AppError> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => serde_json::from_str(text)?,
        _ => json!({}),
    };
    let object = value.as_object_mut().ok_or_else(|| {
        AppError::Serialization("opencode.json must contain a JSON object".into())
    })?;
    let mcp = object.entry("mcp").or_insert_with(|| json!({}));
    if !mcp.is_object() {
        return Err(AppError::Serialization(
            "opencode.json mcp must be a JSON object".into(),
        ));
    }
    mcp.as_object_mut().expect("mcp is an object").insert(
        "lmbrain".into(),
        json!({
            "type": "local",
            "command": [command, "--root", root],
            "enabled": true
        }),
    );
    Ok(serde_json::to_string_pretty(&value)?)
}

pub fn register_opencode_mcp_server(root: &Path, command: &str) -> Result<PathBuf, AppError> {
    let config_path = root.join("opencode.json");
    let existing = match std::fs::read_to_string(&config_path) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let content = build_opencode_config(existing.as_deref(), command, &root.to_string_lossy())?;
    if existing.as_deref() != Some(content.as_str()) {
        let temp = root.join("opencode.json.tmp");
        let backup = root.join("opencode.json.bak");
        std::fs::write(&temp, &content)?;
        if config_path.exists() {
            if backup.exists() {
                std::fs::remove_file(&backup)?;
            }
            std::fs::rename(&config_path, &backup)?;
        }
        if let Err(error) = std::fs::rename(&temp, &config_path) {
            if backup.exists() {
                let _ = std::fs::rename(&backup, &config_path);
            }
            let _ = std::fs::remove_file(&temp);
            return Err(error.into());
        }
        if backup.exists() {
            std::fs::remove_file(&backup)?;
        }
    }
    Ok(config_path)
}

#[cfg(test)]
mod tests {
    use super::{build_opencode_config, register_opencode_mcp_server};
    use serde_json::Value;

    #[test]
    fn creates_native_local_mcp_configuration() {
        let output = build_opencode_config(None, "/bin/lmbrain-mcp", "/workspace").unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["mcp"]["lmbrain"]["type"], "local");
        assert_eq!(value["mcp"]["lmbrain"]["enabled"], true);
        assert_eq!(value["mcp"]["lmbrain"]["command"][0], "/bin/lmbrain-mcp");
        assert_eq!(value["mcp"]["lmbrain"]["command"][1], "--root");
        assert_eq!(value["mcp"]["lmbrain"]["command"][2], "/workspace");
    }

    #[test]
    fn preserves_unrelated_configuration_and_is_idempotent() {
        let existing = r#"{"$schema":"https://opencode.ai/config.json","provider":{"ollama":{"name":"Local"}},"mcp":{"github":{"type":"remote","url":"https://example.test"}}}"#;
        let first = build_opencode_config(Some(existing), "lmbrain-mcp", "/ws").unwrap();
        let second = build_opencode_config(Some(&first), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&first).unwrap();
        assert_eq!(value["provider"]["ollama"]["name"], "Local");
        assert_eq!(value["mcp"]["github"]["url"], "https://example.test");
        assert_eq!(first, second);
    }

    #[test]
    fn rejects_malformed_or_structurally_incompatible_configuration() {
        assert!(build_opencode_config(Some("{"), "lmbrain-mcp", "/ws").is_err());
        assert!(build_opencode_config(Some("[]"), "lmbrain-mcp", "/ws").is_err());
        assert!(build_opencode_config(Some(r#"{"mcp":[]}"#), "lmbrain-mcp", "/ws").is_err());
    }

    #[test]
    fn writes_project_configuration_without_leaving_temporary_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = register_opencode_mcp_server(dir.path(), "lmbrain-mcp").unwrap();
        register_opencode_mcp_server(dir.path(), "lmbrain-mcp").unwrap();
        assert!(path.is_file());
        assert!(!dir.path().join("opencode.json.tmp").exists());
        assert!(!dir.path().join("opencode.json.bak").exists());
    }
}
