use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::commands::parser::{self, fm_string, fm_string_array};
use crate::errors::AppError;
use crate::models::adr::{Adr, AdrStatus};
use crate::models::agent::{AgentProfile, AgentProposal, AgentProposalStatus, AgentStatus};
use crate::models::file::ParsedDocument;
use crate::models::handoff::{Handoff, HandoffStatus};
use crate::models::mcp::{McpProposal, McpProposalStatus, McpRecord, McpStatus};
use crate::models::pulse::{ActionItem, MetricCard, PulseData};
use crate::models::review::{Review, ReviewStatus};
use crate::models::roadmap::{
    Milestone, MilestoneAdrSummary, MilestoneDetail, MilestoneOverview, MilestoneReviewSummary,
    MilestoneSpecSummary, Roadmap,
};
use crate::models::spec::{Spec, SpecStatus};
use crate::models::wiki::{WikiNode, WikiNodeKind, WikiTree};
use crate::models::workspace::{DiagnosticSeverity, KitDiagnostic};

const WIKI_CONTENT_DIRS: &[(&str, WikiNodeKind)] = &[
    ("decisions", WikiNodeKind::Decisions),
    ("knowledge", WikiNodeKind::Knowledge),
    ("specs", WikiNodeKind::Specs),
];

#[derive(Debug, Clone)]
struct CommonFields {
    id: String,
    title: String,
    body: String,
    path: String,
    created: String,
    updated: String,
    tags: Vec<String>,
    links: Vec<String>,
    malformed: Option<bool>,
}

/// Build specs from the specs directory.
pub fn build_specs(root: &Path) -> Result<Vec<Spec>, AppError> {
    build_status_dir_artifacts(
        &root.join(".lmbrain/specs"),
        SpecStatus::all(),
        |status| status.as_str(),
        |status, parsed, path| {
            let common = common_fields(parsed, path);
            Ok(Spec {
                id: common.id,
                title: common.title,
                status: status.clone(),
                priority: fm_string(&parsed.frontmatter, "priority"),
                area: fm_string(&parsed.frontmatter, "area"),
                milestone: fm_string(&parsed.frontmatter, "milestone"),
                recommended_agent: fm_string(&parsed.frontmatter, "recommended_agent"),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                related_tasks: fm_string_array(&parsed.frontmatter, "related_tasks"),
                related_decisions: fm_string_array(&parsed.frontmatter, "related_decisions"),
                malformed: common.malformed,
            })
        },
    )
}

/// Build reviews from the reviews directory.
pub fn build_reviews(root: &Path) -> Result<Vec<Review>, AppError> {
    let statuses = [
        ReviewStatus::Pending,
        ReviewStatus::Accepted,
        ReviewStatus::ChangesRequested,
        ReviewStatus::Blocked,
        ReviewStatus::Superseded,
    ];
    build_status_dir_artifacts(
        &root.join(".lmbrain/reviews"),
        &statuses,
        |status| status.as_str(),
        |status, parsed, path| {
            let common = common_fields(parsed, path);
            Ok(Review {
                id: common.id,
                title: common.title,
                status: status.clone(),
                spec_id: fm_string(&parsed.frontmatter, "spec"),
                reviewer: fm_string(&parsed.frontmatter, "reviewer"),
                findings: Vec::new(),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

/// Build ADRs from the decisions directory.
pub fn build_adrs(root: &Path) -> Result<Vec<Adr>, AppError> {
    let mut adrs = build_flat_artifacts(
        &root.join(".lmbrain/decisions"),
        Some("ADR-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(Adr {
                id: common.id,
                title: common.title,
                status: parse_adr_status(&parsed.frontmatter),
                decision_date: fm_string(&parsed.frontmatter, "decision_date"),
                decider: fm_string(&parsed.frontmatter, "decider"),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )?;
    adrs.sort_by(|left, right| right.created.cmp(&left.created));
    Ok(adrs)
}

/// Build agent profiles from the agents/profiles directory.
pub fn build_agents(root: &Path) -> Result<Vec<AgentProfile>, AppError> {
    build_flat_artifacts(
        &root.join(".lmbrain/agents/profiles"),
        Some("AGENT-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(AgentProfile {
                id: common.id,
                title: common.title,
                mnemonic_name: fm_string(&parsed.frontmatter, "mnemonic_name"),
                status: parse_agent_status(&parsed.frontmatter),
                role: fm_string(&parsed.frontmatter, "role"),
                activation: fm_string(&parsed.frontmatter, "activation"),
                can_implement: parser::fm_bool(&parsed.frontmatter, "can_implement"),
                can_review: parser::fm_bool(&parsed.frontmatter, "can_review"),
                // V3 specialization metadata (optional, backward-compatible)
                domains: parser::fm_string_array_opt(&parsed.frontmatter, "domains"),
                primary_files: parser::fm_string_array_opt(&parsed.frontmatter, "primary_files"),
                review_focus: parser::fm_string_array_opt(&parsed.frontmatter, "review_focus"),
                context_pack: fm_string(&parsed.frontmatter, "context_pack"),
                constraints: parser::fm_string_array_opt(&parsed.frontmatter, "constraints"),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

/// Build agent proposals from the agents/proposals directory.
pub fn build_agent_proposals(root: &Path) -> Result<Vec<AgentProposal>, AppError> {
    build_flat_artifacts(
        &root.join(".lmbrain/agents/proposals"),
        Some("AGENT-PROP-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(AgentProposal {
                id: common.id,
                title: common.title,
                status: parse_agent_proposal_status(&parsed.frontmatter),
                proposed_mnemonic_name: fm_string(&parsed.frontmatter, "proposed_mnemonic_name"),
                // V3: proposal type and target profile (optional, backward-compatible)
                proposal_type: fm_string(&parsed.frontmatter, "proposal_type"),
                target_profile: fm_string(&parsed.frontmatter, "target_profile"),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

/// Build MCP records from the mcp/specs directory.
pub fn build_mcp_records(root: &Path) -> Result<Vec<McpRecord>, AppError> {
    build_flat_artifacts(
        &root.join(".lmbrain/mcp/specs"),
        Some("MCP-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(McpRecord {
                id: common.id,
                title: common.title,
                status: parse_mcp_status(&parsed.frontmatter),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

/// Build MCP proposals from the mcp/proposals directory.
pub fn build_mcp_proposals(root: &Path) -> Result<Vec<McpProposal>, AppError> {
    build_flat_artifacts(
        &root.join(".lmbrain/mcp/proposals"),
        Some("MCP-PROP-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(McpProposal {
                id: common.id,
                title: common.title,
                status: parse_mcp_proposal_status(&parsed.frontmatter),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

/// Build handoffs from the handoffs/active directory.
pub fn build_handoffs(root: &Path) -> Result<Vec<Handoff>, AppError> {
    build_flat_artifacts(
        &root.join(".lmbrain/handoffs/active"),
        Some("HANDOFF-"),
        |parsed, path| {
            let common = common_fields(parsed, path);
            Ok(Handoff {
                id: common.id,
                title: common.title,
                status: parse_handoff_status(&parsed.frontmatter),
                body: common.body,
                path: common.path,
                created: common.created,
                updated: common.updated,
                tags: common.tags,
                links: common.links,
                malformed: common.malformed,
            })
        },
    )
}

fn build_status_dir_artifacts<TStatus, TArtifact, FStatus, FMap>(
    root: &Path,
    statuses: &[TStatus],
    status_dir: FStatus,
    mapper: FMap,
) -> Result<Vec<TArtifact>, AppError>
where
    TStatus: Clone,
    FStatus: Fn(&TStatus) -> &'static str,
    FMap: Fn(&TStatus, &ParsedDocument, &Path) -> Result<TArtifact, AppError>,
{
    let mut artifacts = Vec::new();
    if !root.exists() {
        return Ok(artifacts);
    }

    for status in statuses {
        let dir = root.join(status_dir(status));
        for path in read_md_files(&dir)? {
            let parsed = parse_document(&path)?;
            artifacts.push(mapper(status, &parsed, &path)?);
        }
    }

    Ok(artifacts)
}

fn build_flat_artifacts<TArtifact, FMap>(
    dir: &Path,
    required_prefix: Option<&str>,
    mapper: FMap,
) -> Result<Vec<TArtifact>, AppError>
where
    FMap: Fn(&ParsedDocument, &Path) -> Result<TArtifact, AppError>,
{
    let mut artifacts = Vec::new();
    for path in read_md_files(dir)? {
        let parsed = parse_document(&path)?;
        if let Some(prefix) = required_prefix {
            if !fm_string(&parsed.frontmatter, "id").is_some_and(|id| id.starts_with(prefix)) {
                continue;
            }
        }
        artifacts.push(mapper(&parsed, &path)?);
    }
    Ok(artifacts)
}

fn common_fields(parsed: &ParsedDocument, path: &Path) -> CommonFields {
    CommonFields {
        id: fm_string(&parsed.frontmatter, "id").unwrap_or_else(|| "UNKNOWN".into()),
        title: fm_string(&parsed.frontmatter, "title").unwrap_or_default(),
        body: parsed.body.clone(),
        path: path.to_string_lossy().to_string(),
        created: fm_string(&parsed.frontmatter, "created").unwrap_or_default(),
        updated: fm_string(&parsed.frontmatter, "updated").unwrap_or_default(),
        tags: fm_string_array(&parsed.frontmatter, "tags"),
        links: fm_string_array(&parsed.frontmatter, "links"),
        malformed: Some(parsed.malformed),
    }
}

fn parse_document(path: &Path) -> Result<ParsedDocument, AppError> {
    let content = fs::read_to_string(path)
        .map_err(|error| AppError::Io(format!("Failed to read {}: {}", path.display(), error)))?;
    Ok(parser::parse_markdown_file(
        &path.to_string_lossy(),
        &content,
    ))
}

fn read_md_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| {
        AppError::Io(format!(
            "Failed to read directory {}: {}",
            dir.display(),
            error
        ))
    })? {
        let path = entry
            .map_err(|error| AppError::Io(format!("Failed to read directory entry: {error}")))?
            .path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(files)
}

fn scan_md_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| {
        AppError::Io(format!(
            "Failed to read directory {}: {}",
            dir.display(),
            error
        ))
    })? {
        let path = entry
            .map_err(|error| AppError::Io(format!("Failed to read directory entry: {error}")))?
            .path();
        if path.is_dir() {
            files.extend(scan_md_files(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(files)
}

fn parse_adr_status(frontmatter: &HashMap<String, serde_json::Value>) -> AdrStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("accepted") => AdrStatus::Accepted,
        Some("rejected") => AdrStatus::Rejected,
        Some("superseded") => AdrStatus::Superseded,
        Some("deprecated") => AdrStatus::Deprecated,
        _ => AdrStatus::Proposed,
    }
}

fn parse_agent_status(frontmatter: &HashMap<String, serde_json::Value>) -> AgentStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("active") => AgentStatus::Active,
        Some("inactive") => AgentStatus::Inactive,
        Some("retired") => AgentStatus::Retired,
        _ => AgentStatus::Proposed,
    }
}

fn parse_agent_proposal_status(
    frontmatter: &HashMap<String, serde_json::Value>,
) -> AgentProposalStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("approved") => AgentProposalStatus::Approved,
        Some("rejected") => AgentProposalStatus::Rejected,
        _ => AgentProposalStatus::Proposed,
    }
}

fn parse_mcp_status(frontmatter: &HashMap<String, serde_json::Value>) -> McpStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("active") => McpStatus::Active,
        Some("inactive") => McpStatus::Inactive,
        Some("deprecated") => McpStatus::Deprecated,
        _ => McpStatus::Specified,
    }
}

fn parse_mcp_proposal_status(
    frontmatter: &HashMap<String, serde_json::Value>,
) -> McpProposalStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("approved") => McpProposalStatus::Approved,
        Some("rejected") => McpProposalStatus::Rejected,
        Some("implemented") => McpProposalStatus::Implemented,
        Some("blocked") => McpProposalStatus::Blocked,
        _ => McpProposalStatus::Proposed,
    }
}

fn parse_handoff_status(frontmatter: &HashMap<String, serde_json::Value>) -> HandoffStatus {
    match fm_string(frontmatter, "status").as_deref() {
        Some("consumed") => HandoffStatus::Consumed,
        Some("superseded") => HandoffStatus::Superseded,
        Some("archived") => HandoffStatus::Archived,
        _ => HandoffStatus::Ready,
    }
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

    let mut children = Vec::new();
    let mut file_count = 0;
    for (directory, kind) in WIKI_CONTENT_DIRS {
        let path = lmbrain.join(directory);
        if !path.is_dir() {
            continue;
        }
        let child =
            build_tree_node_with_kind(&path, &format!(".lmbrain/{directory}"), kind.clone())?;
        file_count += child.count.unwrap_or(0);
        children.push(child);
    }
    children.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(WikiTree {
        root: WikiNode {
            name: ".lmbrain".into(),
            path: ".lmbrain".into(),
            kind: WikiNodeKind::Folder,
            children,
            count: Some(file_count),
        },
    })
}

fn build_tree_node(dir: &Path, relative: &str) -> Result<WikiNode, AppError> {
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
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

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        let child_relative = format!("{relative}/{name}");
        if path.is_dir() {
            let child = build_tree_node(&path, &child_relative)?;
            file_count += child.count.unwrap_or(0);
            children.push(child);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
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

    children.sort_by(|left, right| {
        right
            .kind
            .ne(&WikiNodeKind::File)
            .cmp(&left.kind.ne(&WikiNodeKind::File))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(WikiNode {
        name,
        path: relative.to_string(),
        kind,
        children,
        count: Some(file_count),
    })
}

fn build_tree_node_with_kind(
    dir: &Path,
    relative: &str,
    kind: WikiNodeKind,
) -> Result<WikiNode, AppError> {
    let mut node = build_tree_node(dir, relative)?;
    node.kind = kind;
    Ok(node)
}

/// Build pulse data from all parsed artifacts.
pub fn build_pulse_data(
    root: &Path,
    specs: &[Spec],
    _reviews: &[Review],
    _adrs: &[Adr],
    handoffs: &[Handoff],
) -> Result<PulseData, AppError> {
    let status_path = root.join(".lmbrain/STATUS.md");
    let (focus, milestone) = if let Ok(content) = fs::read_to_string(&status_path) {
        (extract_focus(&content), extract_milestone(&content))
    } else {
        (None, None)
    };

    let count_status =
        |status: SpecStatus| specs.iter().filter(|spec| spec.status == status).count();
    let metrics = vec![
        MetricCard {
            label: "Ready for handoff".into(),
            count: count_status(SpecStatus::Ready),
            accent: "#7c6cf6".into(),
        },
        MetricCard {
            label: "In progress".into(),
            count: count_status(SpecStatus::Working),
            accent: "#5b8def".into(),
        },
        MetricCard {
            label: "Awaiting review".into(),
            count: count_status(SpecStatus::Review),
            accent: "#e0a23a".into(),
        },
        MetricCard {
            label: "Done".into(),
            count: count_status(SpecStatus::Done),
            accent: "#46b07d".into(),
        },
    ];

    let actions = specs
        .iter()
        .filter(|spec| spec.status == SpecStatus::Ready)
        .take(3)
        .map(|spec| ActionItem {
            title: format!(
                "Start {} on {}",
                spec.recommended_agent.as_deref().unwrap_or("specialist"),
                spec.id
            ),
            description: "Spec is ready — copy the handoff prompt and launch the agent manually."
                .to_string(),
            action_type: "handoff".into(),
            spec_id: Some(spec.id.clone()),
            agent: spec.recommended_agent.clone(),
        })
        .collect();

    let ready_handoffs: Vec<Handoff> = handoffs
        .iter()
        .filter(|handoff| handoff.status == HandoffStatus::Ready)
        .cloned()
        .collect();

    Ok(PulseData {
        focus,
        milestone,
        milestone_progress: None,
        milestone_due: None,
        metrics,
        actions,
        blockers: Vec::new(),
        recent_activity: Vec::new(),
        ready_handoffs: ready_handoffs.clone(),
        active_handoff: ready_handoffs.into_iter().next(),
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

fn extract_section_after_heading(content: &str, heading: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() == heading {
            for next_line in lines.iter().skip(index + 1) {
                let trimmed = next_line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("##") {
                    return Some(trimmed.to_string());
                }
                if trimmed.starts_with("##") {
                    break;
                }
            }
        }
    }
    None
}

/// Build a wikilink index: for each .md file under .lmbrain/, parse its
/// wikilinks and record which pages link to which target.
pub fn build_wikilink_index(root: &Path) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    let lmbrain = root.join(".lmbrain");
    let entries = wiki_content_files(&lmbrain);

    for file_path in entries {
        if let Ok(parsed) = parse_document(&file_path) {
            let source = file_path
                .strip_prefix(&lmbrain)
                .ok()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default();

            for link in &parsed.wikilinks {
                index
                    .entry(link.to_lowercase())
                    .or_default()
                    .push(source.clone());
            }
        }
    }

    index
}

fn wiki_content_files(lmbrain: &Path) -> Vec<PathBuf> {
    WIKI_CONTENT_DIRS
        .iter()
        .filter_map(|(directory, _)| scan_md_files(&lmbrain.join(directory)).ok())
        .flatten()
        .collect()
}

/// Scan all .md files under .lmbrain/ for malformed frontmatter and
/// status-directory/frontmatter mismatches.
pub fn build_diagnostics(root: &Path) -> Vec<KitDiagnostic> {
    let mut diagnostics = Vec::new();
    let lmbrain = root.join(".lmbrain");
    let Ok(entries) = scan_md_files(&lmbrain) else {
        return diagnostics;
    };

    for file_path in entries {
        let Ok(parsed) = parse_document(&file_path) else {
            diagnostics.push(KitDiagnostic {
                message: format!("Failed to read {}", file_path.display()),
                severity: DiagnosticSeverity::Warning,
                path: Some(
                    file_path
                        .strip_prefix(&lmbrain)
                        .ok()
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            });
            continue;
        };

        for diagnostic in &parsed.diagnostics {
            diagnostics.push(KitDiagnostic {
                message: diagnostic.clone(),
                severity: DiagnosticSeverity::Warning,
                path: Some(
                    file_path
                        .strip_prefix(&lmbrain)
                        .ok()
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            });
        }

        if let Some(frontmatter_status) = fm_string(&parsed.frontmatter, "status") {
            if let Some(parent_dir) = file_path.parent() {
                let status_dir = parent_dir
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();

                if let Some(grandparent) = parent_dir.parent() {
                    let artifact_type = grandparent
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if (artifact_type == "specs" || artifact_type == "reviews")
                        && !lmbrain_core::invariants::folder_matches_status(&file_path)
                    {
                        diagnostics.push(KitDiagnostic {
                            message: format!(
                                "Status mismatch: file is in '{}/{}' but frontmatter status is '{}'",
                                artifact_type, status_dir, frontmatter_status
                            ),
                            severity: DiagnosticSeverity::Warning,
                            path: Some(
                                file_path
                                    .strip_prefix(&lmbrain)
                                    .ok()
                                    .map(|path| path.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                            ),
                        });
                    }
                }
            }
        }
    }

    if let Ok(specs) = build_specs(root) {
        let agents = build_agents(root).unwrap_or_default();
        for spec in &specs {
            let Some(agent) = spec
                .recommended_agent
                .as_deref()
                .map(str::trim)
                .filter(|agent| !agent.is_empty())
            else {
                continue;
            };

            if !lmbrain_core::invariants::recommended_agent_resolves(root, Some(agent)) {
                let rel_path = Path::new(&spec.path)
                    .strip_prefix(&lmbrain)
                    .ok()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_else(|| spec.path.clone());
                diagnostics.push(KitDiagnostic {
                    message: format!(
                        "Missing reference: spec {} recommends agent '{}', which is not an existing agent profile",
                        spec.id, agent
                    ),
                    severity: DiagnosticSeverity::Warning,
                    path: Some(rel_path),
                });
            }

            // V3: check if spec area matches agent domains
            if let Some(area) = &spec.area {
                if let Some(profile) = agents.iter().find(|a| a.id == agent) {
                    if let Some(domains) = &profile.domains {
                        if !domains.is_empty()
                            && !domains
                                .iter()
                                .any(|d| area.contains(d.as_str()) || d.as_str().contains(area))
                        {
                            let rel_path = Path::new(&spec.path)
                                .strip_prefix(&lmbrain)
                                .ok()
                                .map(|path| path.to_string_lossy().to_string())
                                .unwrap_or_else(|| spec.path.clone());
                            diagnostics.push(KitDiagnostic {
                                message: format!(
                                    "Area mismatch: spec {} area '{}' does not match agent {} domains {:?}",
                                    spec.id, area, agent, domains
                                ),
                                severity: DiagnosticSeverity::Warning,
                                path: Some(rel_path),
                            });
                        }
                    }
                }
            }
        }
    }

    diagnostics
}

/// Search .lmbrain markdown content for a query string.
pub fn search_content(root: &Path, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let lmbrain = root.join(".lmbrain");
    if !lmbrain.exists() || query.is_empty() {
        return results;
    }

    let query_lower = query.to_lowercase();
    let Ok(entries) = scan_md_files(&lmbrain) else {
        return results;
    };

    for file_path in entries {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if content.to_lowercase().contains(&query_lower) {
                let relative = file_path
                    .strip_prefix(&lmbrain)
                    .ok()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_default();
                let snippet = content
                    .lines()
                    .find(|line| line.to_lowercase().contains(&query_lower))
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

    results
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        input.to_string()
    } else {
        format!("{}…", &input[..max])
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub snippet: String,
}

pub fn build_roadmap(root: &Path) -> Result<Roadmap, AppError> {
    let roadmap_path = root.join(".lmbrain/ROADMAP.md");
    if !roadmap_path.exists() {
        return Err(AppError::FileNotFound("ROADMAP.md not found".into()));
    }
    let content = fs::read_to_string(&roadmap_path)?;
    Ok(parse_roadmap_content(&content))
}

fn parse_roadmap_content(content: &str) -> Roadmap {
    let parsed = parser::parse_frontmatter(content);
    let title = fm_string(&parsed.frontmatter, "title").unwrap_or_else(|| "Roadmap".to_string());
    let body = if parsed.frontmatter.is_empty() && content.trim_start().starts_with("---") {
        content
    } else {
        parsed.body.as_str()
    };

    let mut milestones = Vec::new();
    let mut current_milestone: Option<Milestone> = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading_level = trimmed.chars().take_while(|ch| *ch == '#').count();
            let heading_content = trimmed.trim_start_matches('#').trim();
            let (id, milestone_title) = split_milestone_heading(heading_content);
            let is_milestone = is_milestone_id(&id);
            if is_milestone {
                if let Some(milestone) = current_milestone.take() {
                    milestones.push(milestone);
                }
                current_milestone = Some(Milestone {
                    id,
                    title: milestone_title,
                    status: String::new(),
                    outcome: String::new(),
                    specs: Vec::new(),
                    decisions: Vec::new(),
                    risks: Vec::new(),
                    depends_on: None,
                });
            } else if heading_level <= 3 {
                if let Some(milestone) = current_milestone.take() {
                    milestones.push(milestone);
                }
            }
        } else if (trimmed.starts_with("- ") || trimmed.starts_with("* "))
            && current_milestone.is_some()
        {
            let list_content = &trimmed[2..];
            let parts: Vec<&str> = list_content.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim().trim_matches('`').trim();
            let value = parts[1].trim();
            if let Some(milestone) = current_milestone.as_mut() {
                match key {
                    "status" => milestone.status = value.to_string(),
                    "outcome" => milestone.outcome = value.to_string(),
                    "depends_on" => milestone.depends_on = Some(value.to_string()),
                    "specs" => milestone.specs = parse_list_items(value),
                    "decisions" => milestone.decisions = parse_list_items(value),
                    "risks" => milestone.risks = parse_list_items(value),
                    _ => {}
                }
            }
        }
    }

    if let Some(milestone) = current_milestone {
        milestones.push(milestone);
    }

    Roadmap { title, milestones }
}

fn parse_list_items(value: &str) -> Vec<String> {
    let bracketed = value
        .match_indices('[')
        .filter_map(|(start, _)| {
            let rest = &value[start + 1..];
            let end = rest.find(']')?;
            Some(rest[..end].to_string())
        })
        .flat_map(|inside| {
            inside
                .split(',')
                .map(|item| clean_reference_item(item).to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
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

fn split_milestone_heading(heading: &str) -> (String, String) {
    for delimiter in [" — ", " – ", " - ", "—", "–"] {
        if let Some((id, title)) = heading.split_once(delimiter) {
            return (id.trim().to_string(), title.trim().to_string());
        }
    }

    let mut parts = heading.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or_default().trim().to_string();
    let title = parts.next().unwrap_or_default().trim().to_string();
    (id, title)
}

fn is_milestone_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix('M') else {
        return false;
    };
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

fn clean_reference_item(item: &str) -> &str {
    item.trim()
        .trim_matches('`')
        .trim_matches(|ch| ch == '[' || ch == ']')
        .trim()
}

/// Build a derived milestone overview with joined spec, review, ADR, and diagnostic data.
pub fn build_milestone_overview(root: &Path) -> Result<MilestoneOverview, AppError> {
    let roadmap = build_roadmap(root).unwrap_or(Roadmap {
        title: "Roadmap".into(),
        milestones: Vec::new(),
    });
    let specs = build_specs(root).unwrap_or_default();
    let reviews = build_reviews(root).unwrap_or_default();
    let adrs = build_adrs(root).unwrap_or_default();

    let defined_ids: std::collections::HashSet<String> =
        roadmap.milestones.iter().map(|m| m.id.clone()).collect();

    let warnings = Vec::new();
    let mut unmapped_specs = Vec::new();

    // Group specs by milestone
    let mut specs_by_milestone: std::collections::HashMap<String, Vec<&Spec>> =
        std::collections::HashMap::new();
    for spec in &specs {
        if let Some(ref ms) = spec.milestone {
            if defined_ids.contains(ms) {
                specs_by_milestone.entry(ms.clone()).or_default().push(spec);
            } else {
                unmapped_specs.push(MilestoneSpecSummary {
                    id: spec.id.clone(),
                    title: spec.title.clone(),
                    status: spec.status.as_str().to_string(),
                    priority: spec.priority.clone(),
                    area: spec.area.clone(),
                    recommended_agent: spec.recommended_agent.clone(),
                    path: Some(spec.path.clone()),
                });
            }
        }
    }

    // Build ADR lookup
    let adr_map: std::collections::HashMap<String, &Adr> =
        adrs.iter().map(|a| (a.id.clone(), a)).collect();

    // Build review lookup by spec_id
    let mut reviews_by_spec: std::collections::HashMap<String, Vec<&Review>> =
        std::collections::HashMap::new();
    for review in &reviews {
        if let Some(ref spec_id) = review.spec_id {
            reviews_by_spec
                .entry(spec_id.clone())
                .or_default()
                .push(review);
        }
    }

    let mut milestones = Vec::new();

    for milestone in &roadmap.milestones {
        let milestone_specs = specs_by_milestone.remove(&milestone.id).unwrap_or_default();
        let total = milestone_specs.len();

        // Count specs by status
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut spec_summaries = Vec::new();
        let mut all_reviews = Vec::new();
        let mut seen_review_ids = std::collections::HashSet::new();

        for spec in &milestone_specs {
            *counts.entry(spec.status.as_str().to_string()).or_insert(0) += 1;
            spec_summaries.push(MilestoneSpecSummary {
                id: spec.id.clone(),
                title: spec.title.clone(),
                status: spec.status.as_str().to_string(),
                priority: spec.priority.clone(),
                area: spec.area.clone(),
                recommended_agent: spec.recommended_agent.clone(),
                path: Some(spec.path.clone()),
            });

            // Collect reviews for this spec
            if let Some(spec_reviews) = reviews_by_spec.get(&spec.id) {
                for r in spec_reviews {
                    if seen_review_ids.insert(r.id.clone()) {
                        all_reviews.push(MilestoneReviewSummary {
                            id: r.id.clone(),
                            title: r.title.clone(),
                            status: r.status.as_str().to_string(),
                            spec_id: r.spec_id.clone(),
                            path: Some(r.path.clone()),
                        });
                    }
                }
            }
        }

        // Resolve linked decisions
        let mut decision_summaries = Vec::new();
        let mut unresolved_refs = Vec::new();
        for adr_id in &milestone.decisions {
            if let Some(adr) = adr_map.get(adr_id) {
                decision_summaries.push(MilestoneAdrSummary {
                    id: adr.id.clone(),
                    title: adr.title.clone(),
                    status: adr.status.as_str().to_string(),
                    path: Some(adr.path.clone()),
                });
            } else {
                unresolved_refs.push(format!(
                    "ADR {adr_id} referenced in milestone {} not found",
                    milestone.id
                ));
            }
        }

        // Check dependency resolution
        if let Some(ref dep) = milestone.depends_on {
            if !defined_ids.contains(dep) {
                unresolved_refs.push(format!(
                    "Milestone {} depends on {dep} which is not a defined milestone",
                    milestone.id
                ));
            }
        }

        // Determine next action
        let next_action = if total == 0 {
            Some("No specs assigned".into())
        } else if counts.get("ready").copied().unwrap_or(0) > 0 {
            Some(format!(
                "{} ready spec(s) ready for handoff",
                counts.get("ready").unwrap()
            ))
        } else if counts.get("review").copied().unwrap_or(0) > 0 {
            Some(format!(
                "{} spec(s) awaiting review",
                counts.get("review").unwrap()
            ))
        } else if counts.get("working").copied().unwrap_or(0) > 0 {
            Some("Specs in progress".into())
        } else if counts.get("done").copied().unwrap_or(0) == total && total > 0 {
            Some("All specs complete".into())
        } else {
            None
        };

        let done = counts.get("done").copied().unwrap_or(0);
        let progress_pct = if total > 0 {
            (done as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        milestones.push(MilestoneDetail {
            id: milestone.id.clone(),
            title: milestone.title.clone(),
            status: milestone.status.clone(),
            outcome: milestone.outcome.clone(),
            depends_on: milestone.depends_on.clone(),
            risks: milestone.risks.clone(),
            spec_count: total,
            spec_counts_by_status: counts,
            specs: spec_summaries,
            reviews: all_reviews,
            decisions: decision_summaries,
            unresolved_refs,
            next_action,
            progress_pct,
        });
    }

    Ok(MilestoneOverview {
        title: roadmap.title,
        milestones,
        unmapped_specs,
        warnings,
    })
}

/// Write the existing desktop approval/rejection action through lmbrain-core.
pub fn set_artifact_status(
    path_guard: &super::filesystem::PathGuard,
    path: &str,
    target_status: &str,
) -> Result<PathBuf, AppError> {
    let root = path_guard
        .get_root()
        .ok_or_else(|| AppError::PathSafety("No workspace root is set".into()))?;
    lmbrain_core::transitions::transition(
        root,
        path,
        target_status,
        lmbrain_core::transitions::MutationOptions::default(),
    )
    .map(|result| result.path)
    .map_err(|error| AppError::ParseError(error.to_string()))
}
