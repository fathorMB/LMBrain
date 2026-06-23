use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpecStatus {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "working")]
    Working,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "discarded")]
    Discarded,
}

impl SpecStatus {
    pub fn all() -> &'static [SpecStatus] {
        &[
            SpecStatus::Backlog,
            SpecStatus::Ready,
            SpecStatus::Working,
            SpecStatus::Review,
            SpecStatus::Done,
            SpecStatus::Discarded,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SpecStatus::Backlog => "backlog",
            SpecStatus::Ready => "ready",
            SpecStatus::Working => "working",
            SpecStatus::Review => "review",
            SpecStatus::Done => "done",
            SpecStatus::Discarded => "discarded",
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
