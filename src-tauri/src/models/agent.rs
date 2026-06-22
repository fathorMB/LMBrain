use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "retired")]
    Retired,
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Proposed => "proposed",
            AgentStatus::Active => "active",
            AgentStatus::Inactive => "inactive",
            AgentStatus::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub title: String,
    pub status: AgentStatus,
    pub role: Option<String>,
    pub activation: Option<String>,
    pub can_implement: Option<bool>,
    pub can_review: Option<bool>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}
