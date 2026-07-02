use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KitHealth {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitDiagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticSeverity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub path: String,
    pub name: String,
    pub health: KitHealth,
    pub last_opened: String,
    pub branch: Option<String>,
    pub is_clean: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KitMigrationStatus {
    #[serde(rename = "up-to-date")]
    UpToDate,
    #[serde(rename = "migration-available")]
    MigrationAvailable,
    #[serde(rename = "project-newer-than-app")]
    ProjectNewerThanApp,
    #[serde(rename = "unknown-project-version")]
    UnknownProjectVersion,
    #[serde(rename = "unknown-bundled-version")]
    UnknownBundledVersion,
    #[serde(rename = "migration-guidance-missing")]
    MigrationGuidanceMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: String,
    pub name: String,
    pub kit_version: String,
    pub health: KitHealth,
    pub diagnostics: Vec<KitDiagnostic>,
    pub branch: Option<String>,
    pub is_clean: Option<bool>,
    pub spec_count: usize,
    pub task_count: usize,
    pub decision_count: usize,
    pub agent_count: usize,
    pub project_kit_version: String,
    pub bundled_kit_version: String,
    pub bundled_kit_path: String,
    pub kit_migration_status: KitMigrationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRegistry {
    pub recent: Vec<WorkspaceSummary>,
    pub pinned: Vec<String>,
}
