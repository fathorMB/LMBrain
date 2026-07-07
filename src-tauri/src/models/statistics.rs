use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCount {
    pub status: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFamilyStats {
    pub family: String,
    pub label: String,
    pub total: usize,
    pub statuses: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFlowStats {
    pub total_specs: usize,
    pub done_specs: usize,
    pub open_specs: usize,
    pub done_ratio: f64,
    pub by_status: Vec<StatusCount>,
    pub by_priority: Vec<StatusCount>,
    pub by_area: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDimensionStat {
    pub value: String,
    pub reviewed_specs: usize,
    pub specs_with_changes_requested: usize,
    pub change_request_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTrendPoint {
    pub period: String,
    pub total_reviews: usize,
    pub accepted_reviews: usize,
    pub changes_requested_reviews: usize,
    pub reviewed_specs: usize,
    pub specs_with_changes_requested: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQualityStats {
    pub total_reviews: usize,
    pub reviewed_specs: usize,
    pub accepted_reviews: usize,
    pub changes_requested_reviews: usize,
    pub blocked_reviews: usize,
    pub superseded_reviews: usize,
    pub reviews_without_spec: usize,
    pub reviews_without_created: usize,
    pub specs_with_changes_requested: usize,
    pub specs_with_multiple_changes_requested: usize,
    pub change_request_rate: f64,
    pub first_pass_eligible_specs: usize,
    pub first_pass_accepted_specs: usize,
    pub first_pass_acceptance_rate: f64,
    pub average_reviews_per_reviewed_spec: f64,
    pub by_area: Vec<ReviewDimensionStat>,
    pub by_agent: Vec<ReviewDimensionStat>,
    pub trend: Vec<ReviewTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticStats {
    pub total: usize,
    pub warnings: usize,
    pub errors: usize,
    pub by_family: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub artifact_families: Vec<ArtifactFamilyStats>,
    pub spec_flow: SpecFlowStats,
    pub review_quality: ReviewQualityStats,
    pub diagnostics: DiagnosticStats,
}
