use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    frontmatter::Document,
    path::PathGuard,
    workspace_index::{is_artifact_markdown_path, is_artifact_scaffolding_path},
};

pub const DEBT_MIGRATION_SCHEMA_VERSION: &str = "2";
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
    pub scaffolding_items: Vec<DebtMigrationItem>,
    pub review_id_mappings: BTreeMap<String, BTreeMap<String, String>>,
    pub reference_mappings: Vec<DebtMigrationReference>,
    pub mutated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DebtMigrationReference {
    pub path: String,
    pub token: String,
    pub replacement: String,
    pub classification: String,
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
    scaffolding: bool,
}

#[derive(Debug)]
struct MigrationPlan {
    preview: DebtMigrationPreview,
    writes: Vec<PlannedWrite>,
}

#[derive(Debug, Default)]
struct ReviewMigrationAnalysis {
    id_mappings: BTreeMap<String, BTreeMap<String, String>>,
    references: Vec<DebtMigrationReference>,
    issues: Vec<String>,
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
                if write.scaffolding && fs::read(&to)? == write.content {
                    fs::remove_file(&from)?;
                    continue;
                }
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
    let (durable_ids, mut preflight_issues) = collect_durable_ids(root, &brain, &files);
    let review_analysis = collect_review_id_mappings(root, &brain, &files, &durable_ids);
    preflight_issues.extend(review_analysis.issues);
    if !preflight_issues.is_empty() {
        preflight_issues.sort();
        preflight_issues.dedup();
        return Err(DebtMigrationError::Preflight(format!(
            "{} issue(s):\n- {}",
            preflight_issues.len(),
            preflight_issues.join("\n- ")
        )));
    }
    let review_id_mappings = review_analysis.id_mappings;
    let mut reference_mappings = review_analysis.references;
    let mut writes = Vec::new();

    for path in files {
        let relative = path.strip_prefix(root).map_err(|_| {
            DebtMigrationError::Preflight(format!("unsafe path: {}", path.display()))
        })?;
        let relative_text = slash(relative);
        let bytes = fs::read(&path)?;
        let is_markdown = path.extension().and_then(|value| value.to_str()) == Some("md");
        let scaffolding = is_artifact_scaffolding_path(&brain, &path);
        let mut content = if is_markdown {
            String::from_utf8(bytes.clone()).map_err(|_| {
                DebtMigrationError::Preflight(format!("non-UTF-8 Markdown: {relative_text}"))
            })?
        } else {
            String::new()
        };
        let mut changes = Vec::new();

        if is_artifact_markdown_path(&brain, &path) && content.trim_start().starts_with("---") {
            Document::parse(&content).map_err(|error| {
                DebtMigrationError::Preflight(format!(
                    "malformed artifact {relative_text}: {error}"
                ))
            })?;
        }

        if relative_text.starts_with(".lmbrain/reviews/") && is_markdown {
            let mapping = review_id_mappings
                .get(&relative_text)
                .cloned()
                .unwrap_or_default();
            content = replace_classified_references(&content, &mapping);
            if content.contains("## Findings") {
                content = content.replace("## Findings", "## Review findings");
                changes.push("canonical review heading".into());
            }
            if !mapping.is_empty() {
                changes.push("review-local RF identifiers".into());
            }
        }

        if is_markdown {
            if relative_text.starts_with(".lmbrain/findings/")
                && is_artifact_markdown_path(&brain, &path)
            {
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
            let (durable_content, durable_references) =
                replace_durable_references(&content, &durable_ids);
            if durable_content != content {
                changes.push("durable artifact references".into());
                for (token, replacement) in durable_references {
                    reference_mappings.push(DebtMigrationReference {
                        path: relative_text.clone(),
                        token,
                        replacement,
                        classification: "durable".into(),
                    });
                }
                content = durable_content;
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
            changes.push(if scaffolding {
                "scaffolding path".into()
            } else {
                "artifact path".into()
            });
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
                scaffolding,
            });
        }
    }
    writes.sort_by(|left, right| left.item.path.cmp(&right.item.path));
    reference_mappings.sort();
    reference_mappings.dedup();
    let items = writes
        .iter()
        .filter(|write| !write.scaffolding)
        .map(|write| write.item.clone())
        .collect::<Vec<_>>();
    let scaffolding_items = writes
        .iter()
        .filter(|write| write.scaffolding)
        .map(|write| write.item.clone())
        .collect::<Vec<_>>();
    let digest_source = serde_json::to_vec(&(
        source_version.clone(),
        &items,
        &scaffolding_items,
        &review_id_mappings,
        &reference_mappings,
    ))
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
            scaffolding_items,
            review_id_mappings,
            reference_mappings,
            mutated: false,
        },
        writes,
    })
}

fn collect_review_id_mappings(
    root: &Path,
    brain: &Path,
    files: &[PathBuf],
    durable_ids: &BTreeMap<String, String>,
) -> ReviewMigrationAnalysis {
    let token = migration_token_regex();
    let qualified = Regex::new(r"^REVIEW-(\d+)-FINDING-(\d+)$").unwrap();
    let mut output = BTreeMap::new();
    let mut references = Vec::new();
    let mut issues = Vec::new();
    for path in files {
        let Ok(stripped) = path.strip_prefix(root) else {
            issues.push(format!("unsafe review path: {}", path.display()));
            continue;
        };
        let relative = slash(stripped);
        if !relative.starts_with(".lmbrain/reviews/") || !is_artifact_markdown_path(brain, path) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => {
                issues.push(format!("unreadable review {relative}: {error}"));
                continue;
            }
        };
        let document = match Document::parse(&content) {
            Ok(document) => document,
            Err(error) => {
                issues.push(format!("malformed review {relative}: {error}"));
                continue;
            }
        };
        let Some(review_id) = document.value("id") else {
            issues.push(format!("missing review id in {relative}"));
            continue;
        };
        let local_bare_tokens = review_local_bare_tokens(&document.body);
        let mut classified = Vec::new();
        let mut local_keys = BTreeMap::new();
        let mut next_local = 1usize;

        for value in token.find_iter(&content) {
            let legacy = value.as_str().to_string();
            if let Some(captures) = qualified.captures(&legacy) {
                let qualifier = format!("REVIEW-{}", &captures[1]);
                if qualifier != review_id {
                    issues.push(format!(
                        "qualified review-local reference {legacy} in {relative} belongs to {qualifier}, not {review_id}"
                    ));
                    continue;
                }
                let key = local_identity(&legacy);
                let replacement = local_keys
                    .entry(key)
                    .or_insert_with(|| {
                        let replacement = format!("RF-{next_local:03}");
                        next_local += 1;
                        replacement
                    })
                    .clone();
                classified.push((legacy, replacement, "review-local"));
                continue;
            }

            let durable = durable_ids.get(&legacy);
            let local = local_bare_tokens.contains(&legacy);
            match (durable, local) {
                (Some(_), true) => issues.push(format!(
                    "ambiguous durable/local finding reference {legacy} in {relative}: it resolves to a durable artifact and is declared in this review's findings section"
                )),
                (Some(replacement), false) => {
                    classified.push((legacy, replacement.clone(), "durable"));
                }
                (None, true) => {
                    let key = local_identity(&legacy);
                    let replacement = local_keys
                        .entry(key)
                        .or_insert_with(|| {
                            let replacement = format!("RF-{next_local:03}");
                            next_local += 1;
                            replacement
                        })
                        .clone();
                    classified.push((legacy, replacement, "review-local"));
                }
                (None, false) => issues.push(format!(
                    "unresolved durable/local finding reference {legacy} in {relative}"
                )),
            }
        }

        let mut mapping = BTreeMap::new();
        for (legacy, replacement, classification) in classified {
            if classification == "review-local" {
                mapping.insert(legacy.clone(), replacement.clone());
            }
            references.push(DebtMigrationReference {
                path: relative.clone(),
                token: legacy,
                replacement,
                classification: classification.into(),
            });
        }
        if !mapping.is_empty() {
            output.insert(relative, mapping);
        }
    }
    ReviewMigrationAnalysis {
        id_mappings: output,
        references,
        issues,
    }
}

fn collect_durable_ids(
    root: &Path,
    brain: &Path,
    files: &[PathBuf],
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut ids = BTreeMap::new();
    let mut issues = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .map(slash)
            .unwrap_or_else(|_| slash(path));
        if !relative.starts_with(".lmbrain/findings/") || !is_artifact_markdown_path(brain, path) {
            continue;
        }
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                issues.push(format!("unreadable debt source {relative}: {error}"));
                continue;
            }
        };
        let document = match Document::parse(&source) {
            Ok(document) => document,
            Err(error) => {
                issues.push(format!("malformed debt source {relative}: {error}"));
                continue;
            }
        };
        let Some(id) = document.value("id") else {
            issues.push(format!("missing id in debt source {relative}"));
            continue;
        };
        let Some(suffix) = id.strip_prefix("FINDING-") else {
            issues.push(format!(
                "legacy durable id must use FINDING-* in {relative}: {id}"
            ));
            continue;
        };
        if suffix.parse::<u64>().is_err() {
            issues.push(format!(
                "legacy durable id must have a numeric FINDING-* suffix in {relative}: {id}"
            ));
            continue;
        }
        let new = format!("DEBT-{suffix}");
        if ids.insert(id.clone(), new).is_some() {
            issues.push(format!("duplicate durable id {id}"));
        }
    }
    (ids, issues)
}

fn migration_token_regex() -> Regex {
    Regex::new(r"\bREVIEW-\d+-FINDING-\d+\b|\b(?:FINDING|F)-\d+\b").unwrap()
}

fn review_local_bare_tokens(body: &str) -> BTreeSet<String> {
    let token = migration_token_regex();
    let mut output = BTreeSet::new();
    let mut in_findings = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_findings = matches!(heading.trim(), "Findings" | "Review findings");
            continue;
        }
        if !in_findings {
            continue;
        }
        for value in token.find_iter(line) {
            let candidate = value.as_str();
            if !candidate.starts_with("REVIEW-") {
                output.insert(candidate.to_string());
            }
        }
    }
    output
}

fn local_identity(token: &str) -> String {
    let suffix = token
        .rsplit('-')
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.to_string())
        .unwrap_or_else(|| token.to_string());
    format!("local-{suffix}")
}

fn replace_durable_references(
    source: &str,
    ids: &BTreeMap<String, String>,
) -> (String, Vec<(String, String)>) {
    let token = migration_token_regex();
    let mut references = BTreeSet::new();
    let replaced = token
        .replace_all(source, |captures: &regex::Captures<'_>| {
            let legacy = &captures[0];
            if legacy.starts_with("REVIEW-") {
                return legacy.to_string();
            }
            if let Some(replacement) = ids.get(legacy) {
                references.insert((legacy.to_string(), replacement.clone()));
                replacement.clone()
            } else {
                legacy.to_string()
            }
        })
        .into_owned();
    (replaced, references.into_iter().collect())
}

fn replace_classified_references(source: &str, mappings: &BTreeMap<String, String>) -> String {
    migration_token_regex()
        .replace_all(source, |captures: &regex::Captures<'_>| {
            mappings
                .get(&captures[0])
                .cloned()
                .unwrap_or_else(|| captures[0].to_string())
        })
        .into_owned()
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

    fn write_durable(root: &Path, id: &str) {
        let directory = root.join(".lmbrain/findings/resolved");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{id}-sample.md")),
            format!(
                "---\nid: {id}\ntitle: Sample {id}\nstatus: resolved\ncategory: correctness\nseverity: medium\ncreated: 2026-08-13\nupdated: 2026-08-13\norigin_artifact: null\norigin_ref: null\nrelated_specs: []\nrelated_reviews: []\nrelated_decisions: []\ntarget_specs: []\nblocked_by: []\nresolution_refs: []\nfinding_events: []\n---\n## Statement\nSample\n"
            ),
        )
        .unwrap();
    }

    fn write_review(root: &Path, id: &str, body: &str) {
        let directory = root.join(".lmbrain/reviews/accepted");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\n---\n{body}\n"
            ),
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

    #[test]
    fn preview_ignores_all_legacy_findings_scaffolding_as_artifacts() {
        let directory = tempdir().unwrap();
        legacy_workspace(directory.path());
        for status in [
            "",
            "open",
            "planned",
            "deferred",
            "resolved",
            "accepted-risk",
            "superseded",
        ] {
            let parent = directory.path().join(".lmbrain/findings").join(status);
            fs::create_dir_all(&parent).unwrap();
            fs::write(parent.join("README.md"), format!("# {status} findings\n")).unwrap();
        }

        let preview = debt_migration_preview(directory.path()).unwrap();
        assert!(preview
            .items
            .iter()
            .all(|item| !item.path.ends_with("README.md")));
        assert_eq!(preview.scaffolding_items.len(), 7);
        assert!(preview.scaffolding_items.iter().all(|item| {
            item.path.ends_with("README.md")
                && item
                    .changes
                    .iter()
                    .any(|change| change == "scaffolding path")
        }));
        assert!(preview
            .reference_mappings
            .iter()
            .all(|reference| !reference.path.ends_with("README.md")));
    }

    #[test]
    fn migration_reconciles_only_identical_kit_installed_scaffolding() {
        let directory = tempdir().unwrap();
        legacy_workspace(directory.path());
        let findings_readme = directory.path().join(".lmbrain/findings/open/README.md");
        let debts_readme = directory.path().join(".lmbrain/debts/open/README.md");
        fs::create_dir_all(debts_readme.parent().unwrap()).unwrap();
        fs::write(&findings_readme, "# Shared scaffolding\n").unwrap();
        fs::write(&debts_readme, "# Shared scaffolding\n").unwrap();

        let preview = debt_migration_preview(directory.path()).unwrap();
        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        assert!(!directory.path().join(".lmbrain/findings").exists());
        assert_eq!(
            fs::read_to_string(debts_readme).unwrap(),
            "# Shared scaffolding\n"
        );

        let conflicting = tempdir().unwrap();
        legacy_workspace(conflicting.path());
        let findings_readme = conflicting.path().join(".lmbrain/findings/open/README.md");
        let debts_readme = conflicting.path().join(".lmbrain/debts/open/README.md");
        fs::create_dir_all(debts_readme.parent().unwrap()).unwrap();
        fs::write(&findings_readme, "# Legacy scaffolding\n").unwrap();
        fs::write(&debts_readme, "# Kit scaffolding\n").unwrap();
        let preview = debt_migration_preview(conflicting.path()).unwrap();
        let error = debt_migrate(conflicting.path(), &preview.digest, true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("migration destination already exists"));
        assert!(conflicting.path().join(".lmbrain/findings").exists());
    }

    #[test]
    fn bare_wikilinks_resolving_to_durable_artifacts_are_not_ambiguous() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        fs::write(directory.path().join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
        write_durable(directory.path(), "FINDING-009");
        write_durable(directory.path(), "FINDING-011");
        write_review(
            directory.path(),
            "REVIEW-019",
            "## Findings\n\n<!-- Stable form: FINDING-ID | category=... -->\n\nNone.\n\n## Context\nSee [[FINDING-009]] and [[FINDING-011]].",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let review = preview
            .items
            .iter()
            .find(|item| item.path.ends_with("REVIEW-019.md"))
            .unwrap();
        assert!(review
            .changes
            .iter()
            .any(|change| change == "durable artifact references"));
        for (legacy, replacement) in [("FINDING-009", "DEBT-009"), ("FINDING-011", "DEBT-011")] {
            assert!(preview.reference_mappings.iter().any(|reference| {
                reference.path.ends_with("REVIEW-019.md")
                    && reference.token == legacy
                    && reference.replacement == replacement
                    && reference.classification == "durable"
            }));
        }
    }

    #[test]
    fn qualified_local_tokens_are_consumed_before_durable_bare_tokens() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        fs::write(directory.path().join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
        write_durable(directory.path(), "FINDING-003");
        write_durable(directory.path(), "FINDING-042");
        write_review(
            directory.path(),
            "REVIEW-007",
            "## Findings\n\n- REVIEW-007-FINDING-003 local issue\n\n## Context\nSee REVIEW-007-FINDING-003 and [[FINDING-042]].",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let mappings = &preview.review_id_mappings[".lmbrain/reviews/accepted/REVIEW-007.md"];
        assert_eq!(mappings["REVIEW-007-FINDING-003"], "RF-001");
        assert!(!mappings.contains_key("FINDING-003"));
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.token == "REVIEW-007-FINDING-003"
                && reference.replacement == "RF-001"
                && reference.classification == "review-local"
        }));
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.token == "FINDING-042"
                && reference.replacement == "DEBT-042"
                && reference.classification == "durable"
        }));
    }

    #[test]
    fn genuine_bare_collision_still_fails_closed() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".lmbrain")).unwrap();
        fs::write(directory.path().join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
        write_durable(directory.path(), "FINDING-005");
        write_review(
            directory.path(),
            "REVIEW-005",
            "## Findings\n\n- FINDING-005 local issue\n",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("ambiguous durable/local finding reference FINDING-005"));
        assert!(directory.path().join(".lmbrain/findings").exists());
    }

    #[test]
    fn preflight_reports_all_source_and_review_issues_at_once() {
        let directory = tempdir().unwrap();
        let findings = directory.path().join(".lmbrain/findings/open");
        fs::create_dir_all(&findings).unwrap();
        fs::write(directory.path().join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
        fs::write(findings.join("FINDING-001-broken.md"), "broken one\n").unwrap();
        fs::write(findings.join("FINDING-002-broken.md"), "broken two\n").unwrap();
        write_review(
            directory.path(),
            "REVIEW-010",
            "## Findings\n\nNone.\n\n## Context\nSee [[FINDING-010]].",
        );
        write_review(
            directory.path(),
            "REVIEW-011",
            "## Findings\n\nNone.\n\n## Context\nSee [[FINDING-011]].",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        for expected in [
            "FINDING-001-broken.md",
            "FINDING-002-broken.md",
            "REVIEW-010.md",
            "REVIEW-011.md",
        ] {
            assert!(error.contains(expected), "missing {expected} from {error}");
        }
        assert!(error.contains("4 issue(s)"));
    }
}
