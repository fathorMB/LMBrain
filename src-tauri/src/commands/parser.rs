use std::collections::HashMap;

use regex::Regex;
use serde_json::Value;

use crate::models::file::ParsedDocument;

/// Result of parsing frontmatter from a markdown file.
pub struct FrontmatterResult {
    pub frontmatter: HashMap<String, Value>,
    pub body: String,
    pub wikilinks: Vec<String>,
    pub diagnostics: Vec<String>,
    pub malformed: bool,
}

/// Parse frontmatter and Markdown body from a string.
/// Returns frontmatter map, body text, wikilinks, and any parse diagnostics.
pub fn parse_frontmatter(content: &str) -> FrontmatterResult {
    let content = content.trim_start();
    let mut diagnostics = Vec::new();

    if !content.starts_with("---") {
        let body = content.to_string();
        let wikilinks = extract_wikilinks(&body);
        return FrontmatterResult {
            frontmatter: HashMap::new(),
            body,
            wikilinks,
            diagnostics,
            malformed: false,
        };
    }

    match lmbrain_core::frontmatter::Document::parse(content) {
        Ok(document) => {
            let body = document.body.clone();
            let wikilinks = extract_wikilinks(&body);
            FrontmatterResult {
                frontmatter: document.fields(),
                body,
                wikilinks,
                diagnostics,
                malformed: false,
            }
        }
        Err(error) => {
            diagnostics.push(format!("Malformed frontmatter: {error}"));
            FrontmatterResult {
                frontmatter: HashMap::new(),
                body: content.to_string(),
                wikilinks: Vec::new(),
                diagnostics,
                malformed: true,
            }
        }
    }
}

/// Extract [[wikilinks]] from markdown body.
pub fn extract_wikilinks(body: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.captures_iter(body)
        .map(|capture| capture[1].trim().to_string())
        .collect()
}

/// Parse a markdown file and return structured content.
pub fn parse_markdown_file(path: &str, content: &str) -> ParsedDocument {
    let result = parse_frontmatter(content);
    let mut diagnostics = result.diagnostics;

    if content.trim().is_empty() {
        diagnostics.push("File is empty".into());
    }

    ParsedDocument {
        path: path.to_string(),
        frontmatter: result.frontmatter,
        body: result.body,
        wikilinks: result.wikilinks,
        diagnostics,
        malformed: result.malformed,
    }
}

/// Extract a string field from frontmatter.
pub fn fm_string(fm: &HashMap<String, Value>, key: &str) -> Option<String> {
    fm.get(key).and_then(|value| value.as_str().map(str::to_string))
}

/// Extract a string array field from frontmatter.
pub fn fm_string_array(fm: &HashMap<String, Value>, key: &str) -> Vec<String> {
    fm.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract a boolean field from frontmatter.
pub fn fm_bool(fm: &HashMap<String, Value>, key: &str) -> Option<bool> {
    fm.get(key).and_then(Value::as_bool)
}

/// Extract an optional string array field from frontmatter.
/// Returns None when the field is absent, Some(vec) when present (even if empty).
pub fn fm_string_array_opt(fm: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
    fm.get(key).map(|value| {
        value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    })
}
