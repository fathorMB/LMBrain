use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignMockup {
    pub id: String,
    pub name: String,
    pub path: String,
    pub entry_path: String,
    pub kind: DesignMockupKind,
    pub modified: Option<String>,
    pub size: u64,
    pub summary: Option<String>,
    pub manifest_title: Option<String>,
    pub manifest_description: Option<String>,
    pub has_manifest: bool,
    pub has_readme: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesignMockupKind {
    #[serde(rename = "package")]
    Package,
    #[serde(rename = "html-file")]
    HtmlFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignMockupHtml {
    pub path: String,
    pub content: String,
}
