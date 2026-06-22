use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReviewStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "changes-requested")]
    ChangesRequested,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "superseded")]
    Superseded,
}

impl ReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewStatus::Pending => "pending",
            ReviewStatus::Accepted => "accepted",
            ReviewStatus::ChangesRequested => "changes-requested",
            ReviewStatus::Blocked => "blocked",
            ReviewStatus::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub id: String,
    pub text: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub title: String,
    pub status: ReviewStatus,
    pub spec_id: Option<String>,
    pub reviewer: Option<String>,
    pub findings: Vec<ReviewFinding>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
}
