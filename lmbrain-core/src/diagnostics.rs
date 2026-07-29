use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    content_digest, frontmatter::Document, invariants, unsupported_verification_requirements,
    verification_blockers_for_workspace,
};

pub const DIAGNOSTIC_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticFixability {
    Manual,
    GovernedMutation,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub schema_version: String,
    pub id: String,
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub artifact_id: Option<String>,
    pub path: Option<String>,
    pub message: String,
    pub next_action: String,
    pub fixability: DiagnosticFixability,
}

#[derive(Debug, Clone)]
struct Artifact {
    relative: String,
    document: Document,
}

pub fn build_diagnostics(root: &Path) -> Vec<Diagnostic> {
    let lmbrain = root.join(".lmbrain");
    let mut diagnostics = Vec::new();
    let mut paths = markdown_paths(&lmbrain);
    paths.sort();
    let mut artifacts = Vec::new();
    let mut ids: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for path in paths {
        let relative = relative_path(&lmbrain, &path);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(diagnostic(
                    "artifact-unreadable",
                    DiagnosticSeverity::Error,
                    None,
                    Some(relative),
                    format!("Artifact cannot be read: {error}"),
                    "Restore readable permissions or recover the artifact from version control.",
                    DiagnosticFixability::Manual,
                    "read",
                ));
                continue;
            }
        };
        if !source.trim_start_matches('\u{feff}').starts_with("---")
            && !requires_frontmatter(&relative)
        {
            continue;
        }
        let document = match Document::parse(&source) {
            Ok(document) => document,
            Err(error) => {
                diagnostics.push(diagnostic(
                    "frontmatter-malformed",
                    DiagnosticSeverity::Warning,
                    None,
                    Some(relative),
                    format!("Malformed frontmatter: {error}"),
                    "Repair the YAML frontmatter before relying on this artifact.",
                    DiagnosticFixability::Manual,
                    "parse",
                ));
                continue;
            }
        };
        let artifact_id = document.value("id");
        if let Some(id) = artifact_id.as_ref() {
            ids.entry(id.clone()).or_default().push(relative.clone());
        }
        if is_status_artifact(&path)
            && document.value("status").is_some()
            && !invariants::folder_matches_status(&path)
        {
            let folder = path
                .parent()
                .and_then(Path::file_name)
                .map(|value| value.to_string_lossy())
                .unwrap_or_default();
            let status = document.value("status").unwrap_or_default();
            diagnostics.push(diagnostic(
                "status-folder-mismatch",
                DiagnosticSeverity::Warning,
                artifact_id.clone(),
                Some(relative.clone()),
                format!(
                    "Status mismatch: artifact is in folder '{folder}' but frontmatter status is '{status}'"
                ),
                "Use the governed lifecycle transition or restore the file to the matching status folder.",
                DiagnosticFixability::GovernedMutation,
                &status,
            ));
        }
        artifacts.push(Artifact { relative, document });
    }

    for (id, paths) in ids.iter().filter(|(_, paths)| paths.len() > 1) {
        for path in paths {
            diagnostics.push(diagnostic(
                "duplicate-artifact-id",
                DiagnosticSeverity::Error,
                Some(id.clone()),
                Some(path.clone()),
                format!("Artifact ID '{id}' is duplicated in {} files", paths.len()),
                "Assign a unique governed ID before performing any mutation.",
                DiagnosticFixability::Manual,
                id,
            ));
        }
    }

    diagnose_references(root, &artifacts, &mut diagnostics);
    diagnose_spec_dependencies(root, &artifacts, &mut diagnostics);
    diagnose_findings(root, &artifacts, &mut diagnostics);
    diagnose_verification(root, &artifacts, &mut diagnostics);
    diagnose_roadmap(root, &artifacts, &mut diagnostics);
    diagnose_kit_feedback(root, &mut diagnostics);

    let harness = lmbrain.join("HARNESSES.json");
    if harness.exists() {
        if let Err(error) = crate::load_harness_manifest(root) {
            diagnostics.push(diagnostic(
                "harness-manifest-invalid",
                DiagnosticSeverity::Warning,
                None,
                Some("HARNESSES.json".into()),
                format!("Invalid project harness manifest: {error}"),
                "Correct HARNESSES.json and preview the governed harness plan again.",
                DiagnosticFixability::Manual,
                "harness",
            ));
        }
    }

    diagnostics.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.id.cmp(&right.id))
    });
    diagnostics.dedup_by(|left, right| left.id == right.id);
    diagnostics
}

fn diagnose_kit_feedback(root: &Path, diagnostics: &mut Vec<Diagnostic>) {
    let report = root.join(crate::KIT_FEEDBACK_REPORT_PATH);
    if !report.exists() {
        return;
    }
    if let Err(error) = crate::read_kit_feedback(root) {
        diagnostics.push(diagnostic(
            "kit-feedback-invalid",
            DiagnosticSeverity::Warning,
            Some("LMBRAIN-KIT-FEEDBACK".into()),
            Some("reports/lmbrain-kit-feedback.md".into()),
            format!("LMBrain kit feedback report is invalid: {error}"),
            "Repair the report structure before recording or delivering further kit feedback; do not discard existing evidence.",
            DiagnosticFixability::Manual,
            "kit-feedback",
        ));
    }
}

fn diagnose_spec_dependencies(
    root: &Path,
    artifacts: &[Artifact],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let paths = artifacts
        .iter()
        .filter_map(|artifact| {
            let id = artifact.document.value("id")?;
            id.starts_with("SPEC-")
                .then_some((id, artifact.relative.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    for (id, issue) in crate::validate_spec_dependency_graph(root) {
        let code = if issue.contains("cycle") {
            "spec-dependency-cycle"
        } else if issue.contains("missing") {
            "spec-dependency-missing"
        } else if issue.contains("duplicate") {
            "spec-dependency-duplicate"
        } else {
            "spec-dependency-invalid"
        };
        diagnostics.push(diagnostic(
            code,
            DiagnosticSeverity::Error,
            Some(id.clone()),
            paths.get(&id).cloned(),
            issue,
            "Use the governed spec dependency mutation to restore a resolvable acyclic graph.",
            DiagnosticFixability::GovernedMutation,
            &id,
        ));
    }

    for artifact in artifacts {
        let Some(id) = artifact.document.value("id") else {
            continue;
        };
        if !id.starts_with("SPEC-") {
            continue;
        }
        let status = artifact.document.value("status").unwrap_or_default();
        if !matches!(status.as_str(), "ready" | "working") {
            continue;
        }
        let blockers = crate::spec_dependency_blockers(root, &artifact.document);
        if blockers.is_empty() {
            continue;
        }
        diagnostics.push(diagnostic(
            "spec-dependency-lifecycle-blocked",
            DiagnosticSeverity::Error,
            Some(id.clone()),
            Some(artifact.relative.clone()),
            format!(
                "{id} is {status} despite incomplete hard prerequisites: {}",
                blockers
                    .iter()
                    .map(|blocker| format!("{} [{}]", blocker.id, blocker.status))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "Complete the prerequisites or use a reasoned governed lifecycle override; do not start this spec normally.",
            DiagnosticFixability::GovernedMutation,
            &status,
        ));
    }
}

fn diagnose_findings(root: &Path, artifacts: &[Artifact], diagnostics: &mut Vec<Diagnostic>) {
    let spec_statuses = artifacts
        .iter()
        .filter_map(|artifact| {
            let id = artifact.document.value("id")?;
            id.starts_with("SPEC-")
                .then(|| (id, artifact.document.value("status").unwrap_or_default()))
        })
        .collect::<BTreeMap<_, _>>();
    for artifact in artifacts {
        let Some(id) = artifact
            .document
            .value("id")
            .filter(|id| id.starts_with("FINDING-"))
        else {
            continue;
        };
        let status = artifact.document.value("status").unwrap_or_default();
        let path = root.join(".lmbrain").join(&artifact.relative);
        if let Err(error) = crate::validate_finding_document(root, &path, &artifact.document) {
            diagnostics.push(diagnostic(
                "finding-contract-invalid",
                DiagnosticSeverity::Error,
                Some(id.clone()),
                Some(artifact.relative.clone()),
                error.to_string(),
                "Repair the finding through the matching governed semantic operation; do not infer or auto-close it.",
                DiagnosticFixability::GovernedMutation,
                &error.to_string(),
            ));
        }
        if status == "planned" {
            let targets = artifact.document.string_array("target_specs");
            if targets.is_empty() {
                diagnostics.push(diagnostic(
                    "finding-planned-without-target",
                    DiagnosticSeverity::Error,
                    Some(id.clone()),
                    Some(artifact.relative.clone()),
                    format!("Planned finding {id} has no target spec"),
                    "Use finding_plan with at least one existing target spec, or return the finding to open.",
                    DiagnosticFixability::GovernedMutation,
                    "targets",
                ));
            } else if targets.iter().all(|target| {
                spec_statuses
                    .get(target)
                    .is_some_and(|value| value == "done")
            }) {
                diagnostics.push(diagnostic(
                    "finding-targets-done-still-planned",
                    DiagnosticSeverity::Warning,
                    Some(id.clone()),
                    Some(artifact.relative.clone()),
                    format!("Every target spec for planned finding {id} is done, but the finding remains unresolved"),
                    "Review canonical evidence and explicitly resolve, accept risk, defer, or re-plan the finding; target completion is not resolution.",
                    DiagnosticFixability::GovernedMutation,
                    "done-targets",
                ));
            }
        }
        if matches!(status.as_str(), "open" | "planned" | "deferred")
            && matches!(
                artifact.document.value("severity").as_deref(),
                Some("critical" | "high")
            )
            && artifact
                .document
                .value("owner")
                .map_or(true, |owner| owner.trim().is_empty())
        {
            diagnostics.push(diagnostic(
                "finding-high-severity-unowned",
                DiagnosticSeverity::Warning,
                Some(id.clone()),
                Some(artifact.relative.clone()),
                format!("Active high-severity finding {id} has no owner"),
                "Assign an explicit owner or record a reasoned defer/accepted-risk disposition.",
                DiagnosticFixability::GovernedMutation,
                "owner",
            ));
        }
        if status == "open"
            && !artifact
                .document
                .body
                .split("## Resolution criteria")
                .nth(1)
                .is_some_and(|section| {
                    section
                        .split("\n## ")
                        .next()
                        .is_some_and(|value| !value.trim().is_empty())
                })
        {
            diagnostics.push(diagnostic(
                "finding-open-without-next-action",
                DiagnosticSeverity::Warning,
                Some(id.clone()),
                Some(artifact.relative.clone()),
                format!("Open finding {id} has no usable resolution criteria"),
                "Record a concrete resolution criterion before planning implementation work.",
                DiagnosticFixability::Manual,
                "resolution-criteria",
            ));
        }
    }
}

fn diagnose_references(root: &Path, artifacts: &[Artifact], diagnostics: &mut Vec<Diagnostic>) {
    let agents = artifacts
        .iter()
        .filter_map(|artifact| {
            artifact
                .document
                .value("id")
                .filter(|id| id.starts_with("AGENT-"))
                .map(|id| (id, artifact))
        })
        .collect::<BTreeMap<_, _>>();
    let skills = artifacts
        .iter()
        .filter_map(|artifact| {
            artifact
                .document
                .value("id")
                .filter(|id| id.starts_with("SKILL-"))
                .map(|id| (id, artifact))
        })
        .collect::<BTreeMap<_, _>>();

    for artifact in artifacts {
        let id = artifact.document.value("id").unwrap_or_default();
        if id.starts_with("SPEC-") {
            for skill in artifact.document.string_array("skills") {
                if !skills.contains_key(&skill) {
                    diagnostics.push(missing_reference(&id, &artifact.relative, "skill", &skill));
                }
            }
            if let Some(agent) = artifact
                .document
                .value("recommended_agent")
                .filter(|agent| !agent.trim().is_empty())
            {
                if !invariants::recommended_agent_resolves(root, Some(&agent)) {
                    diagnostics.push(missing_reference(
                        &id,
                        &artifact.relative,
                        "recommended agent",
                        &agent,
                    ));
                } else if let (Some(area), Some(profile)) =
                    (artifact.document.value("area"), agents.get(&agent))
                {
                    let domains = profile.document.string_array("domains");
                    if !domains.is_empty()
                        && !domains.iter().any(|domain| {
                            area.contains(domain.as_str()) || domain.contains(area.as_str())
                        })
                    {
                        diagnostics.push(diagnostic(
                            "agent-area-mismatch",
                            DiagnosticSeverity::Warning,
                            Some(id.clone()),
                            Some(artifact.relative.clone()),
                            format!(
                                "Area mismatch: spec {id} area '{area}' does not match agent {agent} domains {domains:?}"
                            ),
                            "Choose a matching agent or update the profile domains through the governed profile workflow.",
                            DiagnosticFixability::GovernedMutation,
                            &agent,
                        ));
                    }
                }
            }
        } else if id.starts_with("AGENT-") {
            for skill in artifact.document.string_array("skills") {
                if !skills.contains_key(&skill) {
                    diagnostics.push(missing_reference(&id, &artifact.relative, "skill", &skill));
                }
            }
        } else if id.starts_with("SKILL-") {
            if let Some(risk) = artifact.document.value("risk") {
                if !matches!(risk.as_str(), "low" | "medium" | "high") {
                    diagnostics.push(diagnostic(
                        "skill-risk-invalid",
                        DiagnosticSeverity::Warning,
                        Some(id.clone()),
                        Some(artifact.relative.clone()),
                        format!(
                            "Invalid skill risk: skill {id} uses '{risk}', expected low, medium, or high"
                        ),
                        "Set risk to low, medium, or high.",
                        DiagnosticFixability::Manual,
                        &risk,
                    ));
                }
            }
            for target in artifact.document.string_array("applies_to") {
                if target != "all" && !agents.contains_key(&target) {
                    diagnostics.push(missing_reference(
                        &id,
                        &artifact.relative,
                        "applicable agent",
                        &target,
                    ));
                }
            }
        }
    }
}

fn diagnose_verification(root: &Path, artifacts: &[Artifact], diagnostics: &mut Vec<Diagnostic>) {
    let mut referenced_gates = Vec::new();
    for artifact in artifacts {
        let id = artifact.document.value("id").unwrap_or_default();
        if !id.starts_with("SPEC-") {
            continue;
        }
        let status = artifact.document.value("status").unwrap_or_default();
        let gates = artifact.document.string_array("verification_gates");
        if !gates.is_empty() {
            referenced_gates.push((id.clone(), artifact.relative.clone(), gates));
        }
        let mut phases = Vec::new();
        if matches!(status.as_str(), "review" | "done") {
            phases.push("before-submit");
        }
        if status == "done" {
            phases.push("before-done");
        }
        for phase in phases {
            for blocker in verification_blockers_for_workspace(root, &artifact.document, phase) {
                diagnostics.push(diagnostic(
                    "verification-unresolved",
                    DiagnosticSeverity::Warning,
                    Some(id.clone()),
                    Some(artifact.relative.clone()),
                    format!(
                        "Unresolved {phase} verification on {id}: {} (owner={}): {}",
                        blocker.requirement_id, blocker.owner, blocker.cause
                    ),
                    "Reconcile the requirement and record fresh evidence; do not reopen a completed spec automatically.",
                    DiagnosticFixability::GovernedMutation,
                    &format!("{phase}:{}", blocker.requirement_id),
                ));
            }
        }
        for (requirement_id, cause) in unsupported_verification_requirements(&artifact.document) {
            diagnostics.push(diagnostic(
                "verification-policy-unsupported",
                DiagnosticSeverity::Warning,
                Some(id.clone()),
                Some(artifact.relative.clone()),
                format!("Unsupported verification policy on {id}: {requirement_id}: {cause}"),
                "Use agent/kit + before-submit or lead/operator + before-done.",
                DiagnosticFixability::Manual,
                &requirement_id,
            ));
        }
    }
    if referenced_gates.is_empty() {
        return;
    }
    let approval_path = crate::default_verification_approval_path(root);
    let status = match crate::verification_manifest_status(root, &approval_path) {
        Ok(status) => status,
        Err(error) => {
            diagnostics.push(diagnostic(
                "verification-status-unavailable",
                DiagnosticSeverity::Warning,
                None,
                Some("verification.toml".into()),
                format!("Verification manifest status is unavailable: {error}"),
                "Inspect verification_manifest_status before running spec_verify.",
                DiagnosticFixability::ReadOnly,
                "status",
            ));
            return;
        }
    };
    let manifest = crate::load_verification_manifest(root).ok();
    let known = manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .gates
                .iter()
                .map(|gate| gate.id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for (spec_id, path, gates) in referenced_gates {
        for gate in gates {
            if manifest.is_some() && !known.contains(gate.as_str()) {
                diagnostics.push(diagnostic(
                    "verification-gate-unknown",
                    DiagnosticSeverity::Warning,
                    Some(spec_id.clone()),
                    Some(path.clone()),
                    format!("Spec {spec_id} references unknown verification gate '{gate}'"),
                    "Add the gate through verification_manifest_init/set or remove the stale spec reference.",
                    DiagnosticFixability::GovernedMutation,
                    &gate,
                ));
            }
        }
        if status.state != crate::VerificationManifestState::Approved {
            diagnostics.push(diagnostic(
                &format!(
                    "verification-manifest-{}",
                    manifest_state_code(&status.state)
                ),
                DiagnosticSeverity::Warning,
                Some(spec_id.clone()),
                Some(path.clone()),
                format!(
                    "Spec {spec_id} references verification gates while manifest state is {:?}",
                    status.state
                ),
                &status.next_action,
                if matches!(
                    status.state,
                    crate::VerificationManifestState::Unapproved
                        | crate::VerificationManifestState::Stale
                        | crate::VerificationManifestState::ApprovalInvalid
                ) {
                    DiagnosticFixability::Manual
                } else {
                    DiagnosticFixability::GovernedMutation
                },
                manifest_state_code(&status.state),
            ));
        }
    }
}

fn manifest_state_code(state: &crate::VerificationManifestState) -> &'static str {
    match state {
        crate::VerificationManifestState::Absent => "absent",
        crate::VerificationManifestState::Invalid => "invalid",
        crate::VerificationManifestState::Unsafe => "unsafe",
        crate::VerificationManifestState::Unapproved => "unapproved",
        crate::VerificationManifestState::Approved => "approved",
        crate::VerificationManifestState::Stale => "stale",
        crate::VerificationManifestState::ApprovalInvalid => "approval-invalid",
    }
}

fn diagnose_roadmap(root: &Path, artifacts: &[Artifact], diagnostics: &mut Vec<Diagnostic>) {
    let roadmap_path = root.join(".lmbrain/ROADMAP.md");
    let status_path = root.join(".lmbrain/STATUS.md");
    if !status_path.exists() {
        diagnostics.push(diagnostic(
            "status-missing",
            DiagnosticSeverity::Warning,
            None,
            Some("STATUS.md".into()),
            "Declared project status is missing".into(),
            "Create STATUS.md with an explicit status and current milestone.",
            DiagnosticFixability::Manual,
            "missing",
        ));
    }
    if !roadmap_path.exists() {
        diagnostics.push(diagnostic(
            "roadmap-missing",
            DiagnosticSeverity::Warning,
            None,
            Some("ROADMAP.md".into()),
            "Project roadmap is missing".into(),
            "Create ROADMAP.md or explicitly document that the project has no milestones.",
            DiagnosticFixability::Manual,
            "missing",
        ));
        return;
    }
    let roadmap_source = match fs::read_to_string(&roadmap_path) {
        Ok(source) => source,
        Err(_) => return,
    };
    let milestones = parse_roadmap_milestones(&roadmap_source);
    let milestone_ids = milestones.keys().cloned().collect::<BTreeSet<_>>();
    let specs = artifacts
        .iter()
        .filter(|artifact| {
            artifact
                .document
                .value("id")
                .is_some_and(|id| id.starts_with("SPEC-"))
        })
        .collect::<Vec<_>>();
    for artifact in &specs {
        let id = artifact.document.value("id").unwrap_or_default();
        if let Some(milestone) = artifact.document.value("milestone") {
            if !milestone_ids.contains(&milestone) {
                diagnostics.push(diagnostic(
                    "spec-milestone-missing",
                    DiagnosticSeverity::Warning,
                    Some(id.clone()),
                    Some(artifact.relative.clone()),
                    format!("Spec milestone '{milestone}' does not exist in ROADMAP.md"),
                    "Add the milestone to ROADMAP.md or assign the spec to an existing milestone.",
                    DiagnosticFixability::Manual,
                    &milestone,
                ));
            } else if !milestones
                .get(&milestone)
                .is_some_and(|entry| entry.specs.contains(&id))
            {
                diagnostics.push(diagnostic(
                    "spec-roadmap-membership-mismatch",
                    DiagnosticSeverity::Warning,
                    Some(id.clone()),
                    Some(artifact.relative.clone()),
                    format!(
                        "Spec declares milestone '{milestone}' but ROADMAP.md does not list {id} under it"
                    ),
                    "Reconcile the spec milestone and ROADMAP.md membership without silently choosing one source.",
                    DiagnosticFixability::Manual,
                    &milestone,
                ));
            }
        }
    }
    for (milestone, entry) in &milestones {
        for spec_id in &entry.specs {
            if let Some(artifact) = specs
                .iter()
                .find(|artifact| artifact.document.value("id").as_deref() == Some(spec_id))
            {
                if artifact.document.value("milestone").as_deref() != Some(milestone) {
                    diagnostics.push(diagnostic(
                        "roadmap-spec-membership-mismatch",
                        DiagnosticSeverity::Warning,
                        Some(spec_id.clone()),
                        Some("ROADMAP.md".into()),
                        format!(
                            "ROADMAP.md lists {spec_id} under '{milestone}' but the spec frontmatter declares {:?}",
                            artifact.document.value("milestone")
                        ),
                        "Reconcile ROADMAP.md and the spec milestone explicitly.",
                        DiagnosticFixability::Manual,
                        milestone,
                    ));
                }
            } else {
                diagnostics.push(diagnostic(
                    "roadmap-spec-missing",
                    DiagnosticSeverity::Warning,
                    Some(spec_id.clone()),
                    Some("ROADMAP.md".into()),
                    format!("ROADMAP.md references missing spec {spec_id}"),
                    "Create the referenced spec or remove the stale roadmap reference.",
                    DiagnosticFixability::Manual,
                    milestone,
                ));
            }
        }
    }

    let status_source = fs::read_to_string(status_path).ok();
    let declared = status_source
        .as_deref()
        .and_then(extract_declared_milestone);
    let derived_candidates = current_milestone_candidates(&specs);
    let derived = (derived_candidates.len() == 1).then(|| derived_candidates[0].clone());
    if status_source.as_deref().is_some_and(|source| {
        !source.lines().any(|line| {
            let lower = line.trim().to_ascii_lowercase();
            lower.starts_with("**status:**") || lower.starts_with("status:")
        })
    }) {
        diagnostics.push(diagnostic(
            "declared-status-missing",
            DiagnosticSeverity::Warning,
            None,
            Some("STATUS.md".into()),
            "STATUS.md does not declare an explicit project status".into(),
            "Add an explicit Status field while preserving the narrative project pulse.",
            DiagnosticFixability::Manual,
            "status",
        ));
    }
    if declared.is_none() && derived.is_some() {
        diagnostics.push(diagnostic(
            "declared-milestone-missing",
            DiagnosticSeverity::Warning,
            None,
            Some("STATUS.md".into()),
            format!(
                "STATUS.md does not declare a current milestone; active spec lifecycle derives '{}'",
                derived.as_deref().unwrap_or_default()
            ),
            "Declare the intended current milestone or explain why lifecycle-derived work should not drive orientation.",
            DiagnosticFixability::Manual,
            "milestone",
        ));
    }
    if derived_candidates.len() > 1 {
        diagnostics.push(diagnostic(
            "current-milestone-ambiguous",
            DiagnosticSeverity::Warning,
            None,
            Some("ROADMAP.md".into()),
            format!(
                "Active spec lifecycle yields multiple equally plausible current milestones: {}",
                derived_candidates.join(", ")
            ),
            "Choose and declare the operational focus; do not infer one milestone arbitrarily.",
            DiagnosticFixability::Manual,
            &derived_candidates.join(":"),
        ));
    }
    match (declared, derived) {
        (Some(declared), Some(derived)) if declared != derived => diagnostics.push(diagnostic(
            "current-milestone-mismatch",
            DiagnosticSeverity::Warning,
            None,
            Some("STATUS.md".into()),
            format!(
                "STATUS.md declares current milestone '{declared}' while active spec lifecycle derives '{derived}'"
            ),
            "Review STATUS.md and ROADMAP.md; preserve both sources until the mismatch is explicitly reconciled.",
            DiagnosticFixability::Manual,
            &format!("{declared}:{derived}"),
        )),
        _ => {}
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RoadmapMilestone {
    pub status: Option<String>,
    pub specs: BTreeSet<String>,
}

pub(crate) fn parse_roadmap_milestones(source: &str) -> BTreeMap<String, RoadmapMilestone> {
    let mut milestones: BTreeMap<String, RoadmapMilestone> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed
            .strip_prefix("### ")
            .or_else(|| trimmed.strip_prefix("## "))
        {
            current = heading
                .split_whitespace()
                .next()
                .filter(|candidate| candidate.starts_with("M-"))
                .map(str::to_string);
            if let Some(id) = current.as_ref() {
                milestones.entry(id.clone()).or_default();
            }
            continue;
        }
        let Some(id) = current.as_ref() else {
            continue;
        };
        let Some((key, value)) = roadmap_property(trimmed) else {
            continue;
        };
        let entry = milestones.entry(id.clone()).or_default();
        match key {
            "status" => entry.status = Some(value.trim().trim_matches('`').to_string()),
            "specs" => {
                // Extract only the bracket-delimited list if present,
                // ignoring parenthetical annotations in trailing prose.
                let source = if let (Some(open), Some(close)) =
                    (value.find('['), value.rfind(']'))
                {
                    &value[open + 1..close]
                } else {
                    value
                };
                for token in source
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
                {
                    if token.starts_with("SPEC-")
                        && token
                            .rsplit_once('-')
                            .is_some_and(|(_, number)| number.chars().all(|c| c.is_ascii_digit()))
                    {
                        entry.specs.insert(token.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    milestones
}

fn current_milestone_candidates(specs: &[&Artifact]) -> Vec<String> {
    let mut scores: BTreeMap<String, usize> = BTreeMap::new();
    for artifact in specs {
        let status = artifact.document.value("status").unwrap_or_default();
        let score = match status.as_str() {
            "working" => 40,
            "review" => 30,
            "ready" => 20,
            "backlog" => 10,
            _ => 0,
        };
        if score > 0 {
            if let Some(milestone) = artifact.document.value("milestone") {
                *scores.entry(milestone).or_default() += score;
            }
        }
    }
    let Some(maximum) = scores.values().copied().max() else {
        return Vec::new();
    };
    scores
        .into_iter()
        .filter(|(_, score)| *score == maximum)
        .map(|(milestone, _)| milestone)
        .collect()
}

pub(crate) fn extract_declared_milestone(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("**current milestone:**")
            || lower.starts_with("**milestone:**")
            || lower.starts_with("current milestone:")
            || lower.starts_with("milestone:")
        {
            let value = trimmed.split_once(':')?.1.trim().trim_matches('*').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn roadmap_property(line: &str) -> Option<(&str, &str)> {
    let line = line.strip_prefix("- `")?;
    let (key, value) = line.split_once("`:")?;
    Some((key, value.trim()))
}

fn missing_reference(artifact_id: &str, path: &str, kind: &str, target: &str) -> Diagnostic {
    let family = if artifact_id.starts_with("SPEC-") {
        "spec"
    } else if artifact_id.starts_with("AGENT-") {
        "agent"
    } else if artifact_id.starts_with("SKILL-") {
        "skill"
    } else {
        "artifact"
    };
    let message = if family == "skill" && kind == "applicable agent" {
        format!(
            "Missing reference: skill {artifact_id} applies to '{target}', which is not an existing agent profile"
        )
    } else {
        format!(
            "Missing reference: {family} {artifact_id} references {kind} '{target}', which does not exist"
        )
    };
    diagnostic(
        "missing-reference",
        DiagnosticSeverity::Warning,
        Some(artifact_id.to_string()),
        Some(path.to_string()),
        message,
        "Create the referenced artifact or remove the stale reference through the governed workflow.",
        DiagnosticFixability::GovernedMutation,
        &format!("{kind}:{target}"),
    )
}

fn diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    artifact_id: Option<String>,
    path: Option<String>,
    message: String,
    next_action: &str,
    fixability: DiagnosticFixability,
    discriminator: &str,
) -> Diagnostic {
    let identity = format!(
        "{code}|{}|{}|{discriminator}",
        artifact_id.as_deref().unwrap_or("workspace"),
        path.as_deref().unwrap_or("workspace")
    );
    let digest = content_digest(identity.as_bytes());
    Diagnostic {
        schema_version: DIAGNOSTIC_SCHEMA_VERSION.into(),
        id: format!("DIAG-{}", &digest[..16]),
        code: code.into(),
        severity,
        artifact_id,
        path,
        message,
        next_action: next_action.into(),
        fixability,
    }
}

fn markdown_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return paths;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("templates") {
                continue;
            }
            paths.extend(markdown_paths(&path));
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    paths
}

fn relative_path(lmbrain: &Path, path: &Path) -> String {
    path.strip_prefix(lmbrain)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_status_artifact(path: &Path) -> bool {
    path.parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .is_some_and(|family| {
            matches!(
                family.to_string_lossy().as_ref(),
                "specs" | "reviews" | "skills" | "findings"
            )
        })
}

fn requires_frontmatter(relative: &str) -> bool {
    let normalized = relative.replace('\\', "/");
    let filename = normalized.rsplit('/').next().unwrap_or_default();
    if filename.eq_ignore_ascii_case("README.md") {
        return false;
    }
    [
        "specs/",
        "reviews/",
        "decisions/",
        "agents/profiles/",
        "agents/proposals/",
        "mcp/specs/",
        "mcp/proposals/",
        "handoffs/active/",
        "skills/",
        "findings/",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn severity_rank(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Info => 0,
        DiagnosticSeverity::Warning => 1,
        DiagnosticSeverity::Error => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn diagnostics_are_stable_actionable_and_reconcile_roadmap_state() {
        let directory = tempdir().unwrap();
        let lmbrain = directory.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/working")).unwrap();
        fs::write(
            lmbrain.join("STATUS.md"),
            "# Status\n\n**Status:** active\n**Current milestone:** M-OLD\n",
        )
        .unwrap();
        fs::write(
            lmbrain.join("ROADMAP.md"),
            "# Roadmap\n\n## M-NEW - Current\n\n- `status`: active\n- `specs`: []\n",
        )
        .unwrap();
        fs::write(
            lmbrain.join("specs/working/SPEC-001.md"),
            "---\nid: SPEC-001\ntitle: Work\nstatus: working\nmilestone: M-NEW\nrecommended_agent: AGENT-MISSING\n---\n",
        )
        .unwrap();

        let first = build_diagnostics(directory.path());
        let second = build_diagnostics(directory.path());
        assert_eq!(first, second);
        assert!(first.iter().all(|item| {
            !item.id.is_empty() && !item.next_action.is_empty() && item.path.is_some()
        }));
        assert!(first
            .iter()
            .any(|item| item.code == "current-milestone-mismatch"));
        assert!(first
            .iter()
            .any(|item| item.code == "spec-roadmap-membership-mismatch"));
        assert!(first.iter().any(|item| item.code == "missing-reference"));
    }

    #[test]
    fn roadmap_parser_accepts_h2_h3_and_xenomark_shaped_spec_annotations() {
        let milestones = parse_roadmap_milestones(
            "## M-01 — First\n- `status`: active\n- `specs`: [SPEC-001, SPEC-002]\n\n### M-02 — Second\n- `status`: done\n- `specs`: [SPEC-003 (done: evidence), SPEC-004]\n",
        );
        assert_eq!(milestones.len(), 2);
        assert_eq!(
            milestones["M-02"].specs,
            BTreeSet::from(["SPEC-003".into(), "SPEC-004".into()])
        );
    }

    #[test]
    fn referenced_verification_manifest_states_and_unknown_gates_are_actionable() {
        let directory = tempdir().unwrap();
        let specs = directory.path().join(".lmbrain/specs/working");
        fs::create_dir_all(&specs).unwrap();
        fs::write(
            specs.join("SPEC-001.md"),
            "---\nid: SPEC-001\nstatus: working\nverification_gates: [required-gate]\n---\n",
        )
        .unwrap();
        let absent = build_diagnostics(directory.path());
        assert!(absent.iter().any(|diagnostic| {
            diagnostic.code == "verification-manifest-absent"
                && diagnostic.artifact_id.as_deref() == Some("SPEC-001")
        }));

        let manifest = crate::VerificationManifest {
            schema_version: 1,
            gates: vec![crate::VerificationGate {
                id: "other-gate".into(),
                title: None,
                program: "cargo".into(),
                args: vec!["test".into()],
                cwd: ".".into(),
                timeout_seconds: Some(30),
                output_limit_bytes: Some(1024),
                expected_exit_code: Some(0),
                result_matcher: None,
                environment: BTreeMap::new(),
                fingerprint_exclude: Vec::new(),
            }],
        };
        crate::set_verification_manifest(directory.path(), &manifest, None).unwrap();
        let configured = build_diagnostics(directory.path());
        assert!(configured
            .iter()
            .any(|diagnostic| diagnostic.code == "verification-gate-unknown"));
        assert!(configured
            .iter()
            .any(|diagnostic| diagnostic.code == "verification-manifest-unapproved"));
    }
}
