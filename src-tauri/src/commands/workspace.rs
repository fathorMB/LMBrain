use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::commands::filesystem::clean_path;
use crate::errors::AppError;
use crate::models::workspace::{
    DiagnosticSeverity, KitDiagnostic, KitHealth, KitMigrationStatus, WorkspaceInfo,
    WorkspaceRegistry, WorkspaceSummary,
};

/// Manages the workspace registry (recent/pinned workspaces) and kit validation.
pub struct WorkspaceService {
    registry: Mutex<WorkspaceRegistry>,
    config_path: Mutex<Option<PathBuf>>,
}

impl Default for WorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceService {
    pub fn new() -> Self {
        WorkspaceService {
            registry: Mutex::new(WorkspaceRegistry {
                recent: Vec::new(),
                pinned: Vec::new(),
            }),
            config_path: Mutex::new(None),
        }
    }

    pub fn initialize(&self, app_data_dir: &Path) -> Result<(), AppError> {
        let config_dir = app_data_dir.join("lmbrain");
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("workspaces.json");
        *self.config_path.lock().unwrap() = Some(config_path.clone());

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let registry: WorkspaceRegistry = serde_json::from_str(&content)?;
            *self.registry.lock().unwrap() = registry;
        }

        Ok(())
    }

    fn save(&self) -> Result<(), AppError> {
        let path = self
            .config_path
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| AppError::WorkspaceNotFound("Config path not initialized".into()))?;

        let registry = self.registry.lock().unwrap();
        let content = serde_json::to_string_pretty(&*registry)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn list_recent(&self) -> Vec<WorkspaceSummary> {
        self.registry.lock().unwrap().recent.clone()
    }

    pub fn add_recent(&self, summary: WorkspaceSummary) -> Result<(), AppError> {
        let mut registry = self.registry.lock().unwrap();
        // Remove existing entry with same path
        registry.recent.retain(|w| w.path != summary.path);
        // Add to front
        registry.recent.insert(0, summary);
        // Keep max 20 recent
        registry.recent.truncate(20);
        drop(registry);
        self.save()
    }

    pub fn remove_recent(&self, path: &str) -> Result<(), AppError> {
        let mut registry = self.registry.lock().unwrap();
        registry.recent.retain(|w| w.path != path);
        drop(registry);
        self.save()
    }

    /// Validate a workspace path and return its info.
    pub fn validate_workspace(
        &self,
        path: &str,
        bundled_kit_path: Option<&Path>,
    ) -> Result<WorkspaceInfo, AppError> {
        let root = Path::new(path);
        if !root.exists() {
            return Err(AppError::WorkspaceNotFound(format!(
                "Path does not exist: {}",
                path
            )));
        }
        if !root.is_dir() {
            return Err(AppError::WorkspaceNotFound(format!(
                "Path is not a directory: {}",
                path
            )));
        }

        let root_clean = clean_path(root);
        let lmbrain_dir = root_clean.join(".lmbrain");
        let mut diagnostics = Vec::new();
        let mut health = KitHealth::Ok;
        let bundled_kit_path_display = bundled_kit_path
            .map(display_path_for_prompt)
            .unwrap_or_default();

        // Read bundled version if path is provided
        let bundled_kit_version = if let Some(b_path) = bundled_kit_path {
            let bundled_version_path = b_path.join("VERSION");
            if bundled_version_path.exists() {
                std::fs::read_to_string(&bundled_version_path)
                    .ok()
                    .map(|v| v.trim().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Check .lmbrain directory exists
        if !lmbrain_dir.exists() || !lmbrain_dir.is_dir() {
            return Ok(WorkspaceInfo {
                path: root_clean.to_string_lossy().to_string(),
                name: root_clean
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".into()),
                kit_version: String::new(),
                health: KitHealth::None,
                diagnostics: vec![KitDiagnostic {
                    message: "No .lmbrain directory found".into(),
                    severity: DiagnosticSeverity::Error,
                    path: Some(".lmbrain".into()),
                }],
                branch: None,
                is_clean: None,
                spec_count: 0,
                task_count: 0,
                decision_count: 0,
                agent_count: 0,
                project_kit_version: String::new(),
                bundled_kit_version,
                bundled_kit_path: bundled_kit_path_display,
                kit_migration_status: KitMigrationStatus::UnknownProjectVersion,
            });
        }

        // Check VERSION file
        let version_path = lmbrain_dir.join("VERSION");
        let kit_version = if version_path.exists() {
            std::fs::read_to_string(&version_path)
                .ok()
                .map(|v| v.trim().to_string())
                .unwrap_or_default()
        } else {
            diagnostics.push(KitDiagnostic {
                message: "Missing VERSION file".into(),
                severity: DiagnosticSeverity::Warning,
                path: Some(".lmbrain/VERSION".into()),
            });
            health = KitHealth::Warn;
            String::new()
        };

        // Check STATUS.md
        let status_path = lmbrain_dir.join("STATUS.md");
        if !status_path.exists() {
            diagnostics.push(KitDiagnostic {
                message: "Missing STATUS.md".into(),
                severity: DiagnosticSeverity::Warning,
                path: Some(".lmbrain/STATUS.md".into()),
            });
            health = KitHealth::Warn;
        }

        // Check ROADMAP.md
        let roadmap_path = lmbrain_dir.join("ROADMAP.md");
        if !roadmap_path.exists() {
            diagnostics.push(KitDiagnostic {
                message: "Missing ROADMAP.md".into(),
                severity: DiagnosticSeverity::Info,
                path: Some(".lmbrain/ROADMAP.md".into()),
            });
        }

        // Count artifacts
        let spec_count = count_files_in_dirs(&lmbrain_dir.join("specs"), &["md"]);
        let task_count = count_files_in_dirs(&lmbrain_dir.join("tasks"), &["md"]);
        let decision_count = count_files_in_dirs(&lmbrain_dir.join("decisions"), &["md"]);
        let agent_count = count_files_in_dirs(&lmbrain_dir.join("agents"), &["md"]);

        let name = root_clean
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".into());

        // Determine migration status
        let kit_migration_status = if kit_version.is_empty() {
            KitMigrationStatus::UnknownProjectVersion
        } else if bundled_kit_version.is_empty() {
            KitMigrationStatus::UnknownBundledVersion
        } else {
            let p_semver = semver::Version::parse(&kit_version);
            let b_semver = semver::Version::parse(&bundled_kit_version);

            match (p_semver, b_semver) {
                (Ok(pv), Ok(bv)) => {
                    if pv == bv {
                        KitMigrationStatus::UpToDate
                    } else if pv < bv {
                        // Check if migration guidance exists for the target version in MIGRATIONS.md
                        let mut guidance_exists = false;
                        if let Some(b_path) = bundled_kit_path {
                            let migrations_path = b_path.join("MIGRATIONS.md");
                            if migrations_path.exists() {
                                if let Ok(content) = std::fs::read_to_string(&migrations_path) {
                                    let heading_prefix = format!("### {}", bundled_kit_version);
                                    if content
                                        .lines()
                                        .any(|line| line.trim().starts_with(&heading_prefix))
                                    {
                                        guidance_exists = true;
                                    }
                                }
                            }
                        }
                        if guidance_exists {
                            KitMigrationStatus::MigrationAvailable
                        } else {
                            KitMigrationStatus::MigrationGuidanceMissing
                        }
                    } else {
                        KitMigrationStatus::ProjectNewerThanApp
                    }
                }
                (Err(_), _) => KitMigrationStatus::UnknownProjectVersion,
                (_, Err(_)) => KitMigrationStatus::UnknownBundledVersion,
            }
        };

        Ok(WorkspaceInfo {
            path: root_clean.to_string_lossy().to_string(),
            name,
            kit_version: kit_version.clone(),
            health,
            diagnostics,
            branch: None,
            is_clean: None,
            spec_count,
            task_count,
            decision_count,
            agent_count,
            project_kit_version: kit_version,
            bundled_kit_version,
            bundled_kit_path: bundled_kit_path_display,
            kit_migration_status,
        })
    }

    /// Copy the bundled clean kit into a selected repository. This operation is
    /// intentionally refused when a `.lmbrain` directory already exists.
    pub fn initialize_kit(&self, root: &Path, template: &Path) -> Result<WorkspaceInfo, AppError> {
        let root = root.canonicalize().map_err(|_| {
            AppError::WorkspaceNotFound(format!("Path does not exist: {}", root.display()))
        })?;
        let root = clean_path(&root);
        if !root.is_dir() {
            return Err(AppError::WorkspaceNotFound(format!(
                "Path is not a directory: {}",
                root.display()
            )));
        }

        let destination = root.join(".lmbrain");
        if destination.exists() {
            return Err(AppError::InvalidKit(
                "Refusing to initialize because .lmbrain already exists".into(),
            ));
        }
        if !template.is_dir() {
            return Err(AppError::InvalidKit(format!(
                "Bundled kit is unavailable: {}",
                template.display()
            )));
        }

        let temporary = root.join(format!(".lmbrain.bootstrap-{}", uuid::Uuid::new_v4()));
        let result = copy_directory(template, &temporary)
            .and_then(|_| std::fs::rename(&temporary, &destination).map_err(AppError::from));
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&temporary);
        }
        result?;

        self.validate_workspace(&root.to_string_lossy(), Some(template))
    }
}

fn display_path_for_prompt(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let stripped = raw
        .strip_prefix(r"\\?\")
        .or_else(|| raw.strip_prefix(r"//?/"))
        .unwrap_or(&raw);
    stripped.replace('\\', "/")
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target)?;
        } else {
            return Err(AppError::InvalidKit(format!(
                "Bundled kit contains an unsupported entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn count_files_in_dirs(dir: &Path, extensions: &[&str]) -> usize {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                // Recurse into subdirectories (status dirs)
                if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                    for sub in sub_entries.flatten() {
                        if let Some(ext) = sub.path().extension() {
                            if extensions.contains(&ext.to_str().unwrap_or("")) {
                                count += 1;
                            }
                        }
                    }
                }
            } else if let Some(ext) = entry.path().extension() {
                if extensions.contains(&ext.to_str().unwrap_or("")) {
                    count += 1;
                }
            }
        }
    }
    count
}
