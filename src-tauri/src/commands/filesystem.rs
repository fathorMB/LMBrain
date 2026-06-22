use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::errors::AppError;
use crate::models::file::{DirEntry, FileContent};

/// Thread-safe path safety guard that ensures all file operations
/// stay within an approved workspace root.
pub struct PathGuard {
    approved_root: Mutex<Option<PathBuf>>,
}

impl Default for PathGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PathGuard {
    pub fn new() -> Self {
        PathGuard {
            approved_root: Mutex::new(None),
        }
    }

    pub fn set_root(&self, root: &Path) {
        let canonical = root
            .canonicalize()
            .unwrap_or_else(|_| root.to_path_buf());
        *self.approved_root.lock().unwrap() = Some(canonical);
    }

    pub fn clear_root(&self) {
        *self.approved_root.lock().unwrap() = None;
    }

    pub fn get_root(&self) -> Option<PathBuf> {
        self.approved_root.lock().unwrap().clone()
    }

    /// Resolve a path relative to the approved root and validate it stays inside.
    pub fn resolve(&self, path: &str) -> Result<PathBuf, AppError> {
        let root = self
            .approved_root
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| AppError::PathSafety("No workspace root is set".into()))?;

        let target = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            root.join(path)
        };

        let canonical = target
            .canonicalize()
            .map_err(|_| AppError::PathSafety(format!("Path does not exist: {}", path)))?;

        if !canonical.starts_with(&root) {
            return Err(AppError::PathSafety(format!(
                "Path traversal detected: {} is outside the workspace root",
                path
            )));
        }

        Ok(canonical)
    }

    /// Read a file, validating it's within the approved root.
    pub fn read_file(&self, path: &str) -> Result<FileContent, AppError> {
        let resolved = self.resolve(path)?;
        let metadata = std::fs::metadata(&resolved)?;

        if !metadata.is_file() {
            return Err(AppError::FileNotFound(format!("Not a file: {}", path)));
        }

        let content = std::fs::read_to_string(&resolved)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| {
                    let secs = d.as_secs();
                    // Format as ISO-like string
                    let days = secs / 86400;
                    let hours = (secs % 86400) / 3600;
                    let mins = (secs % 3600) / 60;
                    format!("{}d {}h {}m ago", days, hours, mins)
                })
            })
            .unwrap_or_else(|| "unknown".into());

        Ok(FileContent {
            path: resolved.to_string_lossy().to_string(),
            content,
            size: metadata.len(),
            modified,
        })
    }

    /// List a directory, validating it's within the approved root.
    pub fn list_directory(&self, path: &str) -> Result<Vec<DirEntry>, AppError> {
        let resolved = self.resolve(path)?;

        if !resolved.is_dir() {
            return Err(AppError::FileNotFound(format!("Not a directory: {}", path)));
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&resolved)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format!("{}s", d.as_secs()));

            entries.push(DirEntry {
                name,
                path,
                is_dir: ft.is_dir(),
                size: entry.metadata().ok().map(|m| m.len()),
                modified,
            });
        }

        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(entries)
    }

    /// Check if a path exists within the approved root.
    pub fn exists(&self, path: &str) -> Result<bool, AppError> {
        let resolved = self.resolve(path)?;
        Ok(resolved.exists())
    }
}
