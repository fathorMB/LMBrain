use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("workspace root does not exist: {0}")]
    MissingRoot(String),
    #[error("path is outside the workspace root: {0}")]
    OutsideRoot(String),
    #[error("path does not exist: {0}")]
    Missing(String),
}

/// Strip Windows verbatim prefixes before prefix comparison.
pub fn clean_path(path: &Path) -> PathBuf {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") { PathBuf::from(format!(r"\\{}", rest)) }
    else if let Some(rest) = value.strip_prefix(r"\\?\") { PathBuf::from(rest) }
    else { path.to_path_buf() }
}

#[derive(Debug, Clone)]
pub struct PathGuard { root: PathBuf }
impl PathGuard {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, PathError> {
        let root = root.as_ref().canonicalize().map_err(|_| PathError::MissingRoot(root.as_ref().display().to_string()))?;
        Ok(Self { root: clean_path(&root) })
    }
    pub fn root(&self) -> &Path { &self.root }
    pub fn resolve_existing(&self, path: impl AsRef<Path>) -> Result<PathBuf, PathError> {
        let path = path.as_ref();
        let absolute = if path.is_absolute() { path.to_path_buf() } else { self.root.join(path) };
        let canonical = absolute.canonicalize().map_err(|_| PathError::Missing(path.display().to_string()))?;
        let clean = clean_path(&canonical);
        if !clean.starts_with(&self.root) { return Err(PathError::OutsideRoot(path.display().to_string())); }
        Ok(clean)
    }
    pub fn resolve_new(&self, path: impl AsRef<Path>) -> Result<PathBuf, PathError> {
        let path = path.as_ref();
        let absolute = if path.is_absolute() { path.to_path_buf() } else { self.root.join(path) };
        let parent = absolute.parent().ok_or_else(|| PathError::OutsideRoot(path.display().to_string()))?;
        let canonical_parent = parent.canonicalize().map_err(|_| PathError::Missing(parent.display().to_string()))?;
        let candidate = clean_path(&canonical_parent).join(absolute.file_name().ok_or_else(|| PathError::OutsideRoot(path.display().to_string()))?);
        if !candidate.starts_with(&self.root) { return Err(PathError::OutsideRoot(path.display().to_string())); }
        Ok(candidate)
    }
}
