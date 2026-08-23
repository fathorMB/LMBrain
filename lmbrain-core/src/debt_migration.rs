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

pub const DEBT_MIGRATION_SCHEMA_VERSION: &str = "3";
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
    pub occurrences: usize,
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
    /// Corpus-wide symbol table: every review id mapped to the finding numbers that review
    /// declares in qualified form. A qualified reference names its own scope, so it stays
    /// decidable from this table no matter which document it appears in.
    qualified_declarations: BTreeMap<String, BTreeSet<u64>>,
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
    let qualified_declarations = review_analysis.qualified_declarations;
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

        let inventoried_by_review_analysis = relative_text.starts_with(".lmbrain/reviews/")
            && is_artifact_markdown_path(&brain, &path);
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
                        let replacement = bare_review_local_identifier(replacement);
                        content = replace_token(&content, &local, &replacement);
                        changes.push("review-origin RF reference".into());
                    }
                }
            }
            // Review artifacts already had every qualified token classified and rewritten above.
            // Everywhere else the qualifier is still the least ambiguous form in the corpus, so a
            // decidable qualified reference is carried into its `REVIEW-NNN-RF-MMM` target form
            // instead of being left behind as a stale `FINDING-*` token.
            if !relative_text.starts_with(".lmbrain/reviews/") {
                let (qualified_content, qualified_references) =
                    replace_qualified_review_references(&content, &qualified_declarations);
                if qualified_content != content {
                    changes.push("qualified review-local references".into());
                    for (token, replacement, occurrences) in qualified_references {
                        reference_mappings.push(DebtMigrationReference {
                            path: relative_text.clone(),
                            token,
                            replacement,
                            classification: "cross-review-local".into(),
                            occurrences,
                        });
                    }
                    content = qualified_content;
                }
            }
            let (durable_content, durable_references) =
                replace_durable_references(&content, &durable_ids);
            if durable_content != content {
                changes.push("durable artifact references".into());
                // Review artifacts are inventoried once, by the review analysis, which already
                // classified every token in the file. Re-recording them here would double count.
                if !inventoried_by_review_analysis {
                    for (token, replacement, occurrences) in durable_references {
                        reference_mappings.push(DebtMigrationReference {
                            path: relative_text.clone(),
                            token,
                            replacement,
                            classification: "durable".into(),
                            occurrences,
                        });
                    }
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
    let reference_mappings = aggregate_references(reference_mappings);
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
    let qualified = qualified_token_regex();
    let mut output = BTreeMap::new();
    let mut references = Vec::new();
    let mut issues = Vec::new();

    // Pass one builds the corpus-wide symbol table. A qualified reference carries its own scope,
    // so it has to be resolvable against the review that declares it rather than against whatever
    // document happens to cite it.
    let mut reviews: Vec<(String, String, String, ReviewLocalDeclarations)> = Vec::new();
    let mut qualified_declarations: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
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
        let declarations = review_local_declarations(&content, &document.body, &review_id);
        qualified_declarations
            .entry(review_id.clone())
            .or_default()
            .extend(declarations.qualified_numbers.iter().copied());
        reviews.push((relative, content, review_id, declarations));
    }

    for (relative, content, review_id, declarations) in &reviews {
        let (relative, content, review_id) =
            (relative.clone(), content.as_str(), review_id.clone());
        let mut classified = Vec::new();

        for value in token.find_iter(content) {
            let legacy = value.as_str().to_string();
            if let Some(captures) = qualified.captures(&legacy) {
                let qualifier = format!("REVIEW-{}", &captures[1]);
                let Some(number) = token_number(&legacy) else {
                    issues.push(format!(
                        "unreadable review-local finding number in {legacy} in {relative}"
                    ));
                    continue;
                };
                // Rule 2: a qualified token names its own scope, so it resolves against the
                // review that declares it whether or not that is the citing review. The qualifier
                // is preserved in the target form so the reference keeps surviving the move out
                // of its home document; emitting a bare `RF-MMM` here would silently rebind the
                // reference to the citing review's own finding of that number.
                match qualified_declarations.get(&qualifier) {
                    None => issues.push(format!(
                        "qualified review-local reference {legacy} in {relative} names {qualifier}, which declares no findings in this workspace"
                    )),
                    Some(numbers) if !numbers.contains(&number) => issues.push(format!(
                        "qualified review-local reference {legacy} in {relative} names a finding {qualifier} never declares"
                    )),
                    Some(_) => {
                        let classification = if qualifier == review_id {
                            "review-local"
                        } else {
                            "cross-review-local"
                        };
                        classified.push((
                            legacy,
                            qualified_review_local_identifier(&qualifier, number),
                            classification,
                        ))
                    }
                }
                continue;
            }

            let number = token_number(&legacy);
            let declared_qualified =
                number.is_some_and(|number| declarations.qualified_numbers.contains(&number));
            let declared_bare =
                number.is_some_and(|number| declarations.bare_declared_numbers.contains(&number));
            let durable = durable_ids.get(&legacy);

            match (number, declared_qualified, declared_bare, durable) {
                // Rule 5 guard: the same number declared in both forms while a durable artifact of
                // that number exists is genuinely two objects. Fail closed.
                (Some(_), true, true, Some(_)) => issues.push(format!(
                    "ambiguous durable/local finding reference {legacy} in {relative}: this review declares the same number in both the qualified {review_id}-{legacy} form and the bare form while a durable artifact of that number exists"
                )),
                // Rule 3: the declaring review's local symbol table wins over durable resolution,
                // so an overlapping durable range can never silently capture a local reference.
                (Some(number), true, _, _) => {
                    classified.push((legacy, review_local_identifier(number), "review-local"))
                }
                // Rule 4: a bare token backed by a durable artifact is that durable artifact,
                // whether it appears as a declaration, in prose, or inside a wikilink.
                (_, _, _, Some(replacement)) => {
                    classified.push((legacy, replacement.clone(), "durable"))
                }
                // A bare declaration with no durable artifact is this review's own local finding.
                (Some(number), _, true, None) => {
                    classified.push((legacy, review_local_identifier(number), "review-local"))
                }
                // Rule 5: resolves to nothing.
                _ => issues.push(format!(
                    "unresolved durable/local finding reference {legacy} in {relative}"
                )),
            }
        }

        let mut mapping = BTreeMap::new();
        let mut occurrences: BTreeMap<(String, String, &'static str), usize> = BTreeMap::new();
        for (legacy, replacement, classification) in classified {
            if classification != "durable" {
                mapping.insert(legacy.clone(), replacement.clone());
            }
            *occurrences
                .entry((legacy, replacement, classification))
                .or_default() += 1;
        }
        for ((legacy, replacement, classification), occurrences) in occurrences {
            references.push(DebtMigrationReference {
                path: relative.clone(),
                token: legacy,
                replacement,
                classification: classification.into(),
                occurrences,
            });
        }
        if !mapping.is_empty() {
            output.insert(relative, mapping);
        }
    }
    ReviewMigrationAnalysis {
        id_mappings: output,
        qualified_declarations,
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

fn qualified_token_regex() -> Regex {
    Regex::new(r"^REVIEW-(\d+)-FINDING-(\d+)$").unwrap()
}

/// The finding numbers a single review declares, split by declaration form.
///
/// `qualified_numbers` is the review's local symbol table: bare references to those numbers
/// inside this review resolve to the review's own findings before any durable resolution is
/// attempted. `bare_declared_numbers` records the numbers declared bare in the review's findings
/// section, which is what distinguishes the ordinary promotion convention (bare declaration backed
/// by a durable artifact) from a genuine two-object collision (both forms plus a durable artifact).
#[derive(Debug, Default)]
struct ReviewLocalDeclarations {
    qualified_numbers: BTreeSet<u64>,
    bare_declared_numbers: BTreeSet<u64>,
}

fn review_local_declarations(
    content: &str,
    body: &str,
    review_id: &str,
) -> ReviewLocalDeclarations {
    let token = migration_token_regex();
    let qualified = qualified_token_regex();
    let mut output = ReviewLocalDeclarations::default();

    for value in token.find_iter(content) {
        let candidate = value.as_str();
        let Some(captures) = qualified.captures(candidate) else {
            continue;
        };
        if format!("REVIEW-{}", &captures[1]) != review_id {
            continue;
        }
        if let Some(number) = token_number(candidate) {
            output.qualified_numbers.insert(number);
        }
    }

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
            if candidate.starts_with("REVIEW-") {
                continue;
            }
            if let Some(number) = token_number(candidate) {
                output.bare_declared_numbers.insert(number);
            }
        }
    }

    output
}

fn token_number(token: &str) -> Option<u64> {
    token
        .rsplit('-')
        .next()
        .and_then(|value| value.parse().ok())
}

fn review_local_identifier(number: u64) -> String {
    format!("RF-{number:03}")
}

/// The qualifier-preserving target form for a reference that names its own review scope.
///
/// A qualified reference stays qualified through the migration so that it keeps meaning the same
/// finding wherever it is cited. Reducing it to a bare `RF-MMM` outside its home review would
/// rebind it to the citing review's own finding of that number, which is the corruption the
/// preflight guard exists to prevent.
fn qualified_review_local_identifier(review_id: &str, number: u64) -> String {
    format!("{review_id}-{}", review_local_identifier(number))
}

/// Strip the review qualifier from a review-local identifier.
///
/// `origin_ref` on a durable debt is already scoped by its sibling `origin_artifact`, and the debt
/// contract requires the bare `RF-*` form there, so a qualified replacement is normalised back.
fn bare_review_local_identifier(value: &str) -> String {
    Regex::new(r"^REVIEW-\d+-(RF-\d+)$")
        .unwrap()
        .captures(value)
        .map(|captures| captures[1].to_string())
        .unwrap_or_else(|| value.to_string())
}

/// Rewrite qualified review-local references that the corpus can decide, leaving the rest alone.
///
/// This runs over documents that are not themselves reviews. Those files are not subject to the
/// review preflight, so an undecidable qualified reference there is left untouched rather than
/// turned into a new blocking issue.
fn replace_qualified_review_references(
    source: &str,
    declarations: &BTreeMap<String, BTreeSet<u64>>,
) -> (String, Vec<DurableReplacement>) {
    let qualified = qualified_token_regex();
    let mut references: BTreeMap<(String, String), usize> = BTreeMap::new();
    let replaced = migration_token_regex()
        .replace_all(source, |captures: &regex::Captures<'_>| {
            let legacy = &captures[0];
            let Some(parts) = qualified.captures(legacy) else {
                return legacy.to_string();
            };
            let review_id = format!("REVIEW-{}", &parts[1]);
            let Some(number) = token_number(legacy) else {
                return legacy.to_string();
            };
            if !declarations
                .get(&review_id)
                .is_some_and(|numbers| numbers.contains(&number))
            {
                return legacy.to_string();
            }
            let replacement = qualified_review_local_identifier(&review_id, number);
            *references
                .entry((legacy.to_string(), replacement.clone()))
                .or_default() += 1;
            replacement
        })
        .into_owned();
    (
        replaced,
        references
            .into_iter()
            .map(|((legacy, replacement), occurrences)| (legacy, replacement, occurrences))
            .collect(),
    )
}

fn aggregate_references(references: Vec<DebtMigrationReference>) -> Vec<DebtMigrationReference> {
    let mut totals: BTreeMap<(String, String, String, String), usize> = BTreeMap::new();
    for reference in references {
        *totals
            .entry((
                reference.path,
                reference.token,
                reference.replacement,
                reference.classification,
            ))
            .or_default() += reference.occurrences;
    }
    totals
        .into_iter()
        .map(
            |((path, token, replacement, classification), occurrences)| DebtMigrationReference {
                path,
                token,
                replacement,
                classification,
                occurrences,
            },
        )
        .collect()
}

type DurableReplacement = (String, String, usize);

fn replace_durable_references(
    source: &str,
    ids: &BTreeMap<String, String>,
) -> (String, Vec<DurableReplacement>) {
    let token = migration_token_regex();
    let mut references: BTreeMap<(String, String), usize> = BTreeMap::new();
    let replaced = token
        .replace_all(source, |captures: &regex::Captures<'_>| {
            let legacy = &captures[0];
            if legacy.starts_with("REVIEW-") {
                return legacy.to_string();
            }
            if let Some(replacement) = ids.get(legacy) {
                *references
                    .entry((legacy.to_string(), replacement.clone()))
                    .or_default() += 1;
                replacement.clone()
            } else {
                legacy.to_string()
            }
        })
        .into_owned();
    (
        replaced,
        references
            .into_iter()
            .map(|((legacy, replacement), occurrences)| (legacy, replacement, occurrences))
            .collect(),
    )
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
        let directory = root.join(".lmbrain/findings/open");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{id}-sample.md")),
            format!(
                "---\nid: {id}\ntitle: Sample {id}\nstatus: open\ncategory: correctness\nseverity: medium\ncreated: 2026-08-13\nupdated: 2026-08-13\norigin_artifact: null\norigin_ref: null\nrelated_specs: []\nrelated_reviews: []\nrelated_decisions: []\ntarget_specs: []\nblocked_by: []\nresolution_refs: []\nfinding_events: []\n---\n## Statement\nSample\n"
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
            "RF-007"
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
        assert!(debt.contains("origin_ref: RF-007"));
        let review = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-001.md"),
        )
        .unwrap();
        assert!(review.contains("## Review findings"));
        assert!(review.contains("RF-007 local issue"));
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
            "---\nid: REVIEW-001\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\n---\n## Review findings\n- REVIEW-001-FINDING-001 declared qualified\n- FINDING-001 declared bare\n- FINDING-07 local issue\n",
        )
        .unwrap();
        let error = debt_migration_preview(ambiguous.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("ambiguous durable/local finding reference FINDING-001"),
            "unexpected error: {error}"
        );
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
        assert_eq!(mappings["REVIEW-007-FINDING-003"], "REVIEW-007-RF-003");
        assert!(!mappings.contains_key("FINDING-003"));
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.token == "REVIEW-007-FINDING-003"
                && reference.replacement == "REVIEW-007-RF-003"
                && reference.classification == "review-local"
                && reference.occurrences == 2
        }));
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.token == "FINDING-042"
                && reference.replacement == "DEBT-042"
                && reference.classification == "durable"
        }));
    }

    fn legacy_review_workspace(root: &Path) {
        fs::create_dir_all(root.join(".lmbrain")).unwrap();
        fs::write(root.join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
    }

    #[test]
    fn bare_prose_references_resolve_against_the_declaring_review() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-002",
            "## Findings\n\n- REVIEW-002-FINDING-001 | category=correctness\n- REVIEW-002-FINDING-002 | category=correctness\n\n## Context\nFINDING-001 remains open; FINDING-002 was fixed. FINDING-001 again.",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let mappings = &preview.review_id_mappings[".lmbrain/reviews/accepted/REVIEW-002.md"];
        assert_eq!(mappings["REVIEW-002-FINDING-001"], "REVIEW-002-RF-001");
        assert_eq!(mappings["FINDING-001"], "RF-001");
        assert_eq!(mappings["REVIEW-002-FINDING-002"], "REVIEW-002-RF-002");
        assert_eq!(mappings["FINDING-002"], "RF-002");
        assert!(preview
            .reference_mappings
            .iter()
            .all(|reference| reference.classification == "review-local"));
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.token == "FINDING-001"
                && reference.replacement == "RF-001"
                && reference.occurrences == 2
        }));
    }

    #[test]
    fn bare_declarations_backed_by_durable_artifacts_are_durable() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_durable(directory.path(), "FINDING-013");
        write_durable(directory.path(), "FINDING-014");
        write_review(
            directory.path(),
            "REVIEW-021",
            "## Findings\n\n- FINDING-013 | category=correctness\n- FINDING-014 | category=process\n",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        assert!(!preview
            .review_id_mappings
            .contains_key(".lmbrain/reviews/accepted/REVIEW-021.md"));
        for (legacy, replacement) in [("FINDING-013", "DEBT-013"), ("FINDING-014", "DEBT-014")] {
            assert!(
                preview.reference_mappings.iter().any(|reference| {
                    reference.path.ends_with("REVIEW-021.md")
                        && reference.token == legacy
                        && reference.replacement == replacement
                        && reference.classification == "durable"
                        && reference.occurrences == 1
                }),
                "missing durable mapping for {legacy}"
            );
        }
    }

    #[test]
    fn same_number_declared_in_both_forms_still_fails_closed() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_durable(directory.path(), "FINDING-013");
        write_review(
            directory.path(),
            "REVIEW-021",
            "## Findings\n\n- REVIEW-021-FINDING-013 | category=correctness\n- FINDING-013 | category=correctness\n",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("ambiguous durable/local finding reference FINDING-013"),
            "unexpected error: {error}"
        );
        assert!(directory.path().join(".lmbrain/findings").exists());
    }

    #[test]
    fn overlapping_local_and_durable_ranges_bind_local_first() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_durable(directory.path(), "FINDING-003");
        write_review(
            directory.path(),
            "REVIEW-030",
            "## Findings\n\n- REVIEW-030-FINDING-003 | category=correctness\n\n## Context\nFINDING-003 is still open in this review.",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let mappings = &preview.review_id_mappings[".lmbrain/reviews/accepted/REVIEW-030.md"];
        assert_eq!(mappings["REVIEW-030-FINDING-003"], "REVIEW-030-RF-003");
        assert_eq!(mappings["FINDING-003"], "RF-003");
        assert!(preview
            .reference_mappings
            .iter()
            .filter(|reference| reference.path.ends_with("REVIEW-030.md"))
            .all(|reference| reference.classification == "review-local"));

        let review = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-030.md"),
        )
        .unwrap();
        assert!(review.contains("FINDING-003"));
        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let migrated = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-030.md"),
        )
        .unwrap();
        assert!(migrated.contains("RF-003 is still open"));
        assert!(!migrated.contains("DEBT-003 is still open"));
    }

    #[test]
    fn unresolved_bare_references_still_fail_closed() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-031",
            "## Findings\n\nNone.\n\n## Context\nSee [[FINDING-099]].",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unresolved durable/local finding reference FINDING-099"),
            "unexpected error: {error}"
        );
    }

    fn write_review_with_events(root: &Path, id: &str, events: &str, body: &str) {
        let directory = root.join(".lmbrain/reviews/accepted");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\nreview_events:\n{events}---\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn qualified_cross_review_references_resolve_against_the_declaring_review() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-003 | category=usability | severity=medium\n",
        );
        write_review(
            directory.path(),
            "REVIEW-010",
            "## Findings\n\nNone.\n\n## Context\nREVIEW-009-FINDING-003, carried into this spec, is closed.",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        assert_eq!(
            preview.review_id_mappings[".lmbrain/reviews/accepted/REVIEW-010.md"]
                ["REVIEW-009-FINDING-003"],
            "REVIEW-009-RF-003"
        );
        let reference = preview
            .reference_mappings
            .iter()
            .find(|reference| reference.path.ends_with("REVIEW-010.md"))
            .expect("citing review is inventoried");
        assert_eq!(reference.token, "REVIEW-009-FINDING-003");
        assert_eq!(reference.replacement, "REVIEW-009-RF-003");
        assert_eq!(reference.classification, "cross-review-local");

        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let citing = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-010.md"),
        )
        .unwrap();
        assert!(citing.contains("REVIEW-009-RF-003, carried into this spec, is closed."));
        assert!(!citing.contains("FINDING"));
    }

    #[test]
    fn cross_review_citations_never_collapse_to_a_bare_local_identifier() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-004 | category=usability\n",
        );
        // REVIEW-010 declares a finding 004 of its own. A bare `RF-004` in this file would rebind
        // the citation to that finding, which is exactly the corruption the guard prevents.
        write_review(
            directory.path(),
            "REVIEW-010",
            "## Findings\n\n- REVIEW-010-FINDING-004 | category=process\n\n## Context\nSee REVIEW-009-FINDING-004.",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let cross = preview
            .reference_mappings
            .iter()
            .filter(|reference| reference.token == "REVIEW-009-FINDING-004")
            .collect::<Vec<_>>();
        assert!(!cross.is_empty());
        for reference in cross {
            assert_eq!(
                reference.replacement, "REVIEW-009-RF-004",
                "cross-review citation must keep its qualifier"
            );
            assert!(!reference.replacement.starts_with("RF-"));
        }

        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let citing = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-010.md"),
        )
        .unwrap();
        assert!(citing.contains("- REVIEW-010-RF-004 | category=process"));
        assert!(citing.contains("See REVIEW-009-RF-004."));
    }

    #[test]
    fn qualified_references_inside_managed_frontmatter_are_rewritten_without_corrupting_yaml() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-004 | category=usability\n",
        );
        let events = concat!(
            "  - event: accepted\n",
            "    at: 2026-08-14T09:00:00Z\n",
            "    actor: project-lead\n",
            "    reason: \"la regola flex che chiude REVIEW-009-FINDING-004, viewport a 1920x1080\"\n",
            "  - event: superseded\n",
            "    at: 2026-08-15T09:00:00Z\n",
            "    actor: project-lead\n",
            "    reason: \"the dead region is REVIEW-009-FINDING-004's shape rotated.</reason>\\n<parameter name=\\\"evidence_refs\\\">[\\\"SPEC-029\\\"]\"\n",
        );
        write_review_with_events(
            directory.path(),
            "REVIEW-012",
            events,
            "## Findings\n\nNone.\n\n## Context\nREVIEW-009-FINDING-004 cannot reopen through that path.",
        );

        let preview = debt_migration_preview(directory.path()).unwrap();
        let reference = preview
            .reference_mappings
            .iter()
            .find(|reference| reference.path.ends_with("REVIEW-012.md"))
            .unwrap();
        assert_eq!(reference.token, "REVIEW-009-FINDING-004");
        assert_eq!(reference.replacement, "REVIEW-009-RF-004");
        assert_eq!(reference.classification, "cross-review-local");
        assert_eq!(reference.occurrences, 3);

        let before = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-012.md"),
        )
        .unwrap();
        let before_document = Document::parse(&before).unwrap();
        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let after = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/reviews/accepted/REVIEW-012.md"),
        )
        .unwrap();

        // The rewrite is the identifier and nothing else. Normalising the token on the "before"
        // side and the canonical heading on the "after" side leaves two byte-identical files.
        assert_ne!(before, after);
        assert_eq!(
            before
                .replace("REVIEW-009-FINDING-004", "REVIEW-009-RF-004")
                .replace("## Findings", "## Review findings"),
            after
        );
        assert!(!after.contains("FINDING-004"));

        let document = Document::parse(&after).expect("frontmatter still parses");
        assert_eq!(document.value("id").as_deref(), Some("REVIEW-012"));
        let events_before = before_document.string_array("review_events");
        let events_after = document.string_array("review_events");
        assert_eq!(events_before.len(), events_after.len());
    }

    #[test]
    fn qualified_reference_to_an_unknown_review_fails_closed() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-010",
            "## Findings\n\nNone.\n\n## Context\nSee REVIEW-999-FINDING-001.",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "qualified review-local reference REVIEW-999-FINDING-001 in .lmbrain/reviews/accepted/REVIEW-010.md names REVIEW-999, which declares no findings in this workspace"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn qualified_reference_to_an_undeclared_number_fails_closed() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-001 | category=usability\n- REVIEW-009-FINDING-008 | category=process\n",
        );
        write_review(
            directory.path(),
            "REVIEW-010",
            "## Findings\n\nNone.\n\n## Context\nSee REVIEW-009-FINDING-099.",
        );

        let error = debt_migration_preview(directory.path())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "qualified review-local reference REVIEW-009-FINDING-099 in .lmbrain/reviews/accepted/REVIEW-010.md names a finding REVIEW-009 never declares"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn qualified_references_outside_reviews_follow_the_corpus_without_new_blockers() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-004 | category=usability\n",
        );
        let specs = directory.path().join(".lmbrain/specs/done");
        fs::create_dir_all(&specs).unwrap();
        fs::write(
            specs.join("SPEC-012-sample.md"),
            "---\nid: SPEC-012\ntitle: Sample\nstatus: done\n---\n## Scope\nREVIEW-009-FINDING-004 remains open. REVIEW-404-FINDING-001 is not decidable here.\n",
        )
        .unwrap();

        let preview = debt_migration_preview(directory.path()).unwrap();
        assert!(preview.reference_mappings.iter().any(|reference| {
            reference.path.ends_with("SPEC-012-sample.md")
                && reference.token == "REVIEW-009-FINDING-004"
                && reference.replacement == "REVIEW-009-RF-004"
                && reference.classification == "cross-review-local"
        }));

        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let spec = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/specs/done/SPEC-012-sample.md"),
        )
        .unwrap();
        assert!(spec.contains("REVIEW-009-RF-004 remains open."));
        // An undecidable qualified reference outside a review is left alone rather than turned
        // into a new blocking issue.
        assert!(spec.contains("REVIEW-404-FINDING-001 is not decidable here."));
    }

    #[test]
    fn review_origin_refs_stay_in_the_bare_contract_form() {
        let directory = tempdir().unwrap();
        legacy_review_workspace(directory.path());
        write_review(
            directory.path(),
            "REVIEW-009",
            "## Findings\n\n- REVIEW-009-FINDING-004 | category=usability\n",
        );
        let findings = directory.path().join(".lmbrain/findings/open");
        fs::create_dir_all(&findings).unwrap();
        fs::write(
            findings.join("FINDING-001-sample.md"),
            "---\nid: FINDING-001\ntitle: Sample\nstatus: open\ncategory: correctness\nseverity: medium\ncreated: 2026-08-13\nupdated: 2026-08-13\norigin_artifact: REVIEW-009\norigin_ref: REVIEW-009-FINDING-004\nrelated_specs: []\nrelated_reviews: [REVIEW-009]\nrelated_decisions: []\ntarget_specs: []\nblocked_by: []\nresolution_refs: []\nfinding_events: []\n---\n## Statement\nSample\n",
        )
        .unwrap();

        let preview = debt_migration_preview(directory.path()).unwrap();
        debt_migrate(directory.path(), &preview.digest, true).unwrap();
        let debt = fs::read_to_string(
            directory
                .path()
                .join(".lmbrain/debts/open/DEBT-001-sample.md"),
        )
        .unwrap();
        assert!(
            debt.contains("origin_ref: RF-004"),
            "unexpected debt: {debt}"
        );
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
