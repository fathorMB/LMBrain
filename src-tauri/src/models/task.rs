use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "planned")]
    Planned,
    #[serde(rename = "in-progress")]
    InProgress,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl TaskStatus {
    pub fn all() -> &'static [TaskStatus] {
        &[
            TaskStatus::Backlog,
            TaskStatus::Planned,
            TaskStatus::InProgress,
            TaskStatus::Review,
            TaskStatus::Done,
            TaskStatus::Blocked,
            TaskStatus::Cancelled,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Backlog => "backlog",
            TaskStatus::Planned => "planned",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Review => "review",
            TaskStatus::Done => "done",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCriteria {
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskActivity {
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub spec: Option<String>,
    pub dependencies: Vec<String>,
    pub criteria: Vec<TaskCriteria>,
    pub activity: Vec<TaskActivity>,
    pub block_reason: Option<String>,
    pub body: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
}
