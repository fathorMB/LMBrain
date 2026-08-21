use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::SkillStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
