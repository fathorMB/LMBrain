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
    pub mnemonic_name: Option<String>,
    pub status: AgentStatus,
    pub role: Option<String>,
    pub activation: Option<String>,
    pub can_implement: Option<bool>,
    pub can_review: Option<bool>,
    // V3 specialization metadata (optional, backward-compatible)
    pub domains: Option<Vec<String>>,
    pub primary_files: Option<Vec<String>>,
    pub review_focus: Option<Vec<String>>,
    pub context_pack: Option<String>,
    pub constraints: Option<Vec<String>>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentProposalStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProposal {
    pub id: String,
    pub title: String,
    pub status: AgentProposalStatus,
    pub proposed_mnemonic_name: Option<String>,
    // V3: proposal type — "new-profile" (default) or "improvement"
    pub proposal_type: Option<String>,
    // V3: target profile ID for improvement proposals
    pub target_profile: Option<String>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}
