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
                can_migrate: false,
                blocker_reason: Some(reason),
            },
            writes: Vec::new(),
        });
    }

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
            let item = KitMigrationItem {
                path: format!(".lmbrain/{}", filename),
                action: "update".into(),
                classification: "kit-owned".into(),
                description: format!("Update kit-owned {}", filename),
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
                    let item = KitMigrationItem {
                        path: format!(".lmbrain/contract/{}", fname_str),
                        action: "update".into(),
                        classification: "kit-owned".into(),
                        description: format!("Update contract capability module {}", fname_str),
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
                    let item = KitMigrationItem {
                        path: format!(".lmbrain/templates/{}", fname_str),
                        action: "update".into(),
                        classification: "kit-owned".into(),
                        description: format!("Update template {}", fname_str),
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
                classification: "kit-owned".into(),
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

    // Compute canonical digest
    let digest = migration_digest(&from_version, &to_version, &items);

    let preview = KitMigrationPreview {
        from_version,
        to_version,
        digest,
        items,
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
