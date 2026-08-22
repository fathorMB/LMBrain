pub mod execution;
pub mod fingerprint;
pub mod manifest;
pub mod transcript;

use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use serde::Serialize;
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document},
    harness_manifest::workspace_identity,
    mutation_lock::WorkspaceLock,
};

pub use execution::{
    minimal_gate_environment, minimal_gate_environment_from, run_gate,
    MinimalGateEnvironment, VerificationGateResult,
};
pub use fingerprint::{
    workspace_content_fingerprint, workspace_content_fingerprint_with,
};
pub use manifest::{
    approve_verification_manifest, canonical_verification_manifest_digest,
    gate_contract_digest, load_verification_manifest, manifest_exclusions,
    parse_manifest, validate_verification_manifest, VerificationApproval,
    VerificationGate, VerificationManifest, VERIFICATION_MANIFEST_PATH,
};
pub use transcript::{
    generated_transcript, generated_transcript_range, has_nonempty_fence,
    metadata, render_transcript, replace_transcript, section_at_level,
    transcript_hash_matches, transcript_state, transcript_state_for_document,
    transcript_state_with_exclusions, TranscriptState, GENERATED_TRANSCRIPT_END,
    GENERATED_TRANSCRIPT_START,
};

#[derive(Debug, Clone, Serialize)]
pub struct VerificationRunReport {
    pub spec_id: String,
    pub manifest_digest: String,
    /// Fingerprint captured after the final gate; kept under the historical
    /// name so 2.9.1 freshness checks keep working.
    pub workspace_fingerprint: String,
    /// Fingerprint captured before the first gate ran.
    pub workspace_fingerprint_before: String,
    pub transcript_hash: String,
    pub all_expectations_met: bool,
    /// True when the workspace changed between the pre- and post-gate
    /// fingerprints; such evidence is never publishable as fresh.
    pub invalidated: bool,
    pub invalidation_reason: Option<String>,
    pub results: Vec<VerificationGateResult>,
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("verification manifest does not exist: {0}")]
    MissingManifest(String),
    #[error("unsafe verification path: {0}")]
    UnsafePath(String),
    #[error("invalid verification manifest: {0}")]
    InvalidManifest(String),
    #[error("verification manifest is not approved for this workspace and digest")]
    ApprovalRequired,
    #[error("spec has no verification_gates references")]
    NoRequiredGates,
    #[error("unknown verification gate '{0}'")]
    UnknownGate(String),
    #[error("cannot read or write verification data: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot parse artifact: {0}")]
    Artifact(String),
    #[error("spec changed while verification was running: {0}")]
    ConcurrentModification(String),
    #[error("cannot launch verification gate '{gate}': {source}")]
    Launch {
        gate: String,
        #[source]
        source: std::io::Error,
    },
}

pub fn execute_spec_verification(
    root: &Path,
    spec_path: &Path,
    approval_store: &Path,
) -> Result<VerificationRunReport, VerificationError> {
    let canonical_root = root.canonicalize()?;
    let canonical_spec = spec_path.canonicalize()?;
    if !canonical_spec.starts_with(&canonical_root) {
        return Err(VerificationError::UnsafePath(
            spec_path.display().to_string(),
        ));
    }
    let manifest = load_verification_manifest(&canonical_root)?;
    let manifest_digest = canonical_verification_manifest_digest(&manifest)?;
    let identity = workspace_identity(&canonical_root)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let approval_source = fs::read_to_string(approval_store).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            VerificationError::ApprovalRequired
        } else {
            VerificationError::Io(error)
        }
    })?;
    let approval: VerificationApproval = serde_json::from_str(&approval_source)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    if approval.workspace_fingerprint != identity.fingerprint
        || approval.manifest_digest != manifest_digest
    {
        return Err(VerificationError::ApprovalRequired);
    }

    let document = Document::parse(&fs::read_to_string(&canonical_spec)?)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let spec_id = document
        .value("id")
        .ok_or_else(|| VerificationError::Artifact("missing spec id".into()))?;
    let required = document.string_array("verification_gates");
    if required.is_empty() {
        return Err(VerificationError::NoRequiredGates);
    }
    let gate_contract_digest = gate_contract_digest(&required);
    let by_id: BTreeMap<_, _> = manifest.gates.iter().map(|gate| (&gate.id, gate)).collect();
    // The final artifact lock only protects the transcript write below; the
    // gate-execution interval itself is snapshot-checked by comparing a
    // fingerprint taken before the first gate with one taken after the last.
    // Full isolated-worktree/per-gate input scoping is deferred to 3.0.0.
    let exclusions = manifest_exclusions(&manifest, &required);
    let pre_fingerprint = workspace_content_fingerprint_with(&canonical_root, &exclusions)?;
    let mut results = Vec::new();
    for id in required {
        let gate = by_id
            .get(&id)
            .ok_or_else(|| VerificationError::UnknownGate(id.clone()))?;
        results.push(run_gate(&canonical_root, gate)?);
    }
    let source_fingerprint = workspace_content_fingerprint_with(&canonical_root, &exclusions)?;
    let invalidation_reason = (pre_fingerprint != source_fingerprint).then(|| {
        "workspace content changed during gate execution; evidence is not snapshot-consistent. \
         If a gate intentionally writes build artifacts, declare its output directory in that \
         gate's fingerprint_exclude"
            .to_string()
    });
    let transcript_without_hash = render_transcript(
        &manifest_digest,
        &pre_fingerprint,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        invalidation_reason.as_deref(),
        None,
    );
    let transcript_hash = manifest::hex_digest(transcript_without_hash.as_bytes());
    let transcript = render_transcript(
        &manifest_digest,
        &pre_fingerprint,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        invalidation_reason.as_deref(),
        Some(&transcript_hash),
    );
    write_verification_transcript(
        &canonical_root,
        &canonical_spec,
        &spec_id,
        &document.string_array("verification_gates"),
        &transcript,
        &transcript_hash,
        &source_fingerprint,
    )?;
    let all_expectations_met =
        invalidation_reason.is_none() && results.iter().all(|result| result.expectation_met);
    Ok(VerificationRunReport {
        spec_id,
        manifest_digest,
        workspace_fingerprint: source_fingerprint,
        workspace_fingerprint_before: pre_fingerprint,
        transcript_hash,
        all_expectations_met,
        invalidated: invalidation_reason.is_some(),
        invalidation_reason,
        results,
    })
}

pub fn write_verification_transcript(
    root: &Path,
    canonical_spec: &Path,
    spec_id: &str,
    required_gates: &[String],
    transcript: &str,
    transcript_hash: &str,
    source_fingerprint: &str,
) -> Result<(), VerificationError> {
    let _lock = WorkspaceLock::acquire(root)?;
    if !canonical_spec.exists()
        || canonical_spec
            .canonicalize()
            .map(|path| path != canonical_spec)
            .unwrap_or(true)
    {
        return Err(VerificationError::ConcurrentModification(
            "the spec was moved, replaced, or deleted; verification evidence was not written"
                .into(),
        ));
    }

    let current_source = fs::read_to_string(canonical_spec)?;
    let mut current = Document::parse(&current_source)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    if current.value("id").as_deref() != Some(spec_id) {
        return Err(VerificationError::ConcurrentModification(
            "the artifact at the original path has a different id".into(),
        ));
    }
    if current.string_array("verification_gates") != required_gates {
        return Err(VerificationError::ConcurrentModification(
            "verification_gates changed; rerun verification against the new gate contract".into(),
        ));
    }

    current.body = replace_transcript(&current.body, transcript)?;
    current.append_activity(&format!(
        "spec_verify generated transcript {transcript_hash} for workspace {source_fingerprint}"
    ));
    if fs::read_to_string(canonical_spec)? != current_source {
        return Err(VerificationError::ConcurrentModification(
            "the spec changed again while verification evidence was being merged".into(),
        ));
    }
    atomic_write(canonical_spec, &current.render())
        .map_err(|error| VerificationError::Artifact(error.to_string()))
}
