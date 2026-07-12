//! Tauri-free, filesystem-backed controlled mutations for LMBrain artifacts.
pub mod context;
pub mod frontmatter;
pub mod harness_manifest;
pub mod invariants;
pub mod path;
pub mod transitions;

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
pub use transitions::{
    ArtifactKind, CreateRequest, MutationOptions, MutationResult, TransitionError,
};
