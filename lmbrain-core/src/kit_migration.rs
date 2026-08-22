use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::debt_migration::copy_tree;
use crate::mutation_lock::WorkspaceLock;
use crate::path::PathGuard;

#[derive(Debug, Error)]
pub enum KitMigrationError {
    #[error("preflight failed: {0}")]
    Preflight(String),
    #[error("kit migration changed since preview: expected {expected}, current {current}")]
    Stale { expected: String, current: String },
    #[error("kit migration requires explicit operator confirmation")]
    ConfirmationRequired,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct KitMigrationItem {
    pub path: String,
    pub action: String,
    pub classification: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct KitMigrationPreview {
    pub from_version: String,
    pub to_version: String,
    pub digest: String,
    pub items: Vec<KitMigrationItem>,
    /// Kit-owned paths whose current content differs from the content the
    /// installed kit shipped. Realigning them discards a local edit.
    pub locally_modified: Vec<String>,
    pub can_migrate: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct KitMigrationResult {
    pub previous_version: String,
    pub current_version: String,
    pub updated_items: Vec<String>,
    pub backed_up_to: String,
}

#[derive(Debug)]
struct PlannedWrite {
    pub item: KitMigrationItem,
    pub target_relative: PathBuf,
    pub content: Option<Vec<u8>>,
    pub is_delete: bool,
}

#[derive(Debug)]
struct MigrationPlan {
    pub preview: KitMigrationPreview,
    pub writes: Vec<PlannedWrite>,
}

const KIT_OWNED_FILES: &[&str] = &[
    "CONTRACT.md",
    "AGENT.md",
    "QUALITY.md",
    "UPGRADING.md",
    "VERSION",
];

/// Digests of the kit-owned files as the kit shipped them, recorded at the
/// installed version. It is the only way to tell an intentional local edit
/// apart from a file that is simply older than the bundled kit.
const KIT_BASELINE_FILE: &str = ".kit-baseline.json";

/// The workspace copy matches what the kit shipped; realignment is lossless.
const CLASS_KIT_OWNED: &str = "kit-owned";
/// The workspace copy was edited after installation; realignment discards it.
const CLASS_KIT_OWNED_MODIFIED: &str = "kit-owned-modified";
/// No baseline covers this file, so a local edit cannot be ruled out.
const CLASS_KIT_OWNED_UNVERIFIED: &str = "kit-owned-unverified";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct KitBaseline {
    version: String,
    files: BTreeMap<String, String>,
}

fn content_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn load_baseline(target_lmbrain: &Path) -> Option<KitBaseline> {
    let raw = fs::read(target_lmbrain.join(KIT_BASELINE_FILE)).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn kit_owned_relative_paths(bundled_lmbrain: &Path) -> Vec<String> {
    let mut paths: Vec<String> = KIT_OWNED_FILES
        .iter()
        .filter(|name| bundled_lmbrain.join(name).is_file())
        .map(|name| (*name).to_string())
        .collect();
    for directory in ["contract", "templates"] {
        let Ok(entries) = fs::read_dir(bundled_lmbrain.join(directory)) else {
            continue;
        };
        let mut nested: Vec<String> = entries
            .flatten()
            .filter(|entry| entry.path().is_file())
            .map(|entry| format!("{directory}/{}", entry.file_name().to_string_lossy()))
            .collect();
        nested.sort();
        paths.extend(nested);
    }
    paths
}

/// Digests every kit-owned file the bundled kit ships, so the next migration
/// classifies the workspace copies instead of guessing.
fn build_baseline(
    bundled_lmbrain: &Path,
    to_version: &str,
) -> Result<KitBaseline, KitMigrationError> {
    let mut files = BTreeMap::new();
    for relative in kit_owned_relative_paths(bundled_lmbrain) {
        let content = fs::read(bundled_lmbrain.join(&relative))?;
        files.insert(relative, content_digest(&content));
    }
    Ok(KitBaseline {
        version: to_version.to_string(),
        files,
    })
}

/// Returns the action, the classification, and a description suffix naming the
/// consequence, for one kit-owned file the bundled kit would replace.
fn classify_kit_owned(
    target_file: &Path,
    relative: &str,
    baseline: Option<&KitBaseline>,
) -> (&'static str, &'static str, String) {
    let Ok(current) = fs::read(target_file) else {
        return ("create", CLASS_KIT_OWNED, String::new());
    };
    match baseline.and_then(|base| base.files.get(relative)) {
        Some(recorded) if *recorded == content_digest(&current) => {
            ("update", CLASS_KIT_OWNED, String::new())
        }
        Some(_) => (
            "update",
            CLASS_KIT_OWNED_MODIFIED,
            " (locally modified since installation; the current content is replaced and recoverable only from the migration backup)"
                .into(),
        ),
        None => (
            "update",
            CLASS_KIT_OWNED_UNVERIFIED,
            " (no baseline recorded for the installed kit; a local edit cannot be ruled out)".into(),
        ),
    }
}

fn action_verb(action: &str) -> &'static str {
    if action == "create" {
        "Add"
    } else {
        "Update"
    }
}

/// Records, for a freshly installed kit, the digests it shipped. Without this
/// a new workspace would report every kit-owned file as unverified on its first
/// upgrade, because nothing states what the kit originally wrote.
pub fn record_kit_baseline(
    workspace_root: &Path,
    bundled_kit_root: &Path,
) -> Result<(), KitMigrationError> {
    let guard = PathGuard::new(workspace_root)
        .map_err(|e| KitMigrationError::Preflight(e.to_string()))?;
    let target_lmbrain = guard.root().join(".lmbrain");
    if !target_lmbrain.is_dir() {
        return Err(KitMigrationError::Preflight(
            "Target .lmbrain does not exist".into(),
        ));
    }
    let bundled_lmbrain = if bundled_kit_root.join(".lmbrain").exists() {
        bundled_kit_root.join(".lmbrain")
    } else {
        bundled_kit_root.to_path_buf()
    };
    let version = fs::read_to_string(bundled_lmbrain.join("VERSION"))
        .map(|value| value.trim_start_matches('\u{feff}').trim().to_string())
        .map_err(|_| KitMigrationError::Preflight("bundled kit VERSION does not exist".into()))?;
    let baseline = build_baseline(&bundled_lmbrain, &version)?;
    let content = serde_json::to_vec_pretty(&baseline)
        .map_err(|error| KitMigrationError::Preflight(error.to_string()))?;
    fs::write(target_lmbrain.join(KIT_BASELINE_FILE), content)?;
    Ok(())
}

pub fn kit_migration_preview(
    workspace_root: &Path,
    bundled_kit_root: &Path,
) -> Result<KitMigrationPreview, KitMigrationError> {
    Ok(build_plan(workspace_root, bundled_kit_root)?.preview)
}

pub fn kit_migrate(
    workspace_root: &Path,
    bundled_kit_root: &Path,
    expected_preview_digest: &str,
    confirmed: bool,
) -> Result<KitMigrationResult, KitMigrationError> {
    if !confirmed {
        return Err(KitMigrationError::ConfirmationRequired);
    }
    let guard = PathGuard::new(workspace_root)
        .map_err(|e| KitMigrationError::Preflight(e.to_string()))?;
    // The lock is deliberately outside `.lmbrain`, so it remains held for the
    // entire staging and swap operation on every platform.
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let plan = build_plan(guard.root(), bundled_kit_root)?;
    if !plan.preview.can_migrate {
        return Err(KitMigrationError::Preflight(
            plan.preview.blocker_reason.unwrap_or_else(|| "migration blocked".into()),
        ));
    }
    if plan.preview.digest != expected_preview_digest {
        return Err(KitMigrationError::Stale {
            expected: expected_preview_digest.into(),
            current: plan.preview.digest,
        });
    }

    let source = guard.root().join(".lmbrain");
    let stage_root = guard.root().join(format!(
        ".lmbrain-kit-migration-stage-{}",
        std::process::id()
    ));
    let backup = guard.root().join(format!(
        ".lmbrain-kit-migration-backup-{}",
        std::process::id()
    ));

    if stage_root.exists() || backup.exists() {
        return Err(KitMigrationError::Preflight(
            "stale migration staging or backup directory exists".into(),
        ));
    }

    let stage_brain = stage_root.join(".lmbrain");
    fs::create_dir(&stage_root)?;
    if let Err(err) = copy_tree(&source, &stage_brain) {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(KitMigrationError::Preflight(err.to_string()));
    }

    let mut updated_items = Vec::new();
    let staged_res = (|| -> Result<(), KitMigrationError> {
        for write in &plan.writes {
            let target_path = stage_brain.join(&write.target_relative);
            if write.is_delete {
                if target_path.exists() {
                    if target_path.is_dir() {
                        fs::remove_dir_all(&target_path)?;
                    } else {
                        fs::remove_file(&target_path)?;
                    }
                }
            } else if let Some(ref content) = write.content {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target_path, content)?;
            }
            updated_items.push(write.item.path.clone());
        }
        Ok(())
    })();

    if let Err(err) = staged_res {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(err);
    }

    if let Err(err) = validate_staged_tree(&stage_brain, &plan) {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(err);
    }

    // Atomic swap: rename source -> backup, rename stage -> source
    if let Err(err) = fs::rename(&source, &backup) {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(err.into());
    }

    if let Err(err) = fs::rename(&stage_brain, &source) {
        let _ = fs::rename(&backup, &source);
        let _ = fs::remove_dir_all(&stage_root);
        return Err(err.into());
    }

    let _ = fs::remove_dir_all(&stage_root);

    Ok(KitMigrationResult {
        previous_version: plan.preview.from_version,
        current_version: plan.preview.to_version,
        updated_items,
        backed_up_to: backup
            .strip_prefix(guard.root())
            .unwrap_or(&backup)
            .to_string_lossy()
            .into_owned(),
    })
}

fn build_plan(workspace_root: &Path, bundled_kit_root: &Path) -> Result<MigrationPlan, KitMigrationError> {
    let guard = PathGuard::new(workspace_root)
        .map_err(|e| KitMigrationError::Preflight(e.to_string()))?;
    let target_lmbrain = guard.root().join(".lmbrain");
    if !target_lmbrain.exists() {
        return Err(KitMigrationError::Preflight("Target .lmbrain does not exist".into()));
    }

    let clean_version = |s: String| s.trim_start_matches('\u{feff}').trim().to_string();

    let from_version = fs::read_to_string(target_lmbrain.join("VERSION"))
        .map(clean_version)
        .unwrap_or_else(|_| "unknown".into());

    let bundled_lmbrain = if bundled_kit_root.join(".lmbrain").exists() {
        bundled_kit_root.join(".lmbrain")
    } else {
        bundled_kit_root.to_path_buf()
    };

    if !bundled_lmbrain.is_dir() {
        return Err(KitMigrationError::Preflight(
            "bundled kit directory does not exist".into(),
        ));
    }

    let to_version = fs::read_to_string(bundled_lmbrain.join("VERSION"))
        .map(clean_version)
        .map_err(|_| KitMigrationError::Preflight("bundled kit VERSION does not exist".into()))?;

    let same_directory = target_lmbrain.canonicalize()? == bundled_lmbrain.canonicalize()?;
    if same_directory || from_version == to_version {
        let reason = if same_directory {
            "bundled kit resolves to the target workspace".into()
        } else {
            "workspace kit is already at the bundled version".into()
        };
        let digest = migration_digest(&from_version, &to_version, &[]);
        return Ok(MigrationPlan {
            preview: KitMigrationPreview {
                from_version,
                to_version,
                digest,
                items: Vec::new(),
                locally_modified: Vec::new(),
                can_migrate: false,
                blocker_reason: Some(reason),
            },
            writes: Vec::new(),
        });
    }

    let baseline = load_baseline(&target_lmbrain);

    let mut writes = Vec::new();
    let mut items = Vec::new();

    // 1. Kit-owned top-level files
    for &filename in KIT_OWNED_FILES {
        let bundled_file = bundled_lmbrain.join(filename);
        if bundled_file.exists() {
            let content = fs::read(&bundled_file)?;
            if fs::read(target_lmbrain.join(filename)).ok().as_deref() == Some(content.as_slice()) {
                continue;
            }
            let (action, classification, note) =
                classify_kit_owned(&target_lmbrain.join(filename), filename, baseline.as_ref());
            let item = KitMigrationItem {
                path: format!(".lmbrain/{}", filename),
                action: action.into(),
                classification: classification.into(),
                description: format!("{} kit-owned {}{}", action_verb(action), filename, note),
            };
            writes.push(PlannedWrite {
                item: item.clone(),
                target_relative: PathBuf::from(filename),
                content: Some(content),
                is_delete: false,
            });
            items.push(item);
        }
    }

    // 2. Contract capability modules
    let bundled_contract_dir = bundled_lmbrain.join("contract");
    if bundled_contract_dir.exists() {
        if let Ok(entries) = fs::read_dir(&bundled_contract_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = entry.file_name();
                    let fname_str = fname.to_string_lossy();
                    let content = fs::read(&p)?;
                    if fs::read(target_lmbrain.join("contract").join(&*fname_str))
                        .ok()
                        .as_deref()
                        == Some(content.as_slice())
                    {
                        continue;
                    }
                    let relative = format!("contract/{}", fname_str);
                    let (action, classification, note) = classify_kit_owned(
                        &target_lmbrain.join("contract").join(&*fname_str),
                        &relative,
                        baseline.as_ref(),
                    );
                    let item = KitMigrationItem {
                        path: format!(".lmbrain/{}", relative),
                        action: action.into(),
                        classification: classification.into(),
                        description: format!(
                            "{} contract capability module {}{}",
                            action_verb(action),
                            fname_str,
                            note
                        ),
                    };
                    writes.push(PlannedWrite {
                        item: item.clone(),
                        target_relative: PathBuf::from("contract").join(&*fname_str),
                        content: Some(content),
                        is_delete: false,
                    });
                    items.push(item);
                }
            }
        }
    }

    // 3. Templates
    let bundled_templates_dir = bundled_lmbrain.join("templates");
    if bundled_templates_dir.exists() {
        if let Ok(entries) = fs::read_dir(&bundled_templates_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = entry.file_name();
                    let fname_str = fname.to_string_lossy();
                    let content = fs::read(&p)?;
                    if fs::read(target_lmbrain.join("templates").join(&*fname_str))
                        .ok()
                        .as_deref()
                        == Some(content.as_slice())
                    {
                        continue;
                    }
                    let relative = format!("templates/{}", fname_str);
                    let (action, classification, note) = classify_kit_owned(
                        &target_lmbrain.join("templates").join(&*fname_str),
                        &relative,
                        baseline.as_ref(),
                    );
                    let item = KitMigrationItem {
                        path: format!(".lmbrain/{}", relative),
                        action: action.into(),
                        classification: classification.into(),
                        description: format!("{} template {}{}", action_verb(action), fname_str, note),
                    };
                    writes.push(PlannedWrite {
                        item: item.clone(),
                        target_relative: PathBuf::from("templates").join(&*fname_str),
                        content: Some(content),
                        is_delete: false,
                    });
                    items.push(item);
                }
            }
        }
    }

    // 4. Retired files in .lmbrain to remove (MIGRATIONS.md, CHANGELOG.md)
    for &retired in &["MIGRATIONS.md", "CHANGELOG.md"] {
        let existing = target_lmbrain.join(retired);
        if existing.exists() {
            let item = KitMigrationItem {
                path: format!(".lmbrain/{}", retired),
                action: "delete".into(),
                classification: CLASS_KIT_OWNED.into(),
                description: format!("Remove retired {}", retired),
            };
            writes.push(PlannedWrite {
                item: item.clone(),
                target_relative: PathBuf::from(retired),
                content: None,
                is_delete: true,
            });
            items.push(item);
        }
    }

    // 5. Record the baseline so the next migration can classify local edits.
    let new_baseline = build_baseline(&bundled_lmbrain, &to_version)?;
    let baseline_content = serde_json::to_vec_pretty(&new_baseline)
        .map_err(|error| KitMigrationError::Preflight(error.to_string()))?;
    if fs::read(target_lmbrain.join(KIT_BASELINE_FILE)).ok().as_deref()
        != Some(baseline_content.as_slice())
    {
        let item = KitMigrationItem {
            path: format!(".lmbrain/{}", KIT_BASELINE_FILE),
            action: if target_lmbrain.join(KIT_BASELINE_FILE).exists() {
                "update".into()
            } else {
                "create".into()
            },
            classification: CLASS_KIT_OWNED.into(),
            description: format!("Record kit baseline digests for {}", to_version),
        };
        writes.push(PlannedWrite {
            item: item.clone(),
            target_relative: PathBuf::from(KIT_BASELINE_FILE),
            content: Some(baseline_content),
            is_delete: false,
        });
        items.push(item);
    }

    let locally_modified = items
        .iter()
        .filter(|item| item.classification == CLASS_KIT_OWNED_MODIFIED)
        .map(|item| item.path.clone())
        .collect();

    // Compute canonical digest
    let digest = migration_digest(&from_version, &to_version, &items);

    let preview = KitMigrationPreview {
        from_version,
        to_version,
        digest,
        items,
        locally_modified,
        can_migrate: true,
        blocker_reason: None,
    };

    Ok(MigrationPlan { preview, writes })
}

fn migration_digest(from_version: &str, to_version: &str, items: &[KitMigrationItem]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(from_version.as_bytes());
    hasher.update(b"->");
    hasher.update(to_version.as_bytes());
    for item in items {
        hasher.update(item.path.as_bytes());
        hasher.update(item.action.as_bytes());
        hasher.update(item.classification.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn validate_staged_tree(stage_brain: &Path, plan: &MigrationPlan) -> Result<(), KitMigrationError> {
    for write in &plan.writes {
        let staged = stage_brain.join(&write.target_relative);
        if write.is_delete {
            if staged.exists() {
                return Err(KitMigrationError::Preflight(format!(
                    "staging validation failed: {} was not removed", write.item.path
                )));
            }
        } else if fs::read(&staged).ok().as_deref() != write.content.as_deref() {
            return Err(KitMigrationError::Preflight(format!(
                "staging validation failed: {} does not match the bundled kit", write.item.path
            )));
        }
    }
    let version = fs::read_to_string(stage_brain.join("VERSION"))
        .map(|value| value.trim_start_matches('\u{feff}').trim().to_string())
        .map_err(|_| KitMigrationError::Preflight("staging validation failed: VERSION is missing".into()))?;
    if version != plan.preview.to_version {
        return Err(KitMigrationError::Preflight(format!(
            "staging validation failed: VERSION is {version}, expected {}", plan.preview.to_version
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_fixture() -> (tempfile::TempDir, tempfile::TempDir) {
        let workspace = tempdir().unwrap();
        let bundled = tempdir().unwrap();

        let ws_brain = workspace.path().join(".lmbrain");
        fs::create_dir_all(&ws_brain).unwrap();
        fs::write(ws_brain.join("VERSION"), "3.1.3\n").unwrap();
        fs::write(ws_brain.join("CONTRACT.md"), "old contract").unwrap();
        fs::write(ws_brain.join("PROJECT.md"), "my project").unwrap();
        fs::write(ws_brain.join("MIGRATIONS.md"), "old migrations").unwrap();

        let b_brain = bundled.path().join(".lmbrain");
        fs::create_dir_all(b_brain.join("contract")).unwrap();
        fs::create_dir_all(b_brain.join("templates")).unwrap();
        fs::write(b_brain.join("VERSION"), "5.0.0\n").unwrap();
        fs::write(b_brain.join("CONTRACT.md"), "new contract").unwrap();
        fs::write(b_brain.join("AGENT.md"), "new agent").unwrap();
        fs::write(b_brain.join("QUALITY.md"), "new quality").unwrap();
        fs::write(b_brain.join("UPGRADING.md"), "upgrading guide").unwrap();
        fs::write(b_brain.join("contract/verification.md"), "verification module").unwrap();
        fs::write(b_brain.join("templates/spec.md"), "spec template").unwrap();

        (workspace, bundled)
    }

    #[test]
    fn kit_migration_preview_is_deterministic_and_read_only() {
        let (workspace, bundled) = setup_fixture();
        let preview1 = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        let preview2 = kit_migration_preview(workspace.path(), bundled.path()).unwrap();

        assert_eq!(preview1.from_version, "3.1.3");
        assert_eq!(preview1.to_version, "5.0.0");
        assert_eq!(preview1.digest, preview2.digest);
        assert!(preview1.can_migrate);
        assert_eq!(preview1.items.len(), preview2.items.len());

        // Target workspace is unchanged
        let ws_brain = workspace.path().join(".lmbrain");
        assert_eq!(fs::read_to_string(ws_brain.join("CONTRACT.md")).unwrap(), "old contract");
        assert!(ws_brain.join("MIGRATIONS.md").exists());
    }

    #[test]
    fn kit_migrate_swaps_atomically_and_preserves_project_files() {
        let (workspace, bundled) = setup_fixture();
        let preview = kit_migration_preview(workspace.path(), bundled.path()).unwrap();

        let result = kit_migrate(
            workspace.path(),
            bundled.path(),
            &preview.digest,
            true,
        ).unwrap();

        assert_eq!(result.previous_version, "3.1.3");
        assert_eq!(result.current_version, "5.0.0");
        assert!(workspace.path().join(&result.backed_up_to).exists());

        let ws_brain = workspace.path().join(".lmbrain");
        // Kit-owned updated
        assert_eq!(fs::read_to_string(ws_brain.join("VERSION")).unwrap().trim(), "5.0.0");
        assert_eq!(fs::read_to_string(ws_brain.join("CONTRACT.md")).unwrap(), "new contract");
        assert_eq!(fs::read_to_string(ws_brain.join("UPGRADING.md")).unwrap(), "upgrading guide");
        assert_eq!(fs::read_to_string(ws_brain.join("contract/verification.md")).unwrap(), "verification module");
        assert_eq!(fs::read_to_string(ws_brain.join("templates/spec.md")).unwrap(), "spec template");

        // Retired deleted
        assert!(!ws_brain.join("MIGRATIONS.md").exists());

        // Project-owned strictly preserved
        assert_eq!(fs::read_to_string(ws_brain.join("PROJECT.md")).unwrap(), "my project");
    }

    #[test]
    fn migration_records_a_baseline_and_reports_no_local_modification() {
        let (workspace, bundled) = setup_fixture();
        let preview = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        // Nothing can be claimed about a workspace installed before baselines existed.
        assert!(preview.locally_modified.is_empty());
        assert!(preview
            .items
            .iter()
            .any(|item| item.path == ".lmbrain/CONTRACT.md"
                && item.classification == CLASS_KIT_OWNED_UNVERIFIED));

        kit_migrate(workspace.path(), bundled.path(), &preview.digest, true).unwrap();

        let baseline: KitBaseline = serde_json::from_slice(
            &fs::read(workspace.path().join(".lmbrain").join(KIT_BASELINE_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(baseline.version, "5.0.0");
        assert_eq!(
            baseline.files.get("CONTRACT.md"),
            Some(&content_digest(b"new contract"))
        );
        assert!(baseline.files.contains_key("contract/verification.md"));
        assert!(baseline.files.contains_key("templates/spec.md"));
    }

    #[test]
    fn a_kit_owned_file_edited_after_installation_is_reported_as_locally_modified() {
        let (workspace, bundled) = setup_fixture();
        let ws_brain = workspace.path().join(".lmbrain");
        let b_brain = bundled.path().join(".lmbrain");

        // Install the bundled kit so a baseline exists.
        let first = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        kit_migrate(workspace.path(), bundled.path(), &first.digest, true).unwrap();

        // The operator adapts AGENT.md; a later kit revision would replace it.
        fs::write(ws_brain.join("AGENT.md"), "new agent\n\nProject rule: ask first.").unwrap();
        fs::write(b_brain.join("VERSION"), "5.1.0\n").unwrap();
        fs::write(b_brain.join("AGENT.md"), "agent v5.1").unwrap();

        let preview = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        assert_eq!(preview.locally_modified, vec![".lmbrain/AGENT.md".to_string()]);
        let agent = preview
            .items
            .iter()
            .find(|item| item.path == ".lmbrain/AGENT.md")
            .unwrap();
        assert_eq!(agent.classification, CLASS_KIT_OWNED_MODIFIED);
        assert!(agent.description.contains("locally modified"));

        // An untouched kit-owned file stays a lossless realignment.
        let quality = preview
            .items
            .iter()
            .find(|item| item.path == ".lmbrain/QUALITY.md");
        assert!(quality.is_none_or(|item| item.classification == CLASS_KIT_OWNED));

        // The digest is classification-bound, so editing after a preview invalidates it.
        let stale = preview.digest.clone();
        fs::write(ws_brain.join("AGENT.md"), "new agent").unwrap();
        let refreshed = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        assert_ne!(refreshed.digest, stale);
        assert!(refreshed.locally_modified.is_empty());
    }

    #[test]
    fn a_freshly_initialized_kit_records_its_baseline_so_the_first_upgrade_is_precise() {
        let (workspace, bundled) = setup_fixture();
        let ws_brain = workspace.path().join(".lmbrain");
        let b_brain = bundled.path().join(".lmbrain");

        // Simulate `initialize_kit`: the bundled kit is copied in as-is.
        for name in ["VERSION", "CONTRACT.md", "AGENT.md", "QUALITY.md", "UPGRADING.md"] {
            fs::write(ws_brain.join(name), fs::read(b_brain.join(name)).unwrap()).unwrap();
        }
        record_kit_baseline(workspace.path(), bundled.path()).unwrap();
        assert!(ws_brain.join(KIT_BASELINE_FILE).is_file());

        // A later kit revision arrives, and the operator has adapted QUALITY.md.
        fs::write(ws_brain.join("QUALITY.md"), "new quality + project rule").unwrap();
        fs::write(b_brain.join("VERSION"), "5.1.0\n").unwrap();
        fs::write(b_brain.join("CONTRACT.md"), "contract v5.1").unwrap();
        fs::write(b_brain.join("QUALITY.md"), "quality v5.1").unwrap();

        let preview = kit_migration_preview(workspace.path(), bundled.path()).unwrap();
        assert_eq!(
            preview.locally_modified,
            vec![".lmbrain/QUALITY.md".to_string()]
        );
        let contract = preview
            .items
            .iter()
            .find(|item| item.path == ".lmbrain/CONTRACT.md")
            .unwrap();
        // Untouched since installation, so it is a lossless realignment rather
        // than the blanket "unverified" a baseline-less workspace would report.
        assert_eq!(contract.classification, CLASS_KIT_OWNED);
    }

    #[test]
    fn kit_migrate_fails_without_confirmation() {
        let (workspace, bundled) = setup_fixture();
        let preview = kit_migration_preview(workspace.path(), bundled.path()).unwrap();

        let err = kit_migrate(
            workspace.path(),
            bundled.path(),
            &preview.digest,
            false,
        ).unwrap_err();

        assert!(matches!(err, KitMigrationError::ConfirmationRequired));
    }

    #[test]
    fn kit_migration_refuses_the_target_as_its_own_bundle() {
        let (workspace, _) = setup_fixture();
        let preview = kit_migration_preview(workspace.path(), workspace.path()).unwrap();

        assert!(!preview.can_migrate);
        assert_eq!(
            preview.blocker_reason.as_deref(),
            Some("bundled kit resolves to the target workspace")
        );
    }
}
