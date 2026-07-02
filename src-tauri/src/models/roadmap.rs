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

// ─── V3 milestone intelligence ─────────────────────────────────────

/// Derived spec summary for milestone overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReviewSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub spec_id: Option<String>,
    pub path: Option<String>,
}

/// Derived ADR summary for milestone overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneAdrSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub path: Option<String>,
}

/// Per-milestone derived intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneOverview {
    pub title: String,
    pub milestones: Vec<MilestoneDetail>,
    pub unmapped_specs: Vec<MilestoneSpecSummary>,
    pub warnings: Vec<String>,
}
