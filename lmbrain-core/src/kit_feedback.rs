use std::{fs, path::Path};

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::ArtifactMutationLock,
    path::{PathError, PathGuard},
};

pub const KIT_FEEDBACK_SCHEMA_VERSION: &str = "1";
pub const KIT_FEEDBACK_REPORT_PATH: &str = ".lmbrain/reports/lmbrain-kit-feedback.md";
const CATEGORIES: &[&str] = &[
    "bug",
    "usability",
    "workflow",
    "documentation",
    "compatibility",
    "performance",
    "improvement",
];
const SEVERITIES: &[&str] = &["blocking", "high", "medium", "low", "info"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitFeedbackInput {
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub observed_behavior: String,
    pub expected_behavior: String,
    pub impact: String,
    pub evidence: String,
    pub workaround: Option<String>,
    pub suggested_improvement: Option<String>,
    pub related_note: Option<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitFeedbackNote {
    pub id: String,
    pub timestamp: String,
    pub lmbrain_version: String,
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub observed_behavior: String,
    pub expected_behavior: String,
    pub impact: String,
    pub evidence: String,
    pub workaround: Option<String>,
    pub suggested_improvement: Option<String>,
    pub related_note: Option<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitFeedbackReport {
    pub schema_version: String,
    pub path: String,
    pub updated: String,
    pub total: usize,
    pub counts_by_category: std::collections::BTreeMap<String, usize>,
    pub counts_by_severity: std::collections::BTreeMap<String, usize>,
    pub notes: Vec<KitFeedbackNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitFeedbackMutation {
    pub path: String,
    pub note: KitFeedbackNote,
    pub total: usize,
    pub mutated: bool,
}

#[derive(Debug, Error)]
pub enum KitFeedbackError {
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("invalid LMBrain kit feedback: {0}")]
    Invalid(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn record_kit_feedback(
    root: &Path,
    input: KitFeedbackInput,
) -> Result<KitFeedbackMutation, KitFeedbackError> {
    validate_input(&input)?;
    let guard = PathGuard::new(root)?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), "lmbrain-kit-feedback")?;
    let path = guard.root().join(KIT_FEEDBACK_REPORT_PATH);
    let source = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        initial_report(&read_version(guard.root()))
    };
    let mut document = Document::parse(&source)?;
    validate_report_document(&document)?;
    let existing = parse_notes(&document)?;

    if let Some(related) = input.related_note.as_deref() {
        if !existing.iter().any(|note| note.id == related) {
            return Err(KitFeedbackError::Invalid(format!(
                "related_note '{related}' does not exist in the report"
            )));
        }
    }

    let note = KitFeedbackNote {
        id: format!("KIT-NOTE-{:03}", next_note_number(&existing)),
        timestamp: Local::now().to_rfc3339(),
        lmbrain_version: read_version(guard.root()),
        category: input.category.trim().to_string(),
        severity: input.severity.trim().to_string(),
        summary: input.summary.trim().to_string(),
        observed_behavior: input.observed_behavior.trim().to_string(),
        expected_behavior: input.expected_behavior.trim().to_string(),
        impact: input.impact.trim().to_string(),
        evidence: input.evidence.trim().to_string(),
        workaround: clean_optional(input.workaround),
        suggested_improvement: clean_optional(input.suggested_improvement),
        related_note: clean_optional(input.related_note),
        actor: input.actor.trim().to_string(),
    };
    document.append_object(
        "notes",
        &[
            ("schema_version".into(), json!(KIT_FEEDBACK_SCHEMA_VERSION)),
            ("id".into(), json!(note.id)),
            ("timestamp".into(), json!(note.timestamp)),
            ("lmbrain_version".into(), json!(note.lmbrain_version)),
            ("category".into(), json!(note.category)),
            ("severity".into(), json!(note.severity)),
            ("summary".into(), json!(note.summary)),
            ("observed_behavior".into(), json!(note.observed_behavior)),
            ("expected_behavior".into(), json!(note.expected_behavior)),
            ("impact".into(), json!(note.impact)),
            ("evidence".into(), json!(note.evidence)),
            ("workaround".into(), json!(note.workaround)),
            (
                "suggested_improvement".into(),
                json!(note.suggested_improvement),
            ),
            ("related_note".into(), json!(note.related_note)),
            ("actor".into(), json!(note.actor)),
        ],
    )?;
    document.set("updated", &Local::now().format("%Y-%m-%d").to_string());
    fs::create_dir_all(path.parent().ok_or_else(|| {
        KitFeedbackError::Invalid("feedback report has no parent directory".into())
    })?)?;
    atomic_write(&path, &document.render())?;
    Ok(KitFeedbackMutation {
        path: KIT_FEEDBACK_REPORT_PATH.into(),
        note,
        total: existing.len() + 1,
        mutated: true,
    })
}

pub fn read_kit_feedback(root: &Path) -> Result<KitFeedbackReport, KitFeedbackError> {
    let guard = PathGuard::new(root)?;
    let path = guard.root().join(KIT_FEEDBACK_REPORT_PATH);
    let source = if path.exists() {
        fs::read_to_string(path)?
    } else {
        initial_report(&read_version(guard.root()))
    };
    let document = Document::parse(&source)?;
    validate_report_document(&document)?;
    let notes = parse_notes(&document)?;
    let mut counts_by_category = std::collections::BTreeMap::new();
    let mut counts_by_severity = std::collections::BTreeMap::new();
    for note in &notes {
        *counts_by_category.entry(note.category.clone()).or_insert(0) += 1;
        *counts_by_severity.entry(note.severity.clone()).or_insert(0) += 1;
    }
    Ok(KitFeedbackReport {
        schema_version: KIT_FEEDBACK_SCHEMA_VERSION.into(),
        path: KIT_FEEDBACK_REPORT_PATH.into(),
        updated: document.value("updated").unwrap_or_default(),
        total: notes.len(),
        counts_by_category,
        counts_by_severity,
        notes,
    })
}

fn validate_input(input: &KitFeedbackInput) -> Result<(), KitFeedbackError> {
    for (label, value) in [
        ("category", input.category.as_str()),
        ("severity", input.severity.as_str()),
        ("summary", input.summary.as_str()),
        ("observed_behavior", input.observed_behavior.as_str()),
        ("expected_behavior", input.expected_behavior.as_str()),
        ("impact", input.impact.as_str()),
        ("evidence", input.evidence.as_str()),
        ("actor", input.actor.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(KitFeedbackError::Invalid(format!(
                "{label} cannot be empty"
            )));
        }
    }
    if !CATEGORIES.contains(&input.category.trim()) {
        return Err(KitFeedbackError::Invalid(format!(
            "category must be one of {}",
            CATEGORIES.join(", ")
        )));
    }
    if !SEVERITIES.contains(&input.severity.trim()) {
        return Err(KitFeedbackError::Invalid(format!(
            "severity must be one of {}",
            SEVERITIES.join(", ")
        )));
    }
    for (label, value, limit) in [
        ("summary", input.summary.as_str(), 240usize),
        ("observed_behavior", input.observed_behavior.as_str(), 1200),
        ("expected_behavior", input.expected_behavior.as_str(), 1200),
        ("impact", input.impact.as_str(), 1200),
        ("evidence", input.evidence.as_str(), 2400),
        ("actor", input.actor.as_str(), 120),
    ] {
        if value.chars().count() > limit {
            return Err(KitFeedbackError::Invalid(format!(
                "{label} exceeds the {limit}-character report bound"
            )));
        }
        if contains_secret_like_value(value) {
            return Err(KitFeedbackError::Invalid(format!(
                "{label} appears to contain a credential or secret assignment; redact it before recording feedback"
            )));
        }
    }
    for (label, value) in [
        ("workaround", input.workaround.as_deref()),
        (
            "suggested_improvement",
            input.suggested_improvement.as_deref(),
        ),
        ("related_note", input.related_note.as_deref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(KitFeedbackError::Invalid(format!(
                "{label} cannot be blank when provided"
            )));
        }
    }
    Ok(())
}

fn validate_report_document(document: &Document) -> Result<(), KitFeedbackError> {
    if document.value("id").as_deref() != Some("LMBRAIN-KIT-FEEDBACK") {
        return Err(KitFeedbackError::Invalid(
            "report id must be LMBRAIN-KIT-FEEDBACK".into(),
        ));
    }
    if document.value("schema_version").as_deref() != Some(KIT_FEEDBACK_SCHEMA_VERSION) {
        return Err(KitFeedbackError::Invalid(format!(
            "report schema_version must be {KIT_FEEDBACK_SCHEMA_VERSION}"
        )));
    }
    let fields = document.fields();
    if fields.get("notes").is_some_and(|value| !value.is_array()) {
        return Err(KitFeedbackError::Invalid(
            "notes must be an array of typed objects".into(),
        ));
    }
    Ok(())
}

fn parse_notes(document: &Document) -> Result<Vec<KitFeedbackNote>, KitFeedbackError> {
    let notes = document
        .object_array("notes")
        .into_iter()
        .map(|note| {
            if note
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                != Some(KIT_FEEDBACK_SCHEMA_VERSION)
            {
                return Err(KitFeedbackError::Invalid(
                    "feedback note has an unsupported schema_version".into(),
                ));
            }
            serde_json::from_value::<KitFeedbackNote>(serde_json::Value::Object(note)).map_err(
                |error| KitFeedbackError::Invalid(format!("malformed feedback note: {error}")),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ids = notes
        .iter()
        .map(|note| note.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != notes.len() {
        return Err(KitFeedbackError::Invalid(
            "feedback note IDs must be unique".into(),
        ));
    }
    for note in &notes {
        if !note.id.starts_with("KIT-NOTE-")
            || note
                .id
                .strip_prefix("KIT-NOTE-")
                .and_then(|value| value.parse::<usize>().ok())
                .is_none()
        {
            return Err(KitFeedbackError::Invalid(format!(
                "invalid feedback note ID '{}'",
                note.id
            )));
        }
        if !CATEGORIES.contains(&note.category.as_str())
            || !SEVERITIES.contains(&note.severity.as_str())
        {
            return Err(KitFeedbackError::Invalid(format!(
                "feedback note '{}' has invalid taxonomy",
                note.id
            )));
        }
        if let Some(related) = note.related_note.as_deref() {
            if !ids.contains(related) {
                return Err(KitFeedbackError::Invalid(format!(
                    "feedback note '{}' references missing related_note '{}'",
                    note.id, related
                )));
            }
        }
    }
    Ok(notes)
}

fn next_note_number(notes: &[KitFeedbackNote]) -> usize {
    notes
        .iter()
        .filter_map(|note| note.id.strip_prefix("KIT-NOTE-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
        + 1
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string())
}

fn contains_secret_like_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "github_pat_",
        "ghp_",
        "sk-",
        "password=",
        "password:",
        "token=",
        "token:",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn read_version(root: &Path) -> String {
    fs::read_to_string(root.join(".lmbrain/VERSION"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn initial_report(version: &str) -> String {
    format!(
        "---\nid: LMBRAIN-KIT-FEEDBACK\nschema_version: {KIT_FEEDBACK_SCHEMA_VERSION}\nupdated: {}\nlmbrain_version: {version}\nnotes: []\n---\n# LMBrain kit feedback\n\nThis append-only report records evidence-backed observations about LMBrain itself. It is not project backlog, a FINDING-* registry, or lifecycle authority.\n",
        Local::now().format("%Y-%m-%d")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(summary: &str) -> KitFeedbackInput {
        KitFeedbackInput {
            category: "usability".into(),
            severity: "medium".into(),
            summary: summary.into(),
            observed_behavior: "Operator-facing text uses unexplained abbreviations.".into(),
            expected_behavior: "Operator-facing text is concise and understandable.".into(),
            impact: "The operator must ask for a second explanation.".into(),
            evidence: "Observed in a Project Lead handoff response.".into(),
            workaround: Some("Ask the Lead to explain the shorthand.".into()),
            suggested_improvement: Some("Separate operator and agent communication styles.".into()),
            related_note: None,
            actor: "AGENT-LEAD".into(),
        }
    }

    #[test]
    fn report_is_created_append_only_and_readable() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain")).unwrap();
        fs::write(dir.path().join(".lmbrain/VERSION"), "3.1.0\n").unwrap();
        let first = record_kit_feedback(dir.path(), input("Human-facing language")).unwrap();
        let mut second_input = input("Repeated workaround");
        second_input.related_note = Some(first.note.id.clone());
        record_kit_feedback(dir.path(), second_input).unwrap();
        let report = read_kit_feedback(dir.path()).unwrap();
        assert_eq!(report.total, 2);
        assert_eq!(report.notes[1].related_note, Some("KIT-NOTE-001".into()));
        assert_eq!(report.counts_by_category["usability"], 2);
        assert!(
            fs::read_to_string(dir.path().join(KIT_FEEDBACK_REPORT_PATH))
                .unwrap()
                .contains("KIT-NOTE-002")
        );
    }

    #[test]
    fn invalid_taxonomy_and_missing_related_note_fail_without_mutation() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/reports")).unwrap();
        let mut invalid = input("Invalid");
        invalid.category = "project-bug".into();
        assert!(record_kit_feedback(dir.path(), invalid).is_err());
        assert!(!dir.path().join(KIT_FEEDBACK_REPORT_PATH).exists());

        let mut missing = input("Missing relation");
        missing.related_note = Some("KIT-NOTE-999".into());
        assert!(record_kit_feedback(dir.path(), missing).is_err());
        assert!(!dir.path().join(KIT_FEEDBACK_REPORT_PATH).exists());
    }

    #[test]
    fn absent_report_can_be_read_without_creating_it() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain")).unwrap();
        let report = read_kit_feedback(dir.path()).unwrap();
        assert_eq!(report.total, 0);
        assert!(!dir.path().join(KIT_FEEDBACK_REPORT_PATH).exists());
    }

    #[test]
    fn likely_credentials_and_oversized_evidence_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain")).unwrap();
        let mut secret = input("Secret");
        secret.evidence = "token=do-not-store".into();
        assert!(record_kit_feedback(dir.path(), secret).is_err());
        let mut oversized = input("Oversized");
        oversized.evidence = "x".repeat(2401);
        assert!(record_kit_feedback(dir.path(), oversized).is_err());
        assert!(!dir.path().join(KIT_FEEDBACK_REPORT_PATH).exists());
    }

    #[test]
    fn malformed_existing_report_surfaces_a_shared_diagnostic() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/reports")).unwrap();
        fs::write(
            dir.path().join(KIT_FEEDBACK_REPORT_PATH),
            "---\nid: WRONG\nschema_version: 1\nupdated: 2026-07-29\nnotes: []\n---\n",
        )
        .unwrap();
        assert!(crate::build_diagnostics(dir.path())
            .iter()
            .any(|diagnostic| diagnostic.code == "kit-feedback-invalid"));
    }
}
