use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::{Roadmap, RoadmapMilestone as Milestone};

// ─── V3 milestone intelligence ─────────────────────────────────────

/// Derived spec summary for milestone overview.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MilestoneSpecSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub recommended_agent: Option<String>,
    pub path: Option<String>,
}

/// Derived review summary for milestone overview.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MilestoneReviewSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub spec_id: Option<String>,
    pub path: Option<String>,
}

/// Derived ADR summary for milestone overview.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MilestoneAdrSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub path: Option<String>,
}

/// Per-milestone derived intelligence.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MilestoneDetail {
    pub id: String,
    pub title: String,
    pub status: String,
    pub outcome: String,
    pub depends_on: Option<String>,
    pub risks: Vec<String>,
    pub spec_count: usize,
    pub spec_counts_by_status: std::collections::HashMap<String, usize>,
    pub specs: Vec<MilestoneSpecSummary>,
    pub reviews: Vec<MilestoneReviewSummary>,
    pub decisions: Vec<MilestoneAdrSummary>,
    pub unresolved_refs: Vec<String>,
    pub next_action: Option<String>,
    pub progress_pct: f64,
}

/// Full milestone overview returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct MilestoneOverview {
    pub title: String,
    pub milestones: Vec<MilestoneDetail>,
    pub unmapped_specs: Vec<MilestoneSpecSummary>,
    pub warnings: Vec<String>,
}
