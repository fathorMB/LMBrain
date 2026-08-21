use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{frontmatter::Document, path::PathGuard};

pub const DEBT_MIGRATION_SCHEMA_VERSION: &str = "1";
pub const DEBT_CONTRACT_VERSION: &str = "4.2.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtMigrationItem {
    pub path: String,
    pub destination: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtMigrationPreview {
    pub schema_version: String,
    pub source_version: Option<String>,
    pub target_version: String,
    pub digest: String,
    pub items: Vec<DebtMigrationItem>,
    pub review_id_mappings: BTreeMap<String, BTreeMap<String, String>>,
    pub mutated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtMigrationResult {
    pub schema_version: String,
    pub previous_version: Option<String>,
    pub version: String,
    pub preview_digest: String,
    pub migrated_files: usize,
}

#[derive(Debug, Error)]
pub enum DebtMigrationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("debt migration preflight failed: {0}")]
    Preflight(String),
    #[error("debt migration changed since preview: expected {expected}, current {current}")]
    Stale { expected: String, current: String },
    #[error("debt migration requires explicit operator confirmation")]
    ConfirmationRequired,
}

#[derive(Debug)]
struct PlannedWrite {
    item: DebtMigrationItem,
    content: Vec<u8>,
}

#[derive(Debug)]
struct MigrationPlan {
    preview: DebtMigrationPreview,
    writes: Vec<PlannedWrite>,
}

pub fn debt_migration_preview(root: &Path) -> Result<DebtMigrationPreview, DebtMigrationError> {
    Ok(build_plan(root)?.preview)
}

pub fn debt_migrate(
    root: &Path,
    expected_preview_digest: &str,
    confirmed: bool,
) -> Result<DebtMigrationResult, DebtMigrationError> {
    if !confirmed {
        return Err(DebtMigrationError::ConfirmationRequired);
    }
    let guard =
        PathGuard::new(root).map_err(|error| DebtMigrationError::Preflight(error.to_string()))?;
    let plan = build_plan(guard.root())?;
    if plan.preview.digest != expected_preview_digest {
        return Err(DebtMigrationError::Stale {
            expected: expected_preview_digest.into(),
            current: plan.preview.digest,
        });
    }

    let source = guard.root().join(".lmbrain");
    let stage_root = guard.root().join(format!(
        ".lmbrain-debt-migration-stage-{}",
        std::process::id()
    ));
    let backup = guard.root().join(format!(
        ".lmbrain-debt-migration-backup-{}",
        std::process::id()
    ));
    if stage_root.exists() || backup.exists() {
        return Err(DebtMigrationError::Preflight(
            "stale migration staging or backup directory exists".into(),
        ));
    }

    let stage_brain = stage_root.join(".lmbrain");
    fs::create_dir(&stage_root)?;
    if let Err(error) = copy_tree(&source, &stage_brain) {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(error);
    }

    let staged = (|| -> Result<(), DebtMigrationError> {
        for write in &plan.writes {
            let from = stage_root.join(&write.item.path);
            let to = stage_root.join(&write.item.destination);
            if from != to && to.exists() {
                return Err(DebtMigrationError::Preflight(format!(
                    "migration destination already exists: {}",
                    write.item.destination
                )));
            }
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&to, &write.content)?;
            if from != to {
                fs::remove_file(from)?;
            }
        }
        remove_empty_tree(&stage_brain.join("findings"))?;
        fs::write(
            stage_brain.join("VERSION"),
            format!("{DEBT_CONTRACT_VERSION}\n"),
        )?;
        validate_staged_workspace(&stage_root)?;
        Ok(())
    })();
    if let Err(error) = staged {
        let _ = fs::remove_dir_all(&stage_root);
        return Err(error);
    }

    fs::rename(&source, &backup)?;
    if let Err(error) = fs::rename(&stage_brain, &source) {
        let _ = fs::rename(&backup, &source);
        let _ = fs::remove_dir_all(&stage_root);
        return Err(DebtMigrationError::Io(error));
    }
    let _ = fs::remove_dir_all(&backup);
    let _ = fs::remove_dir_all(&stage_root);

    Ok(DebtMigrationResult {
        schema_version: DEBT_MIGRATION_SCHEMA_VERSION.into(),
        previous_version: plan.preview.source_version,
        version: DEBT_CONTRACT_VERSION.into(),
        preview_digest: expected_preview_digest.into(),
        migrated_files: plan.writes.len(),
    })
}

fn build_plan(root: &Path) -> Result<MigrationPlan, DebtMigrationError> {
    let brain = root.join(".lmbrain");
    if !brain.is_dir() {
        return Err(DebtMigrationError::Preflight(
            ".lmbrain directory is missing".into(),
        ));
    }
    let source_version = fs::read_to_string(brain.join("VERSION"))
        .ok()
        .map(|value| value.trim().to_string());
    let files = regular_files(&brain)?;
    let durable_ids = collect_durable_ids(&files)?;
    let review_id_mappings = collect_review_id_mappings(root, &files)?;
    let mut writes = Vec::new();

    for path in files {
        let relative = path.strip_prefix(root).map_err(|_| {
            DebtMigrationError::Preflight(format!("unsafe path: {}", path.display()))
        })?;
        let relative_text = slash(relative);
        let bytes = fs::read(&path)?;
        let is_markdown = path.extension().and_then(|value| value.to_str()) == Some("md");
        let mut content = if is_markdown {
            String::from_utf8(bytes.clone()).map_err(|_| {
                DebtMigrationError::Preflight(format!("non-UTF-8 Markdown: {relative_text}"))
            })?
        } else {
            String::new()
        };
        let mut changes = Vec::new();

        if is_markdown && content.trim_start().starts_with("---") {
            Document::parse(&content).map_err(|error| {
                DebtMigrationError::Preflight(format!(
                    "malformed artifact {relative_text}: {error}"
                ))
            })?;
        }

        if relative_text.starts_with(".lmbrain/reviews/") && is_markdown {
            if content.contains("[[FINDING-") {
                return Err(DebtMigrationError::Preflight(format!(
                    "ambiguous durable/local finding wikilink in {relative_text}"
                )));
            }
            let mapping = review_id_mappings
                .get(&relative_text)
                .cloned()
                .unwrap_or_default();
            for (old, new) in &mapping {
                content = replace_token(&content, old, new);
            }
            if content.contains("## Findings") {
                content = content.replace("## Findings", "## Review findings");
                changes.push("canonical review heading".into());
            }
            if !mapping.is_empty() {
                changes.push("review-local RF identifiers".into());
            }
        }

        if is_markdown {
            if relative_text.starts_with(".lmbrain/findings/") {
                let document = Document::parse(&content).map_err(|error| {
                    DebtMigrationError::Preflight(format!(
                        "malformed debt source {relative_text}: {error}"
                    ))
                })?;
                if let (Some(review), Some(local)) = (
                    document.value("origin_artifact"),
                    document.value("origin_ref"),
                ) {
                    if review.starts_with("REVIEW-") {
                        let replacement = review_id_mappings
                            .iter()
                            .find(|(path, _)| path.contains(&review))
                            .and_then(|(_, mapping)| mapping.get(&local))
                            .ok_or_else(|| {
                                DebtMigrationError::Preflight(format!(
                                    "unresolved review-local origin {review}/{local} in {relative_text}"
                                ))
                            })?;
                        content = replace_token(&content, &local, replacement);
                        changes.push("review-origin RF reference".into());
                    }
                }
            }
            for (old, new) in &durable_ids {
                content = replace_token(&content, old, new);
            }
            let replaced = content
                .replace("finding_events", "debt_events")
                .replace("finding_context", "debt_context")
                .replace("finding_candidates", "debt_candidates")
                .replace("finding_create", "debt_create")
                .replace("finding_plan", "debt_plan")
                .replace("finding_defer", "debt_defer")
                .replace("finding_resolve", "debt_resolve")
                .replace("finding_accept_risk", "debt_accept_risk")
                .replace("finding_supersede", "debt_supersede")
                .replace("finding_reopen", "debt_reopen");
            if replaced != content {
                changes.push("durable lifecycle references".into());
                content = replaced;
            }
        }

        let destination = migrated_path(&relative_text, &durable_ids);
        if destination != relative_text {
            changes.push("artifact path".into());
        }
        let output = if is_markdown {
            content.into_bytes()
        } else {
            bytes.clone()
        };
        if output != bytes || destination != relative_text {
            if output != bytes && changes.is_empty() {
                changes.push("artifact references".into());
            }
            writes.push(PlannedWrite {
                item: DebtMigrationItem {
                    path: relative_text,
                    destination,
                    changes,
                },
                content: output,
            });
        }
    }
    writes.sort_by(|left, right| left.item.path.cmp(&right.item.path));
    let items = writes
        .iter()
        .map(|write| write.item.clone())
        .collect::<Vec<_>>();
    let digest_source = serde_json::to_vec(&(source_version.clone(), &items, &review_id_mappings))
        .map_err(|error| DebtMigrationError::Preflight(error.to_string()))?;
    let digest = Sha256::digest(digest_source)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(MigrationPlan {
        preview: DebtMigrationPreview {
            schema_version: DEBT_MIGRATION_SCHEMA_VERSION.into(),
            source_version,
            target_version: DEBT_CONTRACT_VERSION.into(),
            digest,
            items,
            review_id_mappings,
            mutated: false,
        },
        writes,
    })
}

fn collect_review_id_mappings(
    root: &Path,
    files: &[PathBuf],
) -> Result<BTreeMap<String, BTreeMap<String, String>>, DebtMigrationError> {
    let token = Regex::new(r"\b(?:FINDING|F)-[A-Za-z0-9-]+\b").unwrap();
    let mut output = BTreeMap::new();
    for path in files {
        let relative = slash(path.strip_prefix(root).map_err(|_| {
            DebtMigrationError::Preflight(format!("unsafe review path: {}", path.display()))
        })?);
        if !relative.starts_with(".lmbrain/reviews/")
            || path.extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let content = fs::read_to_string(path)?;
        if content.contains("[[FINDING-") {
            return Err(DebtMigrationError::Preflight(format!(
                "ambiguous durable/local finding wikilink in {relative}"
            )));
        }
        let mut mapping = BTreeMap::new();
        for value in token.find_iter(&content) {
            let next = format!("RF-{:03}", mapping.len() + 1);
            mapping.entry(value.as_str().to_string()).or_insert(next);
        }
        if !mapping.is_empty() {
            output.insert(relative, mapping);
        }
    }
    Ok(output)
}

fn collect_durable_ids(files: &[PathBuf]) -> Result<BTreeMap<String, String>, DebtMigrationError> {
    let mut ids = BTreeMap::new();
    for path in files {
        if !slash(path).contains("/.lmbrain/findings/") {
            continue;
        }
        let source = fs::read_to_string(path)?;
        let document = Document::parse(&source).map_err(|error| {
            DebtMigrationError::Preflight(format!(
                "malformed debt source {}: {error}",
                path.display()
            ))
        })?;
        let id = document.value("id").ok_or_else(|| {
            DebtMigrationError::Preflight(format!("missing id: {}", path.display()))
        })?;
        let suffix = id.strip_prefix("FINDING-").ok_or_else(|| {
            DebtMigrationError::Preflight(format!("legacy durable id must use FINDING-*: {id}"))
        })?;
        let new = format!("DEBT-{suffix}");
        if ids.insert(id.clone(), new).is_some() {
            return Err(DebtMigrationError::Preflight(format!(
                "duplicate durable id {id}"
            )));
        }
    }
    Ok(ids)
}

fn migrated_path(path: &str, ids: &BTreeMap<String, String>) -> String {
    let mut migrated = path.replace(".lmbrain/findings/", ".lmbrain/debts/");
    if migrated == ".lmbrain/templates/finding.md" {
        migrated = ".lmbrain/templates/debt.md".into();
    }
    for (old, new) in ids {
        migrated = migrated.replace(old, new);
    }
    migrated
}

fn replace_token(source: &str, old: &str, new: &str) -> String {
    Regex::new(&format!(r"\b{}\b", regex::escape(old)))
        .unwrap()
        .replace_all(source, new)
        .into_owned()
}

fn regular_files(root: &Path) -> Result<Vec<PathBuf>, DebtMigrationError> {
    fn walk(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), DebtMigrationError> {
        let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(DebtMigrationError::Preflight(format!(
                    "symlinks are not migrated: {}",
                    entry.path().display()
                )));
            }
            if file_type.is_dir() {
                walk(&entry.path(), output)?;
            } else if file_type.is_file() {
                output.push(entry.path());
            }
        }
        Ok(())
    }
    let mut output = Vec::new();
    walk(root, &mut output)?;
    Ok(output)
}

pub(crate) fn copy_tree(source: &Path, destination: &Path) -> Result<(), DebtMigrationError> {
    fs::create_dir(destination)?;
    for path in regular_files(source)? {
        let relative = path.strip_prefix(source).map_err(|_| {
            DebtMigrationError::Preflight(format!("unsafe copy path: {}", path.display()))
        })?;
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, target)?;
    }
    Ok(())
}

fn remove_empty_tree(path: &Path) -> Result<(), DebtMigrationError> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            remove_empty_tree(&entry.path())?;
        }
    }
    if fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

fn validate_staged_workspace(root: &Path) -> Result<(), DebtMigrationError> {
    let brain = root.join(".lmbrain");
    if brain.join("findings").exists() {
        return Err(DebtMigrationError::Preflight(
            "legacy findings directory remains after migration".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    for path in regular_files(&brain)? {
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        if source.contains("finding_events") || source.contains("waived=FINDING-") {
            return Err(DebtMigrationError::Preflight(format!(
                "operational legacy residue remains in {}",
                path.display()
            )));
        }
        if source.trim_start().starts_with("---") {
            let document = Document::parse(&source).map_err(|error| {
                DebtMigrationError::Preflight(format!("malformed staged artifact: {error}"))
            })?;
            if let Some(id) = document.value("id") {
                if !ids.insert(id.clone()) {
                    return Err(DebtMigrationError::Preflight(format!("duplicate id {id}")));
                }
                if id.starts_with("DEBT-") {
                    crate::validate_debt_document(root, &path, &document)
                        .map_err(|error| DebtMigrationError::Preflight(error.to_string()))?;
                }
            }
        }
    }
    Ok(())
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn legacy_workspace(root: &Path) {
        fs::create_dir_all(root.join(".lmbrain/findings/open")).unwrap();
        fs::create_dir_all(root.join(".lmbrain/reviews/accepted")).unwrap();
        fs::write(root.join(".lmbrain/VERSION"), "2.1.2\n").unwrap();
        fs::write(
            root.join(".lmbrain/findings/open/FINDING-001-sample.md"),
            "---\nid: FINDING-001\ntitle: Sample\nstatus: open\ncategory: correctness\nseverity: medium\ncreated: 2026-08-13\nupdated: 2026-08-13\norigin_artifact: REVIEW-001\norigin_ref: FINDING-07\nrelated_specs: []\nrelated_reviews: [REVIEW-001]\nrelated_decisions: []\ntarget_specs: []\nblocked_by: []\nresolution_refs: []\nfinding_events: []\n---\n## Statement\nSample\n",
        )
        .unwrap();
        fs::write(
            root.join(".lmbrain/reviews/accepted/REVIEW-001.md"),
            "---\nid: REVIEW-001\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\n---\n## Findings\n- FINDING-07 local issue\n",
        )
        .unwrap();
    }

    #[test]
    fn preview_is_deterministic_and_read_only() {
        let directory = tempdir().unwrap();
        legacy_workspace(directory.path());
        let first = debt_migration_preview(directory.path()).unwrap();
        let second = debt_migration_preview(directory.path()).unwrap();
        assert_eq!(first, second);
        assert!(!first.mutated);
        assert!(directory
            .path()
            .join(".lmbrain/findings/open/FINDING-001-sample.md")
            .exists());
        assert_eq!(
            first.review_id_mappings[".lmbrain/reviews/accepted/REVIEW-001.md"]["FINDING-07"],
            "RF-001"
        );
    }

    #[test]
    fn migration_is_confirmed_digest_bound_and_complete() {
        let directory = tempdir().unwrap();
        legacy_workspace(directory.path());
        let preview = debt_migration_preview(directory.path()).unwrap();
        assert!(matches!(
            debt_migrate(directory.path(), &preview.digest, false),
            Err(DebtMigrationError::ConfirmationRequired)
        ));
        let result = debt_migrate(directory.path(), &preview.digest, true).unwrap();
        assert_eq!(result.version, DEBT_CONTRACT_VERSION);
        let debt = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/debts/open/DEBT-001-sample.md"),
        )
        .unwrap();
        assert!(debt.contains("id: DEBT-001"));
        assert!(debt.contains("debt_events"));
        assert!(debt.contains("origin_ref: RF-001"));
        let review = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-001.md"),
        )
        .unwrap();
        assert!(review.contains("## Review findings"));
        assert!(review.contains("RF-001 local issue"));
        assert!(!directory.path().join(".lmbrain/findings").exists());
        assert_eq!(
            fs::read_to_string(directory.path().join(".lmbrain/VERSION"))
                .unwrap()
                .trim(),
            DEBT_CONTRACT_VERSION
        );
    }

    #[test]
    fn malformed_or_ambiguous_input_fails_without_writes() {
        let malformed = tempdir().unwrap();
        legacy_workspace(malformed.path());
        fs::write(
            malformed
                .path()
                .join(".lmbrain/findings/open/FINDING-001-sample.md"),
            "broken",
        )
        .unwrap();
        assert!(debt_migration_preview(malformed.path()).is_err());
        assert!(malformed.path().join(".lmbrain/findings").exists());

        let ambiguous = tempdir().unwrap();
        legacy_workspace(ambiguous.path());
        fs::write(
            ambiguous
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-001.md"),
            "---\nid: REVIEW-001\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\n---\n## Review findings\n[[FINDING-001]]\n",
        )
        .unwrap();
        assert!(debt_migration_preview(ambiguous.path()).is_err());
        assert!(ambiguous.path().join(".lmbrain/findings").exists());
    }
}
