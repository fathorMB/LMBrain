use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    analyze_review_lifecycle, content_digest,
    frontmatter::{atomic_write, Document},
    normalize_finding_category,
    path::PathGuard,
    transitions::{create, ArtifactKind, CreateRequest, TransitionError},
    ReviewHistorySource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentImprovementSignal {
    pub target_profile: String,
    pub category: String,
    pub distinct_specs: Vec<String>,
    pub reviews: Vec<String>,
    pub threshold_met: bool,
    pub rationale: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentEffectivenessMetrics {
    pub profile: String,
    pub reviewed_specs: usize,
    pub accepted_specs: usize,
    pub specs_with_changes_requested: usize,
    pub transcript_fast_fail_reviews: usize,
    pub review_cycles: usize,
    pub remediation_cycles: usize,
    pub lead_escalation_reviews: usize,
    pub escalation_count: usize,
    pub takeover_count: usize,
    pub categorized_findings: usize,
    pub uncategorized_reviews: usize,
    pub first_pass_accepted_specs: usize,
    pub first_pass_acceptance_rate: f64,
    pub average_review_cycles: f64,
    pub transcript_fast_fail_rate: f64,
    pub lead_escalation_rate: f64,
    pub lifecycle_known_reviews: usize,
    pub lifecycle_unknown_reviews: usize,
    pub lifecycle_coverage: f64,
    pub category_values: usize,
    pub canonical_category_values: usize,
    pub category_alias_values: usize,
    pub unknown_categories: Vec<String>,
    pub category_coverage: f64,
    pub attribution_basis: String,
    pub confidence: String,
    pub diagnostics: Vec<String>,
    pub data_quality_caveat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposalRequest {
    pub target_profile: String,
    pub category: String,
    pub evidence_reviews: Vec<String>,
    pub evidence_specs: Vec<String>,
    #[serde(default)]
    pub add_review_focus: Vec<String>,
    #[serde(default)]
    pub add_skills: Vec<String>,
    #[serde(default)]
    pub add_constraints: Vec<String>,
    #[serde(default)]
    pub add_primary_files: Vec<String>,
    #[serde(default)]
    pub guidance: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImprovementApplyResult {
    pub proposal_id: String,
    pub target_profile: String,
    pub target_path: String,
    pub before_digest: String,
    pub after_digest: String,
    pub applied: bool,
}

#[derive(Debug, Error)]
pub enum ImprovementError {
    #[error("cannot read or write improvement artifacts: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid improvement artifact: {0}")]
    Invalid(String),
    #[error("target profile changed after proposal creation")]
    StaleTarget,
    #[error("improvement proposal must be approved by the operator before apply")]
    NotApproved,
    #[error(transparent)]
    Transition(#[from] TransitionError),
}

#[derive(Debug, Default)]
struct SignalAccumulator {
    specs: BTreeSet<String>,
    reviews: BTreeSet<String>,
}

pub fn build_agent_improvement_signals(
    root: &Path,
) -> Result<(Vec<AgentImprovementSignal>, Vec<AgentEffectivenessMetrics>), ImprovementError> {
    let reviews_dir = root.join(".lmbrain/reviews");
    let mut signals: BTreeMap<(String, String), SignalAccumulator> = BTreeMap::new();
    let mut metrics: BTreeMap<String, MetricsAccumulator> = BTreeMap::new();
    for path in markdown_files(&reviews_dir) {
        let source = fs::read_to_string(&path)?;
        let Ok(doc) = Document::parse(&source) else {
            continue;
        };
        let agent = doc.value("implementation_agent").unwrap_or_default();
        if agent.is_empty() {
            continue;
        }
        let review_id = doc.value("id").unwrap_or_default();
        let spec = doc.value("spec").unwrap_or_default();
        let status = doc.value("status").unwrap_or_default();
        let tags = doc.string_array("tags");
        let mut categories = doc.string_array("finding_categories");
        if categories.is_empty() {
            if tags
                .iter()
                .any(|tag| tag == "transcript-fast-fail" || tag == "fast-fail")
            {
                categories.push("verification-transcript".into());
            }
            if tags.iter().any(|tag| tag == "evidence-integrity") {
                categories.push("evidence-integrity".into());
            }
        }
        let lifecycle = analyze_review_lifecycle(&doc);
        let metric = metrics.entry(agent.clone()).or_default();
        metric.review_cycles += lifecycle.review_passes;
        metric.remediation_cycles += lifecycle.remediation_cycles;
        metric.escalation_count += lifecycle.escalation_count;
        metric.takeover_count += lifecycle.takeover_count;
        if lifecycle.source == ReviewHistorySource::StatusOnly {
            metric.lifecycle_unknown_reviews += 1;
        } else {
            metric.lifecycle_known_reviews += 1;
        }
        metric.diagnostics.extend(
            lifecycle
                .warnings
                .iter()
                .map(|warning| format!("{review_id}: {warning}")),
        );
        if !spec.is_empty() {
            metric.reviewed_specs.insert(spec.clone());
            metric
                .spec_events
                .entry(spec.clone())
                .or_default()
                .push(SpecReviewEvent {
                    created: doc.value("created").unwrap_or_default(),
                    review_id: review_id.clone(),
                    status: status.clone(),
                    initial_verdict: lifecycle.initial_verdict.clone(),
                    lifecycle_known: lifecycle.source != ReviewHistorySource::StatusOnly,
                });
        }
        if status == "accepted" && !spec.is_empty() {
            metric.accepted_specs.insert(spec.clone());
        }
        if (status == "changes-requested" || lifecycle.remediation_cycles > 0) && !spec.is_empty() {
            metric.changed_specs.insert(spec.clone());
        }
        if tags
            .iter()
            .any(|tag| tag == "transcript-fast-fail" || tag == "fast-fail")
        {
            metric.transcript_fast_fail_reviews += 1;
        }
        if lifecycle.escalation_count > 0
            || agent == "AGENT-LEAD"
            || tags.iter().any(|tag| tag == "lead-escalation")
        {
            metric.lead_escalation_reviews += 1;
        }
        if categories.is_empty() {
            metric.uncategorized_reviews += 1;
        }
        for raw_category in categories {
            metric.category_values += 1;
            let normalized = normalize_finding_category(&raw_category);
            let Some(category) = normalized.canonical else {
                metric.unknown_categories.insert(raw_category.clone());
                metric.diagnostics.push(format!(
                    "{review_id}: unknown finding category '{raw_category}'."
                ));
                continue;
            };
            metric.canonical_category_values += 1;
            metric.categorized_findings += 1;
            if normalized.is_alias {
                metric.category_alias_values += 1;
            }
            let entry = signals.entry((agent.clone(), category)).or_default();
            if !spec.is_empty() {
                entry.specs.insert(spec.clone());
            }
            if !review_id.is_empty() {
                entry.reviews.insert(review_id.clone());
            }
        }
    }
    let signals = signals
        .into_iter()
        .map(|((target_profile, category), evidence)| {
            let integrity = matches!(
                category.as_str(),
                "verification-integrity" | "security-boundary"
            );
            let threshold_met = evidence.specs.len() >= 2 || (integrity && !evidence.reviews.is_empty());
            AgentImprovementSignal {
                target_profile,
                category: category.clone(),
                distinct_specs: evidence.specs.into_iter().collect(),
                reviews: evidence.reviews.into_iter().collect(),
                threshold_met,
                rationale: if integrity {
                    format!("{category} is an integrity-sensitive finding; one evidenced occurrence is sufficient for operator review")
                } else {
                    "default threshold requires the same category on two distinct specs".into()
                },
            }
        })
        .collect();
    let metrics = metrics
        .into_iter()
        .map(|(profile, mut value)| {
            let reviewed_specs = value.reviewed_specs.len();
            let mut first_pass_eligible_specs = 0;
            let first_pass_accepted_specs = value
                .spec_events
                .values_mut()
                .filter_map(|events| {
                    events.sort_by(|left, right| {
                        (&left.created, &left.review_id).cmp(&(&right.created, &right.review_id))
                    });
                    let first = events.first()?;
                    let separate_artifacts_are_ordered = events.len() > 1
                        && events.iter().all(|event| !event.created.trim().is_empty());
                    if !first.lifecycle_known && !separate_artifacts_are_ordered {
                        return None;
                    }
                    first_pass_eligible_specs += 1;
                    Some(
                        first
                            .initial_verdict
                            .as_deref()
                            .unwrap_or(&first.status)
                            == "accepted",
                    )
                })
                .filter(|accepted| *accepted)
                .count();
            let denominator = reviewed_specs.max(1) as f64;
            let cycle_denominator = value.review_cycles.max(1) as f64;
            let review_count = value.lifecycle_known_reviews + value.lifecycle_unknown_reviews;
            let lifecycle_coverage = ratio(value.lifecycle_known_reviews, review_count);
            let category_coverage = ratio(value.canonical_category_values, value.category_values);
            let confidence = if lifecycle_coverage >= 0.9
                && category_coverage >= 0.9
                && reviewed_specs >= 2
            {
                "high"
            } else if lifecycle_coverage >= 0.6 && category_coverage >= 0.6 {
                "medium"
            } else {
                "low"
            };
            AgentEffectivenessMetrics {
            profile,
            reviewed_specs,
            accepted_specs: value.accepted_specs.len(),
            specs_with_changes_requested: value.changed_specs.len(),
            transcript_fast_fail_reviews: value.transcript_fast_fail_reviews,
            review_cycles: value.review_cycles,
            remediation_cycles: value.remediation_cycles,
            lead_escalation_reviews: value.lead_escalation_reviews,
            escalation_count: value.escalation_count,
            takeover_count: value.takeover_count,
            categorized_findings: value.categorized_findings,
            uncategorized_reviews: value.uncategorized_reviews,
            first_pass_accepted_specs,
            first_pass_acceptance_rate: ratio(first_pass_accepted_specs, first_pass_eligible_specs),
            average_review_cycles: value.review_cycles as f64 / denominator,
            transcript_fast_fail_rate: value.transcript_fast_fail_reviews as f64 / cycle_denominator,
            lead_escalation_rate: value.lead_escalation_reviews as f64 / cycle_denominator,
            lifecycle_known_reviews: value.lifecycle_known_reviews,
            lifecycle_unknown_reviews: value.lifecycle_unknown_reviews,
            lifecycle_coverage,
            category_values: value.category_values,
            canonical_category_values: value.canonical_category_values,
            category_alias_values: value.category_alias_values,
            unknown_categories: value.unknown_categories.into_iter().collect(),
            category_coverage,
            attribution_basis: "original-implementation-agent".into(),
            confidence: confidence.into(),
            diagnostics: value.diagnostics,
            data_quality_caveat: format!(
                "Metrics are attributed to the original implementation agent; lifecycle coverage {:.0}% and category coverage {:.0}% yield {confidence} confidence. Escalation and takeover are outcomes, not implementation credit.",
                lifecycle_coverage * 100.0,
                category_coverage * 100.0
            ),
        }})
        .collect();
    Ok((signals, metrics))
}

#[derive(Debug, Default)]
struct MetricsAccumulator {
    reviewed_specs: BTreeSet<String>,
    accepted_specs: BTreeSet<String>,
    changed_specs: BTreeSet<String>,
    transcript_fast_fail_reviews: usize,
    review_cycles: usize,
    remediation_cycles: usize,
    lead_escalation_reviews: usize,
    escalation_count: usize,
    takeover_count: usize,
    categorized_findings: usize,
    uncategorized_reviews: usize,
    lifecycle_known_reviews: usize,
    lifecycle_unknown_reviews: usize,
    category_values: usize,
    canonical_category_values: usize,
    category_alias_values: usize,
    unknown_categories: BTreeSet<String>,
    diagnostics: Vec<String>,
    spec_events: BTreeMap<String, Vec<SpecReviewEvent>>,
}

#[derive(Debug)]
struct SpecReviewEvent {
    created: String,
    review_id: String,
    status: String,
    initial_verdict: Option<String>,
    lifecycle_known: bool,
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

pub fn create_improvement_proposal(
    root: &Path,
    request: &ImprovementProposalRequest,
) -> Result<PathBuf, ImprovementError> {
    validate_request(request)?;
    let target = find_profile(root, &request.target_profile)?;
    let target_source = fs::read_to_string(&target)?;
    let target_digest = content_digest(target_source.as_bytes());
    let title = format!(
        "Improve {} for {}",
        request.target_profile, request.category
    );
    let result = create(
        root,
        CreateRequest {
            kind: ArtifactKind::AgentProposal,
            title,
            status: Some("proposed".into()),
            fields: vec![
                ("proposal_type".into(), "improvement".into()),
                ("target_profile".into(), request.target_profile.clone()),
                ("target_digest".into(), target_digest),
                ("reason".into(), "repeated-review-finding".into()),
                ("finding_category".into(), request.category.clone()),
                ("links".into(), inline_array(&request.evidence_reviews)),
                (
                    "recommended_for".into(),
                    inline_array(&request.evidence_specs),
                ),
                (
                    "add_review_focus".into(),
                    inline_array(&request.add_review_focus),
                ),
                ("add_skills".into(), inline_array(&request.add_skills)),
                (
                    "add_constraints".into(),
                    inline_array(&request.add_constraints),
                ),
                (
                    "add_primary_files".into(),
                    inline_array(&request.add_primary_files),
                ),
                ("tags".into(), "[proposal, improvement]".into()),
            ],
        },
    )?;
    let mut document = Document::parse(&fs::read_to_string(&result.path)?)
        .map_err(|error| ImprovementError::Invalid(error.to_string()))?;
    document.body = format!(
        "# Improvement proposal\n\n## Observed problem\n\nCategory `{}` recurred in the linked review evidence for `{}`.\n\n## Proposed guidance\n\n{}\n\n## Evidence\n\nReviews: {}\n\nSpecs: {}\n\n## Decision requested\n\n- [ ] Approve\n- [ ] Defer\n- [ ] Reject\n",
        request.category,
        request.target_profile,
        request.guidance.as_deref().unwrap_or("No prose guidance; apply only the structured additive fields."),
        request.evidence_reviews.join(", "),
        request.evidence_specs.join(", ")
    );
    atomic_write(&result.path, &document.render())
        .map_err(|error| ImprovementError::Invalid(error.to_string()))?;
    Ok(result.path)
}

pub fn apply_improvement_proposal(
    root: &Path,
    proposal_path: &Path,
) -> Result<ImprovementApplyResult, ImprovementError> {
    let guard = PathGuard::new(root).map_err(TransitionError::from)?;
    let proposal_path = guard
        .resolve_existing(proposal_path)
        .map_err(TransitionError::from)?;
    let mut proposal = Document::parse(&fs::read_to_string(&proposal_path)?)
        .map_err(|error| ImprovementError::Invalid(error.to_string()))?;
    if proposal.value("status").as_deref() != Some("approved") {
        return Err(ImprovementError::NotApproved);
    }
    if proposal.value("proposal_type").as_deref() != Some("improvement") {
        return Err(ImprovementError::Invalid(
            "proposal_type must be improvement".into(),
        ));
    }
    if proposal.bool("applied") == Some(true) {
        let target_profile = proposal.value("target_profile").unwrap_or_default();
        let target = find_profile(root, &target_profile)?;
        let digest = content_digest(&fs::read(&target)?);
        return Ok(ImprovementApplyResult {
            proposal_id: proposal.value("id").unwrap_or_default(),
            target_profile,
            target_path: target.to_string_lossy().into_owned(),
            before_digest: digest.clone(),
            after_digest: digest,
            applied: false,
        });
    }
    let target_profile = proposal.value("target_profile").unwrap_or_default();
    let target_path = find_profile(root, &target_profile)?;
    let target_source = fs::read_to_string(&target_path)?;
    let before_digest = content_digest(target_source.as_bytes());
    if proposal.value("target_digest").as_deref() != Some(before_digest.as_str()) {
        return Err(ImprovementError::StaleTarget);
    }
    let mut target = Document::parse(&target_source)
        .map_err(|error| ImprovementError::Invalid(error.to_string()))?;
    add_values(
        &mut target,
        "review_focus",
        proposal.string_array("add_review_focus"),
    );
    add_values(&mut target, "skills", proposal.string_array("add_skills"));
    add_values(
        &mut target,
        "constraints",
        proposal.string_array("add_constraints"),
    );
    add_values(
        &mut target,
        "primary_files",
        proposal.string_array("add_primary_files"),
    );
    if let Some(replaces) = proposal.value("replaces_text").or_else(|| proposal.value("supersedes_text")) {
        if !replaces.trim().is_empty() {
            target.body = target.body.replace(&replaces, &format!("~~{replaces}~~"));
        }
    }
    if let Some(guidance) = extract_section(&proposal.body, "Proposed guidance") {
        if !guidance.starts_with("No prose guidance") && !guidance.trim().is_empty() {
            target.body.push_str(&format!(
                "\n\n## Approved improvement guidance\n\nSource: [[{}]]\n\n{}\n",
                proposal.value("id").unwrap_or_default(),
                guidance.trim()
            ));
        }
    }
    target.set("updated", &today());
    target.append_activity(&format!(
        "applied improvement {}",
        proposal.value("id").unwrap_or_default()
    ));
    let rendered = target.render();
    atomic_write(&target_path, &rendered)
        .map_err(|error| ImprovementError::Invalid(error.to_string()))?;
    let after_digest = content_digest(rendered.as_bytes());
    proposal.set("applied", "true");
    proposal.set("applied_target_digest", &after_digest);
    proposal.set("updated", &today());
    proposal.append_activity(&format!(
        "applied to {target_profile}: {before_digest} -> {after_digest}"
    ));
    if let Err(error) = atomic_write(&proposal_path, &proposal.render()) {
        let rollback = atomic_write(&target_path, &target_source);
        return Err(ImprovementError::Invalid(match rollback {
            Ok(()) => format!("proposal audit write failed; target profile was rolled back: {error}"),
            Err(rollback_error) => format!(
                "proposal audit write failed and target rollback also failed: {error}; {rollback_error}"
            ),
        }));
    }
    Ok(ImprovementApplyResult {
        proposal_id: proposal.value("id").unwrap_or_default(),
        target_profile,
        target_path: target_path.to_string_lossy().into_owned(),
        before_digest,
        after_digest,
        applied: true,
    })
}

fn validate_request(request: &ImprovementProposalRequest) -> Result<(), ImprovementError> {
    if request.target_profile.trim().is_empty() || request.category.trim().is_empty() {
        return Err(ImprovementError::Invalid(
            "target_profile and category are required".into(),
        ));
    }
    if request.evidence_reviews.is_empty() {
        return Err(ImprovementError::Invalid(
            "at least one evidence review is required".into(),
        ));
    }
    if request
        .guidance
        .as_ref()
        .is_some_and(|value| value.len() > 8 * 1024)
    {
        return Err(ImprovementError::Invalid(
            "guidance exceeds 8192 bytes".into(),
        ));
    }
    for value in request
        .add_review_focus
        .iter()
        .chain(&request.add_skills)
        .chain(&request.add_constraints)
        .chain(&request.add_primary_files)
    {
        if value.trim().is_empty() || value.len() > 512 || value.contains(['\n', '\r']) {
            return Err(ImprovementError::Invalid(
                "additive improvement values must be non-empty single lines of at most 512 bytes"
                    .into(),
            ));
        }
    }
    Ok(())
}

fn add_values(document: &mut Document, key: &str, additions: Vec<String>) {
    if additions.is_empty() {
        return;
    }
    let mut values = document.string_array(key);
    for value in additions {
        if !values.contains(&value) {
            values.push(value);
        }
    }
    document.set(key, &inline_array(&values));
}

fn find_profile(root: &Path, id: &str) -> Result<PathBuf, ImprovementError> {
    for path in markdown_files(&root.join(".lmbrain/agents/profiles")) {
        let source = fs::read_to_string(&path)?;
        if Document::parse(&source)
            .ok()
            .and_then(|doc| doc.value("id"))
            .as_deref()
            == Some(id)
        {
            return Ok(path);
        }
    }
    Err(ImprovementError::Invalid(format!(
        "target profile {id} does not exist"
    )))
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("templates") {
                    continue;
                }
                files.extend(markdown_files(&path));
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    files
}

fn extract_section(body: &str, heading: &str) -> Option<String> {
    let marker = format!("## {heading}");
    let start = body.find(&marker)? + marker.len();
    let tail = &body[start..];
    let end = tail.find("\n## ").unwrap_or(tail.len());
    Some(tail[..end].trim().to_string())
}

fn inline_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", value.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn signals_count_distinct_specs_not_cycles() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/reviews/changes-requested")).unwrap();
        for (id, spec) in [
            ("REVIEW-001", "SPEC-001"),
            ("REVIEW-002", "SPEC-001"),
            ("REVIEW-003", "SPEC-002"),
        ] {
            fs::write(dir.path().join(format!(".lmbrain/reviews/changes-requested/{id}.md")), format!("---\nid: {id}\ntitle: Test\nstatus: changes-requested\nspec: {spec}\nimplementation_agent: AGENT-X\nfinding_categories: [verification-transcript]\ntags: []\n---\n# Review\n")).unwrap();
        }
        let (signals, _) = build_agent_improvement_signals(dir.path()).unwrap();
        assert_eq!(signals[0].distinct_specs.len(), 2);
        assert!(signals[0].threshold_met);
        assert_eq!(signals[0].category, "verification-integrity");
    }

    #[test]
    fn xenomark_shaped_metrics_count_cycles_escalation_takeover_and_aliases() {
        let dir = tempdir().unwrap();
        let accepted = dir.path().join(".lmbrain/reviews/accepted");
        fs::create_dir_all(&accepted).unwrap();
        fs::write(
            accepted.join("REVIEW-047.md"),
            "---\nid: REVIEW-047\nstatus: accepted\nspec: SPEC-047\ncreated: 2026-07-28\nimplementation_agent: AGENT-ENGINE\nfinding_categories: [metric-cannot-fail, evidence-integrity]\nreview_cycles: 3\nescalation: \"takeover correttivo del Project Lead\"\n---\n",
        )
        .unwrap();
        fs::write(
            accepted.join("REVIEW-054.md"),
            "---\nid: REVIEW-054\nstatus: accepted\nspec: SPEC-048\ncreated: 2026-07-29\nimplementation_agent: AGENT-ENGINE\nfinding_categories: [unsound-metric, evidence-accuracy, project-specific]\nremediation_cycles: 2\nescalations: 1\n---\n",
        )
        .unwrap();

        let (signals, metrics) = build_agent_improvement_signals(dir.path()).unwrap();
        let metric = &metrics[0];
        assert_eq!(metric.review_cycles, 6);
        assert_eq!(metric.remediation_cycles, 4);
        assert_eq!(metric.escalation_count, 2);
        assert_eq!(metric.takeover_count, 1);
        assert_eq!(metric.first_pass_accepted_specs, 0);
        assert_eq!(metric.first_pass_acceptance_rate, 0.0);
        assert_eq!(metric.lifecycle_coverage, 1.0);
        assert_eq!(metric.category_values, 5);
        assert_eq!(metric.canonical_category_values, 4);
        assert_eq!(metric.category_coverage, 0.8);
        assert_eq!(metric.unknown_categories, vec!["project-specific"]);
        assert!(metric
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("project-specific")));

        let metrics_signal = signals
            .iter()
            .find(|signal| signal.category == "metrics-integrity")
            .unwrap();
        assert_eq!(metrics_signal.distinct_specs.len(), 2);
        assert!(metrics_signal.threshold_met);
        let evidence_signal = signals
            .iter()
            .find(|signal| signal.category == "verification-integrity")
            .unwrap();
        assert_eq!(evidence_signal.distinct_specs.len(), 2);
    }

    #[test]
    fn status_only_accepted_review_is_not_claimed_as_first_pass() {
        let dir = tempdir().unwrap();
        let accepted = dir.path().join(".lmbrain/reviews/accepted");
        fs::create_dir_all(&accepted).unwrap();
        fs::write(
            accepted.join("REVIEW-001.md"),
            "---\nid: REVIEW-001\nstatus: accepted\nspec: SPEC-001\ncreated: 2026-07-20\nimplementation_agent: AGENT-X\nfinding_categories: [correctness]\n---\n",
        )
        .unwrap();
        let (_, metrics) = build_agent_improvement_signals(dir.path()).unwrap();
        assert_eq!(metrics[0].first_pass_accepted_specs, 0);
        assert_eq!(metrics[0].lifecycle_unknown_reviews, 1);
        assert_eq!(metrics[0].lifecycle_coverage, 0.0);
        assert_eq!(metrics[0].confidence, "low");
    }

    fn write_profile(root: &Path) -> PathBuf {
        let path = root.join(".lmbrain/agents/profiles/AGENT-X.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "---\nid: AGENT-X\ntitle: Test agent\nstatus: active\nreview_focus: [existing]\nskills: []\nconstraints: []\nprimary_files: []\n---\n# Test agent\n",
        ).unwrap();
        path
    }

    fn request() -> ImprovementProposalRequest {
        ImprovementProposalRequest {
            target_profile: "AGENT-X".into(),
            category: "verification-transcript".into(),
            evidence_reviews: vec!["REVIEW-001".into()],
            evidence_specs: vec!["SPEC-001".into()],
            add_review_focus: vec!["transcript-integrity".into()],
            add_skills: vec![],
            add_constraints: vec!["Never synthesize execution evidence".into()],
            add_primary_files: vec![],
            guidance: Some(
                "Run every before-submit gate and paste only attributable output.".into(),
            ),
        }
    }

    #[test]
    fn proposal_requires_approval_then_applies_once_and_preserves_profile() {
        let dir = tempdir().unwrap();
        let profile = write_profile(dir.path());
        let proposal_path = create_improvement_proposal(dir.path(), &request()).unwrap();
        assert!(matches!(
            apply_improvement_proposal(dir.path(), &proposal_path),
            Err(ImprovementError::NotApproved)
        ));
        let mut proposal = Document::parse(&fs::read_to_string(&proposal_path).unwrap()).unwrap();
        proposal.set("status", "approved");
        fs::write(&proposal_path, proposal.render()).unwrap();

        let applied = apply_improvement_proposal(dir.path(), &proposal_path).unwrap();
        assert!(applied.applied);
        let profile_source = fs::read_to_string(&profile).unwrap();
        assert!(profile_source.contains("existing"));
        assert!(profile_source.contains("transcript-integrity"));
        assert!(profile_source.contains("Never synthesize execution evidence"));
        assert!(profile_source.contains("Approved improvement guidance"));

        let repeated = apply_improvement_proposal(dir.path(), &proposal_path).unwrap();
        assert!(!repeated.applied);
    }

    #[test]
    fn apply_fails_closed_when_target_digest_is_stale() {
        let dir = tempdir().unwrap();
        let profile = write_profile(dir.path());
        let proposal_path = create_improvement_proposal(dir.path(), &request()).unwrap();
        let mut proposal = Document::parse(&fs::read_to_string(&proposal_path).unwrap()).unwrap();
        proposal.set("status", "approved");
        fs::write(&proposal_path, proposal.render()).unwrap();
        fs::OpenOptions::new()
            .append(true)
            .open(profile)
            .unwrap()
            .write_all(b"\nOperator change\n")
            .unwrap();
        assert!(matches!(
            apply_improvement_proposal(dir.path(), &proposal_path),
            Err(ImprovementError::StaleTarget)
        ));
    }
}
