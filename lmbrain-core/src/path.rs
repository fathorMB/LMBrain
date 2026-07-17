use std::path::{Component, Path, PathBuf};
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

/// Boundary errors for guarded artifact reads. Every message echoes only the
/// caller-supplied path, never a resolved host filesystem path.
#[derive(Debug, Error)]
pub enum ArtifactReadError {
    #[error("invalid artifact path (must be relative, without parent traversal): {0}")]
    InvalidPath(String),
    #[error("artifact not found in workspace: {0}")]
    NotFound(String),
    #[error("artifact path resolves outside the workspace: {0}")]
    OutsideWorkspace(String),
    #[error("artifact is not a readable file: {0}")]
    NotAFile(String),
    #[error("workspace root is unavailable")]
    WorkspaceUnavailable,
}

/// Read a workspace artifact with strict boundary enforcement.
///
/// The caller-supplied path is checked lexically before any filesystem access
/// (so locations outside the workspace cannot be probed for existence), then
/// canonicalized and prefix-checked so symlink/junction escapes are rejected.
pub fn read_artifact(root: impl AsRef<Path>, relative: &str) -> Result<String, ArtifactReadError> {
    let display = relative.to_string();
    let trimmed = relative.trim();
    if trimmed.is_empty() {
        return Err(ArtifactReadError::InvalidPath(display));
    }
    // Both separators act as segment boundaries regardless of platform so that
    // `..\` cannot slip through a Unix build or `../` through data written on
    // another platform.
    if trimmed
        .split(['/', '\\'])
        .any(|segment| segment == "..")
    {
        return Err(ArtifactReadError::InvalidPath(display));
    }
    let candidate = Path::new(trimmed);
    if candidate.components().any(|component| {
        matches!(
            component,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        // Keep the "no absolute paths in errors, ever" invariant simple to
        // audit: absolute/rooted inputs are not echoed back even though the
        // caller already knows them.
        return Err(ArtifactReadError::InvalidPath(
            "<absolute or rooted path redacted>".into(),
        ));
    }

    let guard = PathGuard::new(root).map_err(|_| ArtifactReadError::WorkspaceUnavailable)?;
    let resolved = guard.resolve_existing(candidate).map_err(|error| match error {
        PathError::OutsideRoot(_) => ArtifactReadError::OutsideWorkspace(display.clone()),
        _ => ArtifactReadError::NotFound(display.clone()),
    })?;
    if !resolved.is_file() {
        return Err(ArtifactReadError::NotAFile(display));
    }
    std::fs::read_to_string(&resolved).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => ArtifactReadError::NotFound(display),
        _ => ArtifactReadError::NotAFile(display),
    })
}

/// Strip Windows verbatim prefixes before prefix comparison.
pub fn clean_path(path: &Path) -> PathBuf {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", rest))
    } else if let Some(rest) = value.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path.to_path_buf()
    }
}

#[derive(Debug, Clone)]
pub struct PathGuard {
    root: PathBuf,
}
impl PathGuard {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, PathError> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|_| PathError::MissingRoot(root.as_ref().display().to_string()))?;
        Ok(Self {
            root: clean_path(&root),
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn resolve_existing(&self, path: impl AsRef<Path>) -> Result<PathBuf, PathError> {
        let path = path.as_ref();
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let canonical = absolute
            .canonicalize()
            .map_err(|_| PathError::Missing(path.display().to_string()))?;
        let clean = clean_path(&canonical);
        if !clean.starts_with(&self.root) {
            return Err(PathError::OutsideRoot(path.display().to_string()));
        }
        Ok(clean)
    }
    pub fn resolve_new(&self, path: impl AsRef<Path>) -> Result<PathBuf, PathError> {
        let path = path.as_ref();
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let parent = absolute
            .parent()
            .ok_or_else(|| PathError::OutsideRoot(path.display().to_string()))?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|_| PathError::Missing(parent.display().to_string()))?;
        let candidate = clean_path(&canonical_parent).join(
            absolute
                .file_name()
                .ok_or_else(|| PathError::OutsideRoot(path.display().to_string()))?,
        );
        if !candidate.starts_with(&self.root) {
            return Err(PathError::OutsideRoot(path.display().to_string()));
        }
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::{read_artifact, ArtifactReadError};
    use std::fs;

    fn workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        fs::write(
            dir.path().join(".lmbrain/specs/backlog/SPEC-001-demo.md"),
            "---\nid: SPEC-001\n---\n\n# Demo\n",
        )
        .unwrap();
        dir
    }

    fn assert_sanitized(error: &ArtifactReadError, root: &std::path::Path) {
        let message = error.to_string();
        let canonical = root.canonicalize().unwrap();
        assert!(
            !message.contains(&super::clean_path(&canonical).display().to_string()),
            "error message leaks the workspace host path: {message}"
        );
    }

    #[test]
    fn reads_valid_nested_artifact() {
        let dir = workspace();
        let source =
            read_artifact(dir.path(), ".lmbrain/specs/backlog/SPEC-001-demo.md").unwrap();
        assert!(source.contains("SPEC-001"));
    }

    #[cfg(windows)]
    #[test]
    fn reads_valid_artifact_with_backslash_separators() {
        let dir = workspace();
        let source =
            read_artifact(dir.path(), r".lmbrain\specs\backlog\SPEC-001-demo.md").unwrap();
        assert!(source.contains("SPEC-001"));
    }

    #[test]
    fn rejects_parent_traversal_with_either_separator() {
        let dir = workspace();
        for path in ["../outside.md", r"..\outside.md", ".lmbrain/../../outside.md"] {
            let error = read_artifact(dir.path(), path).unwrap_err();
            assert!(
                matches!(error, ArtifactReadError::InvalidPath(_)),
                "{path} was not rejected lexically: {error}"
            );
            assert_sanitized(&error, dir.path());
        }
    }

    #[test]
    fn rejects_absolute_and_rooted_paths() {
        let dir = workspace();
        let mut cases = vec!["/etc/passwd".to_string()];
        if cfg!(windows) {
            cases.push(r"C:\Windows\win.ini".into());
            cases.push(r"\\server\share\file.md".into());
            cases.push("C:relative.md".into());
        }
        // A rooted path inside the workspace is still rejected: the contract is
        // workspace-relative only.
        cases.push(
            dir.path()
                .join(".lmbrain/specs/backlog/SPEC-001-demo.md")
                .display()
                .to_string(),
        );
        for path in cases {
            let error = read_artifact(dir.path(), &path).unwrap_err();
            assert!(
                matches!(error, ArtifactReadError::InvalidPath(_)),
                "{path} was not rejected lexically: {error}"
            );
            assert_sanitized(&error, dir.path());
        }
    }

    #[test]
    fn rejects_empty_path() {
        let dir = workspace();
        assert!(matches!(
            read_artifact(dir.path(), "   ").unwrap_err(),
            ArtifactReadError::InvalidPath(_)
        ));
    }

    #[test]
    fn missing_artifact_reports_not_found_without_host_paths() {
        let dir = workspace();
        let error = read_artifact(dir.path(), ".lmbrain/specs/backlog/absent.md").unwrap_err();
        assert!(matches!(error, ArtifactReadError::NotFound(_)));
        assert_sanitized(&error, dir.path());
    }

    #[test]
    fn directory_is_not_a_readable_artifact() {
        let dir = workspace();
        let error = read_artifact(dir.path(), ".lmbrain/specs").unwrap_err();
        assert!(matches!(error, ArtifactReadError::NotAFile(_)));
        assert_sanitized(&error, dir.path());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        let dir = workspace();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.md"), "secret").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("secret.md"),
            dir.path().join(".lmbrain/link.md"),
        )
        .unwrap();
        let error = read_artifact(dir.path(), ".lmbrain/link.md").unwrap_err();
        assert!(matches!(error, ArtifactReadError::OutsideWorkspace(_)));
        assert_sanitized(&error, dir.path());
    }

    #[cfg(windows)]
    #[test]
    fn rejects_junction_escape() {
        let dir = workspace();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.md"), "secret").unwrap();
        // Junctions do not require elevation; skip only if mklink itself fails.
        let link = dir.path().join(".lmbrain/jct");
        let created = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.display().to_string(),
                &outside.path().display().to_string(),
            ])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !created {
            eprintln!("skipping junction escape test: mklink /J unavailable");
            return;
        }
        let error = read_artifact(dir.path(), ".lmbrain/jct/secret.md").unwrap_err();
        assert!(
            matches!(error, ArtifactReadError::OutsideWorkspace(_)),
            "junction escape not rejected: {error}"
        );
        assert_sanitized(&error, dir.path());
    }
}
