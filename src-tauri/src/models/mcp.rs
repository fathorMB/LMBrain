use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum McpStatus {
    #[serde(rename = "specified")]
    Specified,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "deprecated")]
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum McpProposalStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "implemented")]
    Implemented,
    #[serde(rename = "blocked")]
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRecord {
    pub id: String,
    pub title: String,
    pub status: McpStatus,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProposal {
    pub id: String,
    pub title: String,
    pub status: McpProposalStatus,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
}
