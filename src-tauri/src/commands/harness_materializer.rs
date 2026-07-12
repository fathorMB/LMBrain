use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;
use uuid::Uuid;

use lmbrain_core::{content_digest, load_harness_manifest, HarnessHost};

use crate::commands::{
    codex_registration,
    harness_planner::{plan_harness_configuration, PreviewAction},
    mcp_registration, opencode_registration, pi_registration,
};

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
    pub state: String,
    pub expected_digest: String,
    pub actual_digest: Option<String>,
}

pub fn detect_drift(root: &Path, applied: &BTreeMap<String, String>) -> Vec<HarnessDriftEntry> {
    applied
        .iter()
        .filter_map(|(relative, expected)| {
            let actual = fs::read(root.join(relative))
                .ok()
                .map(|bytes| content_digest(&bytes));
            if actual.as_deref() == Some(expected) {
                None
            } else {
                Some(HarnessDriftEntry {
                    path: relative.clone(),
                    state: if actual.is_some() {
                        "changed".into()
                    } else {
                        "missing".into()
                    },
                    expected_digest: expected.clone(),
                    actual_digest: actual,
                })
            }
        })
        .collect()
}

struct PendingWrite {
    target: PathBuf,
    stage: PathBuf,
    backup: PathBuf,
    existed: bool,
}

pub fn apply_harness_configuration(
    root: &Path,
    command: &str,
    expected_digest: &str,
) -> Result<HarnessApplyResult, String> {
    apply_with_failure(root, command, expected_digest, None)
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
        let (relative, content) = render(&root, host, command)?;
        if actions.get(&relative) == Some(&PreviewAction::Preserved) {
            continue;
        }
        let target = root.join(&relative);
        validate_target(&root, &target)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let nonce = Uuid::new_v4();
        let stage = target.with_extension(format!("lmbrain-{nonce}.tmp"));
        let backup = target.with_extension(format!("lmbrain-{nonce}.bak"));
        let mut file = OpenOptions::new()
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
                content_digest: content_digest(&bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(HarnessApplyResult {
        manifest_digest: plan.manifest_digest,
        changed: !pending.is_empty(),
        files,
    })
}

fn render(root: &Path, host: HarnessHost, command: &str) -> Result<(String, String), String> {
    let root_text = root.to_string_lossy();
    let (relative, result) = match host {
        HarnessHost::ClaudeCode => (
            ".mcp.json",
            mcp_registration::build_mcp_config(
                read(root.join(".mcp.json")).as_deref(),
                command,
                &root_text,
            ),
        ),
        HarnessHost::Codex => (
            ".codex/config.toml",
            codex_registration::build_codex_project_config(
                read(root.join(".codex/config.toml")).as_deref(),
                command,
                &root_text,
            ),
        ),
        HarnessHost::Pi => (
            ".pi/mcp.json",
            pi_registration::build_pi_mcp_config(
                read(root.join(".pi/mcp.json")).as_deref(),
                command,
                &root_text,
            ),
        ),
        HarnessHost::OpenCode => (
            "opencode.json",
            opencode_registration::build_opencode_config(
                read(root.join("opencode.json")).as_deref(),
                command,
                &root_text,
            ),
        ),
    };
    result
        .map(|content| (relative.into(), content))
        .map_err(|error| error.to_string())
}

fn read(path: PathBuf) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn validate_target(root: &Path, target: &Path) -> Result<(), String> {
    if !target.starts_with(root) {
        return Err("native target escapes workspace".into());
    }
    if target.exists()
        && fs::symlink_metadata(target)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err(format!("refusing symlink target {}", target.display()));
    }
    if let Some(parent) = target.parent() {
        let mut cursor = parent;
        while cursor.starts_with(root) && cursor != root {
            if cursor.exists()
                && fs::symlink_metadata(cursor)
                    .map_err(|error| error.to_string())?
                    .file_type()
                    .is_symlink()
            {
                return Err(format!("refusing symlink parent {}", cursor.display()));
            }
            cursor = cursor.parent().ok_or("invalid target parent")?;
        }
    }
    Ok(())
}

fn rollback(pending: &[PendingWrite], committed: usize) {
    for write in pending.iter().take(committed).rev() {
        let _ = fs::remove_file(&write.target);
        if write.backup.exists() {
            let _ = fs::rename(&write.backup, &write.target);
        }
    }
    for write in pending {
        let _ = fs::remove_file(&write.stage);
        if write.backup.exists() && !write.target.exists() {
            let _ = fs::rename(&write.backup, &write.target);
        }
    }
}

struct ApplyLock(PathBuf);
impl ApplyLock {
    fn acquire(path: PathBuf) -> Result<Self, String> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    "harness apply already in progress".into()
                } else {
                    error.to_string()
                }
            })?;
        Ok(Self(path))
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
    use lmbrain_core::{set_harness_manifest, HarnessManifest};

    fn workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        let manifest: HarnessManifest = serde_json::from_value(serde_json::json!({"schema_version":1,"hosts":{"claude-code":{"enabled":true},"pi":{"enabled":true}}})).unwrap();
        set_harness_manifest(dir.path(), &manifest).unwrap();
        dir
    }

    #[test]
    fn apply_is_idempotent_and_preserves_unrelated_content() {
        let dir = workspace();
        fs::write(dir.path().join(".mcp.json"), r#"{"keep":true}"#).unwrap();
        let digest =
            lmbrain_core::canonical_manifest_digest(&load_harness_manifest(dir.path()).unwrap())
                .unwrap();
        let first = apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        assert!(first.changed);
        assert_eq!(first.files.len(), 2);
        let second = apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        assert!(!second.changed);
        assert_eq!(second.files.len(), 2);
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["keep"], true);
    }

    #[test]
    fn injected_batch_failure_restores_every_original_file() {
        let dir = workspace();
        fs::create_dir(dir.path().join(".pi")).unwrap();
        let claude = r#"{"keep":"claude"}"#;
        let pi = r#"{"keep":"pi"}"#;
        fs::write(dir.path().join(".mcp.json"), claude).unwrap();
        fs::write(dir.path().join(".pi/mcp.json"), pi).unwrap();
        let digest =
            lmbrain_core::canonical_manifest_digest(&load_harness_manifest(dir.path()).unwrap())
                .unwrap();
        assert!(apply_with_failure(dir.path(), "lmbrain-mcp", &digest, Some(2)).is_err());
        assert_eq!(
            fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            claude
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(".pi/mcp.json")).unwrap(),
            pi
        );
    }

    #[test]
    fn drift_reports_changed_and_missing_files() {
        let dir = workspace();
        let digest =
            lmbrain_core::canonical_manifest_digest(&load_harness_manifest(dir.path()).unwrap())
                .unwrap();
        let applied = apply_harness_configuration(dir.path(), "lmbrain-mcp", &digest).unwrap();
        let expected = applied
            .files
            .iter()
            .map(|file| (file.path.clone(), file.content_digest.clone()))
            .collect();
        fs::write(dir.path().join(".mcp.json"), "{}").unwrap();
        fs::remove_file(dir.path().join(".pi/mcp.json")).unwrap();
        let drift = detect_drift(dir.path(), &expected);
        assert_eq!(drift.len(), 2);
        assert!(drift.iter().any(|entry| entry.state == "changed"));
        assert!(drift.iter().any(|entry| entry.state == "missing"));
    }
}
