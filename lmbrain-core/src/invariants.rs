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
    let criteria = body
        .lines()
        .filter(|line| line.trim_start().starts_with("- ["))
        .collect::<Vec<_>>();

    !criteria.is_empty()
        && criteria.iter().all(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]")
        })
        && body
            .split("## Evidence")
            .nth(1)
            .is_some_and(|value| !value.trim().is_empty())
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
        Some("specs") | Some("reviews") => {
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
