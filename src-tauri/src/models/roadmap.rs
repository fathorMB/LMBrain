use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub status: String,
    pub outcome: String,
    pub specs: Vec<String>,
    pub decisions: Vec<String>,
    pub risks: Vec<String>,
    pub depends_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    pub title: String,
    pub milestones: Vec<Milestone>,
}
