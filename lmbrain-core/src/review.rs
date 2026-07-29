use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    frontmatter::Document,
    taxonomy::{normalize_finding_category, CategoryNormalization, FINDING_TAXONOMY_VERSION},
};

pub const REVIEW_EVENT_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewEventInput {
    pub actor_role: String,
    pub reason: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub remediation_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewLifecycleEvent {
    pub schema_version: String,
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub from_status: String,
    pub to_status: String,
    pub actor_role: String,
    pub reason: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub implementation_agent: Option<String>,
    pub remediation_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewEventHistory {
    pub events: Vec<ReviewLifecycleEvent>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewHistorySource {
    StructuredEvents,
    LegacyExplicit,
    StatusOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewLifecycleAnalysis {
    pub source: ReviewHistorySource,
    pub confidence: String,
    pub review_passes: usize,
    pub remediation_cycles: usize,
    pub initial_verdict: Option<String>,
    pub final_verdict: Option<String>,
    pub escalation_count: usize,
    pub takeover_count: usize,
    pub remediation_agents: Vec<String>,
    pub escalation_owners: Vec<String>,
    pub takeover_owners: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewMigrationItem {
    pub id: String,
    pub path: String,
    pub lifecycle: ReviewLifecycleAnalysis,
    pub categories: Vec<CategoryNormalization>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewMigrationPreview {
    pub taxonomy_version: String,
    pub total_reviews: usize,
    pub structured_reviews: usize,
    pub legacy_explicit_reviews: usize,
    pub status_only_reviews: usize,
    pub reviews_with_warnings: usize,
    pub category_values: usize,
    pub canonical_or_alias_category_values: usize,
    pub lifecycle_coverage: f64,
    pub category_coverage: f64,
    pub items: Vec<ReviewMigrationItem>,
}

pub fn parse_review_event_history(document: &Document) -> ReviewEventHistory {
    let fields = document.fields();
    parse_review_event_value(
        &document.value("id").unwrap_or_default(),
        fields.get("review_events"),
    )
}

pub fn parse_review_event_value(review_id: &str, value: Option<&Value>) -> ReviewEventHistory {
    let Some(value) = value else {
        return ReviewEventHistory {
            events: Vec::new(),
            warnings: vec![
                "Review lifecycle history is absent; prior review cycles are unknown.".into(),
            ],
        };
    };
    let Some(items) = value.as_array() else {
        return ReviewEventHistory {
            events: Vec::new(),
            warnings: vec!["review_events must be an array of typed event objects.".into()],
        };
    };

    let mut ids = BTreeSet::new();
    let mut events = Vec::new();
    let mut warnings = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let event = match serde_json::from_value::<ReviewLifecycleEvent>(item.clone()) {
            Ok(event) => event,
            Err(error) => {
                warnings.push(format!(
                    "review_events entry {} is malformed: {error}",
                    index + 1
                ));
                continue;
            }
        };
        if event.schema_version != REVIEW_EVENT_SCHEMA_VERSION {
            warnings.push(format!(
                "{} uses unsupported review event schema version {}.",
                event.id, event.schema_version
            ));
        }
        if !review_id.is_empty() && !event.id.starts_with(&format!("{review_id}-EVENT-")) {
            warnings.push(format!(
                "{} does not belong to review {}.",
                event.id, review_id
            ));
        }
        if !ids.insert(event.id.clone()) {
            warnings.push(format!("Duplicate review event ID {}.", event.id));
            continue;
        }
        events.push(event);
    }
    if items.is_empty() {
        warnings.push("Review lifecycle history is empty; prior review cycles are unknown.".into());
    }

    ReviewEventHistory { events, warnings }
}

pub(crate) fn next_review_event_id(document: &Document, review_id: &str) -> String {
    let objects = document.object_array("review_events");
    let used = objects
        .iter()
        .filter_map(|event| event.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let mut sequence = objects.len() + 1;
    loop {
        let candidate = format!("{review_id}-EVENT-{sequence:03}");
        if !used.contains(candidate.as_str()) {
            return candidate;
        }
        sequence += 1;
    }
}

pub fn analyze_review_lifecycle(document: &Document) -> ReviewLifecycleAnalysis {
    let history = parse_review_event_history(document);
    let verdicts = history
        .events
        .iter()
        .filter(|event| event.action == "verdict")
        .collect::<Vec<_>>();
    if !history.events.is_empty() {
        let mut warnings = history.warnings;
        let explicit_passes = numeric_field(document, "review_cycles");
        if explicit_passes.is_some_and(|passes| passes != verdicts.len()) {
            warnings.push(format!(
                "review_cycles contradicts structured events: declared {}, observed {}.",
                explicit_passes.unwrap_or_default(),
                verdicts.len()
            ));
        }
        let remediation_cycles = verdicts
            .iter()
            .filter(|event| event.to_status == "changes-requested")
            .count();
        if numeric_field(document, "remediation_cycles")
            .is_some_and(|declared| declared != remediation_cycles)
        {
            warnings.push(format!(
                "remediation_cycles contradicts structured events: declared {}, observed {}.",
                numeric_field(document, "remediation_cycles").unwrap_or_default(),
                remediation_cycles
            ));
        }
        return ReviewLifecycleAnalysis {
            source: ReviewHistorySource::StructuredEvents,
            confidence: if warnings.is_empty() {
                "high"
            } else {
                "medium"
            }
            .into(),
            review_passes: verdicts.len(),
            remediation_cycles,
            initial_verdict: verdicts.first().map(|event| event.to_status.clone()),
            final_verdict: verdicts
                .last()
                .map(|event| event.to_status.clone())
                .or_else(|| document.value("status")),
            escalation_count: history
                .events
                .iter()
                .filter(|event| event.action == "escalation")
                .count(),
            takeover_count: history
                .events
                .iter()
                .filter(|event| event.action == "takeover")
                .count(),
            remediation_agents: distinct_event_agents(&history.events, "remediation"),
            escalation_owners: distinct_event_actors(&history.events, "escalation"),
            takeover_owners: distinct_event_actors(&history.events, "takeover"),
            warnings,
        };
    }

    let review_cycles = numeric_field(document, "review_cycles");
    let remediation_cycles = numeric_field(document, "remediation_cycles");
    let explicit_escalations =
        numeric_field(document, "escalations").or_else(|| document.value("escalation").map(|_| 1));
    let explicit_takeovers =
        numeric_field(document, "takeovers").or_else(|| document.value("takeover").map(|_| 1));
    let activity = document
        .object_array("activity")
        .iter()
        .filter_map(|entry| entry.get("action").and_then(Value::as_str))
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    let activity_escalations = activity
        .iter()
        .filter(|action| action.contains("escalat"))
        .count();
    let activity_takeovers = activity
        .iter()
        .filter(|action| action.contains("takeover"))
        .count();
    let escalation_count = explicit_escalations.unwrap_or(activity_escalations);
    let takeover_count = explicit_takeovers.unwrap_or_else(|| {
        if document
            .value("escalation")
            .is_some_and(|value| value.to_lowercase().contains("takeover"))
        {
            1
        } else {
            activity_takeovers
        }
    });
    let has_explicit_history = review_cycles.is_some()
        || remediation_cycles.is_some()
        || explicit_escalations.is_some()
        || explicit_takeovers.is_some()
        || activity_escalations > 0
        || activity_takeovers > 0;
    if has_explicit_history {
        let passes = review_cycles.unwrap_or_else(|| {
            remediation_cycles.unwrap_or_default()
                + usize::from(document.value("status").as_deref() == Some("accepted"))
        });
        let remediation = remediation_cycles.unwrap_or_else(|| passes.saturating_sub(1));
        let mut warnings = Vec::new();
        if review_cycles.is_some()
            && remediation_cycles.is_some()
            && passes != remediation.saturating_add(1)
        {
            warnings.push(format!(
                "review_cycles ({passes}) and remediation_cycles ({remediation}) contradict."
            ));
        }
        if explicit_escalations.is_some()
            && activity_escalations > 0
            && explicit_escalations != Some(activity_escalations)
        {
            warnings.push("Explicit escalation count contradicts activity entries.".into());
        }
        if explicit_takeovers.is_some()
            && activity_takeovers > 0
            && explicit_takeovers != Some(activity_takeovers)
        {
            warnings.push("Explicit takeover count contradicts activity entries.".into());
        }
        return ReviewLifecycleAnalysis {
            source: ReviewHistorySource::LegacyExplicit,
            confidence: if warnings.is_empty() { "medium" } else { "low" }.into(),
            review_passes: passes,
            remediation_cycles: remediation,
            initial_verdict: if remediation > 0 || passes > 1 {
                Some("changes-requested".into())
            } else {
                document.value("status")
            },
            final_verdict: document.value("status"),
            escalation_count,
            takeover_count,
            remediation_agents: Vec::new(),
            escalation_owners: if escalation_count > 0 {
                vec!["project-lead".into()]
            } else {
                Vec::new()
            },
            takeover_owners: if takeover_count > 0 {
                vec!["project-lead".into()]
            } else {
                Vec::new()
            },
            warnings,
        };
    }

    ReviewLifecycleAnalysis {
        source: ReviewHistorySource::StatusOnly,
        confidence: "low".into(),
        review_passes: 1,
        remediation_cycles: 0,
        initial_verdict: None,
        final_verdict: document.value("status"),
        escalation_count: 0,
        takeover_count: 0,
        remediation_agents: Vec::new(),
        escalation_owners: Vec::new(),
        takeover_owners: Vec::new(),
        warnings: vec![
            "Lifecycle has status only; first-pass outcome and prior remediation are unknown."
                .into(),
        ],
    }
}

fn numeric_field(document: &Document, key: &str) -> Option<usize> {
    document.value(key)?.trim().parse().ok()
}

fn distinct_event_agents(events: &[ReviewLifecycleEvent], action: &str) -> Vec<String> {
    events
        .iter()
        .filter(|event| event.action == action)
        .filter_map(|event| event.remediation_agent.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn distinct_event_actors(events: &[ReviewLifecycleEvent], action: &str) -> Vec<String> {
    events
        .iter()
        .filter(|event| event.action == action)
        .map(|event| event.actor_role.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn build_review_migration_preview(
    root: &Path,
) -> Result<ReviewMigrationPreview, std::io::Error> {
    let reviews_root = root.join(".lmbrain/reviews");
    let mut paths = review_markdown_files(&reviews_root);
    paths.sort();
    let mut items = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path)?;
        let Ok(document) = Document::parse(&source) else {
            continue;
        };
        let lifecycle = analyze_review_lifecycle(&document);
        let categories = document
            .string_array("finding_categories")
            .iter()
            .map(|category| normalize_finding_category(category))
            .collect();
        items.push(ReviewMigrationItem {
            id: document.value("id").unwrap_or_default(),
            path: path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/"),
            lifecycle,
            categories,
        });
    }
    let total_reviews = items.len();
    let structured_reviews = items
        .iter()
        .filter(|item| item.lifecycle.source == ReviewHistorySource::StructuredEvents)
        .count();
    let legacy_explicit_reviews = items
        .iter()
        .filter(|item| item.lifecycle.source == ReviewHistorySource::LegacyExplicit)
        .count();
    let status_only_reviews = total_reviews - structured_reviews - legacy_explicit_reviews;
    let category_values = items
        .iter()
        .map(|item| item.categories.len())
        .sum::<usize>();
    let canonical_or_alias_category_values = items
        .iter()
        .flat_map(|item| &item.categories)
        .filter(|category| category.canonical.is_some())
        .count();
    let reviews_with_warnings = items
        .iter()
        .filter(|item| {
            !item.lifecycle.warnings.is_empty()
                || item
                    .categories
                    .iter()
                    .any(|category| category.canonical.is_none())
        })
        .count();

    Ok(ReviewMigrationPreview {
        taxonomy_version: FINDING_TAXONOMY_VERSION.into(),
        total_reviews,
        structured_reviews,
        legacy_explicit_reviews,
        status_only_reviews,
        reviews_with_warnings,
        category_values,
        canonical_or_alias_category_values,
        lifecycle_coverage: fraction(structured_reviews + legacy_explicit_reviews, total_reviews),
        category_coverage: fraction(canonical_or_alias_category_values, category_values),
        items,
    })
}

fn review_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("templates") {
                    continue;
                }
                files.extend(review_markdown_files(&path));
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    files
}

fn fraction(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_and_malformed_histories_report_uncertainty_without_fabricating_events() {
        let legacy = Document::parse("---\nid: REVIEW-001\nstatus: accepted\n---\n").unwrap();
        let history = parse_review_event_history(&legacy);
        assert!(history.events.is_empty());
        assert_eq!(history.warnings.len(), 1);

        let malformed = Document::parse(
            "---\nid: REVIEW-001\nreview_events:\n  - id: \"REVIEW-001-EVENT-001\"\n---\n",
        )
        .unwrap();
        let history = parse_review_event_history(&malformed);
        assert!(history.events.is_empty());
        assert!(history.warnings[0].contains("malformed"));
    }

    #[test]
    fn xenomark_shaped_explicit_history_is_not_first_pass_success() {
        let review = Document::parse(
            "---\nid: REVIEW-047\nstatus: accepted\nreview_cycles: 3\nescalation: \"takeover correttivo del Project Lead\"\n---\n",
        )
        .unwrap();
        let analysis = analyze_review_lifecycle(&review);
        assert_eq!(analysis.review_passes, 3);
        assert_eq!(analysis.remediation_cycles, 2);
        assert_eq!(
            analysis.initial_verdict.as_deref(),
            Some("changes-requested")
        );
        assert_eq!(analysis.escalation_count, 1);
        assert_eq!(analysis.takeover_count, 1);
        assert_eq!(analysis.source, ReviewHistorySource::LegacyExplicit);
    }

    #[test]
    fn remediation_cycle_field_implies_a_final_review_pass() {
        let review = Document::parse(
            "---\nid: REVIEW-054\nstatus: accepted\nremediation_cycles: 2\nescalations: 1\n---\n",
        )
        .unwrap();
        let analysis = analyze_review_lifecycle(&review);
        assert_eq!(analysis.review_passes, 3);
        assert_eq!(analysis.remediation_cycles, 2);
        assert_eq!(analysis.escalation_count, 1);
        assert_eq!(analysis.takeover_count, 0);
    }

    #[test]
    fn migration_preview_is_deterministic_and_non_mutating() {
        let directory = tempfile::tempdir().unwrap();
        let reviews = directory.path().join(".lmbrain/reviews/accepted");
        fs::create_dir_all(&reviews).unwrap();
        fs::write(
            reviews.join("REVIEW-002.md"),
            "---\nid: REVIEW-002\nstatus: accepted\nfinding_categories: [project-specific]\n---\n",
        )
        .unwrap();
        fs::write(
            reviews.join("REVIEW-001.md"),
            "---\nid: REVIEW-001\nstatus: accepted\nreview_cycles: 2\nfinding_categories: [evidence-integrity]\n---\n",
        )
        .unwrap();
        let before = fs::read_to_string(reviews.join("REVIEW-001.md")).unwrap();
        let first = build_review_migration_preview(directory.path()).unwrap();
        let second = build_review_migration_preview(directory.path()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.total_reviews, 2);
        assert_eq!(first.legacy_explicit_reviews, 1);
        assert_eq!(first.status_only_reviews, 1);
        assert_eq!(first.category_values, 2);
        assert_eq!(first.canonical_or_alias_category_values, 1);
        assert_eq!(first.items[0].id, "REVIEW-001");
        assert_eq!(
            fs::read_to_string(reviews.join("REVIEW-001.md")).unwrap(),
            before
        );
    }
}
