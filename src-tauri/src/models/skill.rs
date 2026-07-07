use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillStatus {
    #[serde(rename = "proposed")]
    Proposed,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "retired")]
    Retired,
}

impl SkillStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillStatus::Proposed => "proposed",
            SkillStatus::Active => "active",
            SkillStatus::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub title: String,
    pub status: SkillStatus,
    pub scope: Option<String>,
    pub kind: Option<String>,
    pub risk: Option<String>,
    pub applies_to: Vec<String>,
    pub domains: Vec<String>,
    pub commands: Vec<String>,
    pub requires_operator_approval: Option<bool>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}
