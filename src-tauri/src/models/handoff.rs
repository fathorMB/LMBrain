use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::HandoffStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Handoff {
    pub id: String,
    pub title: String,
    pub status: HandoffStatus,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}
