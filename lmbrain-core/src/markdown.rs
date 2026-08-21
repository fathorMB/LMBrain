#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItem {
    pub position: usize,
    pub marker: String,
    pub checked: bool,
    pub waived: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingItem<'a> {
    pub level: usize,
    pub text: &'a str,
    pub offset: usize,
}

pub fn is_fence_line(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = trimmed.chars().take_while(|&c| c == marker).count();
    if count < 3 {
        return None;
    }
    let rest = trimmed[count..].trim();
    Some((marker, count, rest))
}

/// Marks lines that belong to fenced code blocks, including their delimiters.
/// Closing fences must use the same marker and at least the opening length.
pub fn fence_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(lines.len());
    let mut fence: Option<(char, usize)> = None;
    for line in lines {
        if let Some((marker, count, _)) = is_fence_line(line) {
            match fence {
                Some((open_marker, open_count)) if open_marker == marker && count >= open_count => {
                    mask.push(true);
                    fence = None;
                    continue;
                }
                None => {
                    mask.push(true);
                    fence = Some((marker, count));
                    continue;
                }
                _ => {}
            }
        }
        mask.push(fence.is_some());
    }
    mask
}

pub fn find_section<'a>(body: &'a str, heading: &str, level: usize) -> Option<&'a str> {
    find_section_with(body, level, |candidate| candidate.eq_ignore_ascii_case(heading))
}

pub fn find_section_any<'a>(body: &'a str, headings: &[&str]) -> Option<&'a str> {
    find_section_with(body, 0, |candidate| {
        headings.iter().any(|h| candidate.eq_ignore_ascii_case(h))
    })
}

fn find_section_with(
    body: &str,
    target_level: usize,
    matches_heading: impl Fn(&str) -> bool,
) -> Option<&str> {
    let mut fence: Option<(char, usize)> = None;
    let mut start: Option<usize> = None;
    let mut found_level = target_level;
    let mut offset = 0usize;

    for raw in body.split_inclusive('\n') {
        let line_start = offset;
        offset += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);

        if let Some((marker, count, _)) = is_fence_line(line) {
            match fence {
                Some((fm, fc)) if fm == marker && count >= fc => fence = None,
                None => fence = Some((marker, count)),
                _ => {}
            }
            continue;
        }

        if fence.is_some() {
            continue;
        }

        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        let is_heading = level > 0 && level <= 6 && trimmed.as_bytes().get(level) == Some(&b' ');

        if is_heading {
            let heading_text = trimmed[level..].trim().trim_matches('#').trim();
            match start {
                None => {
                    if (target_level == 0 || level == target_level) && matches_heading(heading_text) {
                        start = Some(offset);
                        found_level = level;
                    }
                }
                Some(begin) => {
                    if level <= found_level {
                        return Some(&body[begin..line_start]);
                    }
                }
            }
        }
    }

    start.map(|begin| &body[begin..])
}

pub fn find_nonempty_section<'a>(body: &'a str, heading: &str) -> Option<&'a str> {
    let mut fence: Option<(char, usize)> = None;
    let mut start: Option<usize> = None;
    let mut found_level = 0usize;
    let mut offset = 0usize;

    for raw in body.split_inclusive('\n') {
        let line_start = offset;
        offset += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);

        if let Some((marker, count, _)) = is_fence_line(line) {
            match fence {
                Some((fm, fc)) if fm == marker && count >= fc => fence = None,
                None => fence = Some((marker, count)),
                _ => {}
            }
            continue;
        }

        if fence.is_some() {
            continue;
        }

        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        let is_heading = level > 0 && level <= 6 && trimmed.as_bytes().get(level) == Some(&b' ');

        if is_heading {
            let heading_text = trimmed[level..].trim().trim_matches('#').trim();
            if let Some(begin) = start {
                if level <= found_level {
                    let candidate = &body[begin..line_start];
                    if !candidate.trim().is_empty() {
                        return Some(candidate);
                    }
                    start = None;
                }
            }

            if start.is_none() && heading_text.eq_ignore_ascii_case(heading) {
                start = Some(offset);
                found_level = level;
            }
        }
    }

    if let Some(begin) = start {
        let candidate = &body[begin..];
        if !candidate.trim().is_empty() {
            return Some(candidate);
        }
    }

    None
}

pub fn has_heading(body: &str, heading: &str) -> bool {
    let mut fence: Option<(char, usize)> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, count, _)) = is_fence_line(trimmed) {
            match fence {
                Some((fm, fc)) if fm == marker && count >= fc => fence = None,
                None => fence = Some((marker, count)),
                _ => {}
            }
            continue;
        }

        if fence.is_some() {
            continue;
        }

        let level = trimmed.chars().take_while(|&c| c == '#').count();
        let is_heading = level > 0 && level <= 6 && trimmed.as_bytes().get(level) == Some(&b' ');
        if is_heading {
            let heading_text = trimmed[level..].trim().trim_matches('#').trim();
            if heading_text.eq_ignore_ascii_case(heading) {
                return true;
            }
        }
    }

    false
}

pub fn parse_checklist(section: &str) -> Vec<ChecklistItem> {
    let mut items = Vec::new();
    let mut position = 0;
    let mut fence: Option<(char, usize)> = None;

    for line in section.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, count, _)) = is_fence_line(trimmed) {
            match fence {
                Some((fm, fc)) if fm == marker && count >= fc => fence = None,
                None => fence = Some((marker, count)),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        if !trimmed.starts_with("- [") {
            continue;
        }

        if let Some((marker_part, rest)) = trimmed[3..].split_once(']') {
            position += 1;
            let marker = marker_part.to_string();
            let checked = marker.eq_ignore_ascii_case("x");
            let text = rest.trim().to_string();
            let waived = if marker == "~" {
                extract_waived_debt_id(&text).map(str::to_string)
            } else {
                None
            };

            items.push(ChecklistItem {
                position,
                marker,
                checked,
                waived,
                text,
            });
        }
    }

    items
}

pub fn extract_waived_debt_id(text: &str) -> Option<&str> {
    let idx = text.find("waived=")?;
    let rest = &text[idx + "waived=".len()..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '|' || c == ']' || c == ')')
        .unwrap_or(rest.len());
    let debt_id = rest[..end].trim();
    if debt_id.starts_with("DEBT-") {
        Some(debt_id)
    } else {
        None
    }
}

pub fn strip_code_fences(body: &str) -> String {
    let mut lines = Vec::new();
    let mut fence: Option<(char, usize)> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, count, _)) = is_fence_line(trimmed) {
            match fence {
                Some((fm, fc)) if fm == marker && count >= fc => fence = None,
                None => fence = Some((marker, count)),
                _ => {}
            }
            continue;
        }
        if fence.is_none() {
            lines.push(line);
        }
    }

    lines.join("\n")
}

pub fn extract_code_blocks(body: &str, target_lang: Option<&str>) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current_block: Option<(char, usize, String)> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, count, lang)) = is_fence_line(trimmed) {
            match current_block.take() {
                Some((open_m, open_c, content)) if open_m == marker && count >= open_c => {
                    blocks.push(content);
                }
                Some((open_m, open_c, mut content)) => {
                    content.push_str(line);
                    content.push('\n');
                    current_block = Some((open_m, open_c, content));
                }
                None => {
                    let matches = match target_lang {
                        Some(t) => lang.eq_ignore_ascii_case(t),
                        None => true,
                    };
                    if matches {
                        current_block = Some((marker, count, String::new()));
                    }
                }
            }
        } else if let Some((m, c, mut content)) = current_block.take() {
            content.push_str(line);
            content.push('\n');
            current_block = Some((m, c, content));
        }
    }

    blocks
}

pub struct HeadingIterator<'a> {
    body: &'a str,
    offset: usize,
    fence: Option<(char, usize)>,
}

impl<'a> HeadingIterator<'a> {
    pub fn new(body: &'a str) -> Self {
        Self {
            body,
            offset: 0,
            fence: None,
        }
    }
}

impl<'a> Iterator for HeadingIterator<'a> {
    type Item = HeadingItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < self.body.len() {
            let slice = &self.body[self.offset..];
            let line_len = slice.find('\n').map(|idx| idx + 1).unwrap_or(slice.len());
            let raw = &slice[..line_len];
            let current_offset = self.offset;
            self.offset += line_len;

            let line = raw.trim_end_matches(['\n', '\r']);
            let trimmed = line.trim_start();

            if let Some((marker, count, _)) = is_fence_line(trimmed) {
                match self.fence {
                    Some((fm, fc)) if fm == marker && count >= fc => self.fence = None,
                    None => self.fence = Some((marker, count)),
                    _ => {}
                }
                continue;
            }

            if self.fence.is_some() {
                continue;
            }

            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level > 0 && level <= 6 && trimmed.as_bytes().get(level) == Some(&b' ') {
                let text = trimmed[level..].trim().trim_matches('#').trim();
                return Some(HeadingItem {
                    level,
                    text,
                    offset: current_offset,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_section_ignores_fenced_headings() {
        let body = "# Top\n\n```markdown\n## Implementation evidence\ninside fence\n```\n\n## Implementation evidence\nactual content\n";
        let section = find_section(body, "Implementation evidence", 2).unwrap();
        assert_eq!(section.trim(), "actual content");
    }

    #[test]
    fn parse_checklist_extracts_markers_and_waivers() {
        let text = "- [x] Done item\n- [ ] Todo item\n- [~] Waived item | waived=DEBT-001\n- [!] Invalid item\n```\n- [x] Inside fence\n```\n";
        let items = parse_checklist(text);
        assert_eq!(items.len(), 4);
        assert!(items[0].checked);
        assert_eq!(items[0].text, "Done item");
        assert!(!items[1].checked);
        assert_eq!(items[2].waived.as_deref(), Some("DEBT-001"));
        assert_eq!(items[3].marker, "!");
    }

    #[test]
    fn strip_code_fences_and_extract_code_blocks() {
        let body = "Intro\n```rust\nfn main() {}\n```\nOutro";
        let prose = strip_code_fences(body);
        assert_eq!(prose, "Intro\nOutro");

        let blocks = extract_code_blocks(body, Some("rust"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].trim(), "fn main() {}");
    }
}
