use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use lmbrain_core::ReviewStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ReviewFinding {
    pub id: String,
    pub text: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Review {
    pub id: String,
    pub title: String,
    pub status: ReviewStatus,
    pub spec_id: Option<String>,
    pub reviewer: Option<String>,
    pub implementation_agent: Option<String>,
    pub finding_categories: Vec<String>,
    pub findings: Vec<ReviewFinding>,
    pub events: Vec<lmbrain_core::ReviewLifecycleEvent>,
    pub lifecycle: lmbrain_core::ReviewLifecycleAnalysis,
    pub lifecycle_warnings: Vec<String>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub malformed: Option<bool>,
}
