//! Governed replacement of the `verification_gates` set declared by a spec.
//!
//! The contract states that specs reference executable gates through
//! `verification_gates`, but until 4.2.2 no verb wrote that field: a manifest
//! could be approved and never bound to any spec, so every requirement stayed
//! self-declared evidence (KIT-NOTE-001). This mirrors `set_spec_dependencies`
//! — actor, reason, optimistic concurrency on the source digest, validation
//! against the current manifest, and an append-only event.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::WorkspaceLock,
    path::{PathError, PathGuard},
    transitions::{kind_for_id, ArtifactKind},
    verification::{load_verification_manifest, VerificationError},
};

pub const SPEC_VERIFICATION_GATE_EVENT_SCHEMA_VERSION: &str = "1";

/// Statuses in which the gate contract may still be replaced. `review` and
/// `done` are excluded: a spec under or past review has already been verified
/// against a specific gate contract, and silently swapping it would invalidate
/// the recorded transcript instead of re-earning it.
const MUTABLE_STATUSES: &[&str] = &["backlog", "ready", "working"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecVerificationGatesMutation {
    pub id: String,
    pub path: PathBuf,
    pub verification_gates: Vec<String>,
    pub previous_verification_gates: Vec<String>,
    pub updated: String,
    pub source_digest: String,
}

#[derive(Debug, Error)]
pub enum SpecVerificationGatesError {
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("invalid verification gates: {0}")]
    Invalid(String),
    #[error("verification gate mutation conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Replaces the `verification_gates` set of an existing spec.
///
/// An empty set clears the reference and does not require a manifest: a spec
/// must be able to drop gates even when the manifest itself is being reworked.
/// A non-empty set is validated against the gate IDs of the current manifest,
/// so a typo fails here instead of surfacing later as a spec that quietly
/// executes nothing.
pub fn set_spec_verification_gates(
    root: &Path,
    artifact: &Path,
    mut verification_gates: Vec<String>,
    actor: &str,
    reason: &str,
    expected_digest: &str,
) -> Result<SpecVerificationGatesMutation, SpecVerificationGatesError> {
    require_text("actor", actor)?;
    require_text("reason", reason)?;
    require_text("expected_digest", expected_digest)?;
    verification_gates
        .iter_mut()
        .for_each(|gate| *gate = gate.trim().to_string());

    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(artifact)?;
    let initial = Document::parse(&fs::read_to_string(&path)?)?;
    let initial_id = initial
        .value("id")
        .ok_or_else(|| SpecVerificationGatesError::Invalid("artifact is missing id".into()))?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&source)?;
    let id = document.value("id").unwrap_or_default();
    if id != initial_id || kind_for_id(&id) != Some(ArtifactKind::Spec) {
        return Err(SpecVerificationGatesError::Invalid(
            "verification gate mutation requires a stable SPEC-* artifact".into(),
        ));
    }
    let status = document.value("status").unwrap_or_default();
    if !MUTABLE_STATUSES.contains(&status.as_str()) {
        return Err(SpecVerificationGatesError::Invalid(format!(
            "verification gates can only be changed while a spec is in {}; '{id}' is '{status}'",
            MUTABLE_STATUSES.join(", ")
        )));
    }
    let actual_digest = crate::content_digest(source.as_bytes());
    if actual_digest != expected_digest {
        return Err(SpecVerificationGatesError::Conflict(format!(
            "expected source digest '{expected_digest}', found '{actual_digest}'"
        )));
    }

    validate_candidate_gates(root, &verification_gates)?;

    let previous = document.string_array("verification_gates");
    let updated = Local::now().format("%Y-%m-%d").to_string();
    document.set(
        "verification_gates",
        &serde_json::to_string(&verification_gates).unwrap_or_else(|_| "[]".into()),
    );
    document.set("updated", &updated);
    document.append_activity("updated declared verification gates");
    let sequence = document.object_array("verification_gate_events").len() + 1;
    document.append_object(
        "verification_gate_events",
        &[
            (
                "schema_version".into(),
                json!(SPEC_VERIFICATION_GATE_EVENT_SCHEMA_VERSION),
            ),
            (
                "id".into(),
                json!(format!("{id}-VERIFICATION-GATES-{sequence:03}")),
            ),
            ("timestamp".into(), json!(Local::now().to_rfc3339())),
            ("actor".into(), json!(actor.trim())),
            ("reason".into(), json!(reason.trim())),
            ("previous".into(), json!(previous)),
            ("current".into(), json!(verification_gates)),
        ],
    )?;
    if fs::read_to_string(&path)? != source {
        return Err(SpecVerificationGatesError::Conflict(
            "artifact changed while verification gate mutation was prepared".into(),
        ));
    }
    let rendered = document.render();
    atomic_write(&path, &rendered)?;
    Ok(SpecVerificationGatesMutation {
        id,
        path,
        verification_gates,
        previous_verification_gates: previous,
        updated,
        source_digest: crate::content_digest(rendered.as_bytes()),
    })
}

fn validate_candidate_gates(
    root: &Path,
    verification_gates: &[String],
) -> Result<(), SpecVerificationGatesError> {
    let mut seen = HashSet::new();
    for gate in verification_gates {
        if gate.is_empty() {
            return Err(SpecVerificationGatesError::Invalid(
                "verification_gates entries cannot be empty".into(),
            ));
        }
        if !seen.insert(gate) {
            return Err(SpecVerificationGatesError::Invalid(format!(
                "verification_gates contains duplicate '{gate}'"
            )));
        }
    }
    if verification_gates.is_empty() {
        return Ok(());
    }
    let manifest = load_verification_manifest(root).map_err(|error| match error {
        VerificationError::MissingManifest(path) => SpecVerificationGatesError::Invalid(format!(
            "verification gates cannot be declared before a manifest exists at {path}; run verification_manifest_init and verification_manifest_set first"
        )),
        other => SpecVerificationGatesError::Invalid(format!(
            "verification manifest cannot be read: {other}"
        )),
    })?;
    let known = manifest
        .gates
        .iter()
        .map(|gate| gate.id.as_str())
        .collect::<HashSet<_>>();
    for gate in verification_gates {
        if !known.contains(gate.as_str()) {
            let mut available = manifest
                .gates
                .iter()
                .map(|gate| gate.id.clone())
                .collect::<Vec<_>>();
            available.sort();
            return Err(SpecVerificationGatesError::Invalid(format!(
                "verification_gates references '{gate}', absent from the current manifest (known gates: {})",
                if available.is_empty() {
                    "none".to_string()
                } else {
                    available.join(", ")
                }
            )));
        }
    }
    Ok(())
}

fn require_text(label: &str, value: &str) -> Result<(), SpecVerificationGatesError> {
    if value.trim().is_empty() {
        Err(SpecVerificationGatesError::Invalid(format!(
            "{label} cannot be empty"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const MANIFEST: &str = "schema_version = 1\n\n[[gates]]\nid = \"unit\"\nprogram = \"cargo\"\nargs = [\"test\"]\n\n[[gates]]\nid = \"lint\"\nprogram = \"cargo\"\nargs = [\"clippy\"]\n";

    fn workspace(status: &str, gates: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(".lmbrain/specs/working")).unwrap();
        fs::write(root.join(".lmbrain/verification.toml"), MANIFEST).unwrap();
        let spec = root.join(".lmbrain/specs/working/SPEC-001-gates.md");
        fs::write(
            &spec,
            format!(
                "---\nid: SPEC-001\ntitle: Gates\nstatus: {status}\nverification_gates: {gates}\n---\n\n## Acceptance criteria\n\n- [ ] Works\n"
            ),
        )
        .unwrap();
        (dir, spec)
    }

    fn digest(path: &Path) -> String {
        crate::content_digest(&fs::read(path).unwrap())
    }

    #[test]
    fn replaces_gates_and_records_an_event() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        let result = set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into(), "lint".into()],
            "AGENT-LEAD",
            "bind the approved manifest",
            &digest(&spec),
        )
        .unwrap();

        assert_eq!(result.verification_gates, vec!["unit", "lint"]);
        assert!(result.previous_verification_gates.is_empty());
        let document = Document::parse(&fs::read_to_string(&spec).unwrap()).unwrap();
        assert_eq!(
            document.string_array("verification_gates"),
            ["unit", "lint"]
        );
        let events = document.object_array("verification_gate_events");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].get("actor").and_then(|value| value.as_str()),
            Some("AGENT-LEAD")
        );
        assert_eq!(
            events[0].get("id").and_then(|value| value.as_str()),
            Some("SPEC-001-VERIFICATION-GATES-001")
        );
    }

    #[test]
    fn appends_a_second_event_without_rewriting_history() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into()],
            "AGENT-LEAD",
            "first binding",
            &digest(&spec),
        )
        .unwrap();
        let result = set_spec_verification_gates(
            &root,
            &spec,
            vec!["lint".into()],
            "AGENT-LEAD",
            "swap the gate",
            &digest(&spec),
        )
        .unwrap();

        assert_eq!(result.previous_verification_gates, vec!["unit"]);
        let document = Document::parse(&fs::read_to_string(&spec).unwrap()).unwrap();
        let events = document.object_array("verification_gate_events");
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[1].get("previous").and_then(|value| value.as_array()),
            Some(&vec![json!("unit")])
        );
    }

    #[test]
    fn rejects_a_gate_absent_from_the_manifest() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        let error = set_spec_verification_gates(
            &root,
            &spec,
            vec!["typo".into()],
            "AGENT-LEAD",
            "bind",
            &digest(&spec),
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("absent from the current manifest"),
            "unexpected error: {error}"
        );
        assert!(error.to_string().contains("lint, unit"), "{error}");
    }

    #[test]
    fn rejects_duplicates_and_empty_entries() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        let duplicate = set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into(), "unit".into()],
            "AGENT-LEAD",
            "bind",
            &digest(&spec),
        )
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate"), "{duplicate}");

        let empty = set_spec_verification_gates(
            &root,
            &spec,
            vec![String::new()],
            "AGENT-LEAD",
            "bind",
            &digest(&spec),
        )
        .unwrap_err();
        assert!(empty.to_string().contains("cannot be empty"), "{empty}");
    }

    #[test]
    fn clearing_gates_does_not_require_a_manifest() {
        let (dir, spec) = workspace("working", "[unit]");
        let root = dir.path().canonicalize().unwrap();
        fs::remove_file(root.join(".lmbrain/verification.toml")).unwrap();
        let result = set_spec_verification_gates(
            &root,
            &spec,
            Vec::new(),
            "AGENT-LEAD",
            "manifest rework",
            &digest(&spec),
        )
        .unwrap();

        assert!(result.verification_gates.is_empty());
        assert_eq!(result.previous_verification_gates, vec!["unit"]);
    }

    #[test]
    fn rejects_a_stale_source_digest() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        let error = set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into()],
            "AGENT-LEAD",
            "bind",
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap_err();

        assert!(
            matches!(error, SpecVerificationGatesError::Conflict(_)),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_mutation_once_the_spec_is_in_review() {
        let (dir, spec) = workspace("review", "[unit]");
        let root = dir.path().canonicalize().unwrap();
        let error = set_spec_verification_gates(
            &root,
            &spec,
            vec!["lint".into()],
            "AGENT-LEAD",
            "swap under review",
            &digest(&spec),
        )
        .unwrap_err();

        assert!(
            error.to_string().contains("'SPEC-001' is 'review'"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_an_empty_actor_or_reason() {
        let (dir, spec) = workspace("working", "[]");
        let root = dir.path().canonicalize().unwrap();
        let source_digest = digest(&spec);
        assert!(set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into()],
            "   ",
            "bind",
            &source_digest,
        )
        .is_err());
        assert!(set_spec_verification_gates(
            &root,
            &spec,
            vec!["unit".into()],
            "AGENT-LEAD",
            "",
            &source_digest,
        )
        .is_err());
    }
}
