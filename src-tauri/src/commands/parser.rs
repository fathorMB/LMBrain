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
}

/// Parse YAML frontmatter and Markdown body from a string.
/// Returns frontmatter map, body text, wikilinks, and any parse diagnostics.
pub fn parse_frontmatter(content: &str) -> FrontmatterResult {
    let content = content.trim_start();
    let mut diagnostics = Vec::new();

    if !content.starts_with("---") {
        return FrontmatterResult {
            frontmatter: HashMap::new(),
            body: content.to_string(),
            wikilinks: Vec::new(),
            diagnostics,
        };
    }

    // Find the closing ---
    let after_first = &content[3..];
    let end = after_first.find("\n---");
    let close_pos = match end {
        Some(pos) => pos + 3,
        None => {
            diagnostics.push(
                "Unclosed YAML frontmatter: opening `---` has no matching closing `---`".into(),
            );
            return FrontmatterResult {
                frontmatter: HashMap::new(),
                body: content.to_string(),
                wikilinks: Vec::new(),
                diagnostics,
            };
        }
    };

    let yaml_str = &content[3..close_pos].trim();
    let body = content[(close_pos + 3).min(content.len())..]
        .trim()
        .to_string();

    let frontmatter: HashMap<String, Value> = match serde_yaml::from_str(yaml_str) {
        Ok(fm) => fm,
        Err(e) => {
            diagnostics.push(format!("Malformed YAML frontmatter: {}", e));
            HashMap::new()
        }
    };

    let wikilinks = extract_wikilinks(&body);

    FrontmatterResult {
        frontmatter,
        body,
        wikilinks,
        diagnostics,
    }
}

/// Extract [[wikilinks]] from markdown body.
pub fn extract_wikilinks(body: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.captures_iter(body)
        .map(|c| c[1].trim().to_string())
        .collect()
}

/// Parse a markdown file and return structured content.
pub fn parse_markdown_file(path: &str, content: &str) -> ParsedDocument {
    let result = parse_frontmatter(content);

    let mut diagnostics = result.diagnostics;

    // Check for common issues
    if content.trim().is_empty() {
        diagnostics.push("File is empty".into());
    }

    // Convert frontmatter values to JSON Values
    let fm_json: HashMap<String, Value> = result.frontmatter;

    ParsedDocument {
        path: path.to_string(),
        frontmatter: fm_json,
        body: result.body,
        wikilinks: result.wikilinks,
        diagnostics,
    }
}

/// Extract a string field from frontmatter.
pub fn fm_string(fm: &HashMap<String, Value>, key: &str) -> Option<String> {
    fm.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extract a string array field from frontmatter.
pub fn fm_string_array(fm: &HashMap<String, Value>, key: &str) -> Vec<String> {
    fm.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract a boolean field from frontmatter.
pub fn fm_bool(fm: &HashMap<String, Value>, key: &str) -> Option<bool> {
    fm.get(key).and_then(|v| v.as_bool())
}
