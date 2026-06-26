use std::{collections::HashMap, fs, path::Path};

use chrono::Local;
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("missing or malformed YAML frontmatter")]
    Malformed,
    #[error("missing or malformed YAML frontmatter: {0}")]
    Invalid(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Document {
    pub frontmatter: String,
    pub body: String,
    pub newline: &'static str,
    fields: Map<String, Value>,
}

impl Document {
    pub fn parse(source: &str) -> Result<Self, FrontmatterError> {
        let newline = detect_newline(source);
        let (frontmatter, body) = split_frontmatter(source, newline)?;
        let fields = parse_mapping(&frontmatter)?;

        Ok(Self {
            frontmatter,
            body,
            newline,
            fields,
        })
    }

    pub fn value(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(value_as_string)
    }

    pub fn string_array(&self, key: &str) -> Vec<String> {
        self.fields
            .get(key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(value_as_string).collect())
            .unwrap_or_default()
    }

    pub fn bool(&self, key: &str) -> Option<bool> {
        self.fields.get(key).and_then(Value::as_bool)
    }

    pub fn fields(&self) -> HashMap<String, Value> {
        self.fields.clone().into_iter().collect()
    }

    /// Replaces only top-level scalar fields, retaining line order, comments, unrelated
    /// fields, and newline style.
    pub fn set(&mut self, key: &str, value: &str) {
        let mut found = false;
        let mut depth = 0usize;
        let lines: Vec<String> = self
            .frontmatter
            .lines()
            .map(|line| {
                let indent = indent_width(line);
                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    return line.to_string();
                }
                if indent == 0 {
                    depth = 0;
                }
                if !found
                    && depth == 0
                    && indent == 0
                    && top_level_key(trimmed).is_some_and(|candidate| candidate == key)
                {
                    found = true;
                    depth = if trimmed.ends_with(':')
                        || trimmed.ends_with(": |")
                        || trimmed.ends_with(": >")
                    {
                        1
                    } else {
                        0
                    };
                    format!("{key}: {value}")
                } else if found && depth > 0 && indent > 0 {
                    String::new()
                } else {
                    line.to_string()
                }
            })
            .filter(|line| !line.is_empty())
            .collect();

        self.frontmatter = lines.join(self.newline);
        if !found {
            if !self.frontmatter.is_empty() {
                self.frontmatter.push_str(self.newline);
            }
            self.frontmatter.push_str(&format!("{key}: {value}"));
        }
        self.fields.insert(
            key.to_string(),
            parse_inline_value(value).unwrap_or_else(|_| Value::String(value.to_string())),
        );
    }

    pub fn append_activity(&mut self, action: &str) {
        let today = Local::now().format("%Y-%m-%d");
        if self.fields.contains_key("activity") {
            self.frontmatter.push_str(&format!(
                "{}  - date: {}{}    action: {}",
                self.newline,
                today,
                self.newline,
                yaml_scalar(action)
            ));
        } else {
            self.frontmatter.push_str(&format!(
                "{}activity:{}  - date: {}{}    action: {}",
                self.newline,
                self.newline,
                today,
                self.newline,
                yaml_scalar(action)
            ));
            self.fields
                .insert("activity".into(), Value::Array(Vec::new()));
        }
    }

    pub fn append_override_reason(&mut self, reason: &str) {
        self.body.push_str(&format!(
            "{}{}## Mutation override{}{}",
            self.newline,
            self.newline,
            self.newline,
            reason.trim()
        ));
    }

    pub fn render(&self) -> String {
        format!(
            "---{}{}{}---{}{}",
            self.newline, self.frontmatter, self.newline, self.newline, self.body
        )
    }
}

fn detect_newline(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn split_frontmatter(
    source: &str,
    newline: &'static str,
) -> Result<(String, String), FrontmatterError> {
    if !source.starts_with("---") {
        return Err(FrontmatterError::Malformed);
    }

    let marker = format!("{newline}---");
    let end = source[3..].find(&marker).ok_or_else(|| {
        FrontmatterError::Invalid(
            "Unclosed frontmatter: opening `---` has no matching closing `---`".into(),
        )
    })? + 3;
    let after = end + marker.len();

    Ok((
        source[3..end].trim_start_matches(['\r', '\n']).to_string(),
        source[after..].trim_start_matches(['\r', '\n']).to_string(),
    ))
}

fn parse_mapping(input: &str) -> Result<Map<String, Value>, FrontmatterError> {
    let lines: Vec<&str> = input.lines().collect();
    let mut index = 0usize;
    let mut map = Map::new();

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }
        if indent_width(line) != 0 {
            return Err(FrontmatterError::Invalid(format!(
                "unexpected indentation at line {}",
                index + 1
            )));
        }

        let (key, rest) = split_key_value(trimmed).ok_or_else(|| {
            FrontmatterError::Invalid(format!("expected key/value at line {}", index + 1))
        })?;

        let value = if rest.is_empty() {
            match next_content_indent(&lines, index + 1) {
                Some(child_indent) => parse_nested_block(&lines, &mut index, child_indent)?,
                None => Value::Null,
            }
        } else if rest == "|" || rest == ">" {
            parse_block_scalar(&lines, &mut index, 1, rest == ">")?
        } else {
            parse_inline_value(rest)?
        };

        map.insert(key.to_string(), value);
        index += 1;
    }

    Ok(map)
}

fn parse_nested_block(
    lines: &[&str],
    index: &mut usize,
    indent: usize,
) -> Result<Value, FrontmatterError> {
    let next = lines.get(*index + 1).ok_or(FrontmatterError::Malformed)?;
    let trimmed = next.trim_start();
    if trimmed.starts_with("- ") || trimmed == "-" {
        parse_array(lines, index, indent)
    } else {
        let map = parse_indented_map(lines, index, indent)?;
        Ok(Value::Object(map))
    }
}

fn parse_indented_map(
    lines: &[&str],
    index: &mut usize,
    indent: usize,
) -> Result<Map<String, Value>, FrontmatterError> {
    let mut map = Map::new();
    let mut cursor = *index + 1;

    while cursor < lines.len() {
        let line = lines[cursor];
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            cursor += 1;
            continue;
        }

        let line_indent = indent_width(line);
        if line_indent < indent {
            break;
        }
        if line_indent > indent {
            return Err(FrontmatterError::Invalid(format!(
                "unexpected indentation at line {}",
                cursor + 1
            )));
        }

        let trimmed = line.trim_start();
        let (key, rest) = split_key_value(trimmed).ok_or_else(|| {
            FrontmatterError::Invalid(format!("expected key/value at line {}", cursor + 1))
        })?;

        // This line is consumed; nested parsers may advance `*index` further. Setting it
        // up front guarantees `cursor` advances even for inline scalars and empty values,
        // which would otherwise reset `cursor` to a stale `*index` and loop forever.
        *index = cursor;
        let value = if rest.is_empty() {
            match next_content_indent(lines, cursor + 1) {
                Some(child_indent) if child_indent > indent => {
                    parse_nested_block(lines, index, child_indent)?
                }
                _ => Value::Null,
            }
        } else if rest == "|" || rest == ">" {
            parse_block_scalar(lines, index, indent + 1, rest == ">")?
        } else {
            parse_inline_value(rest)?
        };

        map.insert(key.to_string(), value);
        cursor = *index + 1;
    }

    *index = cursor.saturating_sub(1);
    Ok(map)
}

fn parse_array(
    lines: &[&str],
    index: &mut usize,
    indent: usize,
) -> Result<Value, FrontmatterError> {
    let mut items = Vec::new();
    let mut cursor = *index + 1;

    while cursor < lines.len() {
        let line = lines[cursor];
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            cursor += 1;
            continue;
        }

        let line_indent = indent_width(line);
        if line_indent < indent {
            break;
        }
        if line_indent != indent {
            return Err(FrontmatterError::Invalid(format!(
                "unexpected indentation at line {}",
                cursor + 1
            )));
        }

        let trimmed = line.trim_start();
        if !trimmed.starts_with('-') {
            break;
        }

        let rest = trimmed[1..].trim_start();
        if rest.is_empty() {
            let child_indent = next_content_indent(lines, cursor + 1).ok_or_else(|| {
                FrontmatterError::Invalid(format!("missing nested item at line {}", cursor + 1))
            })?;
            *index = cursor;
            items.push(parse_nested_block(lines, index, child_indent)?);
            cursor = *index + 1;
            continue;
        }

        if let Some((key, value)) = split_key_value(rest) {
            let mut object = Map::new();
            if value.is_empty() {
                let child_indent = next_content_indent(lines, cursor + 1);
                if let Some(child_indent) = child_indent.filter(|child| *child > indent) {
                    *index = cursor;
                    object.insert(
                        key.to_string(),
                        parse_nested_block(lines, index, child_indent)?,
                    );
                    let extra = parse_indented_map(lines, index, indent + 2)?;
                    for (extra_key, extra_value) in extra {
                        object.insert(extra_key, extra_value);
                    }
                    items.push(Value::Object(object));
                    cursor = *index + 1;
                    continue;
                }
                object.insert(key.to_string(), Value::Null);
            } else {
                object.insert(key.to_string(), parse_inline_value(value)?);
            }

            let mut map_cursor = cursor;
            let extra = parse_indented_map(lines, &mut map_cursor, indent + 2)?;
            for (extra_key, extra_value) in extra {
                object.insert(extra_key, extra_value);
            }
            cursor = map_cursor + 1;
            items.push(Value::Object(object));
            continue;
        }

        items.push(parse_inline_value(rest)?);
        cursor += 1;
    }

    *index = cursor.saturating_sub(1);
    Ok(Value::Array(items))
}

fn parse_block_scalar(
    lines: &[&str],
    index: &mut usize,
    minimum_indent: usize,
    folded: bool,
) -> Result<Value, FrontmatterError> {
    let indent = next_content_indent(lines, *index + 1).unwrap_or(minimum_indent);
    let mut cursor = *index + 1;
    let mut parts = Vec::new();

    while cursor < lines.len() {
        let line = lines[cursor];
        if line.trim().is_empty() {
            parts.push(String::new());
            cursor += 1;
            continue;
        }

        let line_indent = indent_width(line);
        if line_indent < indent {
            break;
        }

        parts.push(line.chars().skip(indent).collect());
        cursor += 1;
    }

    *index = cursor.saturating_sub(1);
    if folded {
        Ok(Value::String(parts.join(" ").trim().to_string()))
    } else {
        Ok(Value::String(parts.join("\n")))
    }
}

fn parse_inline_value(input: &str) -> Result<Value, FrontmatterError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Value::Null);
    }
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }
    if trimmed == "null" || trimmed == "~" {
        return Ok(Value::Null);
    }
    if trimmed.starts_with('[') {
        return parse_inline_array(trimmed);
    }
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        return parse_quoted_scalar(trimmed).map(Value::String);
    }
    Ok(Value::String(trim_inline_comment(trimmed).to_string()))
}

fn parse_inline_array(input: &str) -> Result<Value, FrontmatterError> {
    if !input.ends_with(']') {
        return Err(FrontmatterError::Invalid(
            "unterminated inline array".to_string(),
        ));
    }

    let inner = &input[1..input.len() - 1];
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote = None;

    for ch in inner.chars() {
        match (quote, ch) {
            (Some(active), c) if c == active => {
                quote = None;
                current.push(c);
            }
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => {
                quote = Some(ch);
                current.push(ch);
            }
            (None, ',') => {
                if !current.trim().is_empty() {
                    items.push(parse_inline_value(current.trim())?);
                }
                current.clear();
            }
            (None, c) => current.push(c),
        }
    }

    if quote.is_some() {
        return Err(FrontmatterError::Invalid(
            "unterminated quoted value".to_string(),
        ));
    }

    if !current.trim().is_empty() {
        items.push(parse_inline_value(current.trim())?);
    }

    Ok(Value::Array(items))
}

fn parse_quoted_scalar(input: &str) -> Result<String, FrontmatterError> {
    let mut chars = input.chars();
    let quote = chars.next().ok_or(FrontmatterError::Malformed)?;
    let mut escaped = false;
    let mut out = String::new();

    for ch in chars.by_ref() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }
        if ch == '\\' && quote == '"' {
            escaped = true;
            continue;
        }
        if ch == quote {
            let remainder: String = chars.collect();
            if !trim_inline_comment(remainder.trim()).is_empty() {
                return Err(FrontmatterError::Invalid(
                    "unexpected trailing characters after quoted value".to_string(),
                ));
            }
            return Ok(out);
        }
        out.push(ch);
    }

    Err(FrontmatterError::Invalid(
        "unterminated quoted value".to_string(),
    ))
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let mut quote = None;
    for (index, ch) in line.char_indices() {
        match (quote, ch) {
            (Some(active), c) if c == active => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(ch),
            (None, ':') => {
                let key = line[..index].trim();
                let value = line[index + 1..].trim_start();
                if key.is_empty() {
                    return None;
                }
                return Some((key, value));
            }
            _ => {}
        }
    }
    None
}

fn top_level_key(line: &str) -> Option<&str> {
    split_key_value(line).map(|(key, _)| key)
}

fn next_content_indent(lines: &[&str], mut start: usize) -> Option<usize> {
    while let Some(line) = lines.get(start) {
        if !line.trim().is_empty() && !line.trim_start().starts_with('#') {
            return Some(indent_width(line));
        }
        start += 1;
    }
    None
}

fn indent_width(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

fn trim_inline_comment(value: &str) -> &str {
    let mut quote = None;
    for (index, ch) in value.char_indices() {
        match (quote, ch) {
            (Some(active), c) if c == active => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(ch),
            (None, '#') => return value[..index].trim_end(),
            _ => {}
        }
    }
    value
}

fn yaml_scalar(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub fn atomic_write(path: &Path, content: &str) -> Result<(), FrontmatterError> {
    let parent = path.parent().ok_or(FrontmatterError::Malformed)?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact"),
        std::process::id()
    ));
    fs::write(&temp, content)?;
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Document {
        Document::parse(source).expect("document should parse")
    }

    #[test]
    fn reads_top_level_scalars() {
        let document = parse("---\nid: SPEC-001\nstatus: ready\n---\nBody");
        assert_eq!(document.value("id").as_deref(), Some("SPEC-001"));
        assert_eq!(document.value("status").as_deref(), Some("ready"));
        assert_eq!(document.body, "Body");
    }

    #[test]
    fn keeps_colons_inside_quoted_values() {
        let document = parse("---\ntitle: \"React UI: list\"\n---\n");
        assert_eq!(document.value("title").as_deref(), Some("React UI: list"));
    }

    #[test]
    fn parses_inline_array() {
        let document = parse("---\ntags: [a, \"b, c\", d]\n---\n");
        assert_eq!(document.string_array("tags"), vec!["a", "b, c", "d"]);
    }

    #[test]
    fn parses_block_scalar() {
        let document = parse("---\nnote: |\n  line one\n  line two\n---\n");
        assert_eq!(document.value("note").as_deref(), Some("line one\nline two"));
    }

    #[test]
    fn parses_nested_map_with_inline_scalars() {
        // Regression: a nested map whose children are inline scalars must not loop.
        let document = parse("---\nmeta:\n  a: 1\n  b: two\nid: X\n---\n");
        assert_eq!(document.value("id").as_deref(), Some("X"));
        let fields = document.fields();
        let meta = fields.get("meta").and_then(Value::as_object).unwrap();
        assert_eq!(meta.get("a").and_then(Value::as_str), Some("1"));
        assert_eq!(meta.get("b").and_then(Value::as_str), Some("two"));
    }

    #[test]
    fn parses_activity_block_without_hanging() {
        // Regression for the infinite loop on `activity:` blocks (array of maps with
        // inline scalar fields) written by every transition/creation.
        let source = "---\nid: SPEC-001\nstatus: ready\nactivity:\n  - date: 2026-06-26\n    action: \"transitioned backlog -> ready\"\n  - date: 2026-06-27\n    action: \"set recommended_agent\"\n---\nBody";
        let document = parse(source);
        assert_eq!(document.value("status").as_deref(), Some("ready"));
        let fields = document.fields();
        let activity = fields.get("activity").and_then(Value::as_array).unwrap();
        assert_eq!(activity.len(), 2);
        assert_eq!(
            activity[0].get("action").and_then(Value::as_str),
            Some("transitioned backlog -> ready")
        );
        assert_eq!(
            activity[1].get("date").and_then(Value::as_str),
            Some("2026-06-27")
        );
    }

    #[test]
    fn round_trips_an_activity_block_through_transition_shapes() {
        // Append an activity entry, render, and re-parse: the cycle the engine performs.
        let mut document = parse("---\nid: SPEC-001\nstatus: backlog\n---\nBody");
        document.set("status", "ready");
        document.append_activity("transitioned backlog -> ready");
        let rendered = document.render();
        let reparsed = parse(&rendered);
        assert_eq!(reparsed.value("status").as_deref(), Some("ready"));
        let fields = reparsed.fields();
        assert_eq!(
            fields
                .get("activity")
                .and_then(Value::as_array)
                .map(|items| items.len()),
            Some(1)
        );
    }

    #[test]
    fn unterminated_frontmatter_is_malformed() {
        assert!(Document::parse("---\nid: X\nno closing marker").is_err());
    }
}
