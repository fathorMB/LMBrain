use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use toml_edit::{DocumentMut, Item};

use lmbrain_core::{load_harness_manifest, CapabilityState, HarnessHost, HostConfiguration};

use crate::commands::{
    codex_registration, mcp_registration, opencode_registration, pi_registration,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewAction {
    Preserved,
    Added,
    Changed,
    Conflicted,
}

#[derive(Debug, Clone, Serialize)]
pub struct NativeFilePreview {
    pub path: String,
    pub owned_paths: Vec<String>,
    pub action: PreviewAction,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolReadiness {
    pub tool: String,
    pub available: bool,
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostPlan {
    pub host: HarnessHost,
    pub effective: HostConfiguration,
    pub supported_capabilities: Vec<String>,
    pub tools: Vec<ToolReadiness>,
    pub lsp: Option<LspReadiness>,
    pub native_files: Vec<NativeFilePreview>,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LspReadiness {
    pub configured: bool,
    pub prerequisite_ready: bool,
    pub state: CapabilityState,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessConfigurationPlan {
    pub manifest_digest: String,
    pub hosts: Vec<HostPlan>,
    pub has_conflicts: bool,
}

pub fn plan_harness_configuration(
    root: &Path,
    command: &str,
) -> Result<HarnessConfigurationPlan, String> {
    let manifest = load_harness_manifest(root).map_err(|error| error.to_string())?;
    let digest =
        lmbrain_core::canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?;
    let mut hosts = Vec::new();
    for (host, effective) in manifest.hosts {
        let tools = effective
            .required_tools
            .iter()
            .map(|tool| {
                let resolved = find_on_path(tool);
                ToolReadiness {
                    tool: tool.clone(),
                    available: resolved.is_some(),
                    resolved_path: resolved.map(|path| path.to_string_lossy().into_owned()),
                }
            })
            .collect::<Vec<_>>();
        let native_files = if effective.enabled {
            vec![plan_native_file(root, host, command)]
        } else {
            Vec::new()
        };
        let lsp = effective.lsp.as_ref().map(|policy| {
            let prerequisite_ready = tools.iter().all(|tool| tool.available);
            let state = if !policy.required {
                CapabilityState::Configured
            } else if !prerequisite_ready {
                CapabilityState::Failed
            } else if host == HarnessHost::OpenCode {
                CapabilityState::InactiveLazy
            } else {
                CapabilityState::Unknown
            };
            LspReadiness {
                configured: true,
                prerequisite_ready,
                state,
            }
        });
        let ready = tools.iter().all(|tool| tool.available)
            && native_files
                .iter()
                .all(|file| file.action != PreviewAction::Conflicted);
        hosts.push(HostPlan {
            host,
            supported_capabilities: supported_capabilities(host),
            effective,
            tools,
            lsp,
            native_files,
            ready,
        });
    }
    let has_conflicts = hosts
        .iter()
        .flat_map(|host| &host.native_files)
        .any(|file| file.action == PreviewAction::Conflicted);
    Ok(HarnessConfigurationPlan {
        manifest_digest: digest,
        hosts,
        has_conflicts,
    })
}

fn plan_native_file(root: &Path, host: HarnessHost, command: &str) -> NativeFilePreview {
    let root_text = root.to_string_lossy();
    let (relative, owned, proposed) = match host {
        HarnessHost::ClaudeCode => {
            let path = ".mcp.json";
            let existing = read_optional(root.join(path));
            if json_path_conflict(existing.as_deref(), &["mcpServers"]) {
                return conflict(
                    path,
                    vec!["mcpServers.lmbrain"],
                    "mcpServers must be a JSON object",
                );
            }
            (
                path,
                vec!["mcpServers.lmbrain"],
                mcp_registration::build_mcp_config(existing.as_deref(), command, &root_text)
                    .map_err(|error| error.to_string()),
            )
        }
        HarnessHost::Codex => {
            let path = ".codex/config.toml";
            let existing = read_optional(root.join(path));
            if let Some(detail) = codex_conflict(existing.as_deref()) {
                return conflict(path, vec!["mcp_servers.lmbrain"], &detail);
            }
            (
                path,
                vec!["mcp_servers.lmbrain"],
                codex_registration::build_codex_project_config(
                    existing.as_deref(),
                    command,
                    &root_text,
                )
                .map_err(|error| error.to_string()),
            )
        }
        HarnessHost::Pi => {
            let path = ".pi/mcp.json";
            let existing = read_optional(root.join(path));
            (
                path,
                vec!["mcpServers.lmbrain"],
                pi_registration::build_pi_mcp_config(existing.as_deref(), command, &root_text)
                    .map_err(|error| error.to_string()),
            )
        }
        HarnessHost::OpenCode => {
            let path = "opencode.json";
            let existing = read_optional(root.join(path));
            (
                path,
                vec![
                    "mcp.lmbrain",
                    "references.workspace",
                    "lsp (only when absent)",
                ],
                opencode_registration::build_opencode_config(
                    existing.as_deref(),
                    command,
                    &root_text,
                )
                .map_err(|error| error.to_string()),
            )
        }
    };
    let existing = read_optional(root.join(relative));
    match proposed {
        Err(message) => conflict(relative, owned, &message),
        Ok(content) => {
            let action = match existing.as_deref() {
                None => PreviewAction::Added,
                Some(current) if semantically_equal(relative, current, &content) => {
                    PreviewAction::Preserved
                }
                Some(_) => PreviewAction::Changed,
            };
            NativeFilePreview {
                path: relative.into(),
                owned_paths: owned.into_iter().map(str::to_string).collect(),
                detail: match action {
                    PreviewAction::Added => "create managed configuration".into(),
                    PreviewAction::Changed => {
                        "update LMBrain-owned paths while preserving unrelated configuration".into()
                    }
                    PreviewAction::Preserved => "already matches effective configuration".into(),
                    PreviewAction::Conflicted => unreachable!(),
                },
                action,
            }
        }
    }
}

fn codex_conflict(source: Option<&str>) -> Option<String> {
    let source = source?;
    let document = match source.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(error) => return Some(format!("invalid Codex TOML: {error}")),
    };
    let Some(servers) = document.get("mcp_servers") else {
        return None;
    };
    let Some(servers) = servers.as_table() else {
        return Some("mcp_servers must be a TOML table".into());
    };
    if let Some(item) = servers.get("lmbrain") {
        if !matches!(item, Item::Table(_)) {
            return Some("mcp_servers.lmbrain must be a TOML table".into());
        }
    }
    None
}

fn conflict(path: &str, owned: Vec<&str>, detail: &str) -> NativeFilePreview {
    NativeFilePreview {
        path: path.into(),
        owned_paths: owned.into_iter().map(str::to_string).collect(),
        action: PreviewAction::Conflicted,
        detail: detail.into(),
    }
}

fn read_optional(path: PathBuf) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn json_path_conflict(source: Option<&str>, path: &[&str]) -> bool {
    let Some(source) = source else {
        return false;
    };
    let Ok(mut value) = serde_json::from_str::<Value>(source) else {
        return true;
    };
    for key in path {
        match value.get_mut(*key) {
            Some(next) if next.is_object() => value = next.take(),
            Some(_) => return true,
            None => return false,
        }
    }
    false
}

fn semantically_equal(path: &str, left: &str, right: &str) -> bool {
    if path.ends_with(".json") {
        serde_json::from_str::<Value>(left).ok() == serde_json::from_str::<Value>(right).ok()
    } else {
        left == right
    }
}

fn supported_capabilities(host: HarnessHost) -> Vec<String> {
    let mut values = vec!["enabled", "required-tools", "environment"];
    if matches!(host, HarnessHost::ClaudeCode | HarnessHost::OpenCode) {
        values.push("lsp");
    }
    values.into_iter().map(str::to_string).collect()
}

fn find_on_path(tool: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let extensions: Vec<String> = if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    };
    for directory in env::split_paths(&path) {
        for extension in &extensions {
            let candidate = directory.join(format!("{tool}{extension}"));
            if candidate.is_file() {
                return candidate.canonicalize().ok().or(Some(candidate));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use lmbrain_core::{set_harness_manifest, HarnessManifest};

    fn workspace(manifest: Value) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        let manifest: HarnessManifest = serde_json::from_value(manifest).unwrap();
        set_harness_manifest(dir.path(), &manifest).unwrap();
        dir
    }

    #[test]
    fn preview_is_deterministic_and_preserves_unrelated_json() {
        let dir = workspace(
            serde_json::json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true}}}),
        );
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{"other":7,"mcpServers":{"other":{"command":"x"}}}"#,
        )
        .unwrap();
        let first = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        let second = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
        assert_eq!(
            first.hosts[0].native_files[0].action,
            PreviewAction::Changed
        );
        assert_eq!(
            serde_json::from_str::<Value>(
                &fs::read_to_string(dir.path().join(".mcp.json")).unwrap()
            )
            .unwrap()["other"],
            7
        );
    }

    #[test]
    fn incompatible_owned_parent_is_a_conflict_and_no_file_is_changed() {
        let dir =
            workspace(serde_json::json!({"schema_version":1,"hosts":{"pi":{"enabled":true}}}));
        let original = r#"{"mcpServers":[]}"#;
        fs::create_dir(dir.path().join(".pi")).unwrap();
        fs::write(dir.path().join(".pi/mcp.json"), original).unwrap();
        let plan = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        assert!(plan.has_conflicts);
        assert_eq!(
            plan.hosts[0].native_files[0].action,
            PreviewAction::Conflicted
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(".pi/mcp.json")).unwrap(),
            original
        );
    }

    #[test]
    fn codex_scalar_owned_parent_is_reported_without_panicking() {
        let dir =
            workspace(serde_json::json!({"schema_version":1,"hosts":{"codex":{"enabled":true}}}));
        fs::create_dir(dir.path().join(".codex")).unwrap();
        fs::write(
            dir.path().join(".codex/config.toml"),
            "mcp_servers = false\n",
        )
        .unwrap();
        let plan = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        assert!(plan.has_conflicts);
        assert_eq!(
            plan.hosts[0].native_files[0].action,
            PreviewAction::Conflicted
        );
    }
}
