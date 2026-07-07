use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::frontmatter::Document;

pub fn spec_has_accepted_review(root: &Path, spec_id: &str) -> bool {
    scan(root.join(".lmbrain/reviews/accepted"))
        .iter()
        .any(|path| read(path, "spec").as_deref() == Some(spec_id))
}

pub fn criteria_complete_with_evidence(body: &str) -> bool {
    let Some(criteria_section) = markdown_section(body, &["acceptance criteria"]) else {
        return false;
    };
    let criteria = criteria_section
        .lines()
        .filter(|line| line.trim_start().starts_with("- ["))
        .collect::<Vec<_>>();

    !criteria.is_empty()
        && criteria.iter().all(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]")
        })
        && markdown_section(body, &["implementation evidence", "evidence"])
            .is_some_and(has_evidence_content)
}

fn markdown_section<'a>(body: &'a str, headings: &[&str]) -> Option<&'a str> {
    let mut section_start = None;
    let mut section_level = 0usize;

    for (offset, line) in line_offsets(body) {
        let Some((level, text)) = heading(line) else {
            continue;
        };

        if let Some(start) = section_start {
            if level <= section_level {
                return Some(&body[start..offset]);
            }
        }

        if section_start.is_none()
            && headings
                .iter()
                .any(|candidate| normalize_heading(text) == *candidate)
        {
            section_start = Some(offset + line.len());
            section_level = level;
        }
    }

    section_start.map(|start| &body[start..])
}

fn line_offsets(body: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    body.split_inclusive('\n').map(move |line| {
        let current = offset;
        offset += line.len();
        (current, line.trim_end_matches(['\r', '\n']))
    })
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let text = trimmed[level..].trim_start();
    if text.is_empty() {
        return None;
    }
    Some((level, text.trim_matches('#').trim()))
}

fn normalize_heading(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn has_evidence_content(section: &str) -> bool {
    section.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && trimmed != "> Filled in by the specialist after completion."
    })
}

pub fn single_ready_handoff(root: &Path, excluding: Option<&Path>) -> bool {
    scan(root.join(".lmbrain/handoffs/active"))
        .into_iter()
        .filter(|path| Some(path.as_path()) != excluding)
        .filter(|path| read(path, "status").as_deref() == Some("ready"))
        .count()
        == 0
}

pub fn recommended_agent_resolves(root: &Path, agent: Option<&str>) -> bool {
    let Some(agent) = agent.filter(|value| !value.trim().is_empty()) else {
        return true;
    };

    !agent.ends_with("-XXX")
        && scan(root.join(".lmbrain/agents/profiles"))
            .iter()
            .any(|path| read(path, "id").as_deref() == Some(agent))
}

pub fn unique_ids(root: &Path) -> bool {
    let mut seen = HashSet::new();
    scan(root.join(".lmbrain"))
        .into_iter()
        .filter_map(|path| read(&path, "id"))
        .all(|id| seen.insert(id))
}

pub fn folder_matches_status(path: &Path) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(document) = Document::parse(&source) else {
        return false;
    };

    match path
        .parent()
        .and_then(Path::parent)
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
    {
        Some("specs") | Some("reviews") | Some("skills") => {
            path.parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                == document.value("status").as_deref()
        }
        _ => true,
    }
}

fn read(path: &Path, key: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| Document::parse(&source).ok())
        .and_then(|document| document.value(key))
}

fn scan(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(scan(path));
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    out
}
