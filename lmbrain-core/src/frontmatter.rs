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

    pub fn object_array(&self, key: &str) -> Vec<Map<String, Value>> {
        self.fields
            .get(key)
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_object).cloned().collect())
            .unwrap_or_default()
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
            let lines = self.frontmatter.lines().collect::<Vec<_>>();
            let start = lines
                .iter()
                .position(|line| line.trim() == "activity:" || line.trim() == "activity: []");
            if let Some(start) = start {
                let line_trimmed = lines[start].trim();
                let end = lines
                    .iter()
                    .enumerate()
                    .skip(start + 1)
                    .find(|(_, line)| !line.trim().is_empty() && indent_width(line) == 0)
                    .map(|(index, _)| index)
                    .unwrap_or(lines.len());
                let mut output = Vec::new();
                if line_trimmed == "activity: []" {
                    output.extend(lines[..start].iter().map(|line| (*line).to_string()));
                    output.push("activity:".to_string());
                } else {
                    output.extend(lines[..end].iter().map(|line| (*line).to_string()));
                }
                output.push(format!("  - date: {today}"));
                output.push(format!("    action: {}", yaml_scalar(action)));
                if line_trimmed == "activity: []" {
                    output.extend(lines[start + 1..].iter().map(|line| (*line).to_string()));
                } else {
                    output.extend(lines[end..].iter().map(|line| (*line).to_string()));
                }
                self.frontmatter = output.join(self.newline);
            }
        } else {
            self.frontmatter.push_str(&format!(
                "{}activity:{}  - date: {}{}    action: {}",
                self.newline,
                self.newline,
                today,
                self.newline,
                yaml_scalar(action)
            ));
        }
        let item = serde_json::json!({
            "date": today.to_string(),
            "action": action,
        });
        self.fields
            .entry("activity".to_string())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .map(|arr| arr.push(item));
    }

    pub fn append_object(
        &mut self,
        key: &str,
        fields: &[(String, Value)],
    ) -> Result<(), FrontmatterError> {
        match self.fields.get(key) {
            Some(Value::Array(items)) if items.iter().all(Value::is_object) => {}
            Some(Value::Array(items)) if items.is_empty() => {}
            Some(_) => {
                return Err(FrontmatterError::Invalid(format!(
                    "'{key}' must be an array of objects"
                )));
            }
            None => {}
        }

        let lines = self.frontmatter.lines().collect::<Vec<_>>();
        let start = lines.iter().position(|line| {
            indent_width(line) == 0
                && top_level_key(line.trim_start()).is_some_and(|candidate| candidate == key)
        });
        let rendered = render_object_item(fields);

        self.frontmatter = if let Some(start) = start {
            let end = lines
                .iter()
                .enumerate()
                .skip(start + 1)
                .find(|(_, line)| !line.trim().is_empty() && indent_width(line) == 0)
                .map(|(index, _)| index)
                .unwrap_or(lines.len());
            let mut output = lines[..start]
                .iter()
                .map(|line| (*line).to_string())
                .collect::<Vec<_>>();
            output.push(format!("{key}:"));
            if lines[start].trim_end().ends_with(':') {
                output.extend(lines[start + 1..end].iter().map(|line| (*line).to_string()));
            }
            output.extend(rendered);
            output.extend(lines[end..].iter().map(|line| (*line).to_string()));
            output.join(self.newline)
        } else {
            let mut output = self.frontmatter.clone();
            if !output.is_empty() {
                output.push_str(self.newline);
            }
            output.push_str(&format!("{key}:{}", self.newline));
            output.push_str(&rendered.join(self.newline));
            output
        };

        let object = fields.iter().cloned().collect::<Map<_, _>>();
        self.fields
            .entry(key.to_string())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| {
                FrontmatterError::Invalid(format!("'{key}' must be an array of objects"))
            })?
            .push(Value::Object(object));
        Ok(())
    }

    pub fn render(&self) -> String {
        format!(
            "---{}{}{}---{}{}",
            self.newline, self.frontmatter, self.newline, self.newline, self.body
        )
    }
}

/// Outcome of a textual duplicate-key repair: the merged keys and, when a
/// merge happened, the full repaired source ready to be re-parsed.
#[derive(Debug, Clone)]
pub struct DuplicateKeyRepair {
    pub merged_keys: Vec<String>,
    pub repaired_source: Option<String>,
}

/// Merges duplicate top-level frontmatter keys produced by failed mutations
/// (e.g. the pre-4.0.2 duplicate `activity:` blocks) without requiring the
/// document to parse first. List-shaped duplicates are concatenated in file
/// order under the first occurrence; identical scalar duplicates keep the
/// first copy. Diverging scalar duplicates or mixed shapes are refused so a
/// repair can never guess at meaning.
pub fn repair_duplicate_top_level_keys(
    source: &str,
) -> Result<DuplicateKeyRepair, FrontmatterError> {
    let newline = detect_newline(source);
    let (frontmatter, body) = split_frontmatter(source, newline)?;
    let lines: Vec<&str> = frontmatter.lines().collect();

    // Segment the frontmatter into top-level blocks: a key line plus every
    // following line that is blank, a comment, or indented deeper.
    struct Block {
        key: Option<String>,
        scalar: Option<String>,
        lines: Vec<String>,
    }
    let mut blocks: Vec<Block> = Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        let is_content = !trimmed.is_empty() && !trimmed.starts_with('#');
        if is_content && indent_width(line) == 0 {
            let (key, rest) = split_key_value(trimmed).ok_or_else(|| {
                FrontmatterError::Invalid(format!("expected key/value in line '{trimmed}'"))
            })?;
            blocks.push(Block {
                key: Some(key.to_string()),
                scalar: (!rest.is_empty()).then(|| rest.to_string()),
                lines: vec![(*line).to_string()],
            });
        } else if let Some(block) = blocks.last_mut() {
            block.lines.push((*line).to_string());
        } else {
            blocks.push(Block {
                key: None,
                scalar: None,
                lines: vec![(*line).to_string()],
            });
        }
    }

    let mut merged_keys: Vec<String> = Vec::new();
    let mut removed: Vec<usize> = Vec::new();
    for index in 0..blocks.len() {
        if removed.contains(&index) {
            continue;
        }
        let Some(key) = blocks[index].key.clone() else {
            continue;
        };
        let duplicates: Vec<usize> = (index + 1..blocks.len())
            .filter(|later| blocks[*later].key.as_deref() == Some(key.as_str()))
            .collect();
        if duplicates.is_empty() {
            continue;
        }

        // Child content lines of a list-shaped block (`- ` items only).
        let list_items = |block: &Block| -> Option<Vec<String>> {
            let scalar_is_list = match block.scalar.as_deref() {
                None => true,
                Some("[]") => true,
                Some(_) => false,
            };
            if !scalar_is_list {
                return None;
            }
            let mut items = Vec::new();
            let mut item_indent: Option<usize> = None;
            for child in &block.lines[1..] {
                let trimmed = child.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let indent = indent_width(child);
                if trimmed.starts_with('-') {
                    item_indent = Some(indent);
                    items.push(child.clone());
                } else if item_indent.is_some_and(|marker| indent > marker) {
                    // Continuation line of the current `- key: value` item.
                    items.push(child.clone());
                } else {
                    return None;
                }
            }
            Some(items)
        };

        let all_scalar = std::iter::once(index)
            .chain(duplicates.iter().copied())
            .all(|position| blocks[position].scalar.is_some() && blocks[position].lines.len() == 1);
        if all_scalar {
            let first = blocks[index].scalar.clone();
            if duplicates
                .iter()
                .any(|position| blocks[*position].scalar != first)
            {
                return Err(FrontmatterError::Invalid(format!(
                    "duplicate key '{key}' has diverging scalar values; repair refused"
                )));
            }
            removed.extend(duplicates.iter().copied());
            merged_keys.push(key);
            continue;
        }

        let mut all_items = list_items(&blocks[index]).ok_or_else(|| {
            FrontmatterError::Invalid(format!(
                "duplicate key '{key}' is not list-shaped; repair refused"
            ))
        })?;
        for position in &duplicates {
            let items = list_items(&blocks[*position]).ok_or_else(|| {
                FrontmatterError::Invalid(format!(
                    "duplicate key '{key}' is not list-shaped; repair refused"
                ))
            })?;
            all_items.extend(items);
        }
        blocks[index].lines = if all_items.is_empty() {
            vec![format!("{key}: []")]
        } else {
            let mut rebuilt = vec![format!("{key}:")];
            rebuilt.extend(all_items);
            rebuilt
        };
        removed.extend(duplicates.iter().copied());
        merged_keys.push(key);
    }

    if merged_keys.is_empty() {
        return Ok(DuplicateKeyRepair {
            merged_keys,
            repaired_source: None,
        });
    }

    let repaired_frontmatter = blocks
        .iter()
        .enumerate()
        .filter(|(position, _)| !removed.contains(position))
        .flat_map(|(_, block)| block.lines.iter().cloned())
        .collect::<Vec<_>>()
        .join(newline);
    let repaired = format!("---{newline}{repaired_frontmatter}{newline}---{newline}{body}");
    // The repaired document must parse cleanly; otherwise refuse to write it.
    Document::parse(&repaired)?;
    Ok(DuplicateKeyRepair {
        merged_keys,
        repaired_source: Some(repaired),
    })
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

        if map.contains_key(key) {
            return Err(FrontmatterError::Invalid(format!(
                "duplicate top-level YAML key '{key}' at line {}",
                index + 1
            )));
        }

        let value = if rest.is_empty() {
            // Only content indented deeper than the key is its nested block. A
            // following line at indent 0 is the next top-level key: without this
            // guard an empty-valued key (e.g. the template's `area: `) swallowed
            // every remaining top-level field as its own nested object.
            match next_content_indent(&lines, index + 1) {
                Some(child_indent) if child_indent > 0 => {
                    parse_nested_block(&lines, &mut index, child_indent)?
                }
                _ => Value::Null,
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

pub(crate) fn parse_inline_value(input: &str) -> Result<Value, FrontmatterError> {
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
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn render_object_item(fields: &[(String, Value)]) -> Vec<String> {
    let mut rendered = Vec::new();
    for (index, (key, value)) in fields.iter().enumerate() {
        let prefix = if index == 0 { "  - " } else { "    " };
        rendered.push(format!("{prefix}{key}: {}", yaml_value(value)));
    }
    rendered
}

fn yaml_value(value: &Value) -> String {
    match value {
        Value::String(value) => yaml_scalar(value),
        Value::Bool(value) => value.to_string(),
        Value::Null => "null".into(),
        Value::Number(value) => value.to_string(),
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(yaml_value).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(_) => yaml_scalar(&value.to_string()),
    }
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
        assert_eq!(
            document.value("note").as_deref(),
            Some("line one\nline two")
        );
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
    fn empty_valued_key_does_not_swallow_following_top_level_keys() {
        // Regression for #82: `area: ` (empty value) made next_content_indent
        // return 0 and parse_nested_block consumed every remaining top-level
        // key as a nested object under `area`, hiding `activity` and everything
        // after it from the fields map.
        let source = "---\nid: SPEC-002\narea: \nmilestone: \nrecommended_agent: AGENT-XXX\ntags: []\nactivity:\n  - date: 2026-08-06\n    action: \"created\"\n---\nBody";
        let document = parse(source);
        let fields = document.fields();
        for key in [
            "id",
            "area",
            "milestone",
            "recommended_agent",
            "tags",
            "activity",
        ] {
            assert!(fields.contains_key(key), "missing top-level key '{key}'");
        }
        assert_eq!(fields.get("area"), Some(&Value::Null));
        assert_eq!(
            fields
                .get("activity")
                .and_then(Value::as_array)
                .map(|items| items.len()),
            Some(1)
        );
    }

    #[test]
    fn create_then_sequential_setters_keep_a_single_activity_key() {
        // Regression for #82/KIT-NOTE-003: with the template's empty-valued
        // keys, every governed setter after create appended a fresh top-level
        // `activity:` block because the parsed fields map never surfaced the
        // existing one.
        let template = "---\nid: SPEC-XXX\ntitle: \"Feature or work item title\"\nstatus: backlog\narea: \nmilestone: \ncapability_tier: \nthinking_level: \nrelated_decisions: []\ncreated: YYYY-MM-DD\nupdated: YYYY-MM-DD\ntags: []\n---\nBody";
        let mut document = parse(template);
        document.set("id", "SPEC-002");
        document.set("created", "2026-08-06");
        document.set("updated", "2026-08-06");
        document.append_activity("created");
        let after_create = document.render();
        assert_eq!(after_create.matches("\nactivity:").count(), 1);

        let mut second = parse(&after_create);
        second.set("capability_tier", "terra");
        second.set("thinking_level", "standard");
        second.set("updated", "2026-08-06");
        second.append_activity("set effort");
        let after_effort = second.render();
        assert_eq!(after_effort.matches("\nactivity:").count(), 1);

        let mut third = parse(&after_effort);
        third.set("tags", "[governance]");
        third.set("updated", "2026-08-06");
        third.append_activity("set tags");
        let after_tags = third.render();
        assert_eq!(after_tags.matches("\nactivity:").count(), 1);
        let reparsed = parse(&after_tags);
        assert_eq!(
            reparsed
                .fields()
                .get("activity")
                .and_then(Value::as_array)
                .map(|items| items.len()),
            Some(3)
        );
    }

    #[test]
    fn repairs_duplicate_activity_blocks_and_refuses_ambiguity() {
        // The exact corruption 4.0.1 wrote in the field: three top-level
        // activity blocks after create + two setters.
        let corrupted = "---\nid: SPEC-005\nstatus: backlog\ntags: []\nactivity:\n  - date: 2026-08-06\n    action: \"created\"\nactivity:\n  - date: 2026-08-06\n    action: \"set effort\"\nactivity:\n  - date: 2026-08-06\n    action: \"set recommended_agent\"\n---\nBody";
        let repair = super::repair_duplicate_top_level_keys(corrupted).unwrap();
        assert_eq!(repair.merged_keys, vec!["activity".to_string()]);
        let repaired = repair.repaired_source.unwrap();
        assert_eq!(repaired.matches("\nactivity:").count(), 1);
        let document = parse(&repaired);
        assert_eq!(
            document
                .fields()
                .get("activity")
                .and_then(Value::as_array)
                .map(|items| items.len()),
            Some(3)
        );
        assert_eq!(document.value("status").as_deref(), Some("backlog"));

        // A clean document reports nothing to repair.
        let clean = super::repair_duplicate_top_level_keys(&repaired).unwrap();
        assert!(clean.merged_keys.is_empty());
        assert!(clean.repaired_source.is_none());

        // Identical scalar duplicates collapse; diverging ones are refused.
        let scalar = "---\nid: SPEC-006\nstatus: backlog\nstatus: backlog\n---\nBody";
        let collapsed = super::repair_duplicate_top_level_keys(scalar).unwrap();
        assert_eq!(collapsed.merged_keys, vec!["status".to_string()]);
        let diverging = "---\nid: SPEC-006\nstatus: backlog\nstatus: ready\n---\nBody";
        assert!(super::repair_duplicate_top_level_keys(diverging).is_err());
    }

    #[test]
    fn appends_typed_objects_and_rejects_an_incompatible_field() {
        let mut document = parse("---\nid: REVIEW-001\nreview_events: []\n---\nBody");
        document
            .append_object(
                "review_events",
                &[
                    ("id".into(), Value::String("REVIEW-001-EVENT-001".into())),
                    (
                        "evidence_refs".into(),
                        Value::Array(vec![Value::String("SPEC-001".into())]),
                    ),
                ],
            )
            .unwrap();
        let reparsed = parse(&document.render());
        let events = reparsed.object_array("review_events");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0]
                .get("evidence_refs")
                .and_then(Value::as_array)
                .and_then(|values| values.first())
                .and_then(Value::as_str),
            Some("SPEC-001")
        );

        let mut invalid = parse("---\nreview_events: legacy\n---\nBody");
        assert!(invalid
            .append_object(
                "review_events",
                &[("id".into(), Value::String("EVENT-001".into()))]
            )
            .is_err());
    }

    #[test]
    fn unterminated_frontmatter_is_malformed() {
        assert!(Document::parse("---\nid: X\nno closing marker").is_err());
    }
}
