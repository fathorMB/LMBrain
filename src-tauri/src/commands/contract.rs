use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::NaiveDate;

use crate::commands::design;
use crate::commands::parser::{self, fm_string, fm_string_array};
use crate::errors::AppError;
use crate::models::adr::{Adr, AdrStatus};
use crate::models::agent::{AgentProfile, AgentProposal, AgentProposalStatus, AgentStatus};
use crate::models::design::{DesignMockup, DesignMockupKind};
use crate::models::file::ParsedDocument;
use crate::models::handoff::{Handoff, HandoffStatus};
use crate::models::mcp::{McpProposal, McpProposalStatus, McpRecord, McpStatus};
use crate::models::pulse::{ActionItem, MetricCard, PulseData};
use crate::models::review::{Review, ReviewFinding, ReviewStatus};
use crate::models::roadmap::{
    MilestoneAdrSummary, MilestoneDetail, MilestoneOverview, MilestoneReviewSummary,
    MilestoneSpecSummary, Roadmap,
};
use crate::models::skill::{Skill, SkillStatus};
use crate::models::spec::{Spec, SpecParkingEvent, SpecStatus};
use crate::models::statistics::{
    ArtifactFamilyStats, DiagnosticStats, ProjectStatistics, ReviewCycleRankingEntry,
    ReviewDimensionStat, ReviewQualityStats, ReviewTrendPoint, SpecFlowStats, StatusCount,
};
use crate::models::wiki::{WikiNode, WikiNodeKind, WikiTree};
use crate::models::workspace::{DiagnosticSeverity, KitDiagnostic, WorkspaceSnapshot};

const WIKI_CONTENT_DIRS: &[(&str, WikiNodeKind)] = &[
    ("decisions", WikiNodeKind::Decisions),
    ("debts", WikiNodeKind::Debts),
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
                capability_tier: fm_string(&parsed.frontmatter, "capability_tier"),
                thinking_level: fm_string(&parsed.frontmatter, "thinking_level"),
                depends_on: fm_string_array(&parsed.frontmatter, "depends_on"),
                parking_events: parsed
                    .frontmatter
                    .get("parking_events")
                    .cloned()
                    .and_then(|value| serde_json::from_value::<Vec<SpecParkingEvent>>(value).ok())
                    .unwrap_or_default(),
                skills: fm_string_array(&parsed.frontmatter, "skills"),
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
            let history = lmbrain_core::parse_review_event_value(
                &common.id,
                parsed.frontmatter.get("review_events"),
            );
            let lifecycle_source = fs::read_to_string(path).map_err(|error| {
                AppError::Io(format!("Failed to read {}: {error}", path.display()))
            })?;
            let lifecycle_document = lmbrain_core::frontmatter::Document::parse(&lifecycle_source)
                .map_err(|error| AppError::ParseError(error.to_string()))?;
            let lifecycle = lmbrain_core::analyze_review_lifecycle(&lifecycle_document);
            Ok(Review {
                id: common.id,
                title: common.title,
                status: status.clone(),
                spec_id: fm_string(&parsed.frontmatter, "spec"),
                reviewer: fm_string(&parsed.frontmatter, "reviewer"),
                implementation_agent: fm_string(&parsed.frontmatter, "implementation_agent"),
                finding_categories: parser::fm_string_array(
                    &parsed.frontmatter,
                    "finding_categories",
                ),
                findings: parse_review_findings(&common.body),
                events: history.events,
                lifecycle_warnings: lifecycle.warnings.clone(),
                lifecycle,
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

fn parse_review_findings(body: &str) -> Vec<ReviewFinding> {
    let mut in_findings = false;
    let mut findings = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            in_findings = trimmed
                .trim_start_matches("## ")
                .trim()
                .eq_ignore_ascii_case("review findings");
            continue;
        }
        if !in_findings {
            continue;
        }
        let candidate = trimmed
            .strip_prefix("### ")
            .or_else(|| trimmed.strip_prefix("- "))
            .unwrap_or("");
        let Some((id, rest)) = candidate.split_once(char::is_whitespace) else {
            continue;
        };
        if !id.starts_with("RF-") {
            continue;
        }
        let severity = rest
            .split(|character: char| {
                character.is_whitespace() || matches!(character, '[' | ']' | '|')
            })
            .find_map(|token| {
                let normalized = token
                    .strip_prefix("severity=")
                    .unwrap_or(token)
                    .trim_matches(|character: char| !character.is_ascii_alphanumeric())
                    .to_ascii_lowercase();
                matches!(
                    normalized.as_str(),
                    "critical" | "high" | "medium" | "low" | "p0" | "p1" | "p2" | "p3"
                )
                .then_some(normalized)
            })
            .unwrap_or_else(|| "unspecified".into());
        findings.push(ReviewFinding {
            id: id.trim_end_matches([':', '—', '-']).to_string(),
            text: rest
                .trim_start_matches([' ', '|', '—', '-'])
                .trim()
                .to_string(),
            severity,
        });
    }
    findings
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
                supersedes: fm_string_array(&parsed.frontmatter, "supersedes"),
                superseded_by: fm_string_array(&parsed.frontmatter, "superseded_by"),
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
                skills: parser::fm_string_array_opt(&parsed.frontmatter, "skills"),
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

/// Build skills from the skills status directories.
pub fn build_skills(root: &Path) -> Result<Vec<Skill>, AppError> {
    let statuses = [
        SkillStatus::Active,
        SkillStatus::Proposed,
        SkillStatus::Retired,
    ];
    build_status_dir_artifacts(
        &root.join(".lmbrain/skills"),
        &statuses,
        |status| status.as_str(),
        |status, parsed, path| {
            let common = common_fields(parsed, path);
            Ok(Skill {
                id: common.id,
                title: common.title,
                status: status.clone(),
                scope: fm_string(&parsed.frontmatter, "scope"),
                kind: fm_string(&parsed.frontmatter, "kind"),
                risk: fm_string(&parsed.frontmatter, "risk"),
                applies_to: fm_string_array(&parsed.frontmatter, "applies_to"),
                domains: fm_string_array(&parsed.frontmatter, "domains"),
                commands: fm_string_array(&parsed.frontmatter, "commands"),
                requires_operator_approval: parser::fm_bool(
                    &parsed.frontmatter,
                    "requires_operator_approval",
                ),
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
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
}

fn parse_agent_status(frontmatter: &HashMap<String, serde_json::Value>) -> AgentStatus {
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
}

fn parse_agent_proposal_status(
    frontmatter: &HashMap<String, serde_json::Value>,
) -> AgentProposalStatus {
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
}

fn parse_mcp_status(frontmatter: &HashMap<String, serde_json::Value>) -> McpStatus {
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
}

fn parse_mcp_proposal_status(
    frontmatter: &HashMap<String, serde_json::Value>,
) -> McpProposalStatus {
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
}

fn parse_handoff_status(frontmatter: &HashMap<String, serde_json::Value>) -> HandoffStatus {
    fm_string(frontmatter, "status")
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default()
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

        if relative.starts_with(".lmbrain/debts") && !path.is_dir() && !name.starts_with("DEBT-") {
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
            label: "Backlog".into(),
            count: count_status(SpecStatus::Backlog),
            accent: "#817a87".into(),
        },
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

pub fn build_project_statistics(root: &Path) -> Result<ProjectStatistics, AppError> {
    let specs = build_specs(root)?;
    let reviews = build_reviews(root)?;
    let debts = lmbrain_core::list_debts(root);
    let adrs = build_adrs(root)?;
    let agents = build_agents(root)?;
    let agent_proposals = build_agent_proposals(root)?;
    let mcp_records = build_mcp_records(root)?;
    let mcp_proposals = build_mcp_proposals(root)?;
    let skills = build_skills(root)?;
    let handoffs = build_handoffs(root)?;
    let design_mockups = design::scan_design_mockups(root).unwrap_or_default();
    let diagnostics = build_diagnostics(root);

    Ok(build_project_statistics_from_collections(
        &specs,
        &reviews,
        &debts,
        &adrs,
        &agents,
        &agent_proposals,
        &mcp_records,
        &mcp_proposals,
        &skills,
        &handoffs,
        &design_mockups,
        &diagnostics,
    ))
}

pub fn build_workspace_snapshot(root: &Path) -> Result<WorkspaceSnapshot, AppError> {
    let specs = build_specs(root)?;
    let reviews = build_reviews(root)?;
    let debts = lmbrain_core::list_debts(root);
    let dreams = lmbrain_core::list_dreams(root);
    let adrs = build_adrs(root)?;
    let agents = build_agents(root)?;
    let agent_proposals = build_agent_proposals(root)?;
    let mcp_records = build_mcp_records(root)?;
    let mcp_proposals = build_mcp_proposals(root)?;
    let skills = build_skills(root)?;
    let handoffs = build_handoffs(root)?;
    let design_mockups = design::scan_design_mockups(root).unwrap_or_default();
    let diagnostics = build_diagnostics(root);
    let pulse_data = build_pulse_data(root, &specs, &reviews, &adrs, &handoffs)?;
    let project_statistics = build_project_statistics_from_collections(
        &specs,
        &reviews,
        &debts,
        &adrs,
        &agents,
        &agent_proposals,
        &mcp_records,
        &mcp_proposals,
        &skills,
        &handoffs,
        &design_mockups,
        &diagnostics,
    );

    Ok(WorkspaceSnapshot {
        pulse_data,
        specs,
        reviews,
        debts,
        dreams,
        adrs,
        agents,
        agent_proposals,
        mcp_records,
        mcp_proposals,
        skills,
        handoffs,
        diagnostics,
        project_statistics,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_project_statistics_from_collections(
    specs: &[Spec],
    reviews: &[Review],
    debts: &[lmbrain_core::Debt],
    adrs: &[Adr],
    agents: &[AgentProfile],
    agent_proposals: &[AgentProposal],
    mcp_records: &[McpRecord],
    mcp_proposals: &[McpProposal],
    skills: &[Skill],
    handoffs: &[Handoff],
    design_mockups: &[DesignMockup],
    diagnostics: &[KitDiagnostic],
) -> ProjectStatistics {
    let artifact_families = vec![
        family_stats(
            "specs",
            "Specs",
            specs.iter().map(|spec| spec.status.as_str().to_string()),
        ),
        family_stats(
            "reviews",
            "Reviews",
            reviews
                .iter()
                .map(|review| review.status.as_str().to_string()),
        ),
        family_stats(
            "debts",
            "Debts",
            debts.iter().map(|debt| debt.status.clone()),
        ),
        family_stats(
            "decisions",
            "Decisions",
            adrs.iter().map(|adr| adr.status.as_str().to_string()),
        ),
        family_stats(
            "agents",
            "Agent profiles",
            agents.iter().map(|agent| agent.status.as_str().to_string()),
        ),
        family_stats(
            "agent-proposals",
            "Agent proposals",
            agent_proposals
                .iter()
                .map(|proposal| agent_proposal_status(&proposal.status).to_string()),
        ),
        family_stats(
            "skills",
            "Skills",
            skills.iter().map(|skill| skill.status.as_str().to_string()),
        ),
        family_stats(
            "mcp",
            "MCP specifications",
            mcp_records
                .iter()
                .map(|record| mcp_status(&record.status).to_string()),
        ),
        family_stats(
            "mcp-proposals",
            "MCP proposals",
            mcp_proposals
                .iter()
                .map(|proposal| mcp_proposal_status(&proposal.status).to_string()),
        ),
        family_stats(
            "handoffs",
            "Handoffs",
            handoffs
                .iter()
                .map(|handoff| handoff_status(&handoff.status).to_string()),
        ),
        family_stats(
            "design",
            "Design mockups",
            design_mockups.iter().map(|mockup| match mockup.kind {
                DesignMockupKind::Package => "package".to_string(),
                DesignMockupKind::HtmlFile => "html-file".to_string(),
            }),
        ),
    ];

    ProjectStatistics {
        artifact_families,
        spec_flow: build_spec_flow_stats(specs),
        review_quality: build_review_quality_stats(specs, reviews),
        diagnostics: build_diagnostic_stats(diagnostics),
    }
}

fn build_spec_flow_stats(specs: &[Spec]) -> SpecFlowStats {
    let done_specs = specs
        .iter()
        .filter(|spec| spec.status == SpecStatus::Done)
        .count();
    let open_specs = specs
        .iter()
        .filter(|spec| !matches!(spec.status, SpecStatus::Done | SpecStatus::Discarded))
        .count();

    SpecFlowStats {
        total_specs: specs.len(),
        done_specs,
        open_specs,
        done_ratio: ratio(done_specs, specs.len()),
        by_status: status_counts(specs.iter().map(|spec| spec.status.as_str().to_string())),
        by_priority: status_counts(specs.iter().map(|spec| {
            spec.priority
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unspecified")
                .to_string()
        })),
        by_area: status_counts(specs.iter().map(|spec| {
            spec.area
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unspecified")
                .to_string()
        })),
    }
}

fn build_review_quality_stats(specs: &[Spec], reviews: &[Review]) -> ReviewQualityStats {
    let spec_by_id: HashMap<&str, &Spec> =
        specs.iter().map(|spec| (spec.id.as_str(), spec)).collect();
    let mut reviews_by_spec: HashMap<&str, Vec<&Review>> = HashMap::new();
    let mut reviews_without_spec = 0;
    let mut reviews_without_created = 0;
    let mut trend: BTreeMap<String, TrendAccumulator> = BTreeMap::new();

    for review in reviews {
        let Some(spec_id) = review
            .spec_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            reviews_without_spec += 1;
            continue;
        };
        reviews_by_spec.entry(spec_id).or_default().push(review);

        if let Some(date) = parse_artifact_date(&review.created) {
            let period = date.format("%Y-%m").to_string();
            let entry = trend.entry(period).or_default();
            entry.total_reviews += 1;
            match review.status {
                ReviewStatus::Accepted => entry.accepted_reviews += 1,
                ReviewStatus::ChangesRequested => entry.changes_requested_reviews += 1,
                _ => {}
            }
            entry.reviewed_specs.insert(spec_id.to_string());
            if review.status == ReviewStatus::ChangesRequested {
                entry
                    .specs_with_changes_requested
                    .insert(spec_id.to_string());
            }
        } else {
            reviews_without_created += 1;
        }
    }

    let reviewed_specs = reviews_by_spec.len();
    let accepted_reviews = reviews
        .iter()
        .filter(|review| review.status == ReviewStatus::Accepted)
        .count();
    let changes_requested_reviews = reviews
        .iter()
        .filter(|review| review.status == ReviewStatus::ChangesRequested)
        .count();
    let blocked_reviews = reviews
        .iter()
        .filter(|review| review.status == ReviewStatus::Blocked)
        .count();
    let superseded_reviews = reviews
        .iter()
        .filter(|review| review.status == ReviewStatus::Superseded)
        .count();
    let total_review_passes = reviews
        .iter()
        .map(|review| review.lifecycle.review_passes)
        .sum();
    let remediation_cycles = reviews
        .iter()
        .map(|review| review.lifecycle.remediation_cycles)
        .sum();
    let escalation_count = reviews
        .iter()
        .map(|review| review.lifecycle.escalation_count)
        .sum();
    let takeover_count = reviews
        .iter()
        .map(|review| review.lifecycle.takeover_count)
        .sum();
    let lifecycle_known_reviews = reviews
        .iter()
        .filter(|review| review.lifecycle.source != lmbrain_core::ReviewHistorySource::StatusOnly)
        .count();

    let mut specs_with_changes_requested = 0;
    let mut specs_with_multiple_changes_requested = 0;
    let mut first_pass_eligible_specs = 0;
    let mut first_pass_accepted_specs = 0;
    let mut area_map: HashMap<String, DimensionAccumulator> = HashMap::new();
    let mut agent_map: HashMap<String, DimensionAccumulator> = HashMap::new();

    for (spec_id, spec_reviews) in &reviews_by_spec {
        let cr_count = spec_reviews
            .iter()
            .map(|review| review.lifecycle.remediation_cycles)
            .sum::<usize>()
            + usize::from(
                spec_reviews
                    .iter()
                    .any(|review| review.status == ReviewStatus::ChangesRequested)
                    && !spec_reviews
                        .iter()
                        .any(|review| review.lifecycle.remediation_cycles > 0),
            );
        let has_changes_requested = cr_count > 0;
        if has_changes_requested {
            specs_with_changes_requested += 1;
        }
        if cr_count > 1 {
            specs_with_multiple_changes_requested += 1;
        }

        if let Some(spec) = spec_by_id.get(spec_id) {
            let area = spec
                .area
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unspecified");
            area_map
                .entry(area.to_string())
                .or_default()
                .add(has_changes_requested);

            let agent = spec
                .recommended_agent
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unspecified");
            agent_map
                .entry(agent.to_string())
                .or_default()
                .add(has_changes_requested);
        }

        let dated = spec_reviews
            .iter()
            .filter_map(|review| parse_artifact_date(&review.created).map(|date| (date, *review)))
            .collect::<Vec<_>>();
        if dated.len() == spec_reviews.len() && !dated.is_empty() {
            let first = dated
                .iter()
                .min_by_key(|(date, review)| (*date, review.id.as_str()))
                .map(|(_, review)| *review);
            let lifecycle_known = first.is_some_and(|review| {
                review.lifecycle.source != lmbrain_core::ReviewHistorySource::StatusOnly
            });
            let separate_ordered_reviews = dated.len() > 1;
            if !lifecycle_known && !separate_ordered_reviews {
                continue;
            }
            first_pass_eligible_specs += 1;
            if first.is_some_and(|review| {
                review
                    .lifecycle
                    .initial_verdict
                    .as_deref()
                    .unwrap_or(review.status.as_str())
                    == "accepted"
            }) {
                first_pass_accepted_specs += 1;
            }
        }
    }

    let mut review_cycle_ranking = reviews_by_spec
        .iter()
        .filter_map(|(spec_id, spec_reviews)| {
            let spec = spec_by_id.get(spec_id)?;
            if spec_reviews.iter().any(|review| {
                review.lifecycle.source == lmbrain_core::ReviewHistorySource::StatusOnly
            }) {
                return None;
            }
            let warnings = spec_reviews
                .iter()
                .flat_map(|review| review.lifecycle.warnings.clone())
                .collect::<Vec<_>>();
            let confidence = if warnings.is_empty()
                && spec_reviews
                    .iter()
                    .all(|review| review.lifecycle.confidence == "high")
            {
                "high"
            } else {
                "medium"
            };
            Some(ReviewCycleRankingEntry {
                spec_id: (*spec_id).to_string(),
                title: spec.title.clone(),
                path: spec.path.clone(),
                status: spec.status.as_str().into(),
                review_count: spec_reviews.len(),
                review_passes: spec_reviews
                    .iter()
                    .map(|review| review.lifecycle.review_passes)
                    .sum(),
                remediation_cycles: spec_reviews
                    .iter()
                    .map(|review| review.lifecycle.remediation_cycles)
                    .sum(),
                history_source: if spec_reviews.iter().all(|review| {
                    review.lifecycle.source == lmbrain_core::ReviewHistorySource::StructuredEvents
                }) {
                    "structured".into()
                } else {
                    "legacy".into()
                },
                confidence: confidence.into(),
                warnings,
            })
        })
        .collect::<Vec<_>>();
    review_cycle_ranking.sort_by(|left, right| {
        right
            .remediation_cycles
            .cmp(&left.remediation_cycles)
            .then_with(|| right.review_passes.cmp(&left.review_passes))
            .then_with(|| left.spec_id.cmp(&right.spec_id))
    });
    let review_cycle_ranking_coverage = review_cycle_ranking.len();

    ReviewQualityStats {
        total_reviews: reviews.len(),
        total_review_passes,
        remediation_cycles,
        escalation_count,
        takeover_count,
        lifecycle_known_reviews,
        lifecycle_coverage: ratio(lifecycle_known_reviews, reviews.len()),
        reviewed_specs,
        accepted_reviews,
        changes_requested_reviews,
        blocked_reviews,
        superseded_reviews,
        reviews_without_spec,
        reviews_without_created,
        specs_with_changes_requested,
        specs_with_multiple_changes_requested,
        change_request_rate: ratio(specs_with_changes_requested, reviewed_specs),
        first_pass_eligible_specs,
        first_pass_accepted_specs,
        first_pass_acceptance_rate: ratio(first_pass_accepted_specs, first_pass_eligible_specs),
        average_reviews_per_reviewed_spec: ratio_f64(
            reviews_by_spec
                .values()
                .flatten()
                .map(|review| review.lifecycle.review_passes)
                .sum::<usize>(),
            reviewed_specs,
        ),
        review_cycle_ranking,
        review_cycle_ranking_coverage,
        by_area: dimension_stats(area_map),
        by_agent: dimension_stats(agent_map),
        trend: trend
            .into_iter()
            .map(|(period, entry)| ReviewTrendPoint {
                period,
                total_reviews: entry.total_reviews,
                accepted_reviews: entry.accepted_reviews,
                changes_requested_reviews: entry.changes_requested_reviews,
                reviewed_specs: entry.reviewed_specs.len(),
                specs_with_changes_requested: entry.specs_with_changes_requested.len(),
            })
            .collect(),
    }
}

fn build_diagnostic_stats(diagnostics: &[KitDiagnostic]) -> DiagnosticStats {
    DiagnosticStats {
        total: diagnostics.len(),
        warnings: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .count(),
        errors: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count(),
        by_family: status_counts(diagnostics.iter().map(|diagnostic| {
            diagnostic
                .path
                .as_deref()
                .and_then(|path| path.split(['/', '\\']).next())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("workspace")
                .to_string()
        })),
    }
}

#[derive(Default)]
struct DimensionAccumulator {
    reviewed_specs: usize,
    specs_with_changes_requested: usize,
}

impl DimensionAccumulator {
    fn add(&mut self, has_changes_requested: bool) {
        self.reviewed_specs += 1;
        if has_changes_requested {
            self.specs_with_changes_requested += 1;
        }
    }
}

#[derive(Default)]
struct TrendAccumulator {
    total_reviews: usize,
    accepted_reviews: usize,
    changes_requested_reviews: usize,
    reviewed_specs: HashSet<String>,
    specs_with_changes_requested: HashSet<String>,
}

fn family_stats(
    family: &str,
    label: &str,
    statuses: impl Iterator<Item = String>,
) -> ArtifactFamilyStats {
    let statuses = status_counts(statuses);
    let total = statuses.iter().map(|status| status.count).sum();
    ArtifactFamilyStats {
        family: family.into(),
        label: label.into(),
        total,
        statuses,
    }
}

fn status_counts(values: impl Iterator<Item = String>) -> Vec<StatusCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(status, count)| StatusCount { status, count })
        .collect()
}

fn dimension_stats(map: HashMap<String, DimensionAccumulator>) -> Vec<ReviewDimensionStat> {
    let mut stats = map
        .into_iter()
        .map(|(value, counts)| ReviewDimensionStat {
            value,
            reviewed_specs: counts.reviewed_specs,
            specs_with_changes_requested: counts.specs_with_changes_requested,
            change_request_rate: ratio(counts.specs_with_changes_requested, counts.reviewed_specs),
        })
        .collect::<Vec<_>>();
    stats.sort_by(|left, right| {
        right
            .specs_with_changes_requested
            .cmp(&left.specs_with_changes_requested)
            .then_with(|| right.reviewed_specs.cmp(&left.reviewed_specs))
            .then_with(|| left.value.cmp(&right.value))
    });
    stats
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn ratio_f64(numerator: usize, denominator: usize) -> f64 {
    ratio(numerator, denominator)
}

fn parse_artifact_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn agent_proposal_status(status: &AgentProposalStatus) -> &'static str {
    match status {
        AgentProposalStatus::Proposed => "proposed",
        AgentProposalStatus::Approved => "approved",
        AgentProposalStatus::Rejected => "rejected",
    }
}

fn mcp_status(status: &McpStatus) -> &'static str {
    match status {
        McpStatus::Specified => "specified",
        McpStatus::Active => "active",
        McpStatus::Inactive => "inactive",
        McpStatus::Deprecated => "deprecated",
    }
}

fn mcp_proposal_status(status: &McpProposalStatus) -> &'static str {
    match status {
        McpProposalStatus::Proposed => "proposed",
        McpProposalStatus::Approved => "approved",
        McpProposalStatus::Rejected => "rejected",
        McpProposalStatus::Implemented => "implemented",
        McpProposalStatus::Blocked => "blocked",
    }
}

fn handoff_status(status: &HandoffStatus) -> &'static str {
    match status {
        HandoffStatus::Ready => "ready",
        HandoffStatus::Consumed => "consumed",
        HandoffStatus::Superseded => "superseded",
        HandoffStatus::Archived => "archived",
    }
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
    let files: Vec<PathBuf> = WIKI_CONTENT_DIRS
        .iter()
        .filter_map(|(directory, _)| scan_md_files(&lmbrain.join(directory)).ok())
        .flatten()
        .filter(|path| {
            if path.components().any(|c| c.as_os_str() == "debts") {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |name| name.starts_with("DEBT-"))
            } else {
                true
            }
        })
        .collect();

    files
}

/// Scan all .md files under .lmbrain/ for malformed frontmatter and
/// status-directory/frontmatter mismatches.
pub fn build_diagnostics(root: &Path) -> Vec<KitDiagnostic> {
    lmbrain_core::build_diagnostics(root)
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
    Ok(lmbrain_core::parse_roadmap(&content))
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
