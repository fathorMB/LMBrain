use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use toml_edit::{DocumentMut, Item};

use lmbrain_core::{
    load_harness_manifest, BrowserMcpCapability, CapabilityState, HarnessHost, HostConfiguration,
};

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
    pub browser_mcp: Option<BrowserMcpReadiness>,
    pub native_files: Vec<NativeFilePreview>,
    pub ready: bool,
}

/// Discovery-only readiness of the governed browser capability (#86): reports
/// the pre-provisioned Playwright MCP package and browser runtime without
/// executing anything. Absence or failure is never reported as active.
#[derive(Debug, Clone, Serialize)]
pub struct BrowserMcpReadiness {
    pub provider: String,
    pub package_available: bool,
    pub package_version: Option<String>,
    pub browser_runtime_found: bool,
    pub state: CapabilityState,
    pub detail: String,
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
            vec![plan_native_file(root, host, &effective, command)]
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
        let browser_mcp = effective
            .browser_mcp
            .as_ref()
            .map(|capability| browser_mcp_readiness(root, capability));
        let ready = tools.iter().all(|tool| tool.available)
            && native_files
                .iter()
                .all(|file| file.action != PreviewAction::Conflicted)
            && browser_mcp
                .as_ref()
                .is_none_or(|readiness| readiness.state == CapabilityState::PrerequisiteReady);
        hosts.push(HostPlan {
            host,
            supported_capabilities: supported_capabilities(host),
            effective,
            tools,
            lsp,
            browser_mcp,
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

fn plan_native_file(
    root: &Path,
    host: HarnessHost,
    effective: &HostConfiguration,
    command: &str,
) -> NativeFilePreview {
    let root_text = root.to_string_lossy();
    let (relative, owned, proposed) = match host {
        HarnessHost::ClaudeCode => {
            let path = ".mcp.json";
            let existing = read_optional(root.join(path));
            if json_path_conflict(existing.as_deref(), &["mcpServers"]) {
                return conflict(
                    path,
                    vec!["mcpServers.lmbrain", "mcpServers.lmbrain-browser"],
                    "mcpServers must be a JSON object",
                );
            }
            (
                path,
                vec!["mcpServers.lmbrain", "mcpServers.lmbrain-browser"],
                mcp_registration::build_mcp_config_with_browser(
                    existing.as_deref(),
                    command,
                    &root_text,
                    mcp_registration::BrowserEntry::Managed(effective.browser_mcp.as_ref()),
                )
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
    if matches!(host, HarnessHost::ClaudeCode) {
        values.push("browser-mcp");
    }
    values.into_iter().map(str::to_string).collect()
}

/// Discovery-only checks for the pre-provisioned Playwright MCP prerequisite:
/// package presence and version under the project-local `node_modules`, and a
/// best-effort browser-runtime probe honoring `PLAYWRIGHT_BROWSERS_PATH`.
/// Nothing is executed and nothing is ever installed.
fn browser_mcp_readiness(root: &Path, capability: &BrowserMcpCapability) -> BrowserMcpReadiness {
    let package_dir = root.join("node_modules").join("@playwright").join("mcp");
    let package_json = package_dir.join("package.json");
    let cli = package_dir.join("cli.js");
    let package_available = package_json.is_file() && cli.is_file();
    let package_version = fs::read_to_string(&package_json)
        .ok()
        .and_then(|source| serde_json::from_str::<Value>(&source).ok())
        .and_then(|value| {
            value
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
    let browser_runtime_found = playwright_chromium_found(root);
    let (state, detail) = if !package_available {
        (
            CapabilityState::Failed,
            "@playwright/mcp is not provisioned project-locally; the operator must install the pinned package under node_modules before approval".to_string(),
        )
    } else if !browser_runtime_found {
        (
            CapabilityState::Failed,
            "no Playwright Chromium runtime found; the operator must provision it (npx playwright install chromium) before approval".to_string(),
        )
    } else {
        (
            CapabilityState::PrerequisiteReady,
            format!(
                "@playwright/mcp {} provisioned with a Chromium runtime; isolated {} profile",
                package_version.as_deref().unwrap_or("(unknown version)"),
                if capability.headed { "headed" } else { "headless" }
            ),
        )
    };
    BrowserMcpReadiness {
        provider: "playwright".into(),
        package_available,
        package_version,
        browser_runtime_found,
        state,
        detail,
    }
}

fn playwright_chromium_found(root: &Path) -> bool {
    let candidates: Vec<PathBuf> = match env::var("PLAYWRIGHT_BROWSERS_PATH").ok().as_deref() {
        Some("0") => vec![root
            .join("node_modules")
            .join("playwright-core")
            .join(".local-browsers")],
        Some(custom) if !custom.trim().is_empty() => vec![PathBuf::from(custom)],
        _ => {
            let mut paths = Vec::new();
            if cfg!(windows) {
                if let Some(local) = env::var_os("LOCALAPPDATA") {
                    paths.push(PathBuf::from(local).join("ms-playwright"));
                }
            } else if let Some(home) = env::var_os("HOME") {
                let home = PathBuf::from(home);
                if cfg!(target_os = "macos") {
                    paths.push(home.join("Library").join("Caches").join("ms-playwright"));
                } else {
                    paths.push(home.join(".cache").join("ms-playwright"));
                }
            }
            paths
        }
    };
    candidates.iter().any(|directory| {
        fs::read_dir(directory)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .starts_with("chromium")
                })
            })
            .unwrap_or(false)
    })
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
    fn browser_capability_gates_readiness_and_derives_a_fixed_entry() {
        let dir = workspace(serde_json::json!({
            "schema_version": 1,
            "hosts": {"claude-code": {"enabled": true, "browser_mcp": {"provider": "playwright", "mode": "isolated"}}}
        }));

        // Nothing provisioned: the capability is Failed and the host not ready,
        // but the preview still shows the exact derived native entry.
        let plan = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        let host = &plan.hosts[0];
        let readiness = host.browser_mcp.as_ref().unwrap();
        assert!(!readiness.package_available);
        assert_eq!(readiness.state, CapabilityState::Failed);
        assert!(!host.ready);
        assert_eq!(
            host.native_files[0].owned_paths,
            vec!["mcpServers.lmbrain", "mcpServers.lmbrain-browser"]
        );

        // Provision the pinned package project-locally: readiness flips once a
        // browser runtime is also visible via PLAYWRIGHT_BROWSERS_PATH=0.
        let package = dir.path().join("node_modules/@playwright/mcp");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@playwright/mcp","version":"0.0.41"}"#,
        )
        .unwrap();
        fs::write(package.join("cli.js"), "// pinned cli").unwrap();
        fs::create_dir_all(
            dir.path()
                .join("node_modules/playwright-core/.local-browsers/chromium-1181"),
        )
        .unwrap();
        env::set_var("PLAYWRIGHT_BROWSERS_PATH", "0");
        let plan = plan_harness_configuration(dir.path(), "lmbrain-mcp").unwrap();
        env::remove_var("PLAYWRIGHT_BROWSERS_PATH");
        let host = &plan.hosts[0];
        let readiness = host.browser_mcp.as_ref().unwrap();
        assert!(readiness.package_available);
        assert_eq!(readiness.package_version.as_deref(), Some("0.0.41"));
        assert!(readiness.browser_runtime_found);
        assert_eq!(readiness.state, CapabilityState::PrerequisiteReady);
        assert!(host.ready);

        // The derived server definition is fixed: node + the project-local cli
        // in isolated mode, no agent-supplied strings.
        let preview = mcp_registration::build_mcp_config_with_browser(
            None,
            "lmbrain-mcp",
            &dir.path().to_string_lossy(),
            mcp_registration::BrowserEntry::Managed(Some(&lmbrain_core::BrowserMcpCapability {
                provider: lmbrain_core::BrowserMcpProvider::Playwright,
                mode: lmbrain_core::BrowserMcpMode::Isolated,
                headed: true,
            })),
        )
        .unwrap();
        let value: Value = serde_json::from_str(&preview).unwrap();
        let browser = &value["mcpServers"]["lmbrain-browser"];
        assert_eq!(browser["command"], "node");
        assert_eq!(
            browser["args"],
            serde_json::json!([
                "node_modules/@playwright/mcp/cli.js",
                "--isolated",
                "--browser",
                "chromium"
            ])
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
