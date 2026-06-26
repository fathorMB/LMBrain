//! Tauri-free, filesystem-backed controlled mutations for LMBrain artifacts.
pub mod frontmatter;
pub mod invariants;
pub mod path;
pub mod transitions;

pub use transitions::{
    ArtifactKind, CreateRequest, MutationOptions, MutationResult, TransitionError,
};
