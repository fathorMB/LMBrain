use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::frontmatter::Document;

// ─── Context-pack data structures ─────────────────────────────────

/// Compact project overview for Project Lead bootstrap and pulse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDigest {
    pub title: String,
    pub status: String,
    pub current_milestone: Option<String>,
    pub ready_specs: Vec<CompactSpec>,
    pub review_specs: Vec<CompactSpec>,
    pub blockers: Vec<String>,
    pub ready_handoffs: Vec<String>,
    pub active_decisions: Vec<CompactAdr>,
    pub diagnostics_summary: DiagnosticsSummary,
    pub version: Option<String>,
    pub warnings: Vec<String>,
    pub markdown: String,
}

/// Compact spec reference for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactSpec {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub recommended_agent: Option<String>,
    pub milestone: Option<String>,
}

/// Compact ADR reference for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactAdr {
    pub id: String,
    pub title: String,
    pub status: String,
    pub decision_date: Option<String>,
}

/// Summary of diagnostics for the digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
}

/// Spec handoff context for specialist orientation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContext {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub area: Option<String>,
    pub milestone: Option<String>,
    pub recommended_agent: Option<String>,
    pub agent_profile: Option<AgentProfileSummary>,
    pub acceptance_criteria: Vec<Criterion>,
    pub linked_decisions: Vec<CompactAdr>,
    pub related_reviews: Vec<CompactReview>,
    pub applicable_skills: Vec<SkillSummary>,
    pub explicit_files: Vec<String>,
    pub explicit_areas: Vec<String>,
    pub diagnostics: Vec<String>,
    pub warnings: Vec<String>,
    pub markdown: String,
}

/// A single acceptance criterion with its check state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub text: String,
    pub checked: bool,
}

/// Compact review reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactReview {
    pub id: String,
    pub title: String,
    pub status: String,
    pub reviewer: Option<String>,
}

/// Agent profile summary for context packs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfileSummary {
    pub id: String,
    pub title: String,
    pub mnemonic_name: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub can_implement: Option<bool>,
    pub can_review: Option<bool>,
    pub skills: Vec<String>,
}

/// Compact skill reference for context packs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub kind: Option<String>,
    pub risk: Option<String>,
    pub requires_operator_approval: Option<bool>,
    pub commands: Vec<String>,
    pub path: String,
}

/// Review context for reviewer orientation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewContext {
    pub spec_id: String,
    pub spec_title: String,
    pub acceptance_criteria: Vec<Criterion>,
    pub implementation_evidence: Option<String>,
    pub linked_reviews: Vec<CompactReview>,
    pub relevant_decisions: Vec<CompactAdr>,
    pub verification_commands: Vec<String>,
    pub applicable_skills: Vec<SkillSummary>,
    pub warnings: Vec<String>,
    pub markdown: String,
}

// ─── Resolution functions ──────────────────────────────────────────

/// Build a project digest from the .lmbrain directory.
pub fn build_project_digest(root: &Path) -> ProjectDigest {
    let lmbrain = root.join(".lmbrain");
    let mut warnings = Vec::new();

    // Read STATUS.md
    let (title, status, milestone) = read_status(&lmbrain);
    let version = read_version(&lmbrain);

    // Scan specs
    let all_specs = scan_specs(&lmbrain);
    let ready_specs: Vec<CompactSpec> = all_specs
        .iter()
        .filter(|s| s.status.as_deref() == Some("ready"))
        .cloned()
        .collect();
    let review_specs: Vec<CompactSpec> = all_specs
        .iter()
        .filter(|s| s.status.as_deref() == Some("review"))
        .cloned()
        .collect();

    // Scan handoffs
    let ready_handoffs = scan_ready_handoffs(&lmbrain);

    // Scan ADRs
    let active_decisions = scan_active_adrs(&lmbrain);

    // Scan diagnostics
    let diagnostics = scan_diagnostics(root);
    let diag_summary = DiagnosticsSummary {
        errors: diagnostics.iter().filter(|d| d.severity == "error").count(),
        warnings: diagnostics
            .iter()
            .filter(|d| d.severity == "warning")
            .count(),
        total: diagnostics.len(),
    };

    // Collect blockers from diagnostics
    let blockers: Vec<String> = diagnostics
        .iter()
        .filter(|d| d.severity == "error")
        .map(|d| d.message.clone())
        .collect();

    // Check for missing recommended agents
    for spec in &all_specs {
        if let Some(agent) = &spec.recommended_agent {
            if !agent_resolves(&lmbrain, agent) {
                warnings.push(format!(
                    "Spec {} recommends agent '{}' which does not resolve",
                    spec.id, agent
                ));
            }
        }
    }

    let markdown = format_project_digest_md(
        &title,
        &status,
        &milestone,
        &ready_specs,
        &review_specs,
        &blockers,
        &ready_handoffs,
        &active_decisions,
        &diag_summary,
        &version,
        &warnings,
    );

    ProjectDigest {
        title,
        status,
        current_milestone: milestone,
        ready_specs,
        review_specs,
        blockers,
        ready_handoffs,
        active_decisions,
        diagnostics_summary: diag_summary,
        version,
        warnings,
        markdown,
    }
}

/// Build spec context for a given spec ID or path.
pub fn build_spec_context(root: &Path, spec_id_or_path: &str) -> Result<SpecContext, String> {
    let lmbrain = root.join(".lmbrain");
    let spec_path = resolve_spec_path(&lmbrain, spec_id_or_path)?;
    let source = fs::read_to_string(&spec_path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let document = Document::parse(&source).map_err(|e| format!("Failed to parse spec: {e}"))?;

    let id = document.value("id").unwrap_or_default();
    let title = document.value("title").unwrap_or_default();
    let status = document.value("status").unwrap_or_default();
    let priority = document.value("priority");
    let area = document.value("area");
    let milestone = document.value("milestone");
    let recommended_agent = document.value("recommended_agent");
    let spec_skills = document.string_array("skills");
    let spec_tags = document.string_array("tags");
    let related_decisions = document.string_array("related_decisions");

    // Parse acceptance criteria from body
    let criteria = parse_criteria(&document.body);

    // Resolve linked decisions
    let mut linked_decisions = Vec::new();
    let mut warnings = Vec::new();
    for adr_id in &related_decisions {
        match resolve_adr(&lmbrain, adr_id) {
            Some(adr) => linked_decisions.push(adr),
            None => warnings.push(format!("Linked ADR {adr_id} not found")),
        }
    }

    // Resolve agent profile
    let agent_profile = recommended_agent
        .as_deref()
        .and_then(|agent| resolve_agent(&lmbrain, agent));
    let applicable_skills = resolve_applicable_skills(
        &lmbrain,
        &spec_skills,
        recommended_agent.as_deref(),
        agent_profile.as_ref(),
        area.as_deref(),
        &spec_tags,
        false,
    );

    // Resolve related reviews
    let related_reviews = resolve_reviews_for_spec(&lmbrain, &id);

    // Extract explicit files/areas from body
    let explicit_files = extract_section_list(&document.body, "Files and areas involved");
    let explicit_areas = extract_section_list(&document.body, "Areas");

    // Collect diagnostics affecting this spec
    let diagnostics = spec_diagnostics(&lmbrain, &id);

    if recommended_agent.is_some() && agent_profile.is_none() {
        warnings.push(format!(
            "Recommended agent '{}' does not resolve to an existing profile",
            recommended_agent.as_deref().unwrap_or("")
        ));
    }

    let markdown = format_spec_context_md(
        &id,
        &title,
        &status,
        &priority,
        &area,
        &milestone,
        &recommended_agent,
        &agent_profile,
        &criteria,
        &linked_decisions,
        &related_reviews,
        &applicable_skills,
        &explicit_files,
        &explicit_areas,
        &diagnostics,
        &warnings,
    );

    Ok(SpecContext {
        id,
        title,
        status,
        priority,
        area,
        milestone,
        recommended_agent,
        agent_profile,
        acceptance_criteria: criteria,
        linked_decisions,
        related_reviews,
        applicable_skills,
        explicit_files,
        explicit_areas,
        diagnostics,
        warnings,
        markdown,
    })
}

/// Build review context for a given spec ID or path.
pub fn build_review_context(root: &Path, spec_id_or_path: &str) -> Result<ReviewContext, String> {
    let lmbrain = root.join(".lmbrain");
    let spec_path = resolve_spec_path(&lmbrain, spec_id_or_path)?;
    let source = fs::read_to_string(&spec_path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let document = Document::parse(&source).map_err(|e| format!("Failed to parse spec: {e}"))?;

    let spec_id = document.value("id").unwrap_or_default();
    let spec_title = document.value("title").unwrap_or_default();
    let spec_skills = document.string_array("skills");
    let spec_tags = document.string_array("tags");
    let area = document.value("area");
    let recommended_agent = document.value("recommended_agent");
    let agent_profile = recommended_agent
        .as_deref()
        .and_then(|agent| resolve_agent(&lmbrain, agent));

    // Parse acceptance criteria
    let criteria = parse_criteria(&document.body);

    // Extract implementation evidence
    let implementation_evidence = extract_section(&document.body, "Implementation evidence");

    // Resolve related reviews
    let linked_reviews = resolve_reviews_for_spec(&lmbrain, &spec_id);

    // Resolve linked decisions
    let related_decisions: Vec<String> = document.string_array("related_decisions");
    let mut relevant_decisions = Vec::new();
    let mut warnings = Vec::new();
    for adr_id in &related_decisions {
        match resolve_adr(&lmbrain, adr_id) {
            Some(adr) => relevant_decisions.push(adr),
            None => warnings.push(format!("Linked ADR {adr_id} not found")),
        }
    }

    // Extract verification commands
    let verification_commands = extract_section_list(&document.body, "Required verification");
    let applicable_skills = resolve_applicable_skills(
        &lmbrain,
        &spec_skills,
        recommended_agent.as_deref(),
        agent_profile.as_ref(),
        area.as_deref(),
        &spec_tags,
        true,
    );

    let markdown = format_review_context_md(
        &spec_id,
        &spec_title,
        &criteria,
        &implementation_evidence,
        &linked_reviews,
        &relevant_decisions,
        &verification_commands,
        &applicable_skills,
        &warnings,
    );

    Ok(ReviewContext {
        spec_id,
        spec_title,
        acceptance_criteria: criteria,
        implementation_evidence,
        linked_reviews,
        relevant_decisions,
        verification_commands,
        applicable_skills,
        warnings,
        markdown,
    })
}

// ─── Internal helpers ──────────────────────────────────────────────

fn read_status(lmbrain: &Path) -> (String, String, Option<String>) {
    let path = lmbrain.join("STATUS.md");
    let content = fs::read_to_string(&path).unwrap_or_default();
    // Title is the first H1 heading, stripping any "Project " prefix
    let title = content
        .lines()
        .find(|l| l.trim().starts_with("# "))
        .map(|l| {
            let raw = l.trim_start_matches("# ").trim();
            raw.strip_prefix("Project ")
                .map(|s| s.to_string())
                .unwrap_or_else(|| raw.to_string())
        })
        .unwrap_or_default();
    let status = extract_section(&content, "Status")
        .or_else(|| {
            content
                .lines()
                .find(|l| l.to_lowercase().contains("status"))
                .map(|l| l.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let milestone = extract_section(&content, "Current milestone");
    (title, status, milestone)
}

fn read_version(lmbrain: &Path) -> Option<String> {
    fs::read_to_string(lmbrain.join("VERSION"))
        .ok()
        .map(|v| v.trim().to_string())
}

fn scan_specs(lmbrain: &Path) -> Vec<CompactSpec> {
    let mut specs = Vec::new();
    let specs_dir = lmbrain.join("specs");
    if !specs_dir.exists() {
        return specs;
    }
    if let Ok(entries) = fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            let status_dir = entry.path();
            if !status_dir.is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(&status_dir) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("md") {
                        continue;
                    }
                    if let Ok(source) = fs::read_to_string(&path) {
                        if let Ok(doc) = Document::parse(&source) {
                            let status_name = status_dir
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| n.to_string());
                            specs.push(CompactSpec {
                                id: doc.value("id").unwrap_or_default(),
                                title: doc.value("title").unwrap_or_default(),
                                status: status_name.or_else(|| doc.value("status")),
                                priority: doc.value("priority"),
                                area: doc.value("area"),
                                recommended_agent: doc.value("recommended_agent"),
                                milestone: doc.value("milestone"),
                            });
                        }
                    }
                }
            }
        }
    }
    specs
}

fn scan_ready_handoffs(lmbrain: &Path) -> Vec<String> {
    let handoffs_dir = lmbrain.join("handoffs/active");
    if !handoffs_dir.exists() {
        return Vec::new();
    }
    let mut handoffs = Vec::new();
    if let Ok(entries) = fs::read_dir(&handoffs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(source) = fs::read_to_string(&path) {
                if let Ok(doc) = Document::parse(&source) {
                    if doc.value("status").as_deref() == Some("ready") {
                        handoffs.push(doc.value("id").unwrap_or_default());
                    }
                }
            }
        }
    }
    handoffs
}

fn scan_active_adrs(lmbrain: &Path) -> Vec<CompactAdr> {
    let decisions_dir = lmbrain.join("decisions");
    if !decisions_dir.exists() {
        return Vec::new();
    }
    let mut adrs = Vec::new();
    if let Ok(entries) = fs::read_dir(&decisions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(source) = fs::read_to_string(&path) {
                if let Ok(doc) = Document::parse(&source) {
                    let id = doc.value("id").unwrap_or_default();
                    if id.starts_with("ADR-") {
                        let status = doc.value("status").unwrap_or_default();
                        if status == "accepted" || status == "proposed" {
                            adrs.push(CompactAdr {
                                id,
                                title: doc.value("title").unwrap_or_default(),
                                status,
                                decision_date: doc.value("decision_date"),
                            });
                        }
                    }
                }
            }
        }
    }
    adrs.sort_by(|a, b| {
        b.decision_date
            .as_deref()
            .unwrap_or("")
            .cmp(a.decision_date.as_deref().unwrap_or(""))
    });
    adrs
}

fn scan_diagnostics(root: &Path) -> Vec<DiagnosticEntry> {
    let lmbrain = root.join(".lmbrain");
    let mut diagnostics = Vec::new();
    let entries = scan_md_files(&lmbrain);

    for file_path in entries {
        let source = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(_) => {
                diagnostics.push(DiagnosticEntry {
                    message: format!("Failed to read {}", file_path.display()),
                    severity: "warning".to_string(),
                });
                continue;
            }
        };

        let doc = match Document::parse(&source) {
            Ok(d) => d,
            Err(e) => {
                diagnostics.push(DiagnosticEntry {
                    message: format!("Malformed frontmatter in {}: {e}", file_path.display()),
                    severity: "warning".to_string(),
                });
                continue;
            }
        };

        // Check status directory/frontmatter mismatch for specs and reviews
        if let Some(parent) = file_path.parent() {
            if let Some(grandparent) = parent.parent() {
                let artifact_type = grandparent
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                if (artifact_type == "specs" || artifact_type == "reviews")
                    && !crate::invariants::folder_matches_status(&file_path)
                {
                    let status_dir = parent.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    let fm_status = doc.value("status").unwrap_or_else(|| "?".into());
                    diagnostics.push(DiagnosticEntry {
                        message: format!(
                            "Status mismatch: file is in '{artifact_type}/{status_dir}' but frontmatter status is '{fm_status}'"
                        ),
                        severity: "warning".to_string(),
                    });
                }
            }
        }
    }

    // Check for unresolved recommended agents across all specs
    let specs = scan_specs(&lmbrain);
    for spec in &specs {
        if let Some(agent) = &spec.recommended_agent {
            if !agent_resolves(&lmbrain, agent) {
                diagnostics.push(DiagnosticEntry {
                    message: format!(
                        "Missing reference: spec {} recommends agent '{agent}', which is not an existing agent profile",
                        spec.id
                    ),
                    severity: "warning".to_string(),
                });
            }
        }
    }

    diagnostics
}

fn resolve_spec_path(lmbrain: &Path, spec_id_or_path: &str) -> Result<PathBuf, String> {
    // Try as a direct path first (relative to .lmbrain/)
    let direct = lmbrain.join(spec_id_or_path);
    if direct.exists() {
        return Ok(direct);
    }

    // Try as a bare spec ID — search all status subdirectories
    let specs_dir = lmbrain.join("specs");
    if specs_dir.exists() {
        if let Ok(entries) = fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let status_dir = entry.path();
                if !status_dir.is_dir() {
                    continue;
                }
                if let Ok(files) = fs::read_dir(&status_dir) {
                    for file in files.flatten() {
                        let path = file.path();
                        if path.extension().and_then(|e| e.to_str()) != Some("md") {
                            continue;
                        }
                        if let Ok(source) = fs::read_to_string(&path) {
                            if let Ok(doc) = Document::parse(&source) {
                                if doc.value("id").as_deref() == Some(spec_id_or_path) {
                                    return Ok(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(format!("Spec {spec_id_or_path} not found"))
}

fn resolve_adr(lmbrain: &Path, adr_id: &str) -> Option<CompactAdr> {
    let decisions_dir = lmbrain.join("decisions");
    if !decisions_dir.exists() {
        return None;
    }
    if let Ok(entries) = fs::read_dir(&decisions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(source) = fs::read_to_string(&path) {
                if let Ok(doc) = Document::parse(&source) {
                    if doc.value("id").as_deref() == Some(adr_id) {
                        return Some(CompactAdr {
                            id: adr_id.to_string(),
                            title: doc.value("title").unwrap_or_default(),
                            status: doc.value("status").unwrap_or_default(),
                            decision_date: doc.value("decision_date"),
                        });
                    }
                }
            }
        }
    }
    None
}

fn resolve_agent(lmbrain: &Path, agent_id: &str) -> Option<AgentProfileSummary> {
    let profiles_dir = lmbrain.join("agents/profiles");
    if !profiles_dir.exists() {
        return None;
    }
    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(source) = fs::read_to_string(&path) {
                if let Ok(doc) = Document::parse(&source) {
                    if doc.value("id").as_deref() == Some(agent_id) {
                        return Some(AgentProfileSummary {
                            id: agent_id.to_string(),
                            title: doc.value("title").unwrap_or_default(),
                            mnemonic_name: doc.value("mnemonic_name"),
                            role: doc.value("role"),
                            status: doc.value("status").unwrap_or_default(),
                            can_implement: doc.bool("can_implement"),
                            can_review: doc.bool("can_review"),
                            skills: doc.string_array("skills"),
                        });
                    }
                }
            }
        }
    }
    None
}

fn agent_resolves(lmbrain: &Path, agent_id: &str) -> bool {
    resolve_agent(lmbrain, agent_id).is_some()
}

fn resolve_applicable_skills(
    lmbrain: &Path,
    explicit_skill_ids: &[String],
    recommended_agent: Option<&str>,
    agent_profile: Option<&AgentProfileSummary>,
    area: Option<&str>,
    tags: &[String],
    review_only: bool,
) -> Vec<SkillSummary> {
    let skills = scan_skills(lmbrain);
    let mut result = Vec::new();
    for (summary, applies_to, domains) in skills {
        if summary.status != "active" {
            continue;
        }
        let explicit = explicit_skill_ids.iter().any(|id| id == &summary.id);
        let agent_default = agent_profile
            .map(|profile| profile.skills.iter().any(|id| id == &summary.id))
            .unwrap_or(false);
        let applies_to_agent = recommended_agent
            .map(|agent| {
                applies_to
                    .iter()
                    .any(|target| target == "all" || target == agent)
            })
            .unwrap_or_else(|| applies_to.iter().any(|target| target == "all"));
        let domain_match = area
            .map(|area| {
                domains
                    .iter()
                    .any(|domain| area.contains(domain) || domain.contains(area))
            })
            .unwrap_or(false)
            || tags
                .iter()
                .any(|tag| domains.iter().any(|domain| tag == domain));
        let review_kind = summary
            .kind
            .as_deref()
            .map(|kind| matches!(kind, "review" | "test" | "diagnostic" | "verification"))
            .unwrap_or(false);

        if explicit
            || agent_default
            || applies_to_agent
            || domain_match
            || (review_only && review_kind)
        {
            if !result
                .iter()
                .any(|skill: &SkillSummary| skill.id == summary.id)
            {
                result.push(summary);
            }
        }
    }
    result
}

fn scan_skills(lmbrain: &Path) -> Vec<(SkillSummary, Vec<String>, Vec<String>)> {
    let skills_dir = lmbrain.join("skills");
    let mut skills = Vec::new();
    for status in ["active", "proposed", "retired"] {
        let dir = skills_dir.join(status);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                if let Ok(source) = fs::read_to_string(&path) {
                    if let Ok(doc) = Document::parse(&source) {
                        let id = doc.value("id").unwrap_or_default();
                        if !id.starts_with("SKILL-") {
                            continue;
                        }
                        let rel_path = path
                            .strip_prefix(lmbrain)
                            .ok()
                            .map(|p| format!(".lmbrain/{}", p.to_string_lossy().replace('\\', "/")))
                            .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"));
                        skills.push((
                            SkillSummary {
                                id,
                                title: doc.value("title").unwrap_or_default(),
                                status: doc.value("status").unwrap_or_else(|| status.to_string()),
                                kind: doc.value("kind"),
                                risk: doc.value("risk"),
                                requires_operator_approval: doc.bool("requires_operator_approval"),
                                commands: doc.string_array("commands"),
                                path: rel_path,
                            },
                            doc.string_array("applies_to"),
                            doc.string_array("domains"),
                        ));
                    }
                }
            }
        }
    }
    skills
}

fn resolve_reviews_for_spec(lmbrain: &Path, spec_id: &str) -> Vec<CompactReview> {
    let reviews_dir = lmbrain.join("reviews");
    if !reviews_dir.exists() {
        return Vec::new();
    }
    let mut reviews = Vec::new();
    if let Ok(entries) = fs::read_dir(&reviews_dir) {
        for entry in entries.flatten() {
            let status_dir = entry.path();
            if !status_dir.is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(&status_dir) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("md") {
                        continue;
                    }
                    if let Ok(source) = fs::read_to_string(&path) {
                        if let Ok(doc) = Document::parse(&source) {
                            if doc.value("spec").as_deref() == Some(spec_id) {
                                reviews.push(CompactReview {
                                    id: doc.value("id").unwrap_or_default(),
                                    title: doc.value("title").unwrap_or_default(),
                                    status: doc.value("status").unwrap_or_default(),
                                    reviewer: doc.value("reviewer"),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    reviews
}

fn spec_diagnostics(lmbrain: &Path, spec_id: &str) -> Vec<String> {
    let mut result = Vec::new();
    let entries = scan_md_files(lmbrain);

    for file_path in entries {
        let source = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Check for malformed frontmatter in files that reference this spec
        if let Ok(doc) = Document::parse(&source) {
            // Check if this file's diagnostics mention the spec
            if let Some(parent) = file_path.parent() {
                if let Some(grandparent) = parent.parent() {
                    let artifact_type = grandparent
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_string())
                        .unwrap_or_default();
                    if (artifact_type == "specs" || artifact_type == "reviews")
                        && !crate::invariants::folder_matches_status(&file_path)
                    {
                        let id = doc.value("id").unwrap_or_default();
                        if id == spec_id {
                            let status_dir =
                                parent.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            let fm_status = doc.value("status").unwrap_or_else(|| "?".into());
                            result.push(format!(
                                "Status mismatch: file is in '{artifact_type}/{status_dir}' but frontmatter status is '{fm_status}'"
                            ));
                        }
                    }
                }
            }
        } else {
            // File has malformed frontmatter — check if it's the spec itself
            if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                if name.contains(spec_id) {
                    result.push(format!("Malformed frontmatter in {name}"));
                }
            }
        }
    }

    result
}

fn parse_criteria(body: &str) -> Vec<Criterion> {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("- [") && trimmed.len() > 5
        })
        .map(|line| {
            let trimmed = line.trim_start();
            let checked = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            let text = trimmed[5..].trim().to_string();
            Criterion { text, checked }
        })
        .collect()
}

fn extract_section(body: &str, heading: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == format!("# {heading}")
            || trimmed == format!("## {heading}")
            || trimmed == format!("### {heading}")
        {
            let mut content = Vec::new();
            for next in lines.iter().skip(i + 1) {
                let next_trimmed = next.trim();
                if next_trimmed.starts_with("#") && !next_trimmed.starts_with("##") {
                    break;
                }
                if next_trimmed.starts_with("##") {
                    break;
                }
                if !next_trimmed.is_empty() {
                    content.push(*next);
                }
            }
            if !content.is_empty() {
                return Some(content.join("\n").trim().to_string());
            }
        }
    }
    None
}

fn extract_section_list(body: &str, heading: &str) -> Vec<String> {
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == format!("# {heading}")
            || trimmed == format!("## {heading}")
            || trimmed == format!("### {heading}")
        {
            let mut items = Vec::new();
            for next in lines.iter().skip(i + 1) {
                let next_trimmed = next.trim();
                if next_trimmed.starts_with("#") && !next_trimmed.starts_with("##") {
                    break;
                }
                if next_trimmed.starts_with("##") {
                    break;
                }
                if let Some(item) = next_trimmed.strip_prefix("- ") {
                    items.push(item.trim().to_string());
                } else if let Some(item) = next_trimmed.strip_prefix("* ") {
                    items.push(item.trim().to_string());
                }
            }
            return items;
        }
    }
    Vec::new()
}

fn scan_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(scan_md_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
    files
}

#[derive(Debug, Clone)]
struct DiagnosticEntry {
    message: String,
    severity: String,
}

// ─── Markdown summary formatters ───────────────────────────────────

fn format_project_digest_md(
    title: &str,
    status: &str,
    milestone: &Option<String>,
    ready_specs: &[CompactSpec],
    review_specs: &[CompactSpec],
    blockers: &[String],
    ready_handoffs: &[String],
    active_decisions: &[CompactAdr],
    diagnostics: &DiagnosticsSummary,
    version: &Option<String>,
    warnings: &[String],
) -> String {
    let mut md = format!("# Project Digest: {title}\n\n**Status:** {status}\n");
    if let Some(ms) = milestone {
        md.push_str(&format!("**Milestone:** {ms}\n"));
    }
    if let Some(v) = version {
        md.push_str(&format!("**Kit version:** {v}\n"));
    }
    md.push('\n');

    if !ready_specs.is_empty() {
        md.push_str(&format!("## Ready specs ({})\n\n", ready_specs.len()));
        for spec in ready_specs {
            md.push_str(&format!("- **{}**: {}", spec.id, spec.title));
            if let Some(agent) = &spec.recommended_agent {
                md.push_str(&format!(" (agent: {agent})"));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    if !review_specs.is_empty() {
        md.push_str(&format!("## Specs in review ({})\n\n", review_specs.len()));
        for spec in review_specs {
            md.push_str(&format!("- **{}**: {}\n", spec.id, spec.title));
        }
        md.push('\n');
    }

    if !blockers.is_empty() {
        md.push_str("## Blockers\n\n");
        for blocker in blockers {
            md.push_str(&format!("- ⛔ {blocker}\n"));
        }
        md.push('\n');
    }

    if !ready_handoffs.is_empty() {
        md.push_str(&format!("## Ready handoffs ({})\n\n", ready_handoffs.len()));
        for h in ready_handoffs {
            md.push_str(&format!("- {h}\n"));
        }
        md.push('\n');
    }

    if !active_decisions.is_empty() {
        md.push_str("## Active decisions\n\n");
        for adr in active_decisions {
            md.push_str(&format!(
                "- **{}**: {} ({})\n",
                adr.id, adr.title, adr.status
            ));
        }
        md.push('\n');
    }

    md.push_str(&format!(
        "## Diagnostics\n\n- Total: {}\n- Errors: {}\n- Warnings: {}\n",
        diagnostics.total, diagnostics.errors, diagnostics.warnings
    ));

    if !warnings.is_empty() {
        md.push_str("\n## Warnings\n\n");
        for w in warnings {
            md.push_str(&format!("- ⚠ {w}\n"));
        }
    }

    md
}

fn format_spec_context_md(
    id: &str,
    title: &str,
    status: &str,
    priority: &Option<String>,
    area: &Option<String>,
    milestone: &Option<String>,
    recommended_agent: &Option<String>,
    agent_profile: &Option<AgentProfileSummary>,
    criteria: &[Criterion],
    linked_decisions: &[CompactAdr],
    related_reviews: &[CompactReview],
    applicable_skills: &[SkillSummary],
    explicit_files: &[String],
    explicit_areas: &[String],
    diagnostics: &[String],
    warnings: &[String],
) -> String {
    let mut md = format!("# Spec Context: {id} — {title}\n\n**Status:** {status}\n");
    if let Some(p) = priority {
        md.push_str(&format!("**Priority:** {p}\n"));
    }
    if let Some(a) = area {
        md.push_str(&format!("**Area:** {a}\n"));
    }
    if let Some(ms) = milestone {
        md.push_str(&format!("**Milestone:** {ms}\n"));
    }
    if let Some(agent) = recommended_agent {
        md.push_str(&format!("**Recommended agent:** {agent}\n"));
        if let Some(profile) = agent_profile {
            if let Some(name) = &profile.mnemonic_name {
                md.push_str(&format!("  - Mnemonic name: {name}\n"));
            }
            md.push_str(&format!(
                "  - Role: {}\n",
                profile.role.as_deref().unwrap_or("unspecified")
            ));
            md.push_str(&format!("  - Status: {}\n", profile.status));
        }
    }
    md.push('\n');

    if !criteria.is_empty() {
        let checked = criteria.iter().filter(|c| c.checked).count();
        md.push_str(&format!(
            "## Acceptance criteria ({checked}/{})\n\n",
            criteria.len()
        ));
        for c in criteria {
            let marker = if c.checked { "[x]" } else { "[ ]" };
            md.push_str(&format!("- {marker} {}\n", c.text));
        }
        md.push('\n');
    }

    if !linked_decisions.is_empty() {
        md.push_str("## Linked decisions\n\n");
        for adr in linked_decisions {
            md.push_str(&format!(
                "- **{}**: {} ({})\n",
                adr.id, adr.title, adr.status
            ));
        }
        md.push('\n');
    }

    if !related_reviews.is_empty() {
        md.push_str("## Related reviews\n\n");
        for r in related_reviews {
            md.push_str(&format!("- **{}**: {} ({})\n", r.id, r.title, r.status));
        }
        md.push('\n');
    }

    if !applicable_skills.is_empty() {
        md.push_str("## Applicable skills\n\n");
        for skill in applicable_skills {
            md.push_str(&format!(
                "- **{}**: {} ({}, risk: {})",
                skill.id,
                skill.title,
                skill.kind.as_deref().unwrap_or("procedure"),
                skill.risk.as_deref().unwrap_or("unspecified")
            ));
            if skill.requires_operator_approval.unwrap_or(false) {
                md.push_str(" - operator approval required");
            }
            md.push('\n');
            for command in &skill.commands {
                md.push_str(&format!("  - `{command}`\n"));
            }
        }
        md.push('\n');
    }

    if !explicit_files.is_empty() {
        md.push_str("## Files involved\n\n");
        for f in explicit_files {
            md.push_str(&format!("- `{f}`\n"));
        }
        md.push('\n');
    }

    if !explicit_areas.is_empty() {
        md.push_str("## Areas involved\n\n");
        for a in explicit_areas {
            md.push_str(&format!("- {a}\n"));
        }
        md.push('\n');
    }

    if !diagnostics.is_empty() {
        md.push_str("## Diagnostics\n\n");
        for d in diagnostics {
            md.push_str(&format!("- ⚠ {d}\n"));
        }
        md.push('\n');
    }

    if !warnings.is_empty() {
        md.push_str("## Warnings\n\n");
        for w in warnings {
            md.push_str(&format!("- ⚠ {w}\n"));
        }
    }

    md
}

fn format_review_context_md(
    spec_id: &str,
    spec_title: &str,
    criteria: &[Criterion],
    implementation_evidence: &Option<String>,
    linked_reviews: &[CompactReview],
    relevant_decisions: &[CompactAdr],
    verification_commands: &[String],
    applicable_skills: &[SkillSummary],
    warnings: &[String],
) -> String {
    let mut md = format!("# Review Context: {spec_id} — {spec_title}\n\n");

    if !criteria.is_empty() {
        let checked = criteria.iter().filter(|c| c.checked).count();
        md.push_str(&format!(
            "## Acceptance criteria ({checked}/{})\n\n",
            criteria.len()
        ));
        for c in criteria {
            let marker = if c.checked { "[x]" } else { "[ ]" };
            md.push_str(&format!("- {marker} {}\n", c.text));
        }
        md.push('\n');
    }

    if let Some(evidence) = implementation_evidence {
        md.push_str("## Implementation evidence\n\n");
        md.push_str(evidence);
        md.push_str("\n\n");
    }

    if !linked_reviews.is_empty() {
        md.push_str("## Reviews\n\n");
        for r in linked_reviews {
            md.push_str(&format!("- **{}**: {} ({})\n", r.id, r.title, r.status));
        }
        md.push('\n');
    }

    if !relevant_decisions.is_empty() {
        md.push_str("## Relevant decisions\n\n");
        for adr in relevant_decisions {
            md.push_str(&format!(
                "- **{}**: {} ({})\n",
                adr.id, adr.title, adr.status
            ));
        }
        md.push('\n');
    }

    if !verification_commands.is_empty() {
        md.push_str("## Verification commands\n\n");
        for cmd in verification_commands {
            md.push_str(&format!("- `{cmd}`\n"));
        }
        md.push('\n');
    }

    if !applicable_skills.is_empty() {
        md.push_str("## Applicable skills\n\n");
        for skill in applicable_skills {
            md.push_str(&format!(
                "- **{}**: {} ({}, risk: {})",
                skill.id,
                skill.title,
                skill.kind.as_deref().unwrap_or("procedure"),
                skill.risk.as_deref().unwrap_or("unspecified")
            ));
            if skill.requires_operator_approval.unwrap_or(false) {
                md.push_str(" - operator approval required");
            }
            md.push('\n');
            for command in &skill.commands {
                md.push_str(&format!("  - `{command}`\n"));
            }
        }
        md.push('\n');
    }

    if !warnings.is_empty() {
        md.push_str("## Warnings\n\n");
        for w in warnings {
            md.push_str(&format!("- ⚠ {w}\n"));
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_brain(root: &Path) {
        let lmbrain = root.join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
        fs::create_dir_all(lmbrain.join("specs/review")).unwrap();
        fs::create_dir_all(lmbrain.join("specs/working")).unwrap();
        fs::create_dir_all(lmbrain.join("specs/done")).unwrap();
        fs::create_dir_all(lmbrain.join("decisions")).unwrap();
        fs::create_dir_all(lmbrain.join("reviews/pending")).unwrap();
        fs::create_dir_all(lmbrain.join("reviews/accepted")).unwrap();
        fs::create_dir_all(lmbrain.join("handoffs/active")).unwrap();
        fs::create_dir_all(lmbrain.join("agents/profiles")).unwrap();
        fs::create_dir_all(lmbrain.join("knowledge")).unwrap();

        // STATUS.md
        fs::write(
            lmbrain.join("STATUS.md"),
            "# Project TestBrain\n\n**Status:** active\n\n## Current milestone\n\nM-03: Context economy\n",
        )
        .unwrap();

        // VERSION
        fs::write(lmbrain.join("VERSION"), "2.1.2").unwrap();

        // Ready spec
        fs::write(
            lmbrain.join("specs/ready/SPEC-023-v3-context-economy.md"),
            r#"---
id: SPEC-023
title: "V3 context economy"
status: ready
priority: critical
area: workflow
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_decisions: [ADR-004]
links: [ADR-004]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, tokens, workflow]
---
# V3 context economy

## Objective

Reduce token waste.

## Acceptance criteria

- [ ] Kit documents context tiers
- [ ] MCP exposes context-pack tools

## Files and areas involved

- lmbrain-mcp/src/main.rs
- lmbrain-core/src/

## Required verification

- cargo test
- pnpm lint
"#,
        )
        .unwrap();

        // Review spec
        fs::write(
            lmbrain.join("specs/review/SPEC-022-something.md"),
            r#"---
id: SPEC-022
title: "Something else"
status: review
priority: high
area: backend
milestone: M-03
recommended_agent: AGENT-BACKEND
related_decisions: []
links: []
created: 2026-06-30
updated: 2026-07-01
tags: []
---
# Something else

## Acceptance criteria

- [x] Done
"#,
        )
        .unwrap();

        // ADR
        fs::write(
            lmbrain.join("decisions/ADR-004-context-economy.md"),
            r#"---
id: ADR-004
title: "Context economy architecture"
status: accepted
decision_date: 2026-07-01
decider: project-lead
created: 2026-07-01
updated: 2026-07-01
tags: [architecture]
links: []
---
# ADR-004
"#,
        )
        .unwrap();

        // Agent profile
        fs::write(
            lmbrain.join("agents/profiles/AGENT-FULLSTACK-DESKTOP.md"),
            r#"---
id: AGENT-FULLSTACK-DESKTOP
title: "Fullstack Desktop Specialist"
mnemonic_name: "Sam Stacktrace"
status: active
role: "Fullstack Rust/TypeScript developer"
activation: manual
can_implement: true
can_review: true
created: 2026-06-01
updated: 2026-07-01
tags: [rust, typescript, tauri]
links: []
---
# Fullstack Desktop Specialist
"#,
        )
        .unwrap();

        // Handoff
        fs::write(
            lmbrain.join("handoffs/active/HANDOFF-001-test.md"),
            r#"---
id: HANDOFF-001
title: "Test handoff"
status: ready
created: 2026-07-02
updated: 2026-07-02
tags: []
links: []
---
# Test handoff
"#,
        )
        .unwrap();

        // Review for SPEC-022
        fs::write(
            lmbrain.join("reviews/accepted/REVIEW-001-something.md"),
            r#"---
id: REVIEW-001
title: "Review of SPEC-022"
status: accepted
spec: SPEC-022
reviewer: project-lead
created: 2026-07-01
updated: 2026-07-01
tags: []
links: []
---
# Review
"#,
        )
        .unwrap();
    }

    #[test]
    fn project_digest_includes_ready_specs() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let digest = build_project_digest(dir.path());
        assert_eq!(digest.title, "TestBrain");
        assert!(digest.ready_specs.iter().any(|s| s.id == "SPEC-023"));
        assert!(digest.review_specs.iter().any(|s| s.id == "SPEC-022"));
        assert_eq!(digest.ready_handoffs.len(), 1);
        assert!(digest.active_decisions.iter().any(|a| a.id == "ADR-004"));
        assert!(digest.markdown.contains("Project Digest"));
    }

    #[test]
    fn spec_context_resolves_linked_adr_and_agent() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let ctx = build_spec_context(dir.path(), "SPEC-023").unwrap();
        assert_eq!(ctx.id, "SPEC-023");
        assert_eq!(ctx.status, "ready");
        assert!(ctx.linked_decisions.iter().any(|a| a.id == "ADR-004"));
        assert!(ctx.agent_profile.is_some());
        assert_eq!(
            ctx.agent_profile.as_ref().unwrap().id,
            "AGENT-FULLSTACK-DESKTOP"
        );
        assert_eq!(
            ctx.agent_profile.as_ref().unwrap().mnemonic_name.as_deref(),
            Some("Sam Stacktrace")
        );
        assert_eq!(ctx.acceptance_criteria.len(), 2);
        assert!(!ctx.acceptance_criteria[0].checked);
        assert!(ctx.explicit_files.iter().any(|f| f.contains("main.rs")));
        assert!(ctx.markdown.contains("Spec Context"));
        assert!(ctx.markdown.contains("Sam Stacktrace"));
    }

    #[test]
    fn spec_context_missing_agent_warning() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        // Create a spec with a non-existent agent
        let lmbrain = dir.path().join(".lmbrain");
        fs::write(
            lmbrain.join("specs/ready/SPEC-099-bogus.md"),
            r#"---
id: SPEC-099
title: "Bogus spec"
status: ready
recommended_agent: AGENT-NONEXISTENT
related_decisions: [ADR-999]
created: 2026-07-02
updated: 2026-07-02
tags: []
links: []
---
# Bogus
"#,
        )
        .unwrap();

        let ctx = build_spec_context(dir.path(), "SPEC-099").unwrap();
        assert!(ctx.warnings.iter().any(|w| w.contains("AGENT-NONEXISTENT")));
        assert!(ctx.warnings.iter().any(|w| w.contains("ADR-999")));
    }

    #[test]
    fn spec_context_missing_link_warning() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let ctx = build_spec_context(dir.path(), "SPEC-023").unwrap();
        // SPEC-023 links ADR-004 which exists, so no warning for that
        assert!(!ctx.warnings.iter().any(|w| w.contains("ADR-004")));
    }

    #[test]
    fn review_context_parses_criteria_and_evidence() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let ctx = build_review_context(dir.path(), "SPEC-023").unwrap();
        assert_eq!(ctx.spec_id, "SPEC-023");
        assert_eq!(ctx.acceptance_criteria.len(), 2);
        assert!(ctx
            .verification_commands
            .iter()
            .any(|c| c.contains("cargo test")));
        assert!(ctx.markdown.contains("Review Context"));
    }

    #[test]
    fn review_context_for_completed_spec() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let ctx = build_review_context(dir.path(), "SPEC-022").unwrap();
        assert_eq!(ctx.spec_id, "SPEC-022");
        // SPEC-022 has checked criteria
        assert!(ctx.acceptance_criteria.iter().any(|c| c.checked));
        // SPEC-022 has a linked review
        assert!(ctx.linked_reviews.iter().any(|r| r.id == "REVIEW-001"));
    }

    #[test]
    fn project_digest_no_brain_returns_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let digest = build_project_digest(dir.path());
        assert!(digest.ready_specs.is_empty());
        assert!(digest.review_specs.is_empty());
        assert!(digest.ready_handoffs.is_empty());
        assert!(digest.active_decisions.is_empty());
    }

    #[test]
    fn spec_context_not_found_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = build_spec_context(dir.path(), "SPEC-999");
        assert!(result.is_err());
    }

    #[test]
    fn parse_criteria_handles_empty_body() {
        let criteria = parse_criteria("");
        assert!(criteria.is_empty());
    }

    #[test]
    fn parse_criteria_handles_mixed_checked() {
        let body = "- [ ] Not done\n- [x] Done\n- [X] Also done\n- Not a criterion\n";
        let criteria = parse_criteria(body);
        assert_eq!(criteria.len(), 3);
        assert!(!criteria[0].checked);
        assert!(criteria[1].checked);
        assert!(criteria[2].checked);
    }

    #[test]
    fn extract_section_returns_none_for_missing() {
        assert!(extract_section("No heading here", "Missing").is_none());
    }

    #[test]
    fn extract_section_list_returns_items() {
        let body = "## Files\n\n- file1.rs\n- file2.rs\n\n## Other\n";
        let items = extract_section_list(body, "Files");
        assert_eq!(items.len(), 2);
        assert!(items.contains(&"file1.rs".to_string()));
    }

    #[test]
    fn diagnostics_detects_malformed_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
        // Write a spec with malformed frontmatter (missing closing ---)
        fs::write(
            lmbrain.join("specs/ready/SPEC-099-malformed.md"),
            "---\nid: SPEC-099\nstatus: ready\nunclosed frontmatter\n",
        )
        .unwrap();
        let diagnostics = scan_diagnostics(dir.path());
        assert!(
            diagnostics.iter().any(|d| d.message.contains("Malformed")),
            "Expected a malformed frontmatter diagnostic, got: {diagnostics:?}"
        );
    }

    #[test]
    fn diagnostics_detects_status_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
        // Write a spec in specs/ready/ but with status: working
        fs::write(
            lmbrain.join("specs/ready/SPEC-099-mismatch.md"),
            r#"---
id: SPEC-099
title: "Mismatch"
status: working
created: 2026-07-02
updated: 2026-07-02
tags: []
links: []
---
# Mismatch
"#,
        )
        .unwrap();
        let diagnostics = scan_diagnostics(dir.path());
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("Status mismatch")),
            "Expected a status mismatch diagnostic, got: {diagnostics:?}"
        );
    }

    #[test]
    fn diagnostics_detects_unresolved_agent() {
        let dir = tempfile::tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
        fs::create_dir_all(lmbrain.join("agents/profiles")).unwrap();
        // Write a spec with a non-existent recommended agent
        fs::write(
            lmbrain.join("specs/ready/SPEC-099-noagent.md"),
            r#"---
id: SPEC-099
title: "No agent"
status: ready
recommended_agent: AGENT-NONEXISTENT
created: 2026-07-02
updated: 2026-07-02
tags: []
links: []
---
# No agent
"#,
        )
        .unwrap();
        let diagnostics = scan_diagnostics(dir.path());
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("AGENT-NONEXISTENT")),
            "Expected an unresolved agent diagnostic, got: {diagnostics:?}"
        );
    }

    #[test]
    fn project_digest_reports_non_empty_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
        fs::create_dir_all(lmbrain.join("agents/profiles")).unwrap();
        // STATUS.md
        fs::write(
            lmbrain.join("STATUS.md"),
            "# Project Test\n\n**Status:** active\n",
        )
        .unwrap();
        // A spec with unresolved agent
        fs::write(
            lmbrain.join("specs/ready/SPEC-099-noagent.md"),
            r#"---
id: SPEC-099
title: "No agent"
status: ready
recommended_agent: AGENT-MISSING
created: 2026-07-02
updated: 2026-07-02
tags: []
links: []
---
# No agent
"#,
        )
        .unwrap();
        let digest = build_project_digest(dir.path());
        assert!(
            digest.diagnostics_summary.total > 0,
            "Expected non-zero diagnostics total, got {}",
            digest.diagnostics_summary.total
        );
        assert!(
            !digest.blockers.is_empty() || !digest.warnings.is_empty(),
            "Expected blockers or warnings from diagnostics"
        );
    }
}
