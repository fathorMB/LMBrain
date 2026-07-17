//! Tauri-free, filesystem-backed controlled mutations for LMBrain artifacts.
pub mod context;
pub mod frontmatter;
pub mod harness_manifest;
pub mod improvement;
pub mod invariants;
mod mutation_lock;
pub mod path;
pub mod transitions;
pub mod verification;

pub use context::{
    build_project_digest, build_review_context, build_spec_context, AgentProfileSummary,
    CompactAdr, CompactReview, CompactSpec, Criterion, DiagnosticsSummary, ProjectDigest,
    ReviewContext, SpecContext,
};
pub use harness_manifest::{
    canonical_manifest_digest, content_digest, load_harness_manifest, parse_harness_manifest,
    set_harness_manifest, validate_harness_manifest, workspace_identity, CapabilityState,
    HarnessHost, HarnessManifest, HarnessManifestError, HarnessManifestMutation,
    HarnessValidationIssue, HostConfiguration, LspRequirement, WorkspaceIdentity,
};
pub use improvement::{
    apply_improvement_proposal, build_agent_improvement_signals, create_improvement_proposal,
    AgentEffectivenessMetrics, AgentImprovementSignal, ImprovementApplyResult, ImprovementError,
    ImprovementProposalRequest,
};
pub use path::{read_artifact, ArtifactReadError};
pub use transitions::{
    ArtifactKind, CreateRequest, MutationOptions, MutationResult, TransitionError,
};
pub use verification::{
    approve_verification_manifest, canonical_verification_manifest_digest,
    execute_spec_verification, load_verification_manifest, transcript_state,
    transcript_state_for_document, validate_verification_manifest, workspace_content_fingerprint,
    TranscriptState, VerificationApproval, VerificationError, VerificationGate,
    VerificationManifest, VerificationRunReport,
};
