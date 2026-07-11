use serde::{Deserialize, Serialize};

use super::session::AgentHost;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessProbeState {
    Installed,
    Missing,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStatus {
    pub host: AgentHost,
    pub label: String,
    pub state: HarnessProbeState,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub detail: Option<String>,
    pub probed_at: String,
    pub install_url: String,
    pub install_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessUpdateRequest {
    pub host: AgentHost,
    pub codex_bin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessUpdateResult {
    pub host: AgentHost,
    pub success: bool,
    pub already_current: bool,
    pub before: HarnessStatus,
    pub after: HarnessStatus,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
}
