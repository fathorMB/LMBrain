use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::ArtifactMutationLock,
    path::{PathError, PathGuard},
    transitions::{kind_for_id, ArtifactKind, MutationResult},
};

pub const DEBT_EVENT_SCHEMA_VERSION: &str = "1";
const ACTIVE_STATUSES: &[&str] = &["open", "planned", "deferred"];
const ALL_STATUSES: &[&str] = &[
    "open",
    "planned",
    "deferred",
    "resolved",
    "accepted-risk",
    "superseded",
];
const SEVERITIES: &[&str] = &["critical", "high", "medium", "low", "info"];
const MAX_CONTEXT_RELATIONS: usize = 50;
const MAX_CANDIDATES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtCreateInput {
    pub title: String,
    pub category: String,
    pub severity: String,
    pub origin_severity: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub owner: Option<String>,
    pub origin_artifact: Option<String>,
    pub origin_ref: Option<String>,
    #[serde(default)]
    pub related_specs: Vec<String>,
    #[serde(default)]
    pub related_reviews: Vec<String>,
    #[serde(default)]
    pub related_decisions: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub statement: String,
    pub evidence: String,
    pub impact: String,
    pub resolution_criteria: String,
    pub actor: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Debt {
    pub id: String,
    pub title: String,
    pub status: String,
    pub category: String,
    pub severity: String,
    pub origin_severity: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub owner: Option<String>,
    pub origin_artifact: Option<String>,
    pub origin_ref: Option<String>,
    pub related_specs: Vec<String>,
    pub related_reviews: Vec<String>,
    pub related_decisions: Vec<String>,
    pub target_specs: Vec<String>,
    pub blocked_by: Vec<String>,
    pub resolution_refs: Vec<String>,
    pub superseded_by: Option<String>,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub body: String,
    pub path: String,
    pub malformed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtContext {
    pub schema_version: String,
    pub debt: Debt,
    pub origin: Option<RelationSummary>,
    pub related_specs: Vec<RelationSummary>,
    pub related_reviews: Vec<RelationSummary>,
    pub related_decisions: Vec<RelationSummary>,
    pub target_specs: Vec<RelationSummary>,
    pub blockers: Vec<RelationSummary>,
    pub resolution_refs: Vec<RelationSummary>,
    pub superseded_by: Option<RelationSummary>,
    pub events: Vec<serde_json::Value>,
    pub warnings: Vec<String>,
    pub omitted_relations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtCandidate {
    pub origin_artifact: String,
    pub origin_ref: String,
    pub summary: String,
    pub promoted_debt: Option<String>,
    pub inference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebtCandidateInventory {
    pub schema_version: String,
    pub candidates: Vec<DebtCandidate>,
    pub total: usize,
    pub omitted: usize,
    pub mutated: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DebtError {
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("invalid debt: {0}")]
    Invalid(String),
    #[error("debt changed concurrently: {0}")]
    Concurrent(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn list_debts(root: &Path) -> Vec<Debt> {
    let mut debts = Vec::new();
    for path in markdown_files(&root.join(".lmbrain/debts")) {
        let file_stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !file_stem.starts_with("DEBT-") {
            continue;
        }
        let relative = relative_path(root, &path);
        match fs::read_to_string(&path)
            .ok()
            .and_then(|source| Document::parse(&source).ok())
        {
            Some(document) => {
                let malformed = debt_shape_is_malformed(&path, &document);
                debts.push(debt_from_document(&document, relative, malformed));
            }
            None => debts.push(Debt {
                id: path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("MALFORMED")
                    .to_string(),
                title: "Malformed debt".into(),
                status: path_status(&path).unwrap_or("unknown").into(),
                category: "unknown".into(),
                severity: "unknown".into(),
                origin_severity: None,
                area: None,
                milestone: None,
                owner: None,
                origin_artifact: None,
                origin_ref: None,
                related_specs: Vec::new(),
                related_reviews: Vec::new(),
                related_decisions: Vec::new(),
                target_specs: Vec::new(),
                blocked_by: Vec::new(),
                resolution_refs: Vec::new(),
                superseded_by: None,
                created: String::new(),
                updated: String::new(),
                tags: Vec::new(),
                body: String::new(),
                path: relative,
                malformed: true,
            }),
        }
    }
    debts.sort_by(|left, right| left.id.cmp(&right.id));
    debts
}

pub fn create_debt(
    root: impl AsRef<Path>,
    input: DebtCreateInput,
) -> Result<MutationResult, DebtError> {
    let guard = PathGuard::new(root)?;
    validate_create_input(guard.root(), &input)?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), "creation-allocation")?;
    validate_unique_origin(
        guard.root(),
        None,
        input.origin_artifact.as_deref(),
        input.origin_ref.as_deref(),
    )?;
    let id = next_debt_id(guard.root());
    validate_blocker_graph(guard.root(), &id, &input.blocked_by)?;
    let dir = guard.root().join(".lmbrain/debts/open");
    let path = dir.join(format!("{}-{}.md", id, slug(&input.title)));
    let date = today();
    let mut document = Document::parse(&render_new_debt(&id, &date, &input))?;
    append_event(
        &mut document,
        &id,
        "created",
        "none",
        "open",
        "project-lead",
        &input.actor,
        &input.rationale,
        &[],
    )?;
    fs::create_dir_all(&dir)?;
    atomic_write(&path, &document.render())?;
    Ok(MutationResult {
        id,
        status: "open".into(),
        path,
        forced: false,
    })
}

pub fn plan_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    target_specs: Vec<String>,
    actor: &str,
    rationale: &str,
) -> Result<MutationResult, DebtError> {
    if target_specs.is_empty() {
        return Err(DebtError::Invalid(
            "planned debts require at least one target spec".into(),
        ));
    }
    mutate_debt(
        root,
        artifact,
        "planned",
        "planned",
        "project-lead",
        actor,
        rationale,
        |root, id, document| {
            validate_refs(
                root,
                "target_specs",
                &target_specs,
                &[ArtifactKind::Spec],
                id,
            )?;
            document.set("target_specs", &yaml_array(&target_specs));
            Ok(target_specs.clone())
        },
    )
}

pub fn defer_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    actor: &str,
    rationale: &str,
    revisit_condition: &str,
) -> Result<MutationResult, DebtError> {
    require_text("revisit_condition", revisit_condition)?;
    mutate_debt(
        root,
        artifact,
        "deferred",
        "deferred",
        "project-lead",
        actor,
        rationale,
        |_root, _id, document| {
            document.set("revisit_condition", &quoted(revisit_condition));
            Ok(Vec::new())
        },
    )
}

pub fn resolve_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    actor: &str,
    rationale: &str,
    resolution_refs: Vec<String>,
    resolution_evidence: &str,
) -> Result<MutationResult, DebtError> {
    if resolution_refs.is_empty() {
        return Err(DebtError::Invalid(
            "resolved debts require resolution_refs".into(),
        ));
    }
    require_text("resolution_evidence", resolution_evidence)?;
    mutate_debt(
        root,
        artifact,
        "resolved",
        "resolved",
        "project-lead",
        actor,
        rationale,
        |root, id, document| {
            validate_evidence_refs(root, id, &resolution_refs)?;
            document.set("resolution_refs", &yaml_array(&resolution_refs));
            append_body_entry(document, "Resolution evidence", resolution_evidence);
            Ok(resolution_refs.clone())
        },
    )
}

pub fn accept_debt_risk(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    actor: &str,
    rationale: &str,
    revisit_condition: &str,
    resolution_refs: Vec<String>,
) -> Result<MutationResult, DebtError> {
    require_text("operator actor", actor)?;
    require_text("rationale", rationale)?;
    require_text(
        "revisit condition or explicit no-revisit statement",
        revisit_condition,
    )?;
    mutate_debt(
        root,
        artifact,
        "accepted-risk",
        "accepted-risk",
        "operator",
        actor,
        rationale,
        |root, id, document| {
            validate_evidence_refs(root, id, &resolution_refs)?;
            document.set("resolution_refs", &yaml_array(&resolution_refs));
            document.set("revisit_condition", &quoted(revisit_condition));
            append_body_entry(
                document,
                "Resolution evidence",
                &format!("Risk accepted by {actor}: {rationale}\n\nRevisit: {revisit_condition}"),
            );
            Ok(resolution_refs.clone())
        },
    )
}

pub fn supersede_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    actor: &str,
    rationale: &str,
    successor: Option<String>,
) -> Result<MutationResult, DebtError> {
    require_text("rationale", rationale)?;
    mutate_debt(
        root,
        artifact,
        "superseded",
        "superseded",
        "project-lead",
        actor,
        rationale,
        |root, id, document| {
            if let Some(successor) = successor.as_deref() {
                validate_refs(
                    root,
                    "superseded_by",
                    &[successor.to_string()],
                    &[ArtifactKind::Debt],
                    id,
                )?;
                document.set("superseded_by", &format!("[{}]", quoted(successor)));
                Ok(vec![successor.to_string()])
            } else {
                document.set("superseded_by", "null");
                Ok(Vec::new())
            }
        },
    )
}

pub fn reopen_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    actor: &str,
    rationale: &str,
) -> Result<MutationResult, DebtError> {
    mutate_debt(
        root,
        artifact,
        "open",
        "reopened",
        "operator",
        actor,
        rationale,
        |_root, _id, document| {
            document.set("resolution_refs", "[]");
            document.set("superseded_by", "null");
            Ok(Vec::new())
        },
    )
}

fn mutate_debt(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    target: &str,
    action: &str,
    actor_role: &str,
    actor: &str,
    rationale: &str,
    prepare: impl FnOnce(&Path, &str, &mut Document) -> Result<Vec<String>, DebtError>,
) -> Result<MutationResult, DebtError> {
    require_text("actor", actor)?;
    require_text("rationale", rationale)?;
    let guard = PathGuard::new(root)?;
    let initial_path = guard.resolve_existing(artifact.as_ref())?;
    let initial_source = fs::read_to_string(&initial_path)?;
    let initial = Document::parse(&initial_source)?;
    let initial_id = initial
        .value("id")
        .ok_or_else(|| DebtError::Invalid("missing debt id".into()))?;
    if kind_for_id(&initial_id) != Some(ArtifactKind::Debt) {
        return Err(DebtError::Invalid(
            "semantic debt operations require DEBT-*".into(),
        ));
    }
    let _lock = ArtifactMutationLock::acquire(guard.root(), &initial_id)?;
    let path = guard.resolve_existing(artifact.as_ref())?;
    let source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&source)?;
    let id = document
        .value("id")
        .ok_or_else(|| DebtError::Invalid("missing debt id".into()))?;
    if id != initial_id {
        return Err(DebtError::Concurrent(
            "identity changed while waiting for lock".into(),
        ));
    }
    let from = document
        .value("status")
        .ok_or_else(|| DebtError::Invalid("missing debt status".into()))?;
    validate_debt_document(guard.root(), &path, &document)?;
    let terminal = matches!(from.as_str(), "resolved" | "accepted-risk" | "superseded");
    let legal = if action == "reopened" {
        terminal
    } else {
        crate::transitions::allowed(ArtifactKind::Debt, &from, target)
    };
    if !legal {
        return Err(DebtError::Invalid(format!(
            "illegal debt transition from '{from}' to '{target}'"
        )));
    }
    if action == "reopened" && from == "superseded" {
        return Err(DebtError::Invalid(
            "a superseded debt is historical and cannot be reopened".into(),
        ));
    }
    let evidence_refs = prepare(guard.root(), &id, &mut document)?;
    document.set("status", target);
    document.set("updated", &today());
    document.append_activity(&format!("{action}: {}", rationale.trim()));
    append_event(
        &mut document,
        &id,
        action,
        &from,
        target,
        actor_role,
        actor,
        rationale,
        &evidence_refs,
    )?;
    let destination = guard.root().join(".lmbrain/debts").join(target).join(
        path.file_name()
            .ok_or_else(|| DebtError::Invalid("missing file name".into()))?,
    );
    validate_debt_document(guard.root(), &destination, &document)?;
    if destination != path && destination.exists() {
        return Err(DebtError::Invalid("debt destination already exists".into()));
    }
    if fs::read_to_string(&path)? != source {
        return Err(DebtError::Concurrent(
            "artifact changed while mutation was prepared".into(),
        ));
    }
    fs::create_dir_all(destination.parent().unwrap())?;
    let rendered = document.render();
    if destination == path {
        atomic_write(&path, &rendered)?;
    } else {
        // Keep a single authoritative file throughout the move. If the
        // directory rename fails (notably on Windows due to an open handle),
        // restore the exact original source before returning an error.
        atomic_write(&path, &rendered)?;
        if let Err(error) = fs::rename(&path, &destination) {
            atomic_write(&path, &source)?;
            return Err(DebtError::Io(error));
        }
    }
    Ok(MutationResult {
        id,
        status: target.into(),
        path: destination,
        forced: false,
    })
}

pub fn validate_debt_document(
    root: &Path,
    path: &Path,
    document: &Document,
) -> Result<(), DebtError> {
    let id = document
        .value("id")
        .ok_or_else(|| DebtError::Invalid("missing id".into()))?;
    if kind_for_id(&id) != Some(ArtifactKind::Debt) {
        return Err(DebtError::Invalid("id must use DEBT-*".into()));
    }
    let status = document
        .value("status")
        .ok_or_else(|| DebtError::Invalid("missing status".into()))?;
    if !ALL_STATUSES.contains(&status.as_str()) {
        return Err(DebtError::Invalid(format!("unknown status '{status}'")));
    }
    if path_status(path).is_some_and(|folder| folder != status) {
        return Err(DebtError::Invalid(format!(
            "status '{status}' does not match containing directory"
        )));
    }
    require_text(
        "category",
        document.value("category").as_deref().unwrap_or_default(),
    )?;
    let severity = document.value("severity").unwrap_or_default();
    if !SEVERITIES.contains(&severity.as_str()) {
        return Err(DebtError::Invalid(format!(
            "severity must be one of {}",
            SEVERITIES.join(", ")
        )));
    }
    let origin = document.value("origin_artifact");
    let origin_ref = document.value("origin_ref");
    if origin.is_some() != origin_ref.is_some() {
        return Err(DebtError::Invalid(
            "origin_artifact and origin_ref must be both present or both absent".into(),
        ));
    }
    if let Some(origin) = origin.as_deref() {
        validate_refs(
            root,
            "origin_artifact",
            &[origin.to_string()],
            &[ArtifactKind::Review, ArtifactKind::Spec, ArtifactKind::Adr],
            &id,
        )?;
        if origin.starts_with("REVIEW-")
            && !origin_ref.as_deref().is_some_and(valid_review_finding_id)
        {
            return Err(DebtError::Invalid(
                "review-origin debts require an RF-* origin_ref".into(),
            ));
        }
    }
    validate_unique_origin(root, Some(&id), origin.as_deref(), origin_ref.as_deref())?;
    validate_refs(
        root,
        "related_specs",
        &document.string_array("related_specs"),
        &[ArtifactKind::Spec],
        &id,
    )?;
    validate_refs(
        root,
        "related_reviews",
        &document.string_array("related_reviews"),
        &[ArtifactKind::Review],
        &id,
    )?;
    validate_refs(
        root,
        "related_decisions",
        &document.string_array("related_decisions"),
        &[ArtifactKind::Adr],
        &id,
    )?;
    let targets = document.string_array("target_specs");
    validate_refs(root, "target_specs", &targets, &[ArtifactKind::Spec], &id)?;
    if status == "planned" && targets.is_empty() {
        return Err(DebtError::Invalid(
            "planned debt requires target_specs".into(),
        ));
    }
    let blockers = document.string_array("blocked_by");
    validate_refs(root, "blocked_by", &blockers, &[ArtifactKind::Debt], &id)?;
    validate_blocker_graph(root, &id, &blockers)?;
    let resolution_refs = document.string_array("resolution_refs");
    if status == "resolved" {
        if resolution_refs.is_empty()
            || !body_section_has_content(&document.body, "Resolution evidence")
        {
            return Err(DebtError::Invalid(
                "resolved debt requires resolution_refs and resolution evidence".into(),
            ));
        }
        validate_evidence_refs(root, &id, &resolution_refs)?;
    }
    if status == "accepted-risk" {
        require_text(
            "revisit_condition",
            document
                .value("revisit_condition")
                .as_deref()
                .unwrap_or_default(),
        )?;
        if !body_section_has_content(&document.body, "Resolution evidence") {
            return Err(DebtError::Invalid(
                "accepted-risk requires operator rationale in resolution evidence".into(),
            ));
        }
    }
    if status == "superseded"
        && document.value("superseded_by").is_none()
        && !document.object_array("debt_events").iter().any(|event| {
            event.get("action").and_then(|value| value.as_str()) == Some("superseded")
                && event
                    .get("rationale")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| !value.trim().is_empty())
        })
    {
        return Err(DebtError::Invalid(
            "superseded debt requires a successor or obsolescence rationale".into(),
        ));
    }
    Ok(())
}

pub fn debt_context(root: &Path, identity: &str) -> Result<DebtContext, DebtError> {
    let index = artifact_index(root);
    let (path, document) = index
        .get(identity)
        .or_else(|| {
            index
                .values()
                .find(|(path, _)| relative_path(root, path) == identity)
        })
        .cloned()
        .ok_or_else(|| DebtError::Invalid(format!("debt '{identity}' not found")))?;
    if kind_for_id(&document.value("id").unwrap_or_default()) != Some(ArtifactKind::Debt) {
        return Err(DebtError::Invalid(format!("'{identity}' is not a debt")));
    }
    validate_debt_document(root, &path, &document)?;
    let debt = debt_from_document(&document, relative_path(root, &path), false);
    let mut omitted = 0;
    let mut warnings = Vec::new();
    let relation = |id: &str| -> Option<RelationSummary> {
        index.get(id).map(|(path, document)| RelationSummary {
            id: id.into(),
            title: document.value("title").unwrap_or_default(),
            status: document.value("status").unwrap_or_default(),
            path: relative_path(root, path),
        })
    };
    let mut resolve_many = |ids: &[String]| {
        let mut out = Vec::new();
        for id in ids {
            if let Some(item) = relation(id) {
                if out.len() < MAX_CONTEXT_RELATIONS {
                    out.push(item);
                } else {
                    omitted += 1;
                }
            } else {
                warnings.push(format!("unresolved relation '{id}'"));
            }
        }
        out
    };
    let origin = debt.origin_artifact.as_deref().and_then(relation);
    let related_specs = resolve_many(&debt.related_specs);
    let related_reviews = resolve_many(&debt.related_reviews);
    let related_decisions = resolve_many(&debt.related_decisions);
    let target_specs = resolve_many(&debt.target_specs);
    let blockers = resolve_many(&debt.blocked_by);
    let resolution_refs = resolve_many(
        &debt
            .resolution_refs
            .iter()
            .filter(|reference| kind_for_id(reference).is_some())
            .cloned()
            .collect::<Vec<_>>(),
    );
    let superseded_by = debt.superseded_by.as_deref().and_then(relation);
    Ok(DebtContext {
        schema_version: "1".into(),
        debt,
        origin,
        related_specs,
        related_reviews,
        related_decisions,
        target_specs,
        blockers,
        resolution_refs,
        superseded_by,
        events: document
            .fields()
            .get("debt_events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
        warnings,
        omitted_relations: omitted,
    })
}

pub fn debt_candidates(root: &Path) -> DebtCandidateInventory {
    let promoted = list_debts(root)
        .into_iter()
        .filter_map(|debt| Some(((debt.origin_artifact?, debt.origin_ref?), debt.id)))
        .collect::<HashMap<_, _>>();
    let token = Regex::new(r"(?i)\b(RF-[A-Z0-9-]+)\b").unwrap();
    let mut all = Vec::new();
    let mut warnings = Vec::new();
    for path in markdown_files(&root.join(".lmbrain/reviews")) {
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("README.md"))
        {
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(document) = Document::parse(&source) else {
            warnings.push(format!(
                "skipped malformed review {}",
                relative_path(root, &path)
            ));
            continue;
        };
        let Some(review_id) = document.value("id") else {
            continue;
        };
        let mut seen = BTreeSet::new();
        let mut in_review_findings = false;
        let mut fence: Option<char> = None;
        for line in document.body.lines() {
            let trimmed = line.trim();
            if let Some(marker) = fence {
                if trimmed.starts_with(&marker.to_string().repeat(3)) {
                    fence = None;
                }
                continue;
            }
            if trimmed.starts_with("```") {
                fence = Some('`');
                continue;
            }
            if trimmed.starts_with("~~~") {
                fence = Some('~');
                continue;
            }
            if trimmed.starts_with("## ") {
                in_review_findings = trimmed[3..].trim().eq_ignore_ascii_case("review findings");
                continue;
            }
            if !in_review_findings {
                continue;
            }
            let Some(capture) = token.captures(line) else {
                continue;
            };
            let local = capture[1].to_ascii_uppercase();
            if !seen.insert(local.clone()) {
                continue;
            }
            all.push(DebtCandidate {
                origin_artifact: review_id.clone(),
                origin_ref: local.clone(),
                summary: line.trim().chars().take(300).collect(),
                promoted_debt: promoted.get(&(review_id.clone(), local)).cloned(),
                inference: "stable-form token only; disposition is not inferred".into(),
            });
        }
    }
    all.sort_by(|left, right| {
        left.origin_artifact
            .cmp(&right.origin_artifact)
            .then_with(|| left.origin_ref.cmp(&right.origin_ref))
    });
    let total = all.len();
    all.truncate(MAX_CANDIDATES);
    DebtCandidateInventory {
        schema_version: "1".into(),
        candidates: all,
        total,
        omitted: total.saturating_sub(MAX_CANDIDATES),
        mutated: false,
        warnings,
    }
}

fn validate_create_input(root: &Path, input: &DebtCreateInput) -> Result<(), DebtError> {
    for (label, value) in [
        ("title", input.title.as_str()),
        ("category", input.category.as_str()),
        ("severity", input.severity.as_str()),
        ("statement", input.statement.as_str()),
        ("evidence", input.evidence.as_str()),
        ("impact", input.impact.as_str()),
        ("resolution_criteria", input.resolution_criteria.as_str()),
        ("actor", input.actor.as_str()),
        ("rationale", input.rationale.as_str()),
    ] {
        require_text(label, value)?;
    }
    if !SEVERITIES.contains(&input.severity.as_str()) {
        return Err(DebtError::Invalid(format!(
            "severity must be one of {}",
            SEVERITIES.join(", ")
        )));
    }
    if input.origin_artifact.is_some() != input.origin_ref.is_some() {
        return Err(DebtError::Invalid(
            "origin_artifact and origin_ref must be both present or both absent".into(),
        ));
    }
    if let Some(origin) = input.origin_artifact.as_deref() {
        validate_refs(
            root,
            "origin_artifact",
            &[origin.to_string()],
            &[ArtifactKind::Review, ArtifactKind::Spec, ArtifactKind::Adr],
            "",
        )?;
        if origin.starts_with("REVIEW-")
            && !input
                .origin_ref
                .as_deref()
                .is_some_and(valid_review_finding_id)
        {
            return Err(DebtError::Invalid(
                "review-origin debts require an RF-* origin_ref".into(),
            ));
        }
    }
    validate_refs(
        root,
        "related_specs",
        &input.related_specs,
        &[ArtifactKind::Spec],
        "",
    )?;
    validate_refs(
        root,
        "related_reviews",
        &input.related_reviews,
        &[ArtifactKind::Review],
        "",
    )?;
    validate_refs(
        root,
        "related_decisions",
        &input.related_decisions,
        &[ArtifactKind::Adr],
        "",
    )?;
    validate_refs(
        root,
        "blocked_by",
        &input.blocked_by,
        &[ArtifactKind::Debt],
        "",
    )?;
    Ok(())
}

fn valid_review_finding_id(value: &str) -> bool {
    value.strip_prefix("RF-").is_some_and(|suffix| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn validate_refs(
    root: &Path,
    field: &str,
    refs: &[String],
    allowed: &[ArtifactKind],
    self_id: &str,
) -> Result<(), DebtError> {
    let index = artifact_index(root);
    let mut seen = HashSet::new();
    for reference in refs {
        if reference == self_id {
            return Err(DebtError::Invalid(format!(
                "{field} cannot contain a self-link"
            )));
        }
        if !seen.insert(reference) {
            return Err(DebtError::Invalid(format!(
                "{field} contains duplicate '{reference}'"
            )));
        }
        let kind = kind_for_id(reference).ok_or_else(|| {
            DebtError::Invalid(format!("{field} contains invalid ID '{reference}'"))
        })?;
        if !allowed.contains(&kind) {
            return Err(DebtError::Invalid(format!(
                "{field} does not allow {kind:?} reference '{reference}'"
            )));
        }
        if !index.contains_key(reference) {
            return Err(DebtError::Invalid(format!(
                "{field} reference '{reference}' does not resolve"
            )));
        }
    }
    Ok(())
}

fn validate_evidence_refs(root: &Path, self_id: &str, refs: &[String]) -> Result<(), DebtError> {
    let index = artifact_index(root);
    for reference in refs {
        if reference == self_id {
            return Err(DebtError::Invalid(
                "resolution_refs cannot contain a self-link".into(),
            ));
        }
        if kind_for_id(reference).is_some() {
            if !index.contains_key(reference) {
                return Err(DebtError::Invalid(format!(
                    "resolution reference '{reference}' does not resolve"
                )));
            }
        } else if !reference.starts_with("operator-observation:") {
            return Err(DebtError::Invalid(format!(
                "resolution reference '{reference}' must be an existing artifact ID or operator-observation:<id>"
            )));
        }
    }
    Ok(())
}

fn validate_unique_origin(
    root: &Path,
    self_id: Option<&str>,
    origin: Option<&str>,
    origin_ref: Option<&str>,
) -> Result<(), DebtError> {
    let (Some(origin), Some(origin_ref)) = (origin, origin_ref) else {
        return Ok(());
    };
    for debt in list_debts(root) {
        if Some(debt.id.as_str()) == self_id || !ACTIVE_STATUSES.contains(&debt.status.as_str()) {
            continue;
        }
        if debt.origin_artifact.as_deref() == Some(origin)
            && debt.origin_ref.as_deref() == Some(origin_ref)
        {
            return Err(DebtError::Invalid(format!(
                "active debt {} already promotes {origin}/{origin_ref}",
                debt.id
            )));
        }
    }
    Ok(())
}

fn validate_blocker_graph(
    root: &Path,
    debt_id: &str,
    proposed: &[String],
) -> Result<(), DebtError> {
    let mut graph = list_debts(root)
        .into_iter()
        .map(|debt| (debt.id, debt.blocked_by))
        .collect::<HashMap<_, _>>();
    graph.insert(debt_id.to_string(), proposed.to_vec());
    fn visit(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if !visiting.insert(node.to_string()) {
            return true;
        }
        if visited.contains(node) {
            visiting.remove(node);
            return false;
        }
        for next in graph.get(node).into_iter().flatten() {
            if visit(next, graph, visiting, visited) {
                return true;
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        false
    }
    if visit(debt_id, &graph, &mut HashSet::new(), &mut HashSet::new()) {
        return Err(DebtError::Invalid(
            "blocked_by relationships contain a cycle".into(),
        ));
    }
    Ok(())
}

fn append_event(
    document: &mut Document,
    debt_id: &str,
    action: &str,
    from: &str,
    to: &str,
    actor_role: &str,
    actor: &str,
    rationale: &str,
    evidence_refs: &[String],
) -> Result<(), DebtError> {
    let sequence = document.object_array("debt_events").len() + 1;
    document.append_object(
        "debt_events",
        &[
            ("schema_version".into(), json!(DEBT_EVENT_SCHEMA_VERSION)),
            ("id".into(), json!(format!("{debt_id}-EVENT-{sequence:03}"))),
            ("timestamp".into(), json!(Local::now().to_rfc3339())),
            ("action".into(), json!(action)),
            ("from_status".into(), json!(from)),
            ("to_status".into(), json!(to)),
            ("actor_role".into(), json!(actor_role)),
            ("actor".into(), json!(actor.trim())),
            ("rationale".into(), json!(rationale.trim())),
            ("evidence_refs".into(), json!(evidence_refs)),
        ],
    )?;
    Ok(())
}

fn render_new_debt(id: &str, date: &str, input: &DebtCreateInput) -> String {
    format!(
        "---\nid: {id}\ntitle: {}\nstatus: open\ncategory: {}\nseverity: {}\norigin_severity: {}\narea: {}\nmilestone: {}\nowner: {}\norigin_artifact: {}\norigin_ref: {}\nrelated_specs: {}\nrelated_reviews: {}\nrelated_decisions: {}\ntarget_specs: []\nblocked_by: {}\nresolution_refs: []\nsuperseded_by: null\nrevisit_condition: null\ncreated: {date}\nupdated: {date}\ntags: {}\nlinks: []\nactivity: []\ndebt_events: []\n---\n# {}\n\n## Statement\n\n{}\n\n## Evidence and provenance\n\n{}\n\n## Impact and scope boundary\n\n{}\n\n## Decision log\n\nCreated by {}: {}\n\n## Resolution criteria\n\n{}\n\n## Resolution evidence\n\n",
        quoted(&input.title),
        quoted(&input.category),
        quoted(&input.severity),
        yaml_optional(input.origin_severity.as_deref()),
        yaml_optional(input.area.as_deref()),
        yaml_optional(input.milestone.as_deref()),
        yaml_optional(input.owner.as_deref()),
        yaml_optional(input.origin_artifact.as_deref()),
        yaml_optional(input.origin_ref.as_deref()),
        yaml_array(&input.related_specs),
        yaml_array(&input.related_reviews),
        yaml_array(&input.related_decisions),
        yaml_array(&input.blocked_by),
        yaml_array(&input.tags),
        input.title.trim(),
        input.statement.trim(),
        input.evidence.trim(),
        input.impact.trim(),
        input.actor.trim(),
        input.rationale.trim(),
        input.resolution_criteria.trim(),
    )
}

fn debt_from_document(document: &Document, path: String, malformed: bool) -> Debt {
    Debt {
        id: document.value("id").unwrap_or_default(),
        title: document.value("title").unwrap_or_default(),
        status: document.value("status").unwrap_or_default(),
        category: document.value("category").unwrap_or_default(),
        severity: document.value("severity").unwrap_or_default(),
        origin_severity: document.value("origin_severity"),
        area: document.value("area"),
        milestone: document.value("milestone"),
        owner: document.value("owner"),
        origin_artifact: document.value("origin_artifact"),
        origin_ref: document.value("origin_ref"),
        related_specs: document.string_array("related_specs"),
        related_reviews: document.string_array("related_reviews"),
        related_decisions: document.string_array("related_decisions"),
        target_specs: document.string_array("target_specs"),
        blocked_by: document.string_array("blocked_by"),
        resolution_refs: document.string_array("resolution_refs"),
        superseded_by: document.value("superseded_by"),
        created: document.value("created").unwrap_or_default(),
        updated: document.value("updated").unwrap_or_default(),
        tags: document.string_array("tags"),
        body: document.body.clone(),
        path,
        malformed,
    }
}

fn debt_shape_is_malformed(path: &Path, document: &Document) -> bool {
    let id = document.value("id").unwrap_or_default();
    let status = document.value("status").unwrap_or_default();
    let severity = document.value("severity").unwrap_or_default();
    let fields = document.fields();
    kind_for_id(&id) != Some(ArtifactKind::Debt)
        || !ALL_STATUSES.contains(&status.as_str())
        || path_status(path).is_some_and(|folder| folder != status)
        || document
            .value("category")
            .map_or(true, |value| value.trim().is_empty())
        || !SEVERITIES.contains(&severity.as_str())
        || [
            "related_specs",
            "related_reviews",
            "related_decisions",
            "target_specs",
            "blocked_by",
            "resolution_refs",
        ]
        .iter()
        .any(|key| fields.get(*key).is_some_and(|value| !value.is_array()))
}

fn artifact_index(root: &Path) -> HashMap<String, (PathBuf, Document)> {
    markdown_files(&root.join(".lmbrain"))
        .into_iter()
        .filter_map(|path| {
            let source = fs::read_to_string(&path).ok()?;
            let document = Document::parse(&source).ok()?;
            let id = document.value("id")?;
            Some((id, (path, document)))
        })
        .collect()
}

fn next_debt_id(root: &Path) -> String {
    let mut max = list_debts(root)
        .iter()
        .filter_map(|debt| debt.id.strip_prefix("DEBT-")?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    let regex = Regex::new(r"\bDEBT-(\d{3,})\b").unwrap();
    for path in markdown_files(&root.join(".lmbrain")) {
        if path.components().any(|c| c.as_os_str() == "templates") {
            continue;
        }
        if let Ok(source) = fs::read_to_string(&path) {
            for cap in regex.captures_iter(&source) {
                if let Ok(num) = cap[1].parse::<u32>() {
                    if num > max {
                        max = num;
                    }
                }
            }
        }
    }

    format!("DEBT-{:03}", max + 1)
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("templates") {
                    continue;
                }
                out.extend(markdown_files(&path));
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn path_status(path: &Path) -> Option<&str> {
    path.parent()?.file_name()?.to_str()
}

fn body_section_has_content(body: &str, heading: &str) -> bool {
    let needle = format!("## {heading}");
    let Some(start) = body.find(&needle) else {
        return false;
    };
    let rest = &body[start + needle.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    !rest[..end].trim().is_empty()
}

fn append_body_entry(document: &mut Document, heading: &str, value: &str) {
    let needle = format!("## {heading}");
    if let Some(start) = document.body.find(&needle) {
        let content_start = start + needle.len();
        let rest = &document.body[content_start..];
        let end = rest
            .find("\n## ")
            .map(|offset| content_start + offset)
            .unwrap_or(document.body.len());
        let mut body = document.body.clone();
        body.replace_range(content_start..end, &format!("\n\n{}\n", value.trim()));
        document.body = body;
    } else {
        document
            .body
            .push_str(&format!("\n\n## {heading}\n\n{}\n", value.trim()));
    }
}

fn require_text(label: &str, value: &str) -> Result<(), DebtError> {
    if value.trim().is_empty() {
        Err(DebtError::Invalid(format!("{label} cannot be empty")))
    } else {
        Ok(())
    }
}

fn yaml_array(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".into())
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value.trim()).unwrap_or_else(|_| "\"\"".into())
}

fn yaml_optional(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(quoted)
        .unwrap_or_else(|| "null".into())
}

fn slug(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "debt".into()
    } else {
        slug
    }
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/reviews/accepted")).unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/done")).unwrap();
        fs::write(
            dir.path().join(".lmbrain/reviews/accepted/REVIEW-054.md"),
            "---\nid: REVIEW-054\ntitle: Review\nstatus: accepted\nspec: SPEC-048\n---\n## Review findings\n- RF-007 routed debt\n",
        )
        .unwrap();
        for (status, id) in [("backlog", "SPEC-059"), ("done", "SPEC-048")] {
            fs::write(
                dir.path().join(format!(".lmbrain/specs/{status}/{id}.md")),
                format!("---\nid: {id}\ntitle: Spec\nstatus: {status}\n---\n"),
            )
            .unwrap();
        }
        dir
    }

    fn input() -> DebtCreateInput {
        DebtCreateInput {
            title: "Routed debt".into(),
            category: "correctness".into(),
            severity: "high".into(),
            origin_severity: Some("blocking".into()),
            area: Some("engine".into()),
            milestone: Some("M-04".into()),
            owner: None,
            origin_artifact: Some("REVIEW-054".into()),
            origin_ref: Some("RF-007".into()),
            related_specs: vec!["SPEC-048".into()],
            related_reviews: vec!["REVIEW-054".into()],
            related_decisions: Vec::new(),
            blocked_by: Vec::new(),
            tags: vec!["debt".into()],
            statement: "The defect remains true.".into(),
            evidence: "REVIEW-054 documents reproduction.".into(),
            impact: "Incorrect output.".into(),
            resolution_criteria: "Regression passes.".into(),
            actor: "AGENT-LEAD".into(),
            rationale: "Survives the originating spec.".into(),
        }
    }

    #[test]
    fn semantic_lifecycle_keeps_planned_unresolved_and_audits_resolution() {
        let dir = workspace();
        let created = create_debt(dir.path(), input()).unwrap();
        let planned = plan_debt(
            dir.path(),
            &created.path,
            vec!["SPEC-059".into()],
            "AGENT-LEAD",
            "Scheduled separately",
        )
        .unwrap();
        let planned_doc = Document::parse(&fs::read_to_string(&planned.path).unwrap()).unwrap();
        assert_eq!(planned_doc.value("status").as_deref(), Some("planned"));
        assert_eq!(
            planned_doc.value("origin_severity").as_deref(),
            Some("blocking")
        );
        assert!(planned_doc.string_array("resolution_refs").is_empty());
        assert_eq!(planned_doc.object_array("debt_events").len(), 2);
        let digest = crate::build_project_digest(dir.path());
        assert_eq!(digest.debts.active.total, 1);
        let target_context = crate::build_spec_context(dir.path(), "SPEC-059").unwrap();
        assert_eq!(target_context.debts[0].relation_roles, vec!["target"]);
        let review_context = crate::build_review_context(dir.path(), "SPEC-048").unwrap();
        assert_eq!(
            review_context.debts[0].relation_roles,
            vec!["origin", "related-review"]
        );

        let resolved = resolve_debt(
            dir.path(),
            &planned.path,
            "AGENT-LEAD",
            "Accepted review proves closure",
            vec!["REVIEW-054".into()],
            "The regression and acceptance evidence are recorded in REVIEW-054.",
        )
        .unwrap();
        let resolved_doc = Document::parse(&fs::read_to_string(&resolved.path).unwrap()).unwrap();
        assert_eq!(resolved_doc.value("status").as_deref(), Some("resolved"));
        assert_eq!(resolved_doc.object_array("debt_events").len(), 3);
    }

    #[test]
    fn duplicate_active_origin_and_invalid_planning_fail_without_mutation() {
        let dir = workspace();
        let created = create_debt(dir.path(), input()).unwrap();
        assert!(create_debt(dir.path(), input()).is_err());
        let source = fs::read_to_string(&created.path).unwrap();
        assert!(plan_debt(
            dir.path(),
            &created.path,
            vec!["SPEC-404".into()],
            "AGENT-LEAD",
            "Invalid"
        )
        .is_err());
        assert_eq!(fs::read_to_string(created.path).unwrap(), source);
    }

    #[test]
    fn terminal_reopen_requires_operator_semantic_action_and_superseded_stays_historical() {
        let dir = workspace();
        let created = create_debt(dir.path(), input()).unwrap();
        let accepted = accept_debt_risk(
            dir.path(),
            &created.path,
            "moren",
            "Known bounded limitation",
            "Revisit in M-05",
            vec!["operator-observation:playtest-1".into()],
        )
        .unwrap();
        let reopened = reopen_debt(
            dir.path(),
            &accepted.path,
            "moren",
            "New evidence invalidates acceptance",
        )
        .unwrap();
        assert_eq!(reopened.status, "open");
        let superseded = supersede_debt(
            dir.path(),
            &reopened.path,
            "AGENT-LEAD",
            "Observation was invalidated",
            None,
        )
        .unwrap();
        assert!(reopen_debt(
            dir.path(),
            superseded.path,
            "moren",
            "Cannot revive history"
        )
        .is_err());
    }

    #[test]
    fn candidate_inventory_is_read_only_and_scopes_duplicate_local_ids_by_review() {
        let dir = workspace();
        fs::write(
            dir.path().join(".lmbrain/reviews/accepted/REVIEW-055.md"),
            "---\nid: REVIEW-055\ntitle: Other\nstatus: accepted\n---\n## Review findings\n- RF-007 another local item\n",
        )
        .unwrap();
        let before =
            fs::read_to_string(dir.path().join(".lmbrain/reviews/accepted/REVIEW-054.md")).unwrap();
        let inventory = debt_candidates(dir.path());
        assert_eq!(inventory.total, 2);
        assert_ne!(
            inventory.candidates[0].origin_artifact,
            inventory.candidates[1].origin_artifact
        );
        assert!(!inventory.mutated);
        assert_eq!(
            fs::read_to_string(dir.path().join(".lmbrain/reviews/accepted/REVIEW-054.md")).unwrap(),
            before
        );
    }

    #[test]
    fn xenomark_fixture_preserves_distinct_dispositions_without_auto_promotion() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/xenomark-review-findings.json"
        ))
        .unwrap();
        assert_eq!(
            fixture
                .pointer("/planned_debt/expected_status")
                .and_then(|value| value.as_str()),
            Some("planned")
        );
        assert_eq!(
            fixture
                .pointer("/documented_limitation/expected_status")
                .and_then(|value| value.as_str()),
            Some("resolved")
        );
        assert_eq!(
            fixture
                .pointer("/design_observations")
                .and_then(|value| value.as_array())
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            fixture.pointer("/duplicate_local_ids/0/origin_ref"),
            fixture.pointer("/duplicate_local_ids/1/origin_ref")
        );
        assert_ne!(
            fixture.pointer("/duplicate_local_ids/0/origin_artifact"),
            fixture.pointer("/duplicate_local_ids/1/origin_artifact")
        );
        assert_eq!(
            fixture
                .pointer("/verification_gate_debt/auto_promoted_debts")
                .and_then(|value| value.as_u64()),
            Some(0)
        );
    }

    #[test]
    fn list_debts_ignores_scaffolding_readmes_and_keeps_malformed_debts_visible() {
        let dir = tempfile::tempdir().unwrap();
        let open_dir = dir.path().join(".lmbrain/debts/open");
        let planned_dir = dir.path().join(".lmbrain/debts/planned");
        fs::create_dir_all(&open_dir).unwrap();
        fs::create_dir_all(&planned_dir).unwrap();

        fs::write(
            dir.path().join(".lmbrain/debts/README.md"),
            "# Debts Scaffolding\n",
        )
        .unwrap();
        fs::write(open_dir.join("README.md"), "# Open Debts Scaffolding\n").unwrap();
        fs::write(
            planned_dir.join("README.md"),
            "# Planned Debts Scaffolding\n",
        )
        .unwrap();

        fs::write(
            planned_dir.join("DEBT-001-good.md"),
            "---\nid: DEBT-001\ntitle: Good debt\nstatus: planned\ncategory: architecture\nseverity: medium\ncreated: '2026-07-29'\nupdated: '2026-07-29'\n---\n## Statement\nValid statement.\n",
        )
        .unwrap();

        fs::write(
            open_dir.join("DEBT-002-malformed.md"),
            "Broken content without frontmatter",
        )
        .unwrap();

        let debts = list_debts(dir.path());
        assert_eq!(debts.len(), 2);
        assert_eq!(debts[0].id, "DEBT-001");
        assert_eq!(debts[0].status, "planned");
        assert!(!debts[0].malformed);

        assert_eq!(debts[1].id, "DEBT-002-malformed");
        assert_eq!(debts[1].status, "open");
        assert!(debts[1].malformed);
    }
}
