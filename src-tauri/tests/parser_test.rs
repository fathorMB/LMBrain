use lmbrain_lib::commands::parser;

#[test]
fn test_parse_frontmatter_basic() {
    let content = r#"---
id: SPEC-001
title: Test Spec
status: ready
---
# Body content"#;

    let result = parser::parse_frontmatter(content);
    assert_eq!(
        result.frontmatter.get("id").and_then(|v| v.as_str()),
        Some("SPEC-001")
    );
    assert_eq!(
        result.frontmatter.get("title").and_then(|v| v.as_str()),
        Some("Test Spec")
    );
    assert_eq!(
        result.frontmatter.get("status").and_then(|v| v.as_str()),
        Some("ready")
    );
    assert!(result.body.contains("Body content"));
    assert!(result.wikilinks.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_parse_frontmatter_no_frontmatter() {
    let content = "# Just a heading\n\nSome body text.";
    let result = parser::parse_frontmatter(content);
    assert!(result.frontmatter.is_empty());
    assert!(result.body.contains("Just a heading"));
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_parse_frontmatter_empty() {
    let content = "";
    let result = parser::parse_frontmatter(content);
    assert!(result.frontmatter.is_empty());
    assert!(result.body.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_parse_frontmatter_malformed_yaml() {
    let content = r#"---
id: SPEC-001
title: [unclosed list
---
body"#;
    let result = parser::parse_frontmatter(content);
    // Malformed YAML should still return body
    assert!(result.body.contains("body"));
    // Should produce a diagnostic
    assert!(!result.diagnostics.is_empty());
    assert!(result.diagnostics[0].contains("Malformed"));
}

#[test]
fn test_parse_frontmatter_unclosed() {
    let content = r#"---
id: SPEC-001
title: Test"#;
    let result = parser::parse_frontmatter(content);
    assert!(result.diagnostics.iter().any(|d| d.contains("Unclosed")));
}

#[test]
fn test_extract_wikilinks() {
    let body = "See [[ADR-001]] and [[SPEC-002]] for details.";
    let links = parser::extract_wikilinks(body);
    assert_eq!(links, vec!["ADR-001", "SPEC-002"]);
}

#[test]
fn test_extract_wikilinks_none() {
    let body = "No wikilinks here.";
    let links = parser::extract_wikilinks(body);
    assert!(links.is_empty());
}

#[test]
fn test_extract_wikilinks_multiple_same_line() {
    let body = "Links: [[ADR-001]] and [[ADR-002]] and [[SPEC-003]]";
    let links = parser::extract_wikilinks(body);
    assert_eq!(links.len(), 3);
}

#[test]
fn test_fm_string() {
    use std::collections::HashMap;
    use serde_json::json;

    let mut fm = HashMap::new();
    fm.insert("title".to_string(), json!("Test"));
    fm.insert("count".to_string(), json!(42));

    assert_eq!(parser::fm_string(&fm, "title"), Some("Test".to_string()));
    assert_eq!(parser::fm_string(&fm, "count"), None);
    assert_eq!(parser::fm_string(&fm, "missing"), None);
}

#[test]
fn test_fm_string_array() {
    use std::collections::HashMap;
    use serde_json::json;

    let mut fm = HashMap::new();
    fm.insert("tags".to_string(), json!(["a", "b", "c"]));
    fm.insert("empty".to_string(), json!([]));

    let tags = parser::fm_string_array(&fm, "tags");
    assert_eq!(tags, vec!["a", "b", "c"]);

    let empty = parser::fm_string_array(&fm, "empty");
    assert!(empty.is_empty());

    let missing = parser::fm_string_array(&fm, "missing");
    assert!(missing.is_empty());
}

#[test]
fn test_fm_bool() {
    use std::collections::HashMap;
    use serde_json::json;

    let mut fm = HashMap::new();
    fm.insert("active".to_string(), json!(true));
    fm.insert("inactive".to_string(), json!(false));

    assert_eq!(parser::fm_bool(&fm, "active"), Some(true));
    assert_eq!(parser::fm_bool(&fm, "inactive"), Some(false));
    assert_eq!(parser::fm_bool(&fm, "missing"), None);
}

#[test]
fn test_fm_depends_on() {
    use std::collections::HashMap;
    use serde_json::json;

    let mut fm = HashMap::new();
    fm.insert("depends_on".to_string(), json!(["TASK-001", "TASK-002"]));

    let deps = parser::fm_string_array(&fm, "depends_on");
    assert_eq!(deps, vec!["TASK-001", "TASK-002"]);
}

#[test]
fn test_fm_spec() {
    use std::collections::HashMap;
    use serde_json::json;

    let mut fm = HashMap::new();
    fm.insert("spec".to_string(), json!("SPEC-001"));

    let spec = parser::fm_string(&fm, "spec");
    assert_eq!(spec, Some("SPEC-001".to_string()));
}

#[test]
fn test_wikilink_preprocessing() {
    let content = "See [[ADR-001]] and [[SPEC-002]] for details.";
    let result = parser::extract_wikilinks(content);
    assert_eq!(result, vec!["ADR-001", "SPEC-002"]);
}

#[test]
fn test_wikilink_preprocessing_no_wikilinks() {
    let content = "No wikilinks here.";
    let result = parser::extract_wikilinks(content);
    assert!(result.is_empty());
}

#[test]
fn test_wikilink_preprocessing_nested() {
    let content = "Link: [[ADR-001]] in text.";
    let result = parser::extract_wikilinks(content);
    assert_eq!(result, vec!["ADR-001"]);
}

#[test]
fn test_parse_markdown_file() {
    let content = r#"---
id: TASK-001
title: Test Task
status: in-progress
priority: High
tags: [test, demo]
---
## Description
This is a test task.

- [ ] Criterion 1
- [x] Criterion 2

See [[SPEC-001]] for details."#;

    let parsed = parser::parse_markdown_file("test.md", content);
    assert_eq!(parsed.path, "test.md");
    assert_eq!(
        parsed.frontmatter.get("id").and_then(|v| v.as_str()),
        Some("TASK-001")
    );
    assert!(parsed.body.contains("Description"));
    assert_eq!(parsed.wikilinks, vec!["SPEC-001"]);
}
