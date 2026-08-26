//! Keeps `skills/registry.md` in agreement with the skill artifacts.
//!
//! `skill_activate` used to move the artifact into `skills/active/` and stamp
//! the activity trail without writing the registry row, leaving a skill active
//! on disk and invisible to every reader of the registry table (KIT-NOTE-001).
//! The sync here derives the row entirely from the artifact so a governed
//! skill transition can restore agreement instead of asking for a hand edit.

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{
    error::CoreError,
    frontmatter::{atomic_write, Document},
    mutation_lock::WorkspaceLock,
    path::PathGuard,
};

const REGISTRY_RELATIVE: &str = ".lmbrain/skills/registry.md";
const SKILL_STATUS_DIRS: &[&str] = &["proposed", "active", "retired"];

const REGISTRY_HEADER: &str =
    "| ID | Skill | Status | Kind | Risk | Applies to | Definition |";
const REGISTRY_SEPARATOR: &str = "| --- | --- | --- | --- | --- | --- | --- |";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRegistrySync {
    pub registry_path: String,
    pub skill_id: String,
    /// `inserted`, `updated`, or `unchanged`.
    pub action: String,
}

#[derive(Debug, Clone)]
struct SkillRow {
    id: String,
    title: String,
    status: String,
    kind: String,
    risk: String,
    applies_to: String,
    definition: String,
}

fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ").trim().to_string()
}

fn skill_row(document: &Document, relative_definition: &str) -> Result<SkillRow, CoreError> {
    let required = |key: &str| {
        document
            .value(key)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| CoreError::Missing(key.into()))
    };
    let applies_to = document
        .fields()
        .get("applies_to")
        .and_then(|value| value.as_array().cloned())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    Ok(SkillRow {
        id: required("id")?,
        title: required("title")?,
        status: required("status")?,
        kind: document.value("kind").unwrap_or_default(),
        risk: document.value("risk").unwrap_or_default(),
        applies_to,
        definition: relative_definition.to_string(),
    })
}

fn render_row(row: &SkillRow) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | `{}` |",
        cell(&row.id),
        cell(&row.title),
        cell(&row.status),
        cell(&row.kind),
        cell(&row.risk),
        cell(&row.applies_to),
        row.definition,
    )
}

fn row_id(line: &str) -> Option<String> {
    let mut cells = line.trim().trim_start_matches('|').split('|');
    let first = cells.next()?.trim();
    (!first.is_empty() && first != "ID" && !first.starts_with("---")).then(|| first.to_string())
}

/// Locates the artifact for `skill_id` under the skill status directories.
fn find_skill_artifact(lmbrain: &Path, skill_id: &str) -> Option<PathBuf> {
    for status in SKILL_STATUS_DIRS {
        let directory = lmbrain.join("skills").join(status);
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let matches = name.ends_with(".md")
                && (name == format!("{skill_id}.md")
                    || name.starts_with(&format!("{skill_id}-")));
            if path.is_file() && matches {
                return Some(path);
            }
        }
    }
    None
}

/// Upserts the registry row for `skill_id` from its artifact's frontmatter.
///
/// The row is derived wholly from the artifact, so re-running the sync after a
/// divergence restores agreement without interpreting the existing table.
pub fn sync_skill_registry(
    root: impl AsRef<Path>,
    skill_id: &str,
) -> Result<SkillRegistrySync, CoreError> {
    let guard = PathGuard::new(root)?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let lmbrain = guard.root().join(".lmbrain");
    let artifact_path = find_skill_artifact(&lmbrain, skill_id)
        .ok_or_else(|| CoreError::NotFound(format!("skill artifact for '{skill_id}'")))?;
    let document = Document::parse(&fs::read_to_string(&artifact_path)?)?;
    let status_dir = artifact_path
        .parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_name = artifact_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    let row = skill_row(&document, &format!("skills/{status_dir}/{file_name}"))?;
    if row.id != skill_id {
        return Err(CoreError::Invariant(format!(
            "skill artifact '{file_name}' declares id '{}' but '{skill_id}' was requested",
            row.id
        )));
    }

    let registry_path = guard.root().join(REGISTRY_RELATIVE);
    let source = fs::read_to_string(&registry_path).map_err(|_| {
        CoreError::NotFound(format!("skill registry at {REGISTRY_RELATIVE}"))
    })?;

    let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
    let header_index = lines
        .iter()
        .position(|line| line.trim().starts_with("| ID |"));
    let rendered = render_row(&row);
    let action;
    match header_index {
        Some(header) => {
            // Rows follow the header and separator until the first non-row line.
            let mut end = header + 1;
            if lines.get(end).is_some_and(|line| line.trim().starts_with("| ---")) {
                end += 1;
            }
            let rows_start = end;
            while lines.get(end).is_some_and(|line| line.trim().starts_with('|')) {
                end += 1;
            }
            let existing = (rows_start..end)
                .find(|index| row_id(&lines[*index]).as_deref() == Some(skill_id));
            match existing {
                Some(index) if lines[index] == rendered => {
                    return Ok(SkillRegistrySync {
                        registry_path: REGISTRY_RELATIVE.into(),
                        skill_id: skill_id.into(),
                        action: "unchanged".into(),
                    });
                }
                Some(index) => {
                    lines[index] = rendered;
                    action = "updated";
                }
                None => {
                    lines.insert(end, rendered);
                    action = "inserted";
                }
            }
        }
        None => {
            // A registry without a table gets one appended in the kit shape.
            if !lines.last().is_some_and(|line| line.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(REGISTRY_HEADER.into());
            lines.push(REGISTRY_SEPARATOR.into());
            lines.push(rendered);
            action = "inserted";
        }
    }

    let mut content = lines.join("\n");
    if source.ends_with('\n') || !content.ends_with('\n') {
        content.push('\n');
    }
    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = if content.starts_with("---") {
        match Document::parse(&content) {
            Ok(mut registry_document) => {
                registry_document.set("updated", &today);
                registry_document.render()
            }
            Err(_) => content,
        }
    } else {
        content
    };
    atomic_write(&registry_path, &content)?;

    Ok(SkillRegistrySync {
        registry_path: REGISTRY_RELATIVE.into(),
        skill_id: skill_id.into(),
        action: action.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKILL: &str = "---\nid: SKILL-004\ntitle: \"Converge two derivations\"\nstatus: active\nkind: verification\nrisk: low\napplies_to: [AGENT-001, AGENT-002]\ncreated: 2026-08-26\nupdated: 2026-08-26\ntags: [verification]\nlinks: []\n---\n# Converge two derivations\n";

    const REGISTRY: &str = "---\ntitle: Skill registry\nupdated: 2026-07-07\n---\n\n# Skill Registry\n\n| ID | Skill | Status | Kind | Risk | Applies to | Definition |\n| --- | --- | --- | --- | --- | --- | --- |\n| SKILL-003 | Delivery boundaries | active | process | low | AGENT-001 | `skills/active/SKILL-003-delivery-boundaries.md` |\n\nRecord only reusable project procedures.\n";

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let skills = dir.path().join(".lmbrain/skills/active");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::write(
            skills.join("SKILL-004-converge-two-derivations.md"),
            SKILL,
        )
        .unwrap();
        std::fs::write(dir.path().join(".lmbrain/skills/registry.md"), REGISTRY).unwrap();
        dir
    }

    #[test]
    fn an_activated_skill_missing_from_the_registry_gains_its_row() {
        let dir = fixture();
        let sync = sync_skill_registry(dir.path(), "SKILL-004").unwrap();
        assert_eq!(sync.action, "inserted");
        let registry =
            std::fs::read_to_string(dir.path().join(".lmbrain/skills/registry.md")).unwrap();
        assert!(registry.contains(
            "| SKILL-004 | Converge two derivations | active | verification | low | AGENT-001, AGENT-002 | `skills/active/SKILL-004-converge-two-derivations.md` |"
        ));
        // The pre-existing row and the trailing prose survive the upsert.
        assert!(registry.contains("| SKILL-003 | Delivery boundaries |"));
        assert!(registry.contains("Record only reusable project procedures."));
    }

    #[test]
    fn a_stale_row_is_rewritten_from_the_artifact() {
        let dir = fixture();
        let registry_path = dir.path().join(".lmbrain/skills/registry.md");
        let stale = REGISTRY.replace(
            "`skills/active/SKILL-003-delivery-boundaries.md` |\n",
            "`skills/active/SKILL-003-delivery-boundaries.md` |\n| SKILL-004 | Converge two derivations | proposed | verification | low | AGENT-001 | `skills/proposed/SKILL-004-converge-two-derivations.md` |\n",
        );
        std::fs::write(&registry_path, stale).unwrap();
        let sync = sync_skill_registry(dir.path(), "SKILL-004").unwrap();
        assert_eq!(sync.action, "updated");
        let registry = std::fs::read_to_string(&registry_path).unwrap();
        assert!(registry.contains("| SKILL-004 | Converge two derivations | active |"));
        assert!(!registry.contains("| SKILL-004 | Converge two derivations | proposed |"));
    }

    #[test]
    fn a_row_already_in_agreement_reports_unchanged() {
        let dir = fixture();
        sync_skill_registry(dir.path(), "SKILL-004").unwrap();
        let sync = sync_skill_registry(dir.path(), "SKILL-004").unwrap();
        assert_eq!(sync.action, "unchanged");
    }

    #[test]
    fn a_missing_artifact_is_an_error_not_a_silent_no_op() {
        let dir = fixture();
        let error = sync_skill_registry(dir.path(), "SKILL-099").unwrap_err();
        assert!(error.to_string().contains("SKILL-099"));
    }
}
