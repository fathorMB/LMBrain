use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum AppError {
    #[error("Path safety violation: {0}")]
    PathSafety(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Invalid kit: {0}")]
    InvalidKit(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Watcher error: {0}")]
    Watcher(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

impl From<lmbrain_core::CoreError> for AppError {
    fn from(e: lmbrain_core::CoreError) -> Self {
        match e {
            lmbrain_core::CoreError::Path(p) => AppError::PathSafety(p.to_string()),
            lmbrain_core::CoreError::NotFound(s) => AppError::FileNotFound(s),
            lmbrain_core::CoreError::Frontmatter(f) => AppError::ParseError(f.to_string()),
            lmbrain_core::CoreError::Io(i) => AppError::Io(i.to_string()),
            other => AppError::ParseError(other.to_string()),
        }
    }
}

