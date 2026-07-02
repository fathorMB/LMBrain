//! Tauri-free, filesystem-backed controlled mutations for LMBrain artifacts.
pub mod context;
pub mod frontmatter;
pub mod invariants;
pub mod path;
pub mod transitions;

pub use context::{
    build_project_digest, build_review_context, build_spec_context, AgentProfileSummary,
    CompactAdr, CompactReview, CompactSpec, Criterion, DiagnosticsSummary, ProjectDigest,
    ReviewContext, SpecContext,
};
pub use transitions::{
    ArtifactKind, CreateRequest, MutationOptions, MutationResult, TransitionError,
};
