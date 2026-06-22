use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiNode {
    pub name: String,
    pub path: String,
    pub kind: WikiNodeKind,
    pub children: Vec<WikiNode>,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WikiNodeKind {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "folder")]
    Folder,
    #[serde(rename = "knowledge")]
    Knowledge,
    #[serde(rename = "decisions")]
    Decisions,
    #[serde(rename = "specs")]
    Specs,
    #[serde(rename = "tasks")]
    Tasks,
    #[serde(rename = "reviews")]
    Reviews,
    #[serde(rename = "handoffs")]
    Handoffs,
    #[serde(rename = "agents")]
    Agents,
    #[serde(rename = "mcp")]
    Mcp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub path: String,
    pub name: String,
    pub content_html: String,
    pub frontmatter: std::collections::HashMap<String, String>,
    pub wikilinks: Vec<String>,
    pub backlinks: Vec<String>,
    pub updated: Option<String>,
    pub word_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiTree {
    pub root: WikiNode,
}
