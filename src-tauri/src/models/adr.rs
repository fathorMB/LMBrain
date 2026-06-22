use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdrStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "superseded")]
    Superseded,
    #[serde(rename = "deprecated")]
    Deprecated,
}

impl AdrStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdrStatus::Proposed => "proposed",
            AdrStatus::Accepted => "accepted",
            AdrStatus::Rejected => "rejected",
            AdrStatus::Superseded => "superseded",
            AdrStatus::Deprecated => "deprecated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub malformed: Option<bool>,
}
