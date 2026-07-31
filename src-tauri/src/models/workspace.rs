use serde::{Deserialize, Serialize};

use crate::models::{
    adr::Adr,
    agent::{AgentProfile, AgentProposal},
    handoff::Handoff,
    mcp::{McpProposal, McpRecord},
    pulse::PulseData,
    review::Review,
    skill::Skill,
    spec::Spec,
    statistics::ProjectStatistics,
};

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
    pub id: String,
    pub code: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub artifact_id: Option<String>,
    pub path: Option<String>,
    pub next_action: String,
    pub fixability: String,
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
    pub finding_count: usize,
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

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSnapshot {
    pub pulse_data: PulseData,
    pub specs: Vec<Spec>,
    pub reviews: Vec<Review>,
    pub findings: Vec<lmbrain_core::Finding>,
    pub adrs: Vec<Adr>,
    pub agents: Vec<AgentProfile>,
    pub agent_proposals: Vec<AgentProposal>,
    pub mcp_records: Vec<McpRecord>,
    pub mcp_proposals: Vec<McpProposal>,
    pub skills: Vec<Skill>,
    pub handoffs: Vec<Handoff>,
    pub diagnostics: Vec<KitDiagnostic>,
    pub project_statistics: ProjectStatistics,
}
