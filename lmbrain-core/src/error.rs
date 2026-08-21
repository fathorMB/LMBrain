use thiserror::Error;
use crate::frontmatter::FrontmatterError;
use crate::path::{ArtifactReadError, PathError};
use crate::transitions::ArtifactKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    ArtifactNotFound,
    IllegalTransition,
    InvariantViolation,
    LockContention,
    FrontmatterMalformed,
    FrontmatterMissing,
    PathEscape,
    ReservedField,
    InvalidField,
    MissingForceReason,
    AuthorityMismatch,
    IoError,
    ValidationFailed,
    ConfigurationError,
    UnsupportedOperation,
    ConcurrentModification,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArtifactNotFound => "artifact-not-found",
            Self::IllegalTransition => "illegal-transition",
            Self::InvariantViolation => "invariant-violation",
            Self::LockContention => "lock-contention",
            Self::FrontmatterMalformed => "frontmatter-malformed",
            Self::FrontmatterMissing => "frontmatter-missing",
            Self::PathEscape => "path-escape",
            Self::ReservedField => "reserved-field",
            Self::InvalidField => "invalid-field",
            Self::MissingForceReason => "missing-force-reason",
            Self::AuthorityMismatch => "authority-mismatch",
            Self::IoError => "io-error",
            Self::ValidationFailed => "validation-failed",
            Self::ConfigurationError => "configuration-error",
            Self::UnsupportedOperation => "unsupported-operation",
            Self::ConcurrentModification => "concurrent-modification",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Path(#[from] PathError),

    #[error(transparent)]
    ArtifactRead(#[from] ArtifactReadError),

    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),

    #[error("artifact not found: {0}")]
    NotFound(String),

    #[error("artifact is missing required field '{0}'")]
    Missing(String),

    #[error("illegal {kind:?} transition from '{from}' to '{to}'")]
    Illegal {
        kind: ArtifactKind,
        from: String,
        to: String,
    },

    #[error("invariant failed: {0}")]
    Invariant(String),

    #[error("invalid creation status '{status}' for {kind:?} artifacts; allowed: {allowed}")]
    InvalidCreationStatus {
        kind: ArtifactKind,
        status: String,
        allowed: String,
    },

    #[error("field '{0}' is core-owned lifecycle metadata and cannot be set at creation")]
    ReservedField(String),

    #[error("invalid field: {0}")]
    InvalidField(String),

    #[error("force requires a non-empty reason")]
    MissingForceReason,

    #[error("authority mismatch: {0}")]
    AuthorityMismatch(String),

    #[error("lock contention: {0}")]
    LockContention(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("invalid: {0}")]
    Invalid(String),

    #[error("concurrent modification: {0}")]
    Concurrent(String),

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl CoreError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::NotFound(_) => ErrorCode::ArtifactNotFound,
            Self::Missing(_) => ErrorCode::FrontmatterMissing,
            Self::Illegal { .. } => ErrorCode::IllegalTransition,
            Self::Invariant(_) => ErrorCode::InvariantViolation,
            Self::InvalidCreationStatus { .. } => ErrorCode::InvalidField,
            Self::ReservedField(_) => ErrorCode::ReservedField,
            Self::InvalidField(_) => ErrorCode::InvalidField,
            Self::MissingForceReason => ErrorCode::MissingForceReason,
            Self::AuthorityMismatch(_) => ErrorCode::AuthorityMismatch,
            Self::LockContention(_) => ErrorCode::LockContention,
            Self::Validation(_) => ErrorCode::ValidationFailed,
            Self::Invalid(_) => ErrorCode::ValidationFailed,
            Self::Concurrent(_) => ErrorCode::ConcurrentModification,
            Self::Configuration(_) => ErrorCode::ConfigurationError,
            Self::Unsupported(_) => ErrorCode::UnsupportedOperation,
            Self::Path(PathError::OutsideRoot(_)) => ErrorCode::PathEscape,
            Self::Path(PathError::MissingRoot(_)) => ErrorCode::ArtifactNotFound,
            Self::Path(PathError::Missing(_)) => ErrorCode::ArtifactNotFound,
            Self::ArtifactRead(ArtifactReadError::OutsideWorkspace(_)) => ErrorCode::PathEscape,
            Self::ArtifactRead(ArtifactReadError::NotFound(_)) => ErrorCode::ArtifactNotFound,
            Self::ArtifactRead(ArtifactReadError::InvalidPath(_)) => ErrorCode::PathEscape,
            Self::ArtifactRead(ArtifactReadError::NotAFile(_)) => ErrorCode::ValidationFailed,
            Self::ArtifactRead(ArtifactReadError::WorkspaceUnavailable) => ErrorCode::ArtifactNotFound,
            Self::Frontmatter(FrontmatterError::Malformed) => ErrorCode::FrontmatterMalformed,
            Self::Frontmatter(FrontmatterError::Invalid(_)) => ErrorCode::FrontmatterMalformed,
            Self::Frontmatter(FrontmatterError::Io(_)) => ErrorCode::IoError,
            Self::Io(_) => ErrorCode::IoError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_and_formatting() {
        let err = CoreError::NotFound("SPEC-999".into());
        assert_eq!(err.code(), ErrorCode::ArtifactNotFound);
        assert_eq!(err.code().as_str(), "artifact-not-found");
        assert_eq!(err.code().to_string(), "artifact-not-found");

        let err = CoreError::Illegal {
            kind: ArtifactKind::Spec,
            from: "backlog".into(),
            to: "done".into(),
        };
        assert_eq!(err.code(), ErrorCode::IllegalTransition);

        let err = CoreError::Invariant("missing verification".into());
        assert_eq!(err.code(), ErrorCode::InvariantViolation);

        let err = CoreError::Frontmatter(FrontmatterError::Malformed);
        assert_eq!(err.code(), ErrorCode::FrontmatterMalformed);
    }
}
