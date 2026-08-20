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

pub fn extract_waived_debt_id(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("- [~]") {
        return None;
    }
    let idx = trimmed.find("waived=")?;
    let rest = &trimmed[idx + "waived=".len()..];
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

/// How a single acceptance criterion reads to the tools.
///
/// Until 4.2.2 an unmet criterion was invisible outside the `spec_done`
/// invariant, so an agent that honestly declared an impediment got exactly the
/// same silence as one that ticked a box it could not verify (KIT-NOTE-002).
/// Classifying the marker makes the difference legible to `lmbrain_validate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceCriterionState {
    /// `- [x]`: satisfied.
    Met,
    /// `- [~] ... | waived=DEBT-xxx` backed by an existing debt.
    Waived(String),
    /// `- [ ]`: declared and not satisfied.
    Unmet,
    /// A marker the contract does not define, such as an invented `- [!]`.
    /// Every transition treats it as unmet; saying so is the point.
    UnknownMarker(String),
    /// `- [~]` without a parseable `| waived=DEBT-xxx` reference.
    WaiverMalformed,
    /// `- [~] ... | waived=DEBT-xxx` naming a debt that does not exist.
    WaiverDebtMissing(String),
}

impl AcceptanceCriterionState {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Met | Self::Waived(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptanceCriterion {
    /// 1-based position within the section, so a diagnostic can name the
    /// criterion even when two share the same prose.
    pub position: usize,
    pub text: String,
    pub state: AcceptanceCriterionState,
}

/// Classifies every acceptance criterion of a spec body. An absent section
/// yields an empty list: whether that is itself a problem is a caller's
/// decision, not this function's.
pub fn classify_acceptance_criteria(root: &Path, body: &str) -> Vec<AcceptanceCriterion> {
    let Some(criteria_section) = markdown_section(body, &["acceptance criteria"]) else {
        return Vec::new();
    };
    criteria_section
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("- ["))
        .enumerate()
        .map(|(index, line)| AcceptanceCriterion {
            position: index + 1,
            text: criterion_text(line),
            state: classify_criterion(root, line),
        })
        .collect()
}

fn classify_criterion(root: &Path, line: &str) -> AcceptanceCriterionState {
    let Some(marker) = line
        .strip_prefix("- [")
        .and_then(|rest| rest.split_once(']'))
        .map(|(marker, _)| marker)
    else {
        return AcceptanceCriterionState::UnknownMarker(String::new());
    };
    match marker {
        "x" | "X" => AcceptanceCriterionState::Met,
        " " | "" => AcceptanceCriterionState::Unmet,
        "~" => match extract_waived_debt_id(line) {
            None => AcceptanceCriterionState::WaiverMalformed,
            Some(debt_id) => {
                if debt_exists(root, debt_id) {
                    AcceptanceCriterionState::Waived(debt_id.to_string())
                } else {
                    AcceptanceCriterionState::WaiverDebtMissing(debt_id.to_string())
                }
            }
        },
        other => AcceptanceCriterionState::UnknownMarker(other.to_string()),
    }
}

fn criterion_text(line: &str) -> String {
    line.split_once(']')
        .map(|(_, rest)| rest.trim())
        .unwrap_or(line)
        .to_string()
}

fn debt_exists(root: &Path, debt_id: &str) -> bool {
    scan(root.join(".lmbrain/debts")).iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(debt_id))
    })
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
            trimmed.starts_with("- [x]")
                || trimmed.starts_with("- [X]")
                || extract_waived_debt_id(trimmed).is_some()
        })
        && markdown_section(body, &["implementation evidence", "evidence"])
            .is_some_and(has_evidence_content)
}

pub fn waived_findings_are_valid(root: &Path, body: &str) -> Result<(), String> {
    let Some(criteria_section) = markdown_section(body, &["acceptance criteria"]) else {
        return Ok(());
    };
    for line in criteria_section.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- [~]") {
            let Some(debt_id) = extract_waived_debt_id(trimmed) else {
                return Err(format!(
                    "waived criterion '{trimmed}' must include '| waived=DEBT-xxx'"
                ));
            };
            if !debt_exists(root, debt_id) {
                return Err(format!(
                    "waived criterion references non-existent debt '{debt_id}'"
                ));
            }
        }
    }
    Ok(())
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

/// A spec becomes `ready` only with a valid Lead-owned implementation estimate.
/// Legacy specs already past `ready` are never rewritten: they surface a
/// diagnostic instead, so this gate only applies to the transition itself.
pub fn spec_effort_is_declared(document: &Document) -> Result<(), String> {
    let raw_tier = document.value("capability_tier").unwrap_or_default();
    if raw_tier.trim().is_empty() {
        return Err(format!(
            "a ready spec requires `capability_tier` (one of {})",
            crate::taxonomy::capability_tiers().join(", ")
        ));
    }
    let Some(tier) = crate::taxonomy::normalize_capability_tier(&raw_tier) else {
        return Err(format!(
            "unknown capability tier `{raw_tier}`; expected one of {}",
            crate::taxonomy::capability_tiers().join(", ")
        ));
    };

    let raw_level = document.value("thinking_level").unwrap_or_default();
    if raw_level.trim().is_empty() {
        return Err(format!(
            "a ready spec requires `thinking_level` (one of {})",
            crate::taxonomy::thinking_levels().join(", ")
        ));
    }
    let Some(level) = crate::taxonomy::normalize_thinking_level(&raw_level) else {
        return Err(format!(
            "unknown thinking level `{raw_level}`; expected one of {}",
            crate::taxonomy::thinking_levels().join(", ")
        ));
    };

    crate::taxonomy::thinking_level_allowed(&tier, &level)
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

/// Review attribution must never persist a template placeholder (AGENT-XXX
/// and friends) or a profile ID that does not exist: both poison per-agent
/// effectiveness metrics with full confidence (#93 / KIT-NOTE-010).
pub fn implementation_agent_resolves(root: &Path, agent: Option<&str>) -> Result<(), String> {
    agent_reference_resolves(root, "implementation_agent", agent)
}

pub fn agent_reference_resolves(
    root: &Path,
    field: &str,
    agent: Option<&str>,
) -> Result<(), String> {
    let Some(agent) = agent.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if agent.ends_with("-XXX") {
        return Err(format!(
            "{field} '{agent}' is an unreplaced template placeholder; name the AGENT-* profile that did the work"
        ));
    }
    if !scan(root.join(".lmbrain/agents/profiles"))
        .iter()
        .any(|path| read(path, "id").as_deref() == Some(agent))
    {
        return Err(format!(
            "{field} '{agent}' does not resolve to an existing AGENT-* profile"
        ));
    }
    Ok(())
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
        Some("specs") | Some("reviews") | Some("skills") | Some("debts") => {
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
                if path.file_name().and_then(|name| name.to_str()) == Some("templates") {
                    continue;
                }
                out.extend(scan(path));
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    out
}

/// Supersession must agree on both sides (issue #48): the superseding decision
/// declares the predecessor, the predecessor names its successor, and the
/// predecessor is no longer presented as authoritative.
///
/// A *proposed* decision declaring `supersedes` is a legitimate pending claim
/// and passes: supersession only takes effect when the successor is accepted.
pub fn supersession_is_consistent(
    superseding_id: &str,
    superseding_status: &str,
    superseded_id: &str,
    superseded_status: &str,
    superseded_superseded_by: &[String],
) -> Result<(), String> {
    if superseding_status != "accepted" {
        return Ok(());
    }
    if superseded_status != "superseded" {
        return Err(format!(
            "{superseded_id} is still `{superseded_status}` although {superseding_id} supersedes it"
        ));
    }
    if !superseded_superseded_by
        .iter()
        .any(|value| value.trim().eq_ignore_ascii_case(superseding_id))
    {
        return Err(format!(
            "{superseded_id} does not record {superseding_id} in `superseded_by`"
        ));
    }
    Ok(())
}
