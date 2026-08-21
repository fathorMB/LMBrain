//! Shared project-environment engine (#87): host-adapter builders, the
//! machine-local approval store, the deterministic native-file planner, and
//! the atomic materializer. Both the desktop app (read-only consultation) and
//! the `lmbrain-mcp` server (the Project Lead's active management surface)
//! call into this module, so every invariant — digest binding, conflict
//! refusal, preservation of unrelated configuration, rollback, and drift
//! reporting — is enforced here rather than in any UI.

use std::{
    collections::BTreeMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use toml_edit::{value, Array, DocumentMut, Item};

use crate::harness_manifest::{
    canonical_manifest_digest, load_harness_manifest, workspace_identity, BrowserMcpCapability,
    CapabilityState, HarnessHost, HarnessManifestError, HostConfiguration,
};

const STORE_SCHEMA_VERSION: u32 = 1;
const HARNESS_AUDIT_PATH: &str = ".lmbrain/HARNESSES.audit.jsonl";
const APP_IDENTIFIER: &str = "com.lmbrain.app";

/// Key of the LMBrain-owned browser MCP entry in `.mcp.json` (#86).
pub const BROWSER_MCP_SERVER_KEY: &str = "lmbrain-browser";

// ---------------------------------------------------------------------------
// Host adapter builders
// ---------------------------------------------------------------------------

/// How a `.mcp.json` build treats the LMBrain-owned browser entry: workspace
/// auto-registration must not touch it (it is governed by the approved harness
/// manifest), while the planner/materializer manage it explicitly.
pub enum BrowserEntry<'a> {
    Untouched,
    Managed(Option<&'a BrowserMcpCapability>),
}

/// Derives the fixed, allow-listed Playwright MCP server definition from the
/// typed capability. The command and arguments are constants of the profile:
/// the operator pre-provisions `@playwright/mcp` project-locally and no
/// agent-supplied string is ever serialized here.
fn browser_server_definition(capability: &BrowserMcpCapability) -> Value {
    let mut args = vec![
        "node_modules/@playwright/mcp/cli.js".to_string(),
        "--isolated".to_string(),
        "--browser".to_string(),
        "chromium".to_string(),
    ];
    if !capability.headed {
        args.push("--headless".to_string());
    }
    json!({ "command": "node", "args": args })
}

/// Build `.mcp.json` content registering the `lmbrain` server, merging into any
/// existing configuration and preserving unrelated keys and other servers.
pub fn build_claude_mcp_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
    browser: BrowserEntry<'_>,
) -> Result<String, String> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        }
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
    let servers = servers.as_object_mut().expect("mcpServers is an object");
    servers.insert(
        "lmbrain".to_string(),
        json!({ "command": command, "args": ["--root", root] }),
    );
    match browser {
        BrowserEntry::Untouched => {}
        BrowserEntry::Managed(Some(capability)) => {
            servers.insert(
                BROWSER_MCP_SERVER_KEY.to_string(),
                browser_server_definition(capability),
            );
        }
        BrowserEntry::Managed(None) => {
            servers.remove(BROWSER_MCP_SERVER_KEY);
        }
    }
    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())
}

/// Build the `.codex/config.toml` content that registers the `lmbrain` MCP
/// server, preserving unrelated project Codex settings.
pub fn build_codex_project_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, String> {
    let mut doc = match existing {
        Some(text) if !text.trim().is_empty() => text
            .parse::<DocumentMut>()
            .map_err(|error| error.to_string())?,
        _ => DocumentMut::new(),
    };
    doc["mcp_servers"]["lmbrain"]["command"] = value(command);
    let mut args = Array::new();
    args.push("--root");
    args.push(root);
    doc["mcp_servers"]["lmbrain"]["args"] = value(args);
    Ok(doc.to_string())
}

/// Build the `.pi/mcp.json` content for Pi's pinned MCP client extension.
pub fn build_pi_mcp_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, String> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        }
        _ => json!({}),
    };
    let object = value
        .as_object_mut()
        .ok_or_else(|| ".pi/mcp.json must contain a JSON object".to_string())?;
    let servers = object.entry("mcpServers").or_insert_with(|| json!({}));
    let servers = servers
        .as_object_mut()
        .ok_or_else(|| ".pi/mcp.json mcpServers must be a JSON object".to_string())?;
    servers.insert(
        "lmbrain".to_string(),
        json!({
            "command": command,
            "args": ["--root", root],
            "transport": "stdio",
            "lifecycle": "eager"
        }),
    );
    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())
}

/// Build the `opencode.json` content registering LMBrain's MCP server.
pub fn build_opencode_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, String> {
    let mut value: Value = match existing {
        Some(text) if !text.trim().is_empty() => {
            serde_json::from_str(text).map_err(|error| error.to_string())?
        }
        _ => json!({}),
    };
    let object = value
        .as_object_mut()
        .ok_or_else(|| "opencode.json must contain a JSON object".to_string())?;
    // OpenCode disables every built-in LSP when this key is absent. Enable its
    // built-ins for LMBrain workspaces, but never override an explicit operator
    // choice (`false`) or custom per-server object.
    object.entry("lsp").or_insert(Value::Bool(true));
    let references = object.entry("references").or_insert_with(|| json!({}));
    if !references.is_object() {
        return Err("opencode.json references must be a JSON object".into());
    }
    references
        .as_object_mut()
        .expect("references is an object")
        .entry("workspace")
        .or_insert_with(|| {
            json!({
                "path": ".",
                "description": "LMBrain project workspace"
            })
        });
    let mcp = object.entry("mcp").or_insert_with(|| json!({}));
    if !mcp.is_object() {
        return Err("opencode.json mcp must be a JSON object".into());
    }
    mcp.as_object_mut().expect("mcp is an object").insert(
        "lmbrain".into(),
        json!({
            "type": "local",
            "command": [command, "--root", root],
            "enabled": true
        }),
    );
    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())
}

// ---------------------------------------------------------------------------
// Machine-local approval store
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ApprovalStoreData {
    schema_version: u32,
    #[serde(default)]
    approvals: BTreeMap<String, ApprovalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalRecord {
    manifest_digest: String,
    approved_at: String,
    #[serde(default)]
    applied_files: BTreeMap<String, String>,
    #[serde(default)]
    applied_at: Option<String>,
    #[serde(default)]
    actor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessApprovalState {
    Unconfigured,
    ApprovalRequired,
    Approved,
    Stale,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessApprovalStatus {
    pub state: HarnessApprovalState,
    pub manifest_digest: Option<String>,
    pub approved_digest: Option<String>,
    pub approved_at: Option<String>,
    pub approved_by: Option<String>,
    pub workspace_fingerprint: String,
}

/// The machine-local approval store shared by the desktop app and the MCP
/// server: `<data dir>/com.lmbrain.app/lmbrain/harness-approvals.json`. This
/// mirrors Tauri's `app_data_dir` so approvals recorded by either surface are
/// visible to the other.
pub fn default_harness_approval_store_path() -> Result<PathBuf, String> {
    let base = if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or("APPDATA is not set")?
    } else if cfg!(target_os = "macos") {
        PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?)
            .join("Library")
            .join("Application Support")
    } else {
        match env::var_os("XDG_DATA_HOME") {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?)
                .join(".local")
                .join("share"),
        }
    };
    Ok(base
        .join(APP_IDENTIFIER)
        .join("lmbrain")
        .join("harness-approvals.json"))
}

fn load_store(path: &Path) -> Result<ApprovalStoreData, String> {
    if !path.exists() {
        return Ok(ApprovalStoreData {
            schema_version: STORE_SCHEMA_VERSION,
            approvals: BTreeMap::new(),
        });
    }
    let parsed = fs::read_to_string(path)
        .map_err(|error| error.to_string())
        .and_then(|source| {
            serde_json::from_str::<ApprovalStoreData>(&source).map_err(|error| error.to_string())
        });
    match parsed {
        Ok(data) if data.schema_version == STORE_SCHEMA_VERSION => Ok(data),
        Ok(_) | Err(_) => {
            // Quarantine the unreadable store: approvals are never reused from
            // a corrupt file.
            let backup = path.with_file_name(format!(
                "harness-approvals.corrupt-{}.json",
                chrono::Utc::now().format("%Y%m%dT%H%M%S")
            ));
            fs::rename(path, backup).map_err(|error| error.to_string())?;
            Ok(ApprovalStoreData {
                schema_version: STORE_SCHEMA_VERSION,
                approvals: BTreeMap::new(),
            })
        }
    }
}

fn save_store(path: &Path, data: &ApprovalStoreData) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = format!(
        "{}\n",
        serde_json::to_string_pretty(data).map_err(|error| error.to_string())?
    );
    crate::frontmatter::atomic_write(path, &content).map_err(|error| error.to_string())?;
    restrict_permissions(path)
}

pub fn harness_approval_status(
    root: &Path,
    store_path: &Path,
) -> Result<HarnessApprovalStatus, String> {
    let identity = workspace_identity(root).map_err(|error| error.to_string())?;
    let manifest_digest = match load_harness_manifest(root) {
        Ok(manifest) => {
            Some(canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?)
        }
        Err(HarnessManifestError::Missing(_)) => None,
        Err(error) => return Err(error.to_string()),
    };
    let data = load_store(store_path)?;
    let record = data.approvals.get(&identity.fingerprint);
    let state = match (&manifest_digest, record) {
        (None, _) => HarnessApprovalState::Unconfigured,
        (Some(_), None) => HarnessApprovalState::ApprovalRequired,
        (Some(current), Some(approved)) if current == &approved.manifest_digest => {
            HarnessApprovalState::Approved
        }
        (Some(_), Some(_)) => HarnessApprovalState::Stale,
    };
    Ok(HarnessApprovalStatus {
        state,
        manifest_digest,
        approved_digest: record.map(|record| record.manifest_digest.clone()),
        approved_at: record.map(|record| record.approved_at.clone()),
        approved_by: record.and_then(|record| record.actor.clone()),
        workspace_fingerprint: identity.fingerprint,
    })
}

/// Digest-bound approval. The caller states the exact digest it previewed; a
/// manifest that changed in the meantime is refused. Every approval is audited
/// with its actor in `.lmbrain/HARNESSES.audit.jsonl`.
pub fn approve_harness_manifest(
    root: &Path,
    store_path: &Path,
    expected_digest: &str,
    actor: &str,
) -> Result<HarnessApprovalStatus, String> {
    let identity = workspace_identity(root).map_err(|error| error.to_string())?;
    let manifest = load_harness_manifest(root).map_err(|error| error.to_string())?;
    let current = canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?;
    if current != expected_digest {
        return Err(
            "manifest changed since preview; refresh and approve the current digest".into(),
        );
    }
    let mut data = load_store(store_path)?;
    data.approvals.insert(
        identity.fingerprint,
        ApprovalRecord {
            manifest_digest: current.clone(),
            approved_at: chrono::Utc::now().to_rfc3339(),
            applied_files: BTreeMap::new(),
            applied_at: None,
            actor: Some(actor.to_string()),
        },
    );
    save_store(store_path, &data)?;
    audit_environment_action(root, "harness_manifest_approve", &current, actor)?;
    harness_approval_status(root, store_path)
}

pub fn revoke_harness_approval(
    root: &Path,
    store_path: &Path,
    actor: &str,
) -> Result<HarnessApprovalStatus, String> {
    let identity = workspace_identity(root).map_err(|error| error.to_string())?;
    let mut data = load_store(store_path)?;
    let removed = data.approvals.remove(&identity.fingerprint);
    save_store(store_path, &data)?;
    if let Some(record) = removed {
        audit_environment_action(
            root,
            "harness_approval_revoke",
            &record.manifest_digest,
            actor,
        )?;
    }
    harness_approval_status(root, store_path)
}

pub fn record_application(
    root: &Path,
    store_path: &Path,
    manifest_digest: &str,
    files: &[(String, String)],
) -> Result<(), String> {
    let identity = workspace_identity(root).map_err(|error| error.to_string())?;
    let mut data = load_store(store_path)?;
    let record = data
        .approvals
        .get_mut(&identity.fingerprint)
        .ok_or("manifest is not approved")?;
    if record.manifest_digest != manifest_digest {
        return Err("approval digest no longer matches apply result".into());
    }
    record.applied_files = files.iter().cloned().collect();
    record.applied_at = Some(chrono::Utc::now().to_rfc3339());
    save_store(store_path, &data)
}

pub fn applied_files(root: &Path, store_path: &Path) -> Result<BTreeMap<String, String>, String> {
    let identity = workspace_identity(root).map_err(|error| error.to_string())?;
    Ok(load_store(store_path)?
        .approvals
        .get(&identity.fingerprint)
        .map(|record| record.applied_files.clone())
        .unwrap_or_default())
}

fn audit_environment_action(
    root: &Path,
    action: &str,
    digest: &str,
    actor: &str,
) -> Result<(), String> {
    let path = root.join(HARNESS_AUDIT_PATH);
    if path.exists()
        && fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err(format!("audit path is a symlink: {}", path.display()));
    }
    let entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "action": action,
        "manifest_digest": digest,
        "actor": actor,
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{entry}").map_err(|error| error.to_string())?;
    file.sync_data().map_err(|error| error.to_string())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

// ---------------------------------------------------------------------------
// Deterministic native-file planner
// ---------------------------------------------------------------------------

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
pub struct LspReadiness {
    pub configured: bool,
    pub prerequisite_ready: bool,
    pub state: CapabilityState,
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
    let browsers_path = env::var("PLAYWRIGHT_BROWSERS_PATH").ok();
    plan_harness_configuration_with_browser_path(root, command, browsers_path.as_deref())
}

fn plan_harness_configuration_with_browser_path(
    root: &Path,
    command: &str,
    browsers_path: Option<&str>,
) -> Result<HarnessConfigurationPlan, String> {
    let manifest = load_harness_manifest(root).map_err(|error| error.to_string())?;
    let digest = canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?;
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
            .map(|capability| browser_mcp_readiness(root, capability, browsers_path));
        let ready = tools.iter().all(|tool| tool.available)
            && native_files
                .iter()
                .all(|file| file.action != PreviewAction::Conflicted)
            && browser_mcp.as_ref().map_or(true, |readiness| {
                readiness.state == CapabilityState::PrerequisiteReady
            });
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
                build_claude_mcp_config(
                    existing.as_deref(),
                    command,
                    &root_text,
                    BrowserEntry::Managed(effective.browser_mcp.as_ref()),
                ),
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
                build_codex_project_config(existing.as_deref(), command, &root_text),
            )
        }
        HarnessHost::Pi => {
            let path = ".pi/mcp.json";
            let existing = read_optional(root.join(path));
            (
                path,
                vec!["mcpServers.lmbrain"],
                build_pi_mcp_config(existing.as_deref(), command, &root_text),
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
                build_opencode_config(existing.as_deref(), command, &root_text),
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
    let servers = document.get("mcp_servers")?;
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
fn browser_mcp_readiness(
    root: &Path,
    capability: &BrowserMcpCapability,
    browsers_path: Option<&str>,
) -> BrowserMcpReadiness {
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
    let browser_runtime_found = playwright_chromium_found_with_path(root, browsers_path);
    let (state, detail) = if !package_available {
        (
            CapabilityState::Failed,
            "@playwright/mcp is not provisioned project-locally; the operator must install the pinned package under node_modules before approval".to_string(),
        )
    } else if !browser_runtime_found {
        (
            CapabilityState::Failed,
            "the Playwright-managed Chromium executable the provider launches is missing; the operator must provision it (npx @playwright/mcp install-browser, or npx playwright install chromium) before approval".to_string(),
        )
    } else {
        (
            CapabilityState::PrerequisiteReady,
            format!(
                "@playwright/mcp {} provisioned with a Chromium runtime; isolated {} profile",
                package_version.as_deref().unwrap_or("(unknown version)"),
                if capability.headed {
                    "headed"
                } else {
                    "headless"
                }
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

fn playwright_chromium_found_with_path(root: &Path, browsers_path: Option<&str>) -> bool {
    let candidates: Vec<PathBuf> = match browsers_path {
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
    let required_revision = playwright_required_chromium_revision(root);
    candidates
        .iter()
        .any(|directory| chromium_executable_present(directory, required_revision.as_deref()))
}

/// The Chromium revision pinned by the provisioned `playwright-core`, read from
/// its `browsers.json`. `None` when the manifest is absent or unreadable, in
/// which case the probe accepts any installed `chromium-*` revision.
fn playwright_required_chromium_revision(root: &Path) -> Option<String> {
    let manifest = root
        .join("node_modules")
        .join("playwright-core")
        .join("browsers.json");
    let value: Value = serde_json::from_str(&fs::read_to_string(manifest).ok()?).ok()?;
    value
        .get("browsers")?
        .as_array()?
        .iter()
        .find_map(|browser| {
            if browser.get("name").and_then(Value::as_str) == Some("chromium") {
                browser
                    .get("revision")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                None
            }
        })
}

/// True only when a `chromium-*` revision directory actually contains the
/// platform executable the provider launches. A bare revision directory left
/// behind by another Playwright install must not count as a runtime
/// (issue #96 / KIT-NOTE-013).
fn chromium_executable_present(directory: &Path, required_revision: Option<&str>) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let matches = match required_revision {
            Some(revision) => name == format!("chromium-{revision}"),
            None => name.starts_with("chromium-"),
        };
        matches && chromium_executable_in(&entry.path())
    })
}

fn chromium_executable_in(revision_dir: &Path) -> bool {
    let candidates: &[&[&str]] = if cfg!(windows) {
        &[
            &["chrome-win64", "chrome.exe"],
            &["chrome-win", "chrome.exe"],
        ]
    } else if cfg!(target_os = "macos") {
        &[
            &[
                "chrome-mac",
                "Chromium.app",
                "Contents",
                "MacOS",
                "Chromium",
            ],
            &[
                "chrome-mac-arm64",
                "Chromium.app",
                "Contents",
                "MacOS",
                "Chromium",
            ],
        ]
    } else {
        &[&["chrome-linux", "chrome"], &["chrome-linux64", "chrome"]]
    };
    candidates.iter().any(|segments| {
        let mut path = revision_dir.to_path_buf();
        for segment in *segments {
            path.push(segment);
        }
        path.is_file()
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
            let candidate = directory.join(format!("{tool}{}", extension.to_lowercase()));
            if candidate.is_file() {
                return Some(candidate);
            }
            let candidate = directory.join(format!("{tool}{extension}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Atomic materializer and drift detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AppliedNativeFile {
    pub path: String,
    pub content_digest: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessApplyResult {
    pub manifest_digest: String,
    pub changed: bool,
    pub files: Vec<AppliedNativeFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessDriftEntry {
    pub path: String,
    pub expected_digest: String,
    pub current_digest: Option<String>,
}

pub fn detect_drift(root: &Path, applied: &BTreeMap<String, String>) -> Vec<HarnessDriftEntry> {
    let mut drift = Vec::new();
    for (relative, expected) in applied {
        let current = fs::read(root.join(relative))
            .ok()
            .map(|bytes| crate::harness_manifest::content_digest(&bytes));
        if current.as_deref() != Some(expected.as_str()) {
            drift.push(HarnessDriftEntry {
                path: relative.clone(),
                expected_digest: expected.clone(),
                current_digest: current,
            });
        }
    }
    drift
}

pub fn apply_harness_configuration(
    root: &Path,
    command: &str,
    expected_digest: &str,
) -> Result<HarnessApplyResult, String> {
    apply_with_failure(root, command, expected_digest, None)
}

struct PendingWrite {
    existed: bool,
    target: PathBuf,
    stage: PathBuf,
    backup: PathBuf,
}

fn apply_with_failure(
    root: &Path,
    command: &str,
    expected_digest: &str,
    fail_after: Option<usize>,
) -> Result<HarnessApplyResult, String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let _lock = ApplyLock::acquire(root.join(".lmbrain/.harness-config.lock"))?;
    let plan = plan_harness_configuration(&root, command)?;
    if plan.manifest_digest != expected_digest {
        return Err("manifest changed since approval; apply refused".into());
    }
    if plan.has_conflicts {
        return Err("native configuration conflicts must be resolved before apply".into());
    }
    let manifest = load_harness_manifest(&root).map_err(|error| error.to_string())?;
    let mut actions = BTreeMap::new();
    for host in &plan.hosts {
        for file in &host.native_files {
            actions.insert(file.path.clone(), file.action.clone());
        }
    }
    let mut pending = Vec::new();
    for (host, config) in manifest.hosts {
        if !config.enabled {
            continue;
        }
        let (relative, content) = render(&root, host, &config, command)?;
        if actions.get(&relative) == Some(&PreviewAction::Preserved) {
            continue;
        }
        let target = root.join(&relative);
        validate_target(&root, &target)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let nonce = format!(
            "{}-{}",
            std::process::id(),
            chrono::Utc::now().format("%Y%m%dT%H%M%S%f")
        );
        let stage = target.with_extension(format!("lmbrain-{nonce}.tmp"));
        let backup = target.with_extension(format!("lmbrain-{nonce}.bak"));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)
            .map_err(|error| error.to_string())?;
        file.write_all(content.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|error| error.to_string())?;
        pending.push(PendingWrite {
            existed: target.exists(),
            target,
            stage,
            backup,
        });
    }
    let mut committed = 0usize;
    for write in &pending {
        let result = (|| -> Result<(), String> {
            if write.existed {
                fs::rename(&write.target, &write.backup).map_err(|error| error.to_string())?;
            }
            fs::rename(&write.stage, &write.target).map_err(|error| error.to_string())?;
            committed += 1;
            if fail_after == Some(committed) {
                return Err("injected apply failure".into());
            }
            Ok(())
        })();
        if let Err(error) = result {
            rollback(&pending, committed);
            return Err(error);
        }
    }
    for write in &pending {
        if write.backup.exists() {
            fs::remove_file(&write.backup).map_err(|error| error.to_string())?;
        }
    }
    let files = plan
        .hosts
        .iter()
        .flat_map(|host| &host.native_files)
        .map(|preview| {
            let bytes = fs::read(root.join(&preview.path)).map_err(|error| error.to_string())?;
            Ok(AppliedNativeFile {
                path: preview.path.clone(),
                content_digest: crate::harness_manifest::content_digest(&bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(HarnessApplyResult {
        manifest_digest: plan.manifest_digest,
        changed: !pending.is_empty(),
        files,
    })
}

/// One end-to-end governed apply for the MCP surface: verifies the approval is
/// current for the stated digest, materializes, records the applied hashes for
/// drift detection, and audits the action with its actor.
pub fn apply_approved_harness_configuration(
    root: &Path,
    store_path: &Path,
    command: &str,
    expected_digest: &str,
    actor: &str,
) -> Result<HarnessApplyResult, String> {
    let status = harness_approval_status(root, store_path)?;
    if status.state != HarnessApprovalState::Approved {
        return Err(format!(
            "manifest is not approved for this workspace (state: {:?}); approve the current digest first",
            status.state
        ));
    }
    if status.approved_digest.as_deref() != Some(expected_digest) {
        return Err("expected_digest does not match the approved digest".into());
    }
    let result = apply_harness_configuration(root, command, expected_digest)?;
    let files = result
        .files
        .iter()
        .map(|file| (file.path.clone(), file.content_digest.clone()))
        .collect::<Vec<_>>();
    record_application(root, store_path, &result.manifest_digest, &files)?;
    audit_environment_action(root, "harness_config_apply", &result.manifest_digest, actor)?;
    Ok(result)
}

fn render(
    root: &Path,
    host: HarnessHost,
    config: &HostConfiguration,
    command: &str,
) -> Result<(String, String), String> {
    let root_text = root.to_string_lossy();
    let (relative, result) = match host {
        HarnessHost::ClaudeCode => (
            ".mcp.json",
            build_claude_mcp_config(
                read_optional(root.join(".mcp.json")).as_deref(),
                command,
                &root_text,
                BrowserEntry::Managed(config.browser_mcp.as_ref()),
            ),
        ),
        HarnessHost::Codex => (
            ".codex/config.toml",
            build_codex_project_config(
                read_optional(root.join(".codex/config.toml")).as_deref(),
                command,
                &root_text,
            ),
        ),
        HarnessHost::Pi => (
            ".pi/mcp.json",
            build_pi_mcp_config(
                read_optional(root.join(".pi/mcp.json")).as_deref(),
                command,
                &root_text,
            ),
        ),
        HarnessHost::OpenCode => (
            "opencode.json",
            build_opencode_config(
                read_optional(root.join("opencode.json")).as_deref(),
                command,
                &root_text,
            ),
        ),
    };
    result.map(|content| (relative.into(), content))
}

fn validate_target(root: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        let metadata = fs::symlink_metadata(target).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "native target is not a regular file: {}",
                target.display()
            ));
        }
        let canonical = target.canonicalize().map_err(|error| error.to_string())?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "native target escapes the workspace: {}",
                target.display()
            ));
        }
    }
    Ok(())
}

fn rollback(pending: &[PendingWrite], committed: usize) {
    for write in pending.iter().take(committed) {
        if write.existed {
            let _ = fs::rename(&write.backup, &write.target);
        } else {
            let _ = fs::remove_file(&write.target);
        }
    }
    for write in pending {
        let _ = fs::remove_file(&write.stage);
    }
}

struct ApplyLock(PathBuf);

impl ApplyLock {
    fn acquire(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(Self(path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err("another harness configuration operation is in progress".into())
            }
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Drop for ApplyLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness_manifest::{set_harness_manifest, HarnessManifest};

    fn workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        dir
    }

    fn manifest(json: Value) -> HarnessManifest {
        serde_json::from_value(json).unwrap()
    }

    fn store(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("store/harness-approvals.json")
    }

    #[test]
    fn lead_lifecycle_approve_apply_and_drift_end_to_end() {
        let dir = workspace();
        let store_path = store(&dir);
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true}}})),
        )
        .unwrap();

        let status = harness_approval_status(dir.path(), &store_path).unwrap();
        assert_eq!(status.state, HarnessApprovalState::ApprovalRequired);
        let digest = status.manifest_digest.unwrap();

        // Stale digests are refused; the current digest approves and audits.
        assert!(approve_harness_manifest(dir.path(), &store_path, "0", "project-lead").is_err());
        let approved =
            approve_harness_manifest(dir.path(), &store_path, &digest, "project-lead").unwrap();
        assert_eq!(approved.state, HarnessApprovalState::Approved);
        assert_eq!(approved.approved_by.as_deref(), Some("project-lead"));

        let applied = apply_approved_harness_configuration(
            dir.path(),
            &store_path,
            "lmbrain-mcp",
            &digest,
            "project-lead",
        )
        .unwrap();
        assert!(applied.changed);
        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["mcpServers"]["lmbrain"]["command"], "lmbrain-mcp");

        // Drift is empty right after apply, and reports a mutated file.
        let recorded = applied_files(dir.path(), &store_path).unwrap();
        assert!(detect_drift(dir.path(), &recorded).is_empty());
        fs::write(dir.path().join(".mcp.json"), "{}").unwrap();
        let drift = detect_drift(dir.path(), &recorded);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].path, ".mcp.json");

        // The audit trail names every Lead action.
        let audit = fs::read_to_string(dir.path().join(".lmbrain/HARNESSES.audit.jsonl")).unwrap();
        assert!(audit.contains("harness_manifest_approve"));
        assert!(audit.contains("harness_config_apply"));
        assert!(audit.contains("project-lead"));
    }

    #[test]
    fn apply_refuses_unapproved_and_mismatched_digests() {
        let dir = workspace();
        let store_path = store(&dir);
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true}}})),
        )
        .unwrap();
        let digest = harness_approval_status(dir.path(), &store_path)
            .unwrap()
            .manifest_digest
            .unwrap();

        // Not approved yet.
        assert!(apply_approved_harness_configuration(
            dir.path(),
            &store_path,
            "lmbrain-mcp",
            &digest,
            "project-lead",
        )
        .is_err());

        approve_harness_manifest(dir.path(), &store_path, &digest, "project-lead").unwrap();

        // Approved, but the caller states a different digest.
        assert!(apply_approved_harness_configuration(
            dir.path(),
            &store_path,
            "lmbrain-mcp",
            "not-the-digest",
            "project-lead",
        )
        .is_err());

        // The manifest changes after approval: state goes stale and apply
        // refuses even the previously approved digest.
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"claude-code":{"enabled":false}}})),
        )
        .unwrap();
        assert_eq!(
            harness_approval_status(dir.path(), &store_path)
                .unwrap()
                .state,
            HarnessApprovalState::Stale
        );
        assert!(apply_approved_harness_configuration(
            dir.path(),
            &store_path,
            "lmbrain-mcp",
            &digest,
            "project-lead",
        )
        .is_err());
    }

    #[test]
    fn preview_is_deterministic_and_preserves_unrelated_json() {
        let dir = workspace();
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true}}})),
        )
        .unwrap();
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
    }

    #[test]
    fn incompatible_owned_parent_is_a_conflict_and_no_file_is_changed() {
        let dir = workspace();
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"pi":{"enabled":true}}})),
        )
        .unwrap();
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
        let dir = workspace();
        set_harness_manifest(
            dir.path(),
            &manifest(json!({
                "schema_version": 1,
                "hosts": {"claude-code": {"enabled": true, "browser_mcp": {"provider": "playwright", "mode": "isolated"}}}
            })),
        )
        .unwrap();

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

        let package = dir.path().join("node_modules/@playwright/mcp");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@playwright/mcp","version":"0.0.41"}"#,
        )
        .unwrap();
        fs::write(package.join("cli.js"), "// pinned cli").unwrap();
        let revision_dir = dir
            .path()
            .join("node_modules/playwright-core/.local-browsers/chromium-1181");
        fs::create_dir_all(&revision_dir).unwrap();

        // A bare revision directory without the launchable executable is the
        // KIT-NOTE-013 false positive: it must NOT count as a runtime.
        let plan =
            plan_harness_configuration_with_browser_path(dir.path(), "lmbrain-mcp", Some("0"))
                .unwrap();
        let readiness = plan.hosts[0].browser_mcp.as_ref().unwrap();
        assert!(readiness.package_available);
        assert!(!readiness.browser_runtime_found);
        assert_eq!(readiness.state, CapabilityState::Failed);
        assert!(readiness.detail.contains("install-browser"));

        let executable = revision_dir.join(if cfg!(windows) {
            "chrome-win64/chrome.exe"
        } else if cfg!(target_os = "macos") {
            "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
        } else {
            "chrome-linux/chrome"
        });
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, "").unwrap();
        let plan =
            plan_harness_configuration_with_browser_path(dir.path(), "lmbrain-mcp", Some("0"))
                .unwrap();
        let host = &plan.hosts[0];
        let readiness = host.browser_mcp.as_ref().unwrap();
        assert!(readiness.package_available);
        assert_eq!(readiness.package_version.as_deref(), Some("0.0.41"));
        assert!(readiness.browser_runtime_found);
        assert_eq!(readiness.state, CapabilityState::PrerequisiteReady);
        assert!(host.ready);
    }

    #[test]
    fn browser_runtime_probe_requires_the_pinned_revision_when_known() {
        let dir = workspace();
        let browsers = dir.path().join("node_modules/playwright-core");
        fs::create_dir_all(&browsers).unwrap();
        fs::write(
            browsers.join("browsers.json"),
            r#"{"browsers":[{"name":"chromium","revision":"1237"}]}"#,
        )
        .unwrap();
        let local = browsers.join(".local-browsers");
        let stale = local.join("chromium-1181");
        let executable_tail = if cfg!(windows) {
            "chrome-win64/chrome.exe"
        } else if cfg!(target_os = "macos") {
            "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
        } else {
            "chrome-linux/chrome"
        };
        let stale_executable = stale.join(executable_tail);
        fs::create_dir_all(stale_executable.parent().unwrap()).unwrap();
        fs::write(&stale_executable, "").unwrap();

        // A different revision, even fully installed, is not the runtime the
        // provisioned provider launches.
        assert!(!playwright_chromium_found_with_path(dir.path(), Some("0")));

        let pinned_executable = local.join("chromium-1237").join(executable_tail);
        fs::create_dir_all(pinned_executable.parent().unwrap()).unwrap();
        fs::write(&pinned_executable, "").unwrap();
        let found = playwright_chromium_found_with_path(dir.path(), Some("0"));
        assert!(found);
    }

    #[test]
    fn apply_is_idempotent_and_injected_failure_rolls_back() {
        let dir = workspace();
        set_harness_manifest(
            dir.path(),
            &manifest(
                json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true},"pi":{"enabled":true}}}),
            ),
        )
        .unwrap();
        let claude = r#"{"keep":"claude"}"#;
        let pi = r#"{"keep":"pi"}"#;
        fs::create_dir(dir.path().join(".pi")).unwrap();
        fs::write(dir.path().join(".mcp.json"), claude).unwrap();
        fs::write(dir.path().join(".pi/mcp.json"), pi).unwrap();
        let digest =
            canonical_manifest_digest(&load_harness_manifest(dir.path()).unwrap()).unwrap();

        // A failure mid-batch restores every original file.
        assert!(apply_with_failure(dir.path(), "lmbrain-mcp", &digest, Some(2)).is_err());
        assert_eq!(
            fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            claude
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(".pi/mcp.json")).unwrap(),
            pi
        );

        let first = apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        assert!(first.changed);
        let second = apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        assert!(!second.changed);
        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["keep"], "claude");
    }

    #[test]
    fn browser_capability_applies_and_is_removed_when_dropped_from_the_manifest() {
        let dir = workspace();
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{"keep":true,"mcpServers":{"other":{"command":"x"}}}"#,
        )
        .unwrap();
        let with_browser = manifest(json!({
            "schema_version": 1,
            "hosts": {"claude-code": {"enabled": true, "browser_mcp": {"provider": "playwright", "mode": "isolated"}}}
        }));
        set_harness_manifest(dir.path(), &with_browser).unwrap();
        let digest = canonical_manifest_digest(&with_browser).unwrap();
        apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["mcpServers"]["lmbrain-browser"]["command"], "node");
        assert_eq!(value["keep"], true);
        assert_eq!(value["mcpServers"]["other"]["command"], "x");

        let without_browser = manifest(json!({
            "schema_version": 1,
            "hosts": {"claude-code": {"enabled": true}}
        }));
        set_harness_manifest(dir.path(), &without_browser).unwrap();
        let digest = canonical_manifest_digest(&without_browser).unwrap();
        apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(value["mcpServers"]["lmbrain-browser"].is_null());
        assert_eq!(value["mcpServers"]["other"]["command"], "x");
        assert_eq!(value["keep"], true);
    }

    #[test]
    fn revoke_is_idempotent_and_corrupt_store_is_quarantined() {
        let dir = workspace();
        let store_path = store(&dir);
        set_harness_manifest(
            dir.path(),
            &manifest(json!({"schema_version":1,"hosts":{"codex":{"enabled":false}}})),
        )
        .unwrap();
        let digest = harness_approval_status(dir.path(), &store_path)
            .unwrap()
            .manifest_digest
            .unwrap();
        approve_harness_manifest(dir.path(), &store_path, &digest, "project-lead").unwrap();
        assert_eq!(
            revoke_harness_approval(dir.path(), &store_path, "project-lead")
                .unwrap()
                .state,
            HarnessApprovalState::ApprovalRequired
        );
        assert_eq!(
            revoke_harness_approval(dir.path(), &store_path, "project-lead")
                .unwrap()
                .state,
            HarnessApprovalState::ApprovalRequired
        );

        fs::create_dir_all(store_path.parent().unwrap()).unwrap();
        fs::write(&store_path, "not json").unwrap();
        let status = harness_approval_status(dir.path(), &store_path).unwrap();
        assert_eq!(status.state, HarnessApprovalState::ApprovalRequired);
        assert!(fs::read_dir(store_path.parent().unwrap())
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("harness-approvals.corrupt-")));
    }
}
