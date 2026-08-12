use serde::{Deserialize, Serialize};

use crate::frontmatter::Document;

pub const ROADMAP_STATUSES: &[&str] = &["proposed", "active", "completed"];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoadmapMilestone {
    pub id: String,
    pub title: String,
    pub status: String,
    pub outcome: String,
    pub specs: Vec<String>,
    pub decisions: Vec<String>,
    pub risks: Vec<String>,
    pub depends_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Roadmap {
    pub title: String,
    pub milestones: Vec<RoadmapMilestone>,
}

pub fn parse_roadmap(source: &str) -> Roadmap {
    let (title, body) = match Document::parse(source) {
        Ok(document) => (
            document.value("title").unwrap_or_else(|| "Roadmap".into()),
            document.body,
        ),
        Err(_) => ("Roadmap".into(), source.to_string()),
    };
    let mut milestones = Vec::new();
    let mut current: Option<RoadmapMilestone> = None;
    let mut fence: Option<(char, usize)> = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some((marker, length)) = fence_marker(trimmed) {
            match fence {
                Some((open_marker, open_length))
                    if marker == open_marker && length >= open_length =>
                {
                    fence = None;
                }
                None => fence = Some((marker, length)),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        if let Some((level, heading)) = markdown_heading(trimmed) {
            if matches!(level, 2 | 3) {
                if let Some(milestone) = current.take() {
                    milestones.push(milestone);
                }
                if let Some((id, title)) = milestone_heading(heading) {
                    current = Some(RoadmapMilestone {
                        id,
                        title,
                        ..RoadmapMilestone::default()
                    });
                }
            }
            continue;
        }

        let Some(milestone) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = roadmap_property(trimmed) else {
            continue;
        };
        match key {
            "status" => milestone.status = clean_scalar(value),
            "outcome" => milestone.outcome = clean_scalar(value),
            "depends_on" => milestone.depends_on = Some(clean_scalar(value)),
            "specs" => milestone.specs = parse_reference_items(value, "SPEC-"),
            "decisions" => milestone.decisions = parse_reference_items(value, "ADR-"),
            "risks" => milestone.risks = parse_list_items(value),
            _ => {}
        }
    }
    if let Some(milestone) = current {
        milestones.push(milestone);
    }

    Roadmap { title, milestones }
}

pub fn is_roadmap_status(status: &str) -> bool {
    ROADMAP_STATUSES.contains(&status)
}

fn fence_marker(line: &str) -> Option<(char, usize)> {
    let marker = line.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    (length >= 3).then_some((marker, length))
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if level == 0 || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    Some((level, line[level + 1..].trim()))
}

fn milestone_heading(heading: &str) -> Option<(String, String)> {
    let mut parts = heading.splitn(2, char::is_whitespace);
    let id = parts.next()?.trim();
    let number = id.strip_prefix("M-")?;
    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let title = parts
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches(['-', '–', '—'])
        .trim()
        .to_string();
    Some((id.to_string(), title))
}

fn roadmap_property(line: &str) -> Option<(&str, &str)> {
    let property = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))?;
    let (key, value) = property.split_once(':')?;
    Some((key.trim().trim_matches('`'), value.trim()))
}

fn clean_scalar(value: &str) -> String {
    value.trim().trim_matches('`').trim().to_string()
}

fn parse_list_items(value: &str) -> Vec<String> {
    let bracketed = value
        .match_indices('[')
        .filter_map(|(start, _)| {
            let rest = &value[start + 1..];
            let end = rest.find(']')?;
            Some(&rest[..end])
        })
        .flat_map(|inside| inside.split(','))
        .map(clean_reference_item)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !bracketed.is_empty() {
        return bracketed;
    }
    value
        .split(',')
        .map(clean_reference_item)
        .filter(|item| !item.is_empty() && *item != "(backlog)")
        .map(str::to_string)
        .collect()
}

fn parse_reference_items(value: &str, prefix: &str) -> Vec<String> {
    parse_list_items(value)
        .into_iter()
        .filter_map(|item| {
            item.split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
                .find(|token| {
                    token.strip_prefix(prefix).is_some_and(|number| {
                        !number.is_empty()
                            && number.chars().all(|character| character.is_ascii_digit())
                    })
                })
                .map(str::to_string)
        })
        .collect()
}

fn clean_reference_item(item: &str) -> &str {
    item.trim()
        .trim_matches('`')
        .trim_matches(|character| matches!(character, '[' | ']'))
        .trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_numeric_h2_h3_milestones_outside_fences() {
        let roadmap = parse_roadmap(
            "---\ntitle: Product roadmap\n---\n# Roadmap\n\n## M-01 — One\n- `status`: active\n\n### M-2 - Two\n- `status`: proposed\n\n### M-NN — Placeholder\n\n### M-<number> — Placeholder\n\n### M- — Placeholder\n\n### M0 — Legacy\n\n```markdown\n## M-03 — Example\n```\n~~~\n### M-04 — Example\n~~~\n",
        );
        assert_eq!(roadmap.title, "Product roadmap");
        assert_eq!(
            roadmap
                .milestones
                .iter()
                .map(|milestone| milestone.id.as_str())
                .collect::<Vec<_>>(),
            vec!["M-01", "M-2"]
        );
    }

    #[test]
    fn preserves_unknown_status_and_all_supported_properties() {
        let roadmap = parse_roadmap(
            "## M-7 — Release\n- `status`: completata (2026-08-12)\n- `outcome`: Ship it\n- `specs`: [SPEC-001] delivered; [SPEC-002]\n- `decisions`: [ADR-001]\n- `risks`: [compatibility]\n- `depends_on`: M-6\n",
        );
        let milestone = &roadmap.milestones[0];
        assert_eq!(milestone.status, "completata (2026-08-12)");
        assert_eq!(milestone.specs, ["SPEC-001", "SPEC-002"]);
        assert_eq!(milestone.decisions, ["ADR-001"]);
        assert_eq!(milestone.risks, ["compatibility"]);
        assert_eq!(milestone.depends_on.as_deref(), Some("M-6"));
        assert!(!is_roadmap_status(&milestone.status));
    }
}
