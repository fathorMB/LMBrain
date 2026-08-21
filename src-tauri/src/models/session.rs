use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
pub enum AgentHost {
    Claude,
    Codex,
    Pi,
    Opencode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
pub enum ModelRoute {
    Native,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Running,
    Exited,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionStartRequest {
    pub host: AgentHost,
    pub route: ModelRoute,
    pub model: Option<String>,
    pub label: Option<String>,
    pub codex_bin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionInfo {
    pub id: String,
    pub label: String,
    pub host: AgentHost,
    pub route: ModelRoute,
    pub model: Option<String>,
    pub status: SessionStatus,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionOutputPayload {
    pub id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SessionExitPayload {
    pub id: String,
    pub code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct OllamaModel {
    pub name: String,
    pub cloud: bool,
    pub capabilities: Vec<String>,
}
