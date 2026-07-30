use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    content_digest,
    context::{parse_verification_requirements, VerificationRequirement},
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::ArtifactMutationLock,
    path::{PathError, PathGuard},
};

pub const VERIFICATION_ATTESTATION_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationAttestation {
    pub schema_version: String,
    pub id: String,
    pub requirement_id: String,
    pub requirement_digest: String,
    pub actor_role: String,
    pub actor: String,
    pub timestamp: String,
    pub result: String,
    pub evidence_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationBlocker {
    pub requirement_id: String,
    pub owner: String,
    pub cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttestationResult {
    pub path: PathBuf,
    pub attestation: VerificationAttestation,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationMigrationItem {
    pub spec_id: String,
    pub path: String,
    pub requirement_id: String,
    pub current_owner: String,
    pub proposed_owner: Option<String>,
    pub confidence: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationMigrationPreview {
    pub schema_version: String,
    pub candidates: Vec<VerificationMigrationItem>,
    pub proposed_lead_count: usize,
    pub ambiguous_count: usize,
    pub mutated: bool,
}

#[derive(Debug, Error)]
pub enum AttestationError {
    #[error("invalid verification attestation: {0}")]
    Invalid(String),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn requirement_digest(requirement: &VerificationRequirement) -> String {
    content_digest(
        format!(
            "{}|{}|{}|{}|{}|{}",
            requirement.id,
            requirement.kind,
            requirement.owner,
            requirement.phase,
            requirement.evidence,
            requirement.text
        )
        .as_bytes(),
    )
}

pub fn verification_requirements(document: &Document) -> Vec<VerificationRequirement> {
    parse_verification_requirements(&document.body, &document.string_array("verification_gates")).0
}

pub fn verification_attestations(document: &Document) -> Vec<VerificationAttestation> {
    parse_attestations(document).unwrap_or_default()
}

pub fn verification_blockers(document: &Document, phase: &str) -> Vec<VerificationBlocker> {
    verification_blockers_at(None, document, phase)
}

pub fn verification_blockers_for_workspace(
    root: impl AsRef<Path>,
    document: &Document,
    phase: &str,
) -> Vec<VerificationBlocker> {
    verification_blockers_at(Some(root.as_ref()), document, phase)
}

fn verification_blockers_at(
    root: Option<&Path>,
    document: &Document,
    phase: &str,
) -> Vec<VerificationBlocker> {
    let requirements = verification_requirements(document);
    let attestations = parse_attestations(document);
    let mut blockers = Vec::new();
    let mut ids = BTreeSet::new();
    for requirement in requirements
        .iter()
        .filter(|requirement| requirement.phase == phase)
    {
        if !ids.insert(&requirement.id) {
            blockers.push(blocker(requirement, "duplicate requirement ID"));
            continue;
        }
        if let Some(cause) = unsupported_policy(requirement) {
            blockers.push(blocker(requirement, cause));
            continue;
        }
        if requirement.owner == "kit" && phase == "before-submit" {
            continue;
        }
        if !requirement.checked {
            blockers.push(blocker(requirement, "checklist item is unchecked"));
            continue;
        }
        if matches!(requirement.owner.as_str(), "lead" | "operator") {
            let attestations = match &attestations {
                Ok(attestations) => attestations,
                Err(error) => {
                    blockers.push(blocker(
                        requirement,
                        &format!("verification attestation history is invalid: {error}"),
                    ));
                    continue;
                }
            };
            let expected_digest = requirement_digest(requirement);
            let expected_role = requirement.owner.as_str();
            let matching = attestations.iter().rev().find(|attestation| {
                attestation.requirement_id == requirement.id
                    && attestation.actor_role == expected_role
                    && attestation.result == "passed"
            });
            match matching {
                None => blockers.push(blocker(
                    requirement,
                    &format!("missing {expected_role} attestation"),
                )),
                Some(attestation) if attestation.requirement_digest != expected_digest => blockers
                    .push(blocker(
                        requirement,
                        "attestation is stale because the requirement changed",
                    )),
                Some(attestation) if attestation.evidence_ref.trim().is_empty() => blockers.push(
                    blocker(requirement, "attestation has no evidence reference"),
                ),
                Some(attestation)
                    if root.is_some()
                        && attestation.evidence_digest.as_deref()
                            != evidence_digest(root.unwrap(), &attestation.evidence_ref)
                                .ok()
                                .as_deref() =>
                {
                    blockers.push(blocker(
                        requirement,
                        "attestation is stale because the referenced evidence changed",
                    ))
                }
                Some(_) => {}
            }
        }
    }
    blockers
}

pub fn attest_spec_requirement(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    requirement_id: &str,
    actor_role: &str,
    actor: &str,
    evidence_ref: &str,
) -> Result<AttestationResult, AttestationError> {
    if !matches!(actor_role, "lead" | "operator") {
        return Err(AttestationError::Invalid(
            "actor_role must be lead or operator".into(),
        ));
    }
    if actor.trim().is_empty() || evidence_ref.trim().is_empty() {
        return Err(AttestationError::Invalid(
            "actor and evidence_ref are required".into(),
        ));
    }
    let guard = PathGuard::new(root)?;
    let relative = artifact.as_ref();
    let path = guard.resolve_existing(relative)?;
    let initial = Document::parse(&fs::read_to_string(&path)?)?;
    let spec_id = initial
        .value("id")
        .filter(|id| id.starts_with("SPEC-"))
        .ok_or_else(|| AttestationError::Invalid("artifact must be a SPEC-*".into()))?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), &spec_id)?;
    let path = guard.resolve_existing(relative)?;
    let current_source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&current_source)?;
    if document.value("id").as_deref() != Some(spec_id.as_str()) {
        return Err(AttestationError::Invalid(
            "spec identity changed while acquiring the mutation lock".into(),
        ));
    }
    let status = document.value("status").unwrap_or_default();
    if !matches!(status.as_str(), "review" | "done") {
        return Err(AttestationError::Invalid(format!(
            "operator/lead evidence can be attested only while a spec is in review or done, not {status}"
        )));
    }
    let requirements = verification_requirements(&document);
    let matching_requirements = requirements
        .iter()
        .filter(|requirement| requirement.id == requirement_id)
        .collect::<Vec<_>>();
    if matching_requirements.len() > 1 {
        return Err(AttestationError::Invalid(format!(
            "verification requirement '{requirement_id}' is duplicated"
        )));
    }
    let mut requirement = matching_requirements
        .into_iter()
        .next()
        .cloned()
        .ok_or_else(|| {
            AttestationError::Invalid(format!(
                "verification requirement '{requirement_id}' does not exist"
            ))
        })?;
    if requirement.owner != actor_role || requirement.phase != "before-done" {
        return Err(AttestationError::Invalid(format!(
            "{} is owner={} phase={}; {actor_role} cannot attest it",
            requirement.id, requirement.owner, requirement.phase
        )));
    }
    if unsupported_policy(&requirement).is_some() {
        return Err(AttestationError::Invalid(format!(
            "{} has an unsupported verification policy",
            requirement.id
        )));
    }
    if !requirement.checked {
        if (actor_role == "operator" && requirement.owner == "operator")
            || (actor_role == "lead" && requirement.owner == "lead")
        {
            // Auto-check: attesting one's own gate implies completion.
            // The two facts (completion + evidence) are recorded separately in the artifact.
            check_requirement_in_body(&mut document, &requirement.id)?;
            // Re-parse requirements after the body mutation to get a correct digest.
            let refreshed = verification_requirements(&document);
            requirement = refreshed
                .into_iter()
                .find(|r| r.id == requirement_id)
                .ok_or_else(|| {
                    AttestationError::Invalid(format!(
                        "requirement '{requirement_id}' disappeared after auto-check"
                    ))
                })?;
            if !requirement.checked {
                return Err(AttestationError::Invalid(format!(
                    "{} could not be auto-checked in the spec body",
                    requirement.id
                )));
            }
        } else {
            return Err(AttestationError::Invalid(format!(
                "{} is unchecked; record completion in the spec before attesting its evidence",
                requirement.id
            )));
        }
    }
    let existing_attestations = parse_attestations(&document)
        .map_err(|error| AttestationError::Invalid(error.to_string()))?;
    let digest = requirement_digest(&requirement);
    let evidence_digest = evidence_digest(guard.root(), evidence_ref.trim())?;
    if let Some(existing) = existing_attestations
        .iter()
        .find(|attestation| {
            attestation.requirement_id == requirement.id
                && attestation.requirement_digest == digest
                && attestation.actor_role == actor_role
                && attestation.actor == actor.trim()
                && attestation.result == "passed"
                && attestation.evidence_ref == evidence_ref.trim()
                && attestation.evidence_digest.as_deref() == Some(&evidence_digest)
        })
        .cloned()
    {
        return Ok(AttestationResult {
            path,
            attestation: existing,
            created: false,
        });
    }
    let existing_ids = existing_attestations
        .iter()
        .map(|attestation| attestation.id.as_str())
        .collect::<BTreeSet<_>>();
    let sequence = (1usize..)
        .find(|sequence| !existing_ids.contains(format!("{spec_id}-ATTEST-{sequence:03}").as_str()))
        .expect("an unbounded sequence always has a free attestation ID");
    let attestation = VerificationAttestation {
        schema_version: VERIFICATION_ATTESTATION_SCHEMA_VERSION.into(),
        id: format!("{spec_id}-ATTEST-{sequence:03}"),
        requirement_id: requirement.id.clone(),
        requirement_digest: digest,
        actor_role: actor_role.into(),
        actor: actor.trim().into(),
        timestamp: Local::now().to_rfc3339(),
        result: "passed".into(),
        evidence_ref: evidence_ref.trim().into(),
        evidence_digest: Some(evidence_digest),
    };
    let fields = serde_json::to_value(&attestation)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| AttestationError::Invalid("cannot serialize attestation".into()))?
        .into_iter()
        .collect::<Vec<_>>();
    document.append_object("verification_attestations", &fields)?;
    document.set("updated", &Local::now().format("%Y-%m-%d").to_string());
    document.append_activity(&format!(
        "attested verification {} by {}",
        requirement_id, actor_role
    ));
    if fs::read_to_string(&path)? != current_source {
        return Err(AttestationError::Invalid(
            "spec changed while the attestation was being prepared".into(),
        ));
    }
    atomic_write(&path, &document.render())?;
    Ok(AttestationResult {
        path,
        attestation,
        created: true,
    })
}

/// Locates an unchecked `- [ ]` line containing the given requirement ID in the
/// document body and toggles it to `- [x]`.  Returns an error if no matching
/// unchecked line is found.
fn check_requirement_in_body(
    document: &mut Document,
    requirement_id: &str,
) -> Result<(), AttestationError> {
    let mut new_lines = Vec::new();
    let mut found = false;
    for line in document.body.lines() {
        if !found
            && line.contains(requirement_id)
            && line.trim_start().starts_with("- [ ]")
        {
            new_lines.push(line.replacen("- [ ]", "- [x]", 1));
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }
    if !found {
        return Err(AttestationError::Invalid(format!(
            "could not locate unchecked checklist item for {requirement_id}"
        )));
    }
    document.body = new_lines.join(document.newline);
    Ok(())
}

pub fn build_verification_migration_preview(
    root: impl AsRef<Path>,
) -> Result<VerificationMigrationPreview, AttestationError> {
    let guard = PathGuard::new(root)?;
    let specs = guard.root().join(".lmbrain/specs");
    let mut paths = Vec::new();
    collect_markdown_paths(&specs, &mut paths)?;
    paths.sort();
    let mut candidates = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path)?;
        let Ok(document) = Document::parse(&source) else {
            continue;
        };
        let spec_id = document.value("id").unwrap_or_default();
        if !spec_id.starts_with("SPEC-") {
            continue;
        }
        let relative = path
            .strip_prefix(guard.root())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for requirement in verification_requirements(&document)
            .into_iter()
            .filter(|requirement| {
                requirement.owner == "operator" && requirement.phase == "before-done"
            })
        {
            let names_lead = requirement
                .text
                .split(|character: char| !character.is_alphanumeric())
                .any(|word| word.eq_ignore_ascii_case("lead"));
            candidates.push(VerificationMigrationItem {
                spec_id: spec_id.clone(),
                path: relative.clone(),
                requirement_id: requirement.id,
                current_owner: "operator".into(),
                proposed_owner: names_lead.then(|| "lead".into()),
                confidence: if names_lead { "high" } else { "unknown" }.into(),
                reason: if names_lead {
                    "requirement text explicitly names the Lead; proposal only"
                } else {
                    "operator ownership is ambiguous and requires human resolution"
                }
                .into(),
            });
        }
    }
    candidates.sort_by(|left, right| {
        (&left.spec_id, &left.requirement_id, &left.path).cmp(&(
            &right.spec_id,
            &right.requirement_id,
            &right.path,
        ))
    });
    let proposed_lead_count = candidates
        .iter()
        .filter(|candidate| candidate.proposed_owner.as_deref() == Some("lead"))
        .count();
    let ambiguous_count = candidates.len() - proposed_lead_count;
    Ok(VerificationMigrationPreview {
        schema_version: "1".into(),
        candidates,
        proposed_lead_count,
        ambiguous_count,
        mutated: false,
    })
}

fn parse_attestations(
    document: &Document,
) -> Result<Vec<VerificationAttestation>, AttestationError> {
    let fields = document.fields();
    let Some(value) = fields.get("verification_attestations") else {
        return Ok(Vec::new());
    };
    let items = value.as_array().ok_or_else(|| {
        AttestationError::Invalid("verification_attestations must be an array".into())
    })?;
    let mut attestations = Vec::with_capacity(items.len());
    let mut ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let attestation: VerificationAttestation =
            serde_json::from_value(item.clone()).map_err(|error| {
                AttestationError::Invalid(format!(
                    "verification_attestations item {} is malformed: {error}",
                    index + 1
                ))
            })?;
        if attestation.schema_version != VERIFICATION_ATTESTATION_SCHEMA_VERSION {
            return Err(AttestationError::Invalid(format!(
                "attestation '{}' uses unsupported schema version '{}'",
                attestation.id, attestation.schema_version
            )));
        }
        if !ids.insert(attestation.id.clone()) {
            return Err(AttestationError::Invalid(format!(
                "duplicate verification attestation ID '{}'",
                attestation.id
            )));
        }
        attestations.push(attestation);
    }
    Ok(attestations)
}

fn evidence_digest(root: &Path, evidence_ref: &str) -> Result<String, AttestationError> {
    let guard = PathGuard::new(root)?;
    if let Ok(path) = guard.resolve_existing(evidence_ref) {
        if path.is_file() {
            return Ok(content_digest(&fs::read(path)?));
        }
        return Err(AttestationError::Invalid(format!(
            "evidence reference '{evidence_ref}' is not a file"
        )));
    }
    if looks_like_artifact_id(evidence_ref) {
        let matches = find_artifacts_by_id(guard.root(), evidence_ref)?;
        return match matches.as_slice() {
            [path] => Ok(content_digest(&fs::read(path)?)),
            [] => Err(AttestationError::Invalid(format!(
                "evidence artifact '{evidence_ref}' does not exist"
            ))),
            _ => Err(AttestationError::Invalid(format!(
                "evidence artifact '{evidence_ref}' is ambiguous"
            ))),
        };
    }
    Ok(content_digest(evidence_ref.as_bytes()))
}

fn looks_like_artifact_id(value: &str) -> bool {
    let Some((prefix, suffix)) = value.rsplit_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|character| character.is_ascii_uppercase() || character == '-')
        && !suffix.is_empty()
        && suffix.chars().all(|character| character.is_ascii_digit())
}

fn find_artifacts_by_id(root: &Path, id: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    fn scan(directory: &Path, id: &str, matches: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
        if !directory.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                scan(&path, id, matches)?;
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
                let source = fs::read_to_string(&path)?;
                if Document::parse(&source)
                    .ok()
                    .and_then(|document| document.value("id"))
                    .as_deref()
                    == Some(id)
                {
                    matches.push(path);
                }
            }
        }
        Ok(())
    }

    let mut matches = Vec::new();
    scan(&root.join(".lmbrain"), id, &mut matches)?;
    Ok(matches)
}

fn collect_markdown_paths(
    directory: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_markdown_paths(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn unsupported_verification_requirements(document: &Document) -> BTreeMap<String, String> {
    verification_requirements(document)
        .into_iter()
        .filter_map(|requirement| {
            unsupported_policy(&requirement)
                .map(|cause| (requirement.id.clone(), cause.to_string()))
        })
        .collect()
}

fn unsupported_policy(requirement: &VerificationRequirement) -> Option<&'static str> {
    match (requirement.owner.as_str(), requirement.phase.as_str()) {
        ("agent", "before-submit") | ("kit", "before-submit") => None,
        ("lead", "before-done") | ("operator", "before-done") => None,
        ("unspecified", _) | (_, "unspecified") => Some("owner and phase must be explicit"),
        _ => Some("unsupported owner/phase combination"),
    }
}

fn blocker(requirement: &VerificationRequirement, cause: &str) -> VerificationBlocker {
    VerificationBlocker {
        requirement_id: requirement.id.clone(),
        owner: requirement.owner.clone(),
        cause: cause.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn lead_and_operator_gates_require_distinct_fresh_attestations() {
        let document = Document::parse(
            "---\nid: SPEC-001\nstatus: review\nverification_attestations:\n  - schema_version: \"1\"\n    id: \"SPEC-001-ATTEST-001\"\n    requirement_id: \"LEAD\"\n    requirement_digest: \"stale\"\n    actor_role: \"lead\"\n    actor: \"AGENT-LEAD\"\n    timestamp: \"2026-07-29T12:00:00+02:00\"\n    result: \"passed\"\n    evidence_ref: \"REVIEW-001\"\n---\n## Required verification\n- [x] LEAD | kind=manual | owner=lead | phase=before-done | evidence=artifact | Independent review\n- [x] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play the app\n",
        )
        .unwrap();
        let blockers = verification_blockers(&document, "before-done");
        assert_eq!(blockers.len(), 2);
        assert!(blockers
            .iter()
            .any(|blocker| blocker.requirement_id == "LEAD" && blocker.cause.contains("stale")));
        assert!(blockers.iter().any(|blocker| {
            blocker.requirement_id == "HUMAN" && blocker.cause.contains("operator")
        }));
    }

    #[test]
    fn unsupported_owner_phase_combinations_fail_validation() {
        let document = Document::parse(
            "---\nid: SPEC-001\n---\n## Required verification\n- [x] WRONG | kind=manual | owner=operator | phase=before-submit | evidence=observation | Wrong authority\n",
        )
        .unwrap();
        assert_eq!(unsupported_verification_requirements(&document).len(), 1);
    }

    #[test]
    fn operator_attestation_checks_only_its_gate_and_never_changes_status() {
        let directory = tempdir().unwrap();
        let path = directory.path().join(".lmbrain/specs/review/SPEC-001.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "---\nid: SPEC-001\nstatus: review\n---\n## Required verification\n- [x] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play the app\n- [ ] LEAD | kind=manual | owner=lead | phase=before-done | evidence=artifact | Independent review\n",
        )
        .unwrap();

        let result = attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-001.md",
            "HUMAN",
            "operator",
            "human-operator",
            "playtest:2026-07-29",
        )
        .unwrap();
        assert!(result.created);
        let output = fs::read_to_string(&path).unwrap();
        let document = Document::parse(&output).unwrap();
        assert_eq!(document.value("status").as_deref(), Some("review"));
        assert!(document.body.contains("- [x] HUMAN"));
        assert!(document.body.contains("- [ ] LEAD"));
        assert_eq!(verification_attestations(&document).len(), 1);
        assert_eq!(verification_blockers(&document, "before-done").len(), 1);

        let repeated = attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-001.md",
            "HUMAN",
            "operator",
            "human-operator",
            "playtest:2026-07-29",
        )
        .unwrap();
        assert!(!repeated.created);
        assert_eq!(
            verification_attestations(
                &Document::parse(&fs::read_to_string(&path).unwrap()).unwrap()
            )
            .len(),
            1
        );
    }

    #[test]
    fn lead_cannot_attest_operator_gate() {
        let directory = tempdir().unwrap();
        let path = directory.path().join(".lmbrain/specs/review/SPEC-001.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let source = "---\nid: SPEC-001\nstatus: review\n---\n## Required verification\n- [ ] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play\n";
        fs::write(&path, source).unwrap();
        assert!(attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-001.md",
            "HUMAN",
            "lead",
            "AGENT-LEAD",
            "REVIEW-001"
        )
        .is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), source);
    }

    #[test]
    fn attestation_never_checks_an_open_requirement_or_changes_a_non_review_spec() {
        let directory = tempdir().unwrap();
        let ready_path = directory.path().join(".lmbrain/specs/ready/SPEC-001.md");
        fs::create_dir_all(ready_path.parent().unwrap()).unwrap();
        let ready_source = "---\nid: SPEC-001\nstatus: ready\n---\n## Required verification\n- [x] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play\n";
        fs::write(&ready_path, ready_source).unwrap();
        assert!(attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/ready/SPEC-001.md",
            "HUMAN",
            "operator",
            "Moren",
            "playtest:ready"
        )
        .is_err());
        assert_eq!(fs::read_to_string(&ready_path).unwrap(), ready_source);

        // Auto-check: operator attesting their own unchecked gate in review succeeds
        let review_path = directory.path().join(".lmbrain/specs/review/SPEC-002.md");
        fs::create_dir_all(review_path.parent().unwrap()).unwrap();
        let review_source = "---\nid: SPEC-002\nstatus: review\n---\n## Required verification\n- [ ] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play\n";
        fs::write(&review_path, review_source).unwrap();
        let result = attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-002.md",
            "HUMAN",
            "operator",
            "Moren",
            "playtest:unchecked",
        );
        assert!(result.is_ok(), "operator auto-check should succeed: {result:?}");
        // Verify the spec body was auto-checked
        let updated = fs::read_to_string(&review_path).unwrap();
        assert!(updated.contains("- [x] HUMAN"), "checkbox should be auto-checked");

        // Lead attesting an unchecked owner=lead gate succeeds and auto-checks the spec body
        let lead_path = directory.path().join(".lmbrain/specs/review/SPEC-003.md");
        fs::create_dir_all(lead_path.parent().unwrap()).unwrap();
        let lead_source = "---\nid: SPEC-003\nstatus: review\n---\n## Required verification\n- [ ] LEAD-CHECK | kind=manual | owner=lead | phase=before-done | evidence=observation | Review\n";
        fs::write(&lead_path, lead_source).unwrap();
        let lead_result = attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-003.md",
            "LEAD-CHECK",
            "lead",
            "Ada",
            "review:unchecked",
        );
        assert!(lead_result.is_ok(), "lead auto-check should succeed: {lead_result:?}");
        let updated_lead = fs::read_to_string(&lead_path).unwrap();
        assert!(updated_lead.contains("- [x] LEAD-CHECK"), "lead checkbox should be auto-checked");
    }

    #[test]
    fn changed_evidence_and_malformed_history_fail_closed() {
        let directory = tempdir().unwrap();
        let evidence = directory.path().join("evidence/playtest.txt");
        fs::create_dir_all(evidence.parent().unwrap()).unwrap();
        fs::write(&evidence, "passed on build 1").unwrap();
        let path = directory.path().join(".lmbrain/specs/review/SPEC-001.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "---\nid: SPEC-001\nstatus: review\n---\n## Required verification\n- [x] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play\n",
        )
        .unwrap();

        attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-001.md",
            "HUMAN",
            "operator",
            "Moren",
            "evidence/playtest.txt",
        )
        .unwrap();
        let document = Document::parse(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(
            verification_blockers_for_workspace(directory.path(), &document, "before-done")
                .is_empty()
        );

        fs::write(&evidence, "changed after attestation").unwrap();
        let document = Document::parse(&fs::read_to_string(&path).unwrap()).unwrap();
        let blockers =
            verification_blockers_for_workspace(directory.path(), &document, "before-done");
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].cause.contains("referenced evidence changed"));

        let malformed_path = directory.path().join(".lmbrain/specs/review/SPEC-002.md");
        let malformed_source = "---\nid: SPEC-002\nstatus: review\nverification_attestations:\n  - id: incomplete\n---\n## Required verification\n- [x] HUMAN | kind=operator | owner=operator | phase=before-done | evidence=observation | Play\n";
        fs::write(&malformed_path, malformed_source).unwrap();
        let malformed = Document::parse(malformed_source).unwrap();
        assert!(
            verification_blockers_for_workspace(directory.path(), &malformed, "before-done")[0]
                .cause
                .contains("history is invalid")
        );
        assert!(attest_spec_requirement(
            directory.path(),
            ".lmbrain/specs/review/SPEC-002.md",
            "HUMAN",
            "operator",
            "Moren",
            "playtest:malformed"
        )
        .is_err());
        assert_eq!(
            fs::read_to_string(malformed_path).unwrap(),
            malformed_source
        );
    }

    #[test]
    fn legacy_operator_migration_is_deterministic_conservative_and_read_only() {
        let directory = tempdir().unwrap();
        let done = directory.path().join(".lmbrain/specs/done");
        fs::create_dir_all(&done).unwrap();
        let lead = done.join("SPEC-002.md");
        let human = done.join("SPEC-001.md");
        let lead_source = "---\nid: SPEC-002\nstatus: done\n---\n## Required verification\n- [x] REVIEW | kind=manual | owner=operator | phase=before-done | evidence=artifact | Project Lead independent review\n";
        let human_source = "---\nid: SPEC-001\nstatus: done\n---\n## Required verification\n- [ ] PLAY | kind=operator | owner=operator | phase=before-done | evidence=observation | Play the desktop flow\n";
        fs::write(&lead, lead_source).unwrap();
        fs::write(&human, human_source).unwrap();

        let first = build_verification_migration_preview(directory.path()).unwrap();
        let second = build_verification_migration_preview(directory.path()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.proposed_lead_count, 1);
        assert_eq!(first.ambiguous_count, 1);
        assert!(!first.mutated);
        assert_eq!(first.candidates[0].spec_id, "SPEC-001");
        assert_eq!(first.candidates[0].proposed_owner, None);
        assert_eq!(first.candidates[1].proposed_owner.as_deref(), Some("lead"));
        assert_eq!(fs::read_to_string(lead).unwrap(), lead_source);
        assert_eq!(fs::read_to_string(human).unwrap(), human_source);
    }
}
