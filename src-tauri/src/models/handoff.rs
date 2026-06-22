use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HandoffStatus {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "consumed")]
    Consumed,
    #[serde(rename = "superseded")]
    Superseded,
    #[serde(rename = "archived")]
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
