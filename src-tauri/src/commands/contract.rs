use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::commands::parser::{self, fm_string, fm_string_array};
use crate::errors::AppError;
use crate::models::adr::{Adr, AdrStatus};
use crate::models::agent::{AgentProfile, AgentStatus};
use crate::models::handoff::{Handoff, HandoffStatus};
use crate::models::mcp::{McpProposal, McpProposalStatus, McpRecord, McpStatus};
use crate::models::pulse::{ActionItem, MetricCard, PulseData};
use crate::models::review::{Review, ReviewStatus};
use crate::models::spec::{Spec, SpecStatus};
use crate::models::task::{Task, TaskCriteria, TaskStatus};
use crate::models::wiki::{WikiNode, WikiNodeKind, WikiTree};
use crate::models::workspace::{DiagnosticSeverity, KitDiagnostic};

/// Build the full task list by reading all task status directories.
pub fn build_tasks(root: &Path) -> Result<Vec<Task>, AppError> {
    let mut tasks = Vec::new();
    let task_dir = root.join(".lmbrain").join("tasks");

    if !task_dir.exists() {
        return Ok(tasks);
    }

    for status in TaskStatus::all() {
        let dir = task_dir.join(status.as_str());
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                    let id =
                        fm_string(&parsed.frontmatter, "id").unwrap_or_else(|| "UNKNOWN".into());
                    let title = fm_string(&parsed.frontmatter, "title").unwrap_or_else(|| {
                        path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    });
                    let priority = fm_string(&parsed.frontmatter, "priority");
                    let area = fm_string(&parsed.frontmatter, "area");
                    let milestone = fm_string(&parsed.frontmatter, "milestone");
                    let spec = fm_string(&parsed.frontmatter, "spec");
                    let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                    let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                    let tags = fm_string_array(&parsed.frontmatter, "tags");
                    let links = fm_string_array(&parsed.frontmatter, "links");
                    let dependencies = fm_string_array(&parsed.frontmatter, "depends_on");

                    // Parse criteria from body (simple checkbox detection)
                    let criteria = parse_criteria(&parsed.body);
                    let block_reason = if *status == TaskStatus::Blocked {
                        fm_string(&parsed.frontmatter, "block_reason")
                            .or_else(|| extract_block_reason(&parsed.body))
                    } else {
                        None
                    };

                    tasks.push(Task {
                        id,
                        title,
                        status: status.clone(),
                        priority,
                        area,
                        milestone,
                        spec,
                        dependencies,
                        criteria,
                        activity: Vec::new(), // Could be derived from git log
                        block_reason,
                        body: parsed.body,
                        path: path.to_string_lossy().to_string(),
                        created,
                        updated,
                        tags,
                        links,
                    });
                }
            }
        }
    }

    Ok(tasks)
}

/// Build specs from the specs directory.
pub fn build_specs(root: &Path) -> Result<Vec<Spec>, AppError> {
    let mut specs = Vec::new();
    let spec_dir = root.join(".lmbrain").join("specs");

    if !spec_dir.exists() {
        return Ok(specs);
    }

    for status in SpecStatus::all() {
        let dir = spec_dir.join(status.as_str());
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                    let id =
                        fm_string(&parsed.frontmatter, "id").unwrap_or_else(|| "UNKNOWN".into());
                    let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                    let priority = fm_string(&parsed.frontmatter, "priority");
                    let area = fm_string(&parsed.frontmatter, "area");
                    let milestone = fm_string(&parsed.frontmatter, "milestone");
                    let agent = fm_string(&parsed.frontmatter, "recommended_agent");
                    let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                    let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                    let tags = fm_string_array(&parsed.frontmatter, "tags");
                    let links = fm_string_array(&parsed.frontmatter, "links");
                    let related_tasks = fm_string_array(&parsed.frontmatter, "related_tasks");
                    let related_decisions =
                        fm_string_array(&parsed.frontmatter, "related_decisions");

                    specs.push(Spec {
                        id,
                        title,
                        status: status.clone(),
                        priority,
                        area,
                        milestone,
                        recommended_agent: agent,
                        body: parsed.body,
                        path: path.to_string_lossy().to_string(),
                        created,
                        updated,
                        tags,
                        links,
                        related_tasks,
                        related_decisions,
                    });
                }
            }
        }
    }

    Ok(specs)
}

/// Build reviews from the reviews directory.
pub fn build_reviews(root: &Path) -> Result<Vec<Review>, AppError> {
    let mut reviews = Vec::new();
    let review_dir = root.join(".lmbrain").join("reviews");

    if !review_dir.exists() {
        return Ok(reviews);
    }

    for status in [
        ReviewStatus::Pending,
        ReviewStatus::Accepted,
        ReviewStatus::ChangesRequested,
        ReviewStatus::Blocked,
        ReviewStatus::Superseded,
    ] {
        let dir = review_dir.join(status.as_str());
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                    let id =
                        fm_string(&parsed.frontmatter, "id").unwrap_or_else(|| "UNKNOWN".into());
                    let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                    let spec_id = fm_string(&parsed.frontmatter, "spec");
                    let reviewer = fm_string(&parsed.frontmatter, "reviewer");
                    let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                    let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                    let tags = fm_string_array(&parsed.frontmatter, "tags");
                    let links = fm_string_array(&parsed.frontmatter, "links");

                    reviews.push(Review {
                        id,
                        title,
                        status: status.clone(),
                        spec_id,
                        reviewer,
                        findings: Vec::new(),
                        body: parsed.body,
                        path: path.to_string_lossy().to_string(),
                        created,
                        updated,
                        tags,
                        links,
                    });
                }
            }
        }
    }

    Ok(reviews)
}

/// Build ADRs from the decisions directory.
pub fn build_adrs(root: &Path) -> Result<Vec<Adr>, AppError> {
    let mut adrs = Vec::new();
    let dir = root.join(".lmbrain").join("decisions");

    if !dir.exists() {
        return Ok(adrs);
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                let id = match fm_string(&parsed.frontmatter, "id") {
                    Some(val) if val.starts_with("ADR-") => val,
                    _ => continue,
                };
                let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                let status_str =
                    fm_string(&parsed.frontmatter, "status").unwrap_or_else(|| "proposed".into());
                let status = match status_str.as_str() {
                    "accepted" => AdrStatus::Accepted,
                    "superseded" => AdrStatus::Superseded,
                    "deprecated" => AdrStatus::Deprecated,
                    _ => AdrStatus::Proposed,
                };
                let decision_date = fm_string(&parsed.frontmatter, "decision_date");
                let decider = fm_string(&parsed.frontmatter, "decider");
                let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                let tags = fm_string_array(&parsed.frontmatter, "tags");
                let links = fm_string_array(&parsed.frontmatter, "links");

                adrs.push(Adr {
                    id,
                    title,
                    status,
                    decision_date,
                    decider,
                    body: parsed.body,
                    path: path.to_string_lossy().to_string(),
                    created,
                    updated,
                    tags,
                    links,
                });
            }
        }
    }

    adrs.sort_by(|a, b| b.created.cmp(&a.created));
    Ok(adrs)
}

/// Build agent profiles from the agents/profiles directory.
pub fn build_agents(root: &Path) -> Result<Vec<AgentProfile>, AppError> {
    let mut agents = Vec::new();
    let dir = root.join(".lmbrain").join("agents").join("profiles");

    if !dir.exists() {
        return Ok(agents);
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                let id = match fm_string(&parsed.frontmatter, "id") {
                    Some(val) if val.starts_with("AGENT-") => val,
                    _ => continue,
                };
                let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                let status_str =
                    fm_string(&parsed.frontmatter, "status").unwrap_or_else(|| "proposed".into());
                let status = match status_str.as_str() {
                    "active" => AgentStatus::Active,
                    "inactive" => AgentStatus::Inactive,
                    "retired" => AgentStatus::Retired,
                    _ => AgentStatus::Proposed,
                };
                let role = fm_string(&parsed.frontmatter, "role");
                let activation = fm_string(&parsed.frontmatter, "activation");
                let can_implement = parser::fm_bool(&parsed.frontmatter, "can_implement");
                let can_review = parser::fm_bool(&parsed.frontmatter, "can_review");
                let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                let tags = fm_string_array(&parsed.frontmatter, "tags");
                let links = fm_string_array(&parsed.frontmatter, "links");

                agents.push(AgentProfile {
                    id,
                    title,
                    status,
                    role,
                    activation,
                    can_implement,
                    can_review,
                    body: parsed.body,
                    path: path.to_string_lossy().to_string(),
                    created,
                    updated,
                    tags,
                    links,
                });
            }
        }
    }

    Ok(agents)
}

/// Build MCP records from the mcp/specs directory.
pub fn build_mcp_records(root: &Path) -> Result<Vec<McpRecord>, AppError> {
    let mut records = Vec::new();
    let dir = root.join(".lmbrain").join("mcp").join("specs");

    if !dir.exists() {
        return Ok(records);
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                let id = match fm_string(&parsed.frontmatter, "id") {
                    Some(val) if val.starts_with("MCP-") => val,
                    _ => continue,
                };
                let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                let status_str =
                    fm_string(&parsed.frontmatter, "status").unwrap_or_else(|| "specified".into());
                let status = match status_str.as_str() {
                    "active" => McpStatus::Active,
                    "inactive" => McpStatus::Inactive,
                    "deprecated" => McpStatus::Deprecated,
                    _ => McpStatus::Specified,
                };
                let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                let tags = fm_string_array(&parsed.frontmatter, "tags");
                let links = fm_string_array(&parsed.frontmatter, "links");

                records.push(McpRecord {
                    id,
                    title,
                    status,
                    body: parsed.body,
                    path: path.to_string_lossy().to_string(),
                    created,
                    updated,
                    tags,
                    links,
                });
            }
        }
    }

    Ok(records)
}

/// Build MCP proposals from the mcp/proposals directory.
pub fn build_mcp_proposals(root: &Path) -> Result<Vec<McpProposal>, AppError> {
    let mut proposals = Vec::new();
    let dir = root.join(".lmbrain").join("mcp").join("proposals");

    if !dir.exists() {
        return Ok(proposals);
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                let id = match fm_string(&parsed.frontmatter, "id") {
                    Some(val) if val.starts_with("MCP-PROP-") => val,
                    _ => continue,
                };
                let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                let status_str =
                    fm_string(&parsed.frontmatter, "status").unwrap_or_else(|| "proposed".into());
                let status = match status_str.as_str() {
                    "approved" => McpProposalStatus::Approved,
                    "rejected" => McpProposalStatus::Rejected,
                    "implemented" => McpProposalStatus::Implemented,
                    "blocked" => McpProposalStatus::Blocked,
                    _ => McpProposalStatus::Proposed,
                };
                let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                let tags = fm_string_array(&parsed.frontmatter, "tags");
                let links = fm_string_array(&parsed.frontmatter, "links");

                proposals.push(McpProposal {
                    id,
                    title,
                    status,
                    body: parsed.body,
                    path: path.to_string_lossy().to_string(),
                    created,
                    updated,
                    tags,
                    links,
                });
            }
        }
    }

    Ok(proposals)
}

/// Build handoffs from the handoffs/active directory.
pub fn build_handoffs(root: &Path) -> Result<Vec<Handoff>, AppError> {
    let mut handoffs = Vec::new();
    let dir = root.join(".lmbrain").join("handoffs").join("active");

    if !dir.exists() {
        return Ok(handoffs);
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed = parser::parse_markdown_file(&path.to_string_lossy(), &content);

                let id = match fm_string(&parsed.frontmatter, "id") {
                    Some(val) if val.starts_with("HANDOFF-") => val,
                    _ => continue,
                };
                let title = fm_string(&parsed.frontmatter, "title").unwrap_or_default();
                let status_str =
                    fm_string(&parsed.frontmatter, "status").unwrap_or_else(|| "ready".into());
                let status = match status_str.as_str() {
                    "consumed" => HandoffStatus::Consumed,
                    "superseded" => HandoffStatus::Superseded,
                    "archived" => HandoffStatus::Archived,
                    _ => HandoffStatus::Ready,
                };
                let created = fm_string(&parsed.frontmatter, "created").unwrap_or_default();
                let updated = fm_string(&parsed.frontmatter, "updated").unwrap_or_default();
                let tags = fm_string_array(&parsed.frontmatter, "tags");
                let links = fm_string_array(&parsed.frontmatter, "links");

                handoffs.push(Handoff {
                    id,
                    title,
                    status,
                    body: parsed.body,
                    path: path.to_string_lossy().to_string(),
                    created,
                    updated,
                    tags,
                    links,
                });
            }
        }
    }

    Ok(handoffs)
}

/// Build the wiki tree from the .lmbrain directory structure.
pub fn build_wiki_tree(root: &Path) -> Result<WikiTree, AppError> {
    let lmbrain = root.join(".lmbrain");
    if !lmbrain.exists() {
        return Ok(WikiTree {
            root: WikiNode {
                name: ".lmbrain".into(),
                path: ".lmbrain".into(),
                kind: WikiNodeKind::Folder,
                children: Vec::new(),
                count: None,
            },
        });
    }

    let root_node = build_tree_node(&lmbrain, ".lmbrain")?;
    Ok(WikiTree { root: root_node })
}

fn build_tree_node(dir: &Path, relative: &str) -> Result<WikiNode, AppError> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let kind = match name.as_str() {
        "knowledge" => WikiNodeKind::Knowledge,
        "decisions" => WikiNodeKind::Decisions,
        "specs" => WikiNodeKind::Specs,
        "tasks" => WikiNodeKind::Tasks,
        "reviews" => WikiNodeKind::Reviews,
        "handoffs" => WikiNodeKind::Handoffs,
        "agents" => WikiNodeKind::Agents,
        "mcp" => WikiNodeKind::Mcp,
        _ => WikiNodeKind::Folder,
    };

    let mut children = Vec::new();
    let mut file_count = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and non-md files at top level
            if name.starts_with('.') {
                continue;
            }

            let child_relative = format!("{}/{}", relative, name);

            if path.is_dir() {
                let child = build_tree_node(&path, &child_relative)?;
                file_count += child.count.unwrap_or(0);
                children.push(child);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                file_count += 1;
                children.push(WikiNode {
                    name: name.trim_end_matches(".md").to_string(),
                    path: child_relative,
                    kind: WikiNodeKind::File,
                    children: Vec::new(),
                    count: None,
                });
            }
        }
    }

    children.sort_by(|a, b| {
        b.kind
            .ne(&WikiNodeKind::File)
            .cmp(&a.kind.ne(&WikiNodeKind::File))
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(WikiNode {
        name,
        path: relative.to_string(),
        kind,
        children,
        count: Some(file_count),
    })
}

/// Build pulse data from all parsed artifacts.
pub fn build_pulse_data(
    root: &Path,
    tasks: &[Task],
    specs: &[Spec],
    _reviews: &[Review],
    _adrs: &[Adr],
    handoffs: &[Handoff],
) -> Result<PulseData, AppError> {
    // Read STATUS.md for focus and milestone
    let status_path = root.join(".lmbrain").join("STATUS.md");
    let (focus, milestone) = if let Ok(content) = std::fs::read_to_string(&status_path) {
        (extract_focus(&content), extract_milestone(&content))
    } else {
        (None, None)
    };

    let ready_specs = specs
        .iter()
        .filter(|s| s.status == SpecStatus::Ready)
        .count();
    let in_progress = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();
    let in_review = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Review)
        .count();
    let blocked = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Blocked)
        .count();
    let _ready_handoff_count = handoffs
        .iter()
        .filter(|h| h.status == HandoffStatus::Ready)
        .count();

    let metrics = vec![
        MetricCard {
            label: "Ready for handoff".into(),
            count: ready_specs,
            accent: "#7c6cf6".into(),
        },
        MetricCard {
            label: "In progress".into(),
            count: in_progress,
            accent: "#5b8def".into(),
        },
        MetricCard {
            label: "Awaiting review".into(),
            count: in_review,
            accent: "#e0a23a".into(),
        },
        MetricCard {
            label: "Blocked".into(),
            count: blocked,
            accent: "#e0584a".into(),
        },
    ];

    let actions = specs
        .iter()
        .filter(|s| s.status == SpecStatus::Ready)
        .take(3)
        .map(|s| ActionItem {
            title: format!(
                "Start {} on {}",
                s.recommended_agent.as_deref().unwrap_or("specialist"),
                s.id
            ),
            description: "Spec is ready — copy the handoff prompt and launch the agent manually."
                .to_string(),
            action_type: "handoff".into(),
            spec_id: Some(s.id.clone()),
            agent: s.recommended_agent.clone(),
        })
        .collect();

    let blockers: Vec<ActionItem> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Blocked)
        .take(3)
        .map(|t| ActionItem {
            title: format!("Unblock {} — {}", t.id, t.title),
            description: t
                .block_reason
                .clone()
                .unwrap_or_else(|| "Blocked by dependency".into()),
            action_type: "blocker".into(),
            spec_id: t.spec.clone(),
            agent: None,
        })
        .collect();

    let ready_handoff_list: Vec<crate::models::handoff::Handoff> = handoffs
        .iter()
        .filter(|h| h.status == HandoffStatus::Ready)
        .cloned()
        .collect();

    let active_handoff = handoffs
        .iter()
        .find(|h| h.status == HandoffStatus::Ready)
        .cloned();

    Ok(PulseData {
        focus,
        milestone,
        milestone_progress: None,
        milestone_due: None,
        metrics,
        actions,
        blockers,
        recent_activity: Vec::new(),
        ready_handoffs: ready_handoff_list,
        active_handoff,
    })
}

pub fn extract_focus_for_test(content: &str) -> Option<String> {
    extract_section_after_heading(content, "## Current focus")
}

pub fn extract_milestone_for_test(content: &str) -> Option<String> {
    extract_section_after_heading(content, "## Current milestone")
}

fn extract_focus(content: &str) -> Option<String> {
    extract_focus_for_test(content)
}

fn extract_milestone(content: &str) -> Option<String> {
    extract_milestone_for_test(content)
}

/// Extract the first non-empty line after a given heading in markdown content.
fn extract_section_after_heading(content: &str, heading: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == heading {
            // Look at subsequent lines for the first non-empty content
            for next_line in lines.iter().skip(i + 1) {
                let trimmed = next_line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("##") {
                    return Some(trimmed.to_string());
                }
                if trimmed.starts_with("##") {
                    break; // Next section reached
                }
            }
        }
    }
    None
}

fn parse_criteria(body: &str) -> Vec<TaskCriteria> {
    let mut criteria = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(text) = trimmed.strip_prefix("- [x] ") {
            criteria.push(TaskCriteria {
                text: text.to_string(),
                completed: true,
            });
        } else if let Some(text) = trimmed.strip_prefix("- [ ] ") {
            criteria.push(TaskCriteria {
                text: text.to_string(),
                completed: false,
            });
        }
    }
    criteria
}

fn extract_block_reason(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("blocked")
            || trimmed.to_lowercase().starts_with("blocker")
        {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Build a wikilink index: for each .md file under .lmbrain/, parse its
/// wikilinks and record which pages link to which target.
/// Returns a map of target → list of source paths.
pub fn build_wikilink_index(root: &Path) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    let lmbrain = root.join(".lmbrain");

    if !lmbrain.exists() {
        return index;
    }

    // Walk all .md files recursively
    if let Ok(entries) = walk_md_files(&lmbrain) {
        for file_path in entries {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let parsed = parser::parse_markdown_file(&file_path.to_string_lossy(), &content);
                let source = file_path
                    .strip_prefix(&lmbrain)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                for link in &parsed.wikilinks {
                    index
                        .entry(link.to_lowercase())
                        .or_default()
                        .push(source.clone());
                }
            }
        }
    }

    index
}

/// Scan all .md files under .lmbrain/ for malformed frontmatter and
/// status-directory/frontmatter mismatches.
pub fn build_diagnostics(root: &Path) -> Vec<KitDiagnostic> {
    let mut diagnostics = Vec::new();
    let lmbrain = root.join(".lmbrain");
    if !lmbrain.exists() {
        return diagnostics;
    }

    if let Ok(entries) = walk_md_files(&lmbrain) {
        for file_path in entries {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let parsed = parser::parse_markdown_file(&file_path.to_string_lossy(), &content);

                // Report parse diagnostics
                for d in &parsed.diagnostics {
                    diagnostics.push(KitDiagnostic {
                        message: d.clone(),
                        severity: DiagnosticSeverity::Warning,
                        path: Some(
                            file_path
                                .strip_prefix(&lmbrain)
                                .ok()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default(),
                        ),
                    });
                }

                // Check status directory vs frontmatter status
                // Path structure: .lmbrain/<artifact_type>/<status_dir>/<id>.md
                // e.g. .lmbrain/specs/ready/SPEC-001.md
                // artifact_type = "specs", status_dir = "ready"
                let status = fm_string(&parsed.frontmatter, "status");
                if let Some(fm_status) = status {
                    if let Some(parent_dir) = file_path.parent() {
                        let status_dir = parent_dir
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // Get grandparent to determine artifact type
                        if let Some(grandparent) = parent_dir.parent() {
                            let artifact_type = grandparent
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();

                            // Only check for specs, tasks, reviews (status-directory artifacts)
                            if artifact_type == "specs"
                                || artifact_type == "tasks"
                                || artifact_type == "reviews"
                            {
                                // The status_dir should match the frontmatter status
                                if status_dir != fm_status {
                                    diagnostics.push(KitDiagnostic {
                                        message: format!(
                                            "Status mismatch: file is in '{}/{}' but frontmatter status is '{}'",
                                            artifact_type, status_dir, fm_status
                                        ),
                                        severity: DiagnosticSeverity::Warning,
                                        path: Some(
                                            file_path
                                                .strip_prefix(&lmbrain)
                                                .ok()
                                                .map(|p| p.to_string_lossy().to_string())
                                                .unwrap_or_default(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    diagnostics
}

fn walk_md_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_md_files(&path)?);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// Search .lmbrain markdown content for a query string.
/// Returns matching file paths with context snippets.
pub fn search_content(root: &Path, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let lmbrain = root.join(".lmbrain");
    if !lmbrain.exists() || query.is_empty() {
        return results;
    }

    let query_lower = query.to_lowercase();

    if let Ok(entries) = walk_md_files(&lmbrain) {
        for file_path in entries {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if content.to_lowercase().contains(&query_lower) {
                    let relative = file_path
                        .strip_prefix(&lmbrain)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Extract a snippet around the first match
                    let snippet = content
                        .lines()
                        .find(|l| l.to_lowercase().contains(&query_lower))
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    results.push(SearchResult {
                        path: relative,
                        snippet: truncate(&snippet, 120),
                    });
                }
            }
        }
    }

    results
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub snippet: String,
}
