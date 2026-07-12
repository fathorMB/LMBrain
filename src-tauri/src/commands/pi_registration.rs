//! Registers the repository-scoped `lmbrain-mcp` server for Pi's pinned MCP
//! client extension. LMBrain owns only `.pi/mcp.json`; package installation
//! remains an explicit operator action.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::{json, Value};

use crate::commands::process::hide_console;
use crate::errors::AppError;

pub const PI_MCP_EXTENSION_NAME: &str = "pi-mcp-extension";
pub const PI_MCP_EXTENSION_VERSION: &str = "1.5.0";
pub const PI_MCP_EXTENSION_SOURCE: &str = "npm:pi-mcp-extension@1.5.0";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PiPreparationStatus {
    Ready,
    Installed,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct PiPreparationResult {
    pub status: PiPreparationStatus,
    pub message: String,
}

pub fn build_pi_mcp_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, AppError> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => serde_json::from_str(text)?,
        _ => json!({}),
    };
    let object = value
        .as_object_mut()
        .ok_or_else(|| AppError::Serialization(".pi/mcp.json must contain a JSON object".into()))?;
    let servers = object.entry("mcpServers").or_insert_with(|| json!({}));
    let servers = servers.as_object_mut().ok_or_else(|| {
        AppError::Serialization(".pi/mcp.json mcpServers must be a JSON object".into())
    })?;
    servers.insert(
        "lmbrain".to_string(),
        json!({
            "command": command,
            "args": ["--root", root],
            "transport": "stdio",
            "lifecycle": "eager"
        }),
    );
    Ok(serde_json::to_string_pretty(&value)?)
}

pub fn register_pi_mcp_server(root: &Path, command: &str) -> Result<PathBuf, AppError> {
    let config_dir = root.join(".pi");
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("mcp.json");
    let backup = config_dir.join("mcp.json.bak");
    if !config_path.exists() && backup.exists() {
        std::fs::rename(&backup, &config_path)?;
    }
    let existing = std::fs::read_to_string(&config_path).ok();
    let content = build_pi_mcp_config(existing.as_deref(), command, &root.to_string_lossy())?;
    if existing.as_deref() != Some(content.as_str()) {
        let temp = config_dir.join("mcp.json.tmp");
        std::fs::write(&temp, &content)?;
        if config_path.exists() {
            if backup.exists() {
                std::fs::remove_file(&backup)?;
            }
            std::fs::rename(&config_path, &backup)?;
            if let Err(error) = std::fs::rename(&temp, &config_path) {
                let _ = std::fs::rename(&backup, &config_path);
                let _ = std::fs::remove_file(&temp);
                return Err(error.into());
            }
            std::fs::remove_file(&backup)?;
        } else {
            std::fs::rename(&temp, &config_path)?;
        }
    }
    Ok(config_path)
}

pub fn has_pinned_pi_mcp_extension(output: &str) -> bool {
    output.lines().any(|line| {
        let normalized = line.to_ascii_lowercase();
        let has_exact_version = normalized
            .split(|character: char| {
                !(character.is_ascii_alphanumeric()
                    || matches!(character, '.' | '-' | '_' | '@' | ':' | '/'))
            })
            .any(|token| {
                token == PI_MCP_EXTENSION_VERSION
                    || token.ends_with(&format!("@{PI_MCP_EXTENSION_VERSION}"))
            });
        normalized.contains(PI_MCP_EXTENSION_NAME) && has_exact_version
    })
}

pub fn prepare_pi_mcp_extension(root: &Path) -> PiPreparationResult {
    let Some(pi) = command_on_path("pi") else {
        return unavailable(
            "Pi CLI is not on the desktop app PATH. Workspace opened without Pi integration.",
        );
    };

    if project_declares_pinned_extension(root) && pi_list_has_pinned_extension(&pi, root) {
        return PiPreparationResult {
            status: PiPreparationStatus::Ready,
            message: "Pi MCP integration is ready.".into(),
        };
    }

    let mut command = Command::new(&pi);
    command
        .arg("install")
        .arg(PI_MCP_EXTENSION_SOURCE)
        .arg("-l")
        .arg("--approve")
        .current_dir(root)
        .env("PI_SKIP_VERSION_CHECK", "1")
        .env("PI_TELEMETRY", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    hide_console(&mut command);
    let output = command.output();

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return unavailable(&format!(
                "Could not run Pi package installation: {error}. Workspace opened without Pi integration."
            ));
        }
    };

    if !output.status.success() {
        let detail = command_failure_detail(&output.stdout, &output.stderr);
        return unavailable(&format!(
            "Could not install {PI_MCP_EXTENSION_SOURCE}: {detail}. Workspace opened without Pi integration."
        ));
    }

    if !project_declares_pinned_extension(root) || !pi_list_has_pinned_extension(&pi, root) {
        return unavailable(
            "Pi reported a successful install but the exact project-local pin could not be verified. Workspace opened without Pi integration.",
        );
    }

    PiPreparationResult {
        status: PiPreparationStatus::Installed,
        message: format!("Installed project-local {PI_MCP_EXTENSION_SOURCE}."),
    }
}

fn pi_list_has_pinned_extension(pi: &Path, root: &Path) -> bool {
    let mut command = Command::new(pi);
    command
        .arg("list")
        .arg("--approve")
        .current_dir(root)
        .env("PI_OFFLINE", "1")
        .env("PI_SKIP_VERSION_CHECK", "1")
        .env("PI_TELEMETRY", "0");
    hide_console(&mut command);
    let output = command.output();
    match output {
        Ok(output) if output.status.success() => has_pinned_pi_mcp_extension(&format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )),
        _ => false,
    }
}

fn project_declares_pinned_extension(root: &Path) -> bool {
    let settings = match std::fs::read_to_string(root.join(".pi").join("settings.json")) {
        Ok(settings) => settings,
        Err(_) => return false,
    };
    let value: Value = match serde_json::from_str(&settings) {
        Ok(value) => value,
        Err(_) => return false,
    };
    json_contains_exact_string(&value, PI_MCP_EXTENSION_SOURCE)
}

fn json_contains_exact_string(value: &Value, expected: &str) -> bool {
    match value {
        Value::String(value) => value == expected,
        Value::Array(values) => values
            .iter()
            .any(|value| json_contains_exact_string(value, expected)),
        Value::Object(values) => values
            .values()
            .any(|value| json_contains_exact_string(value, expected)),
        _ => false,
    }
}

fn command_on_path(command: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) {
        vec![
            format!("{command}.exe"),
            format!("{command}.cmd"),
            format!("{command}.bat"),
            command.to_string(),
        ]
    } else {
        vec![command.to_string()]
    };
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

fn command_failure_detail(stdout: &[u8], stderr: &[u8]) -> String {
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(stderr),
        String::from_utf8_lossy(stdout)
    );
    let compact = combined.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "the installer exited unsuccessfully".into()
    } else {
        compact.chars().take(500).collect()
    }
}

fn unavailable(message: &str) -> PiPreparationResult {
    PiPreparationResult {
        status: PiPreparationStatus::Unavailable,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_pi_mcp_config, has_pinned_pi_mcp_extension, json_contains_exact_string,
        register_pi_mcp_server, PI_MCP_EXTENSION_SOURCE,
    };
    use serde_json::Value;

    #[test]
    fn creates_pi_mcp_config() {
        let out = build_pi_mcp_config(None, "/bin/lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        let server = &value["mcpServers"]["lmbrain"];
        assert_eq!(server["command"], "/bin/lmbrain-mcp");
        assert_eq!(server["args"], serde_json::json!(["--root", "/ws"]));
        assert_eq!(server["transport"], "stdio");
        assert_eq!(server["lifecycle"], "eager");
    }

    #[test]
    fn preserves_unrelated_pi_configuration_and_is_idempotent() {
        let existing =
            r#"{"settings":{"maxRetries":3},"mcpServers":{"other":{"command":"other"}}}"#;
        let first = build_pi_mcp_config(Some(existing), "lmbrain-mcp", "/ws").unwrap();
        let second = build_pi_mcp_config(Some(&first), "lmbrain-mcp", "/ws").unwrap();
        let value: Value = serde_json::from_str(&second).unwrap();
        assert_eq!(first, second);
        assert_eq!(value["settings"]["maxRetries"], 3);
        assert_eq!(value["mcpServers"]["other"]["command"], "other");
    }

    #[test]
    fn rejects_non_object_configuration() {
        assert!(build_pi_mcp_config(Some("[]"), "lmbrain-mcp", "/ws").is_err());
        assert!(build_pi_mcp_config(Some(r#"{"mcpServers":[]}"#), "lmbrain-mcp", "/ws").is_err());
    }

    #[test]
    fn detects_only_the_pinned_extension_version() {
        assert!(has_pinned_pi_mcp_extension(&format!(
            "installed {PI_MCP_EXTENSION_SOURCE}"
        )));
        assert!(!has_pinned_pi_mcp_extension(
            "installed npm:pi-mcp-extension@1.4.0"
        ));
        assert!(!has_pinned_pi_mcp_extension(
            "installed another-package@1.5.0"
        ));
        assert!(!has_pinned_pi_mcp_extension(
            "pi-mcp-extension (global)\nanother-package@1.5.0"
        ));
        assert!(!has_pinned_pi_mcp_extension(
            "installed npm:pi-mcp-extension@11.5.0"
        ));
    }

    #[test]
    fn registration_replaces_existing_file_and_cleans_temporary_state() {
        let dir = tempfile::tempdir().unwrap();
        let pi_dir = dir.path().join(".pi");
        std::fs::create_dir_all(&pi_dir).unwrap();
        std::fs::write(
            pi_dir.join("mcp.json"),
            r#"{"mcpServers":{"other":{"command":"other"}}}"#,
        )
        .unwrap();

        let path = register_pi_mcp_server(dir.path(), "/bin/lmbrain-mcp").unwrap();
        register_pi_mcp_server(dir.path(), "/bin/lmbrain-mcp").unwrap();
        let value: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

        assert_eq!(value["mcpServers"]["other"]["command"], "other");
        assert_eq!(
            value["mcpServers"]["lmbrain"]["command"],
            "/bin/lmbrain-mcp"
        );
        assert!(!pi_dir.join("mcp.json.tmp").exists());
        assert!(!pi_dir.join("mcp.json.bak").exists());
    }

    #[test]
    fn finds_exact_pin_in_string_or_object_package_entries() {
        assert!(json_contains_exact_string(
            &serde_json::json!({"packages": [PI_MCP_EXTENSION_SOURCE]}),
            PI_MCP_EXTENSION_SOURCE
        ));
        assert!(json_contains_exact_string(
            &serde_json::json!({"packages": [{"source": PI_MCP_EXTENSION_SOURCE}]}),
            PI_MCP_EXTENSION_SOURCE
        ));
        assert!(!json_contains_exact_string(
            &serde_json::json!({"packages": ["npm:pi-mcp-extension@latest"]}),
            PI_MCP_EXTENSION_SOURCE
        ));
    }
}
