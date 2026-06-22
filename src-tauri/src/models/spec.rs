use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpecStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "in-progress")]
    InProgress,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "changes-requested")]
    ChangesRequested,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "archived")]
    Archived,
}

impl SpecStatus {
    pub fn all() -> &'static [SpecStatus] {
        &[
            SpecStatus::Proposed,
            SpecStatus::Ready,
            SpecStatus::InProgress,
            SpecStatus::Review,
            SpecStatus::Accepted,
            SpecStatus::ChangesRequested,
            SpecStatus::Rejected,
            SpecStatus::Archived,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SpecStatus::Proposed => "proposed",
            SpecStatus::Ready => "ready",
            SpecStatus::InProgress => "in-progress",
            SpecStatus::Review => "review",
            SpecStatus::Accepted => "accepted",
            SpecStatus::ChangesRequested => "changes-requested",
            SpecStatus::Rejected => "rejected",
            SpecStatus::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub status: SpecStatus,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub recommended_agent: Option<String>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub related_tasks: Vec<String>,
    pub related_decisions: Vec<String>,
    pub malformed: Option<bool>,
}
