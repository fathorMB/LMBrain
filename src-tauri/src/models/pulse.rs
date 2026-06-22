use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub count: usize,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub spec_id: Option<String>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub action: String,
    pub path: String,
    pub description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseData {
    pub focus: Option<String>,
    pub milestone: Option<String>,
    pub milestone_progress: Option<f64>,
    pub milestone_due: Option<String>,
    pub metrics: Vec<MetricCard>,
    pub actions: Vec<ActionItem>,
    pub blockers: Vec<ActionItem>,
    pub recent_activity: Vec<RecentActivity>,
    pub ready_handoffs: Vec<super::handoff::Handoff>,
    pub active_handoff: Option<super::handoff::Handoff>,
}
