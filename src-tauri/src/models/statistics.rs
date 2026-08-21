use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct StatusCount {
    pub status: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ArtifactFamilyStats {
    pub family: String,
    pub label: String,
    pub total: usize,
    pub statuses: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SpecFlowStats {
    pub total_specs: usize,
    pub done_specs: usize,
    pub open_specs: usize,
    pub done_ratio: f64,
    pub by_status: Vec<StatusCount>,
    pub by_priority: Vec<StatusCount>,
    pub by_area: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ReviewDimensionStat {
    pub value: String,
    pub reviewed_specs: usize,
    pub specs_with_changes_requested: usize,
    pub change_request_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ReviewTrendPoint {
    pub period: String,
    pub total_reviews: usize,
    pub accepted_reviews: usize,
    pub changes_requested_reviews: usize,
    pub reviewed_specs: usize,
    pub specs_with_changes_requested: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ReviewCycleRankingEntry {
    pub spec_id: String,
    pub title: String,
    pub path: String,
    pub status: String,
    pub review_count: usize,
    pub review_passes: usize,
    pub remediation_cycles: usize,
    pub history_source: String,
    pub confidence: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ReviewQualityStats {
    pub total_reviews: usize,
    pub total_review_passes: usize,
    pub remediation_cycles: usize,
    pub escalation_count: usize,
    pub takeover_count: usize,
    pub lifecycle_known_reviews: usize,
    pub lifecycle_coverage: f64,
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
    pub review_cycle_ranking: Vec<ReviewCycleRankingEntry>,
    pub review_cycle_ranking_coverage: usize,
    pub by_area: Vec<ReviewDimensionStat>,
    pub by_agent: Vec<ReviewDimensionStat>,
    pub trend: Vec<ReviewTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DiagnosticStats {
    pub total: usize,
    pub warnings: usize,
    pub errors: usize,
    pub by_family: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ProjectStatistics {
    pub artifact_families: Vec<ArtifactFamilyStats>,
    pub spec_flow: SpecFlowStats,
    pub review_quality: ReviewQualityStats,
    pub diagnostics: DiagnosticStats,
}
