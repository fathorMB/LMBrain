use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::AdrStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Adr {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub decision_date: Option<String>,
    pub decider: Option<String>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    /// Decisions this one retires, and the one that retired it (issue #48).
    /// Both are optional in the artifact: records predating the governed
    /// supersession parse to empty rather than failing.
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
    pub malformed: Option<bool>,
}
