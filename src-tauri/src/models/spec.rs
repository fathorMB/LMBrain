use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SpecParkingEvent {
    pub timestamp: String,
    pub actor: String,
    pub reason: String,
    pub revisit_condition: Option<String>,
}

pub use lmbrain_core::SpecStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub status: SpecStatus,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub recommended_agent: Option<String>,
    /// Lead-owned implementation estimate (issue #64). Optional in the parser:
    /// it is mandatory only at the `ready` transition, so legacy specs load.
    pub capability_tier: Option<String>,
    pub thinking_level: Option<String>,
    pub depends_on: Vec<String>,
    pub parking_events: Vec<SpecParkingEvent>,
    pub skills: Vec<String>,
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
