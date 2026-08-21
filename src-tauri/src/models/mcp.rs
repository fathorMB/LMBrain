use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::{McpProposalStatus, McpStatus};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
    pub malformed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
    pub malformed: Option<bool>,
}
