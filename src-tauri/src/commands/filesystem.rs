use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use lmbrain_core::path::{PathError, PathGuard as CorePathGuard};
pub use lmbrain_core::path::clean_path;

use crate::errors::AppError;
use crate::models::file::{DirEntry, FileContent};

/// Thread-safe path safety guard that ensures all file operations
/// stay within an approved workspace root.
pub struct PathGuard {
    approved_guard: Mutex<Option<CorePathGuard>>,
}

impl Default for PathGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PathGuard {
    pub fn new() -> Self {
        PathGuard {
            approved_guard: Mutex::new(None),
        }
    }

    pub fn set_root(&self, root: &Path) {
        let guard = CorePathGuard::new(root).ok();
        *self.lock_guard() = guard;
    }

    pub fn get_root(&self) -> Option<PathBuf> {
        self.lock_guard().as_ref().map(|g| g.root().to_path_buf())
    }

    /// Resolve a path relative to the approved root and validate it stays inside.
    pub fn resolve(&self, path: &str) -> Result<PathBuf, AppError> {
        let guard = self
            .lock_guard()
            .clone()
            .ok_or_else(|| AppError::PathSafety("No workspace root is set".into()))?;

        guard.resolve_existing(path).map_err(|error| match error {
            PathError::OutsideRoot(_) => AppError::PathSafety(format!(
                "Path traversal detected: {} is outside the workspace root",
                path
            )),
            _ => AppError::PathSafety(format!("Path does not exist: {}", path)),
        })
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
            .and_then(format_elapsed_time)
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
                .and_then(format_elapsed_time);

            entries.push(DirEntry {
                name,
                path,
                is_dir: ft.is_dir(),
                size: entry.metadata().ok().map(|m| m.len()),
                modified,
            });
        }

        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

        Ok(entries)
    }
    fn lock_guard(&self) -> std::sync::MutexGuard<'_, Option<CorePathGuard>> {
        self.approved_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn format_elapsed_time(modified: SystemTime) -> Option<String> {
    let elapsed = SystemTime::now().duration_since(modified).ok()?;
    let total_minutes = elapsed.as_secs() / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let minutes = total_minutes % 60;

    Some(format!("{days}d {hours}h {minutes}m ago"))
}
