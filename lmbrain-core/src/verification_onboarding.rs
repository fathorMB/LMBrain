use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{
    canonical_verification_manifest_digest, frontmatter::atomic_write, load_verification_manifest,
    mutation_lock::WorkspaceLock, parse_verification_manifest, path::PathGuard,
    validate_verification_manifest, workspace_identity, VerificationApproval, VerificationError,
    VerificationGate, VerificationManifest, VERIFICATION_MANIFEST_PATH,
};

const MAX_DISCOVERY_FILES: usize = 32;
const MAX_DISCOVERY_BYTES: u64 = 256 * 1024;
const DEFAULT_TIMEOUT_SECONDS: u64 = 900;
const DEFAULT_OUTPUT_BYTES: usize = 128 * 1024;
const PREVIOUS_MANIFEST_PATH: &str = ".lmbrain/verification.toml.previous";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationManifestState {
    Absent,
    Invalid,
    Unsafe,
    Unapproved,
    Approved,
    Stale,
    ApprovalInvalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationManifestStatus {
    pub schema_version: String,
    pub state: VerificationManifestState,
    pub manifest_digest: Option<String>,
    pub approved_digest: Option<String>,
    pub approved_at: Option<String>,
    pub workspace_fingerprint: String,
    pub gate_count: usize,
    pub issues: Vec<String>,
    pub next_action: String,
    pub can_rollback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationGateCandidate {
    pub gate: VerificationGate,
    pub provenance: String,
    pub confidence: String,
    pub selected: bool,
    pub environment_policy: String,
    pub mutation_policy: String,
    pub security_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationManifestPreview {
    pub schema_version: String,
    pub status: VerificationManifestStatus,
    pub candidates: Vec<VerificationGateCandidate>,
    pub conflicts: Vec<String>,
    pub guidance: Vec<String>,
    pub proposed_manifest: VerificationManifest,
    pub proposed_toml: String,
    pub proposed_digest: String,
    pub current_toml: Option<String>,
    pub diff: String,
    pub discovered_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationManifestWriteResult {
    pub path: String,
    pub digest: String,
    pub previous_digest: Option<String>,
    pub approval_required: bool,
    pub rollback_available: bool,
}

#[derive(Debug, Error)]
pub enum VerificationOnboardingError {
    #[error("invalid verification onboarding request: {0}")]
    Invalid(String),
    #[error(transparent)]
    Verification(#[from] VerificationError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn default_verification_approval_path(root: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("LMBRAIN_VERIFICATION_APPROVAL_STORE") {
        return PathBuf::from(path);
    }
    let base = std::env::var_os(if cfg!(windows) {
        "LOCALAPPDATA"
    } else {
        "XDG_DATA_HOME"
    })
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    .unwrap_or_else(std::env::temp_dir);
    let identity = workspace_identity(root)
        .map(|identity| identity.fingerprint)
        .unwrap_or_else(|_| "unknown-workspace".into());
    base.join("lmbrain/verification-approvals")
        .join(format!("{identity}.json"))
}

pub fn verification_manifest_status(
    root: &Path,
    approval_store: &Path,
) -> Result<VerificationManifestStatus, VerificationOnboardingError> {
    let identity = workspace_identity(root)
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    let rollback_available = root.join(PREVIOUS_MANIFEST_PATH).is_file();
    let manifest = match load_verification_manifest(root) {
        Ok(manifest) => manifest,
        Err(VerificationError::MissingManifest(_)) => {
            return Ok(status(
                VerificationManifestState::Absent,
                &identity.fingerprint,
                None,
                None,
                Vec::new(),
                0,
                "Run verification_manifest_init to inspect deterministic suggestions; nothing will execute.",
                rollback_available,
            ));
        }
        Err(VerificationError::InvalidManifest(issue)) => {
            return Ok(status(
                VerificationManifestState::Invalid,
                &identity.fingerprint,
                None,
                read_approval(approval_store).ok().flatten(),
                vec![issue],
                0,
                "Validate a corrected complete manifest, then write it with verification_manifest_set.",
                rollback_available,
            ));
        }
        Err(VerificationError::UnsafePath(issue)) => {
            return Ok(status(
                VerificationManifestState::Unsafe,
                &identity.fingerprint,
                None,
                read_approval(approval_store).ok().flatten(),
                vec![issue],
                0,
                "Replace the unsafe manifest path with a regular workspace-confined file.",
                rollback_available,
            ));
        }
        Err(error) => return Err(error.into()),
    };
    let digest = canonical_verification_manifest_digest(&manifest)?;
    let approval = match read_approval(approval_store) {
        Ok(approval) => approval,
        Err(error) => {
            return Ok(status(
                VerificationManifestState::ApprovalInvalid,
                &identity.fingerprint,
                Some(digest),
                None,
                vec![error.to_string()],
                manifest.gates.len(),
                "Remove or repair the machine-local approval record, then approve the current digest explicitly.",
                rollback_available,
            ));
        }
    };
    let state = match approval.as_ref() {
        None => VerificationManifestState::Unapproved,
        Some(approval)
            if approval.manifest_digest == digest
                && approval.workspace_fingerprint == identity.fingerprint =>
        {
            VerificationManifestState::Approved
        }
        Some(_) => VerificationManifestState::Stale,
    };
    let next_action = match state {
        VerificationManifestState::Approved => {
            "The manifest is approved; spec_verify may execute only referenced named gates."
        }
        VerificationManifestState::Stale => {
            "Review the changed manifest and explicitly approve its current digest before verification."
        }
        VerificationManifestState::Unapproved => {
            "Review the manifest and explicitly approve its digest before verification."
        }
        _ => unreachable!("handled above"),
    };
    Ok(status(
        state,
        &identity.fingerprint,
        Some(digest),
        approval,
        Vec::new(),
        manifest.gates.len(),
        next_action,
        rollback_available,
    ))
}

pub fn discover_verification_manifest(
    root: &Path,
    approval_store: &Path,
) -> Result<VerificationManifestPreview, VerificationOnboardingError> {
    let guard = PathGuard::new(root)
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    let mut candidates = Vec::new();
    let mut guidance = Vec::new();
    discover_cargo(guard.root(), &mut candidates, &mut guidance)?;
    discover_node(guard.root(), &mut candidates, &mut guidance)?;
    discover_tasks(guard.root(), &mut candidates, &mut guidance)?;
    discover_ci(guard.root(), &mut candidates, &mut guidance)?;
    candidates.sort_by(|left, right| {
        left.gate
            .id
            .cmp(&right.gate.id)
            .then_with(|| left.provenance.cmp(&right.provenance))
    });
    let mut conflicts = Vec::new();
    let mut seen: BTreeMap<String, VerificationGate> = BTreeMap::new();
    for candidate in &mut candidates {
        if let Some(existing) = seen.get(&candidate.gate.id) {
            if existing != &candidate.gate {
                conflicts.push(format!(
                    "gate '{}' has conflicting discoveries; review provenance before selection",
                    candidate.gate.id
                ));
                candidate.selected = false;
            }
        } else {
            seen.insert(candidate.gate.id.clone(), candidate.gate.clone());
        }
    }
    let selected = candidates
        .iter()
        .filter(|candidate| candidate.selected)
        .map(|candidate| candidate.gate.clone())
        .collect::<Vec<_>>();
    let proposed_manifest = VerificationManifest {
        schema_version: 1,
        gates: if selected.is_empty() {
            guidance.push(
                "No supported repository-native gate was found; a safe Git repository check is proposed as a starting point, not as proof of product correctness."
                    .into(),
            );
            vec![minimal_safe_gate()]
        } else {
            selected
        },
    };
    let issues = validate_verification_manifest(&proposed_manifest);
    if !issues.is_empty() {
        return Err(VerificationOnboardingError::Invalid(issues.join("; ")));
    }
    let proposed_toml = render_manifest(&proposed_manifest)?;
    let proposed_digest = canonical_verification_manifest_digest(&proposed_manifest)?;
    let current_toml = fs::read_to_string(guard.root().join(VERIFICATION_MANIFEST_PATH)).ok();
    let diff = line_diff(current_toml.as_deref(), &proposed_toml);
    Ok(VerificationManifestPreview {
        schema_version: "1".into(),
        status: verification_manifest_status(guard.root(), approval_store)?,
        candidates,
        conflicts,
        guidance,
        proposed_manifest,
        proposed_toml,
        proposed_digest,
        current_toml,
        diff,
        discovered_only: true,
    })
}

pub fn validate_verification_manifest_source(
    source: &str,
) -> Result<VerificationManifestPreviewValidation, VerificationOnboardingError> {
    let manifest = parse_verification_manifest(source)?;
    Ok(VerificationManifestPreviewValidation {
        valid: true,
        digest: canonical_verification_manifest_digest(&manifest)?,
        manifest,
        canonical_toml: render_manifest(&parse_verification_manifest(source)?)?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationManifestPreviewValidation {
    pub valid: bool,
    pub digest: String,
    pub manifest: VerificationManifest,
    pub canonical_toml: String,
}

pub fn set_verification_manifest(
    root: &Path,
    manifest: &VerificationManifest,
    expected_current_digest: Option<&str>,
) -> Result<VerificationManifestWriteResult, VerificationOnboardingError> {
    let issues = validate_verification_manifest(manifest);
    if !issues.is_empty() {
        return Err(VerificationOnboardingError::Invalid(issues.join("; ")));
    }
    let guard = PathGuard::new(root)
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let path = guard.root().join(VERIFICATION_MANIFEST_PATH);
    let current = current_manifest(&path)?;
    let current_digest = current
        .as_ref()
        .map(canonical_verification_manifest_digest)
        .transpose()?;
    if current_digest.as_deref() != expected_current_digest {
        return Err(VerificationOnboardingError::Invalid(format!(
            "manifest changed since preview: expected {:?}, current {:?}",
            expected_current_digest, current_digest
        )));
    }
    if path.exists() {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(VerificationOnboardingError::Invalid(
                "verification manifest must be a regular file".into(),
            ));
        }
        atomic_write(
            &guard.root().join(PREVIOUS_MANIFEST_PATH),
            &fs::read_to_string(&path)?,
        )
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    }
    let rendered = render_manifest(manifest)?;
    atomic_write(&path, &rendered)
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    Ok(VerificationManifestWriteResult {
        path: VERIFICATION_MANIFEST_PATH.into(),
        digest: canonical_verification_manifest_digest(manifest)?,
        previous_digest: current_digest,
        approval_required: true,
        rollback_available: guard.root().join(PREVIOUS_MANIFEST_PATH).is_file(),
    })
}

pub fn rollback_verification_manifest(
    root: &Path,
    expected_current_digest: &str,
) -> Result<VerificationManifestWriteResult, VerificationOnboardingError> {
    let previous_path = root.join(PREVIOUS_MANIFEST_PATH);
    if !previous_path.is_file() {
        return Err(VerificationOnboardingError::Invalid(
            "no previous verification manifest is available".into(),
        ));
    }
    let source = fs::read_to_string(&previous_path)?;
    let previous = parse_verification_manifest(&source)?;
    set_verification_manifest(root, &previous, Some(expected_current_digest))
}

#[allow(clippy::too_many_arguments)]
fn status(
    state: VerificationManifestState,
    workspace_fingerprint: &str,
    manifest_digest: Option<String>,
    approval: Option<VerificationApproval>,
    issues: Vec<String>,
    gate_count: usize,
    next_action: &str,
    can_rollback: bool,
) -> VerificationManifestStatus {
    VerificationManifestStatus {
        schema_version: "1".into(),
        state,
        manifest_digest,
        approved_digest: approval
            .as_ref()
            .map(|approval| approval.manifest_digest.clone()),
        approved_at: approval.map(|approval| approval.approved_at),
        workspace_fingerprint: workspace_fingerprint.into(),
        gate_count,
        issues,
        next_action: next_action.into(),
        can_rollback,
    }
}

fn read_approval(
    approval_store: &Path,
) -> Result<Option<VerificationApproval>, VerificationOnboardingError> {
    if !approval_store.exists() {
        return Ok(None);
    }
    let source = fs::read_to_string(approval_store)?;
    serde_json::from_str(&source).map(Some).map_err(|error| {
        VerificationOnboardingError::Invalid(format!(
            "machine-local approval record is malformed: {error}"
        ))
    })
}

fn current_manifest(
    path: &Path,
) -> Result<Option<VerificationManifest>, VerificationOnboardingError> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(parse_verification_manifest(&fs::read_to_string(
        path,
    )?)?))
}

fn render_manifest(manifest: &VerificationManifest) -> Result<String, VerificationOnboardingError> {
    let mut rendered = toml::to_string_pretty(manifest)
        .map_err(|error| VerificationOnboardingError::Invalid(error.to_string()))?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

fn discover_cargo(
    root: &Path,
    candidates: &mut Vec<VerificationGateCandidate>,
    guidance: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let paths = discover_named_files(root, "Cargo.toml");
    for path in paths {
        let source = bounded_read(&path)?;
        let cwd = relative_directory(root, &path);
        let workspace = source.contains("[workspace]");
        candidates.push(candidate(
            gate(
                &unique_gate_id("cargo-test", &cwd),
                "cargo",
                if workspace {
                    vec!["test".into(), "--workspace".into()]
                } else {
                    vec!["test".into()]
                },
                &cwd,
                vec![join_relative(&cwd, "target")],
            ),
            format!("cargo:{}", relative_file(root, &path)),
            "high",
        ));
        if candidates.len() >= MAX_DISCOVERY_FILES {
            guidance.push("Cargo discovery was truncated at the bounded file limit.".into());
            break;
        }
    }
    Ok(())
}

fn discover_node(
    root: &Path,
    candidates: &mut Vec<VerificationGateCandidate>,
    guidance: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let paths = discover_named_files(root, "package.json");
    for path in paths {
        let source = bounded_read(&path)?;
        let Ok(value) = serde_json::from_str::<Value>(&source) else {
            guidance.push(format!(
                "Skipped malformed {} during discovery.",
                relative_file(root, &path)
            ));
            continue;
        };
        let Some(scripts) = value.get("scripts").and_then(Value::as_object) else {
            continue;
        };
        let cwd = relative_directory(root, &path);
        let (program, prefix) = node_runner(root, &path);
        for script in ["test", "lint", "typecheck", "check", "build"] {
            if !scripts.get(script).is_some_and(Value::is_string) {
                continue;
            }
            let mut args = prefix.clone();
            args.push(script.into());
            candidates.push(candidate(
                gate(
                    &unique_gate_id(&format!("node-{script}"), &cwd),
                    program,
                    args,
                    &cwd,
                    if script == "build" {
                        vec![join_relative(&cwd, "dist")]
                    } else {
                        Vec::new()
                    },
                ),
                format!("package-script:{}#{script}", relative_file(root, &path)),
                "high",
            ));
        }
        if candidates.len() >= MAX_DISCOVERY_FILES {
            guidance.push("Node discovery was truncated at the bounded file limit.".into());
            break;
        }
    }
    Ok(())
}

fn discover_tasks(
    root: &Path,
    candidates: &mut Vec<VerificationGateCandidate>,
    guidance: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let path = root.join(".vscode/tasks.json");
    if !path.is_file() {
        return Ok(());
    }
    let source = bounded_read(&path)?;
    let Ok(value) = serde_json::from_str::<Value>(&source) else {
        guidance.push("Skipped malformed .vscode/tasks.json during discovery.".into());
        return Ok(());
    };
    for (index, task) in value
        .get("tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        if task.get("type").and_then(Value::as_str) != Some("process") {
            guidance.push(format!(
                "Skipped VS Code task {} because only type=process avoids implicit shell execution.",
                index + 1
            ));
            continue;
        }
        let Some(program) = task.get("command").and_then(Value::as_str) else {
            continue;
        };
        let args = task
            .get("args")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let label = task.get("label").and_then(Value::as_str).unwrap_or("task");
        let proposed = gate(
            &unique_gate_id(&format!("task-{}", slug(label)), "."),
            program,
            args,
            ".",
            Vec::new(),
        );
        if validate_verification_manifest(&VerificationManifest {
            schema_version: 1,
            gates: vec![proposed.clone()],
        })
        .is_empty()
        {
            candidates.push(candidate(
                proposed,
                format!("vscode-task:.vscode/tasks.json#{label}"),
                "medium",
            ));
        } else {
            guidance.push(format!(
                "Skipped unsafe or non-portable VS Code process task '{label}'."
            ));
        }
    }
    Ok(())
}

fn discover_ci(
    root: &Path,
    _candidates: &mut Vec<VerificationGateCandidate>,
    guidance: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let workflows = root.join(".github/workflows");
    if workflows.is_dir()
        && fs::read_dir(workflows)?
            .flatten()
            .any(|entry| entry.path().is_file())
    {
        guidance.push(
            "GitHub Actions workflows were detected. Their shell `run` blocks are shown as provenance guidance only; CI trust and shell snippets are never imported automatically."
                .into(),
        );
    }
    Ok(())
}

fn minimal_safe_gate() -> VerificationGate {
    let mut gate = gate(
        "repository-clean",
        "git",
        vec!["status".into(), "--porcelain".into()],
        ".",
        Vec::new(),
    );
    gate.result_matcher = Some("^$".into());
    gate
}

fn gate(
    id: &str,
    program: &str,
    args: Vec<String>,
    cwd: &str,
    fingerprint_exclude: Vec<String>,
) -> VerificationGate {
    VerificationGate {
        id: id.into(),
        title: Some(id.replace('-', " ")),
        program: program.into(),
        args,
        cwd: cwd.into(),
        timeout_seconds: Some(DEFAULT_TIMEOUT_SECONDS),
        output_limit_bytes: Some(DEFAULT_OUTPUT_BYTES),
        expected_exit_code: Some(0),
        result_matcher: None,
        environment: BTreeMap::new(),
        fingerprint_exclude,
    }
}

fn candidate(
    gate: VerificationGate,
    provenance: String,
    confidence: &str,
) -> VerificationGateCandidate {
    VerificationGateCandidate {
        gate,
        provenance,
        confidence: confidence.into(),
        selected: true,
        environment_policy: "No discovered environment variables are imported.".into(),
        mutation_policy:
            "Only declared fingerprint_exclude build outputs may change without invalidation."
                .into(),
        security_notes: vec![
            "Discovery does not approve or execute this command.".into(),
            "Program and argv remain discrete; no shell interpolation is used.".into(),
        ],
    }
}

fn discover_named_files(root: &Path, filename: &str) -> Vec<PathBuf> {
    fn scan(root: &Path, directory: &Path, filename: &str, output: &mut Vec<PathBuf>) {
        if output.len() >= MAX_DISCOVERY_FILES {
            return;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                if matches!(
                    name.to_string_lossy().as_ref(),
                    ".git" | ".lmbrain" | "node_modules" | "target" | "dist" | "build"
                ) {
                    continue;
                }
                scan(root, &path, filename, output);
            } else if entry.file_name() == filename {
                output.push(path);
            }
            if output.len() >= MAX_DISCOVERY_FILES {
                break;
            }
        }
        let _ = root;
    }
    let mut output = Vec::new();
    scan(root, root, filename, &mut output);
    output.sort();
    output
}

fn bounded_read(path: &Path) -> Result<String, std::io::Error> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_DISCOVERY_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "discovery source exceeds the bounded read limit",
        ));
    }
    fs::read_to_string(path)
}

fn node_runner(root: &Path, package_json: &Path) -> (&'static str, Vec<String>) {
    let directory = package_json.parent().unwrap_or(root);
    if directory.join("pnpm-lock.yaml").is_file() || root.join("pnpm-lock.yaml").is_file() {
        ("pnpm", vec!["run".into()])
    } else if directory.join("yarn.lock").is_file() || root.join("yarn.lock").is_file() {
        ("yarn", Vec::new())
    } else {
        ("npm", vec!["run".into()])
    }
}

fn relative_directory(root: &Path, file: &Path) -> String {
    let directory = file.parent().unwrap_or(root);
    let relative = directory.strip_prefix(root).unwrap_or(directory);
    if relative.as_os_str().is_empty() {
        ".".into()
    } else {
        relative.to_string_lossy().replace('\\', "/")
    }
}

fn relative_file(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn join_relative(cwd: &str, child: &str) -> String {
    if cwd == "." {
        child.into()
    } else {
        format!("{cwd}/{child}")
    }
}

fn unique_gate_id(prefix: &str, cwd: &str) -> String {
    if cwd == "." {
        prefix.into()
    } else {
        format!("{prefix}-{}", slug(cwd))
    }
}

fn slug(value: &str) -> String {
    let mut output = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while output.contains("--") {
        output = output.replace("--", "-");
    }
    output.trim_matches('-').to_string()
}

fn line_diff(current: Option<&str>, proposed: &str) -> String {
    let current = current.unwrap_or_default().lines().collect::<Vec<_>>();
    let proposed = proposed.lines().collect::<Vec<_>>();
    let mut output = String::from("--- current\n+++ proposed\n");
    let maximum = current.len().max(proposed.len());
    for index in 0..maximum {
        match (current.get(index), proposed.get(index)) {
            (Some(left), Some(right)) if left == right => {
                output.push_str(&format!(" {left}\n"));
            }
            (Some(left), Some(right)) => {
                output.push_str(&format!("-{left}\n+{right}\n"));
            }
            (Some(left), None) => output.push_str(&format!("-{left}\n")),
            (None, Some(right)) => output.push_str(&format!("+{right}\n")),
            (None, None) => {}
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn approval_path(root: &Path) -> PathBuf {
        root.join("approval.json")
    }

    #[test]
    fn empty_repository_has_safe_preview_and_typed_absent_status() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        let preview =
            discover_verification_manifest(directory.path(), &approval_path(directory.path()))
                .unwrap();
        assert_eq!(preview.status.state, VerificationManifestState::Absent);
        assert_eq!(preview.proposed_manifest.gates[0].program, "git");
        assert_eq!(
            preview.proposed_manifest.gates[0].args,
            vec!["status", "--porcelain"]
        );
        assert!(preview.discovered_only);
        assert!(preview.proposed_toml.contains("timeout_seconds = 900"));
        assert!(!directory.path().join(VERIFICATION_MANIFEST_PATH).exists());
    }

    #[test]
    fn mixed_discovery_is_deterministic_and_never_imports_script_bodies() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            "[workspace]\nmembers=[]\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("package.json"),
            r#"{"scripts":{"test":"echo $SECRET && curl https://private","build":"vite build"}}"#,
        )
        .unwrap();
        let first =
            discover_verification_manifest(directory.path(), &approval_path(directory.path()))
                .unwrap();
        let second =
            discover_verification_manifest(directory.path(), &approval_path(directory.path()))
                .unwrap();
        assert_eq!(first, second);
        assert!(first
            .candidates
            .iter()
            .any(|candidate| candidate.gate.program == "cargo"));
        assert!(first
            .candidates
            .iter()
            .any(|candidate| candidate.gate.program == "npm"));
        assert!(!first.proposed_toml.contains("SECRET"));
        assert!(!first.proposed_toml.contains("private"));
    }

    #[test]
    fn create_update_stale_approval_and_rollback_are_digest_bound() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        let approval = approval_path(directory.path());
        let preview = discover_verification_manifest(directory.path(), &approval).unwrap();
        let created =
            set_verification_manifest(directory.path(), &preview.proposed_manifest, None).unwrap();
        assert!(created.approval_required);
        assert_eq!(
            verification_manifest_status(directory.path(), &approval)
                .unwrap()
                .state,
            VerificationManifestState::Unapproved
        );
        crate::approve_verification_manifest(directory.path(), &approval).unwrap();
        assert_eq!(
            verification_manifest_status(directory.path(), &approval)
                .unwrap()
                .state,
            VerificationManifestState::Approved
        );

        let mut changed = preview.proposed_manifest.clone();
        changed.gates[0].title = Some("Changed".into());
        let updated =
            set_verification_manifest(directory.path(), &changed, Some(&created.digest)).unwrap();
        assert_eq!(
            verification_manifest_status(directory.path(), &approval)
                .unwrap()
                .state,
            VerificationManifestState::Stale
        );
        assert!(set_verification_manifest(
            directory.path(),
            &preview.proposed_manifest,
            Some("stale-digest")
        )
        .is_err());
        let rolled_back =
            rollback_verification_manifest(directory.path(), &updated.digest).unwrap();
        assert_eq!(rolled_back.digest, created.digest);
    }

    #[test]
    fn unsafe_values_are_rejected_without_writing() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        let manifest = VerificationManifest {
            schema_version: 1,
            gates: vec![VerificationGate {
                id: "unsafe".into(),
                title: None,
                program: "cargo".into(),
                args: vec!["$(steal-token)".into()],
                cwd: ".".into(),
                timeout_seconds: Some(30),
                output_limit_bytes: Some(1024),
                expected_exit_code: Some(0),
                result_matcher: None,
                environment: BTreeMap::new(),
                fingerprint_exclude: Vec::new(),
            }],
        };
        assert!(set_verification_manifest(directory.path(), &manifest, None).is_err());
        assert!(!directory.path().join(VERIFICATION_MANIFEST_PATH).exists());
    }
}
