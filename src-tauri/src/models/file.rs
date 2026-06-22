use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileEventKind {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "modified")]
    Modified,
    #[serde(rename = "removed")]
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub is_clean: Option<bool>,
    pub current_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub path: String,
    pub frontmatter: std::collections::HashMap<String, serde_json::Value>,
    pub body: String,
    pub wikilinks: Vec<String>,
    pub diagnostics: Vec<String>,
}
