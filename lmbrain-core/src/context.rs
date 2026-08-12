use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    frontmatter::Document,
    review::{parse_review_event_history, ReviewLifecycleEvent},
    taxonomy::{normalize_finding_category, CategoryNormalization},
    verification::load_verification_manifest,
};

// ─── Context-pack data structures ─────────────────────────────────

/// Compact project overview for Project Lead bootstrap and pulse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDigest {
    pub schema_version: String,
    pub title: String,
    pub status: String,
    pub current_milestone: Option<String>,
    pub ready_specs: Vec<CompactSpec>,
    pub review_specs: Vec<CompactSpec>,
    pub blockers: Vec<String>,
    pub blockers_omitted: usize,
    pub ready_handoffs: Vec<String>,
    pub active_decisions: Vec<CompactAdr>,
    pub diagnostics_summary: DiagnosticsSummary,
    pub version: Option<String>,
    pub warnings: Vec<String>,
    pub warnings_omitted: usize,
    pub declared_state: DeclaredProjectState,
    pub derived_state: DerivedProjectState,
    pub lifecycle: SpecLifecycleView,
    pub diagnostics: BoundedDiagnosticList,
    pub debts: DebtDigest,
    pub spec_dependencies: SpecDependencyDigest,
    pub branching_strategy: BranchingStrategyDigest,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDependencyDigest {
    pub blocked_total: usize,
    pub blocked_omitted: usize,
    pub blocked: Vec<SpecDependencyDigestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDependencyDigestItem {
    pub id: String,
    pub status: String,
    pub blockers: Vec<crate::SpecDependencyBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredProjectState {
    pub title: String,
    pub status: String,
    pub current_milestone: Option<String>,
    pub source: String,
    pub source_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedProjectState {
    pub current_milestone: Option<String>,
    pub basis: String,
    pub active_milestones: Vec<String>,
    pub planned_milestones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedSpecList {
    pub total: usize,
    pub omitted: usize,
    pub items: Vec<CompactSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecLifecycleView {
    pub counts: BTreeMap<String, usize>,
    pub backlog: BoundedSpecList,
    pub ready: BoundedSpecList,
    pub working: BoundedSpecList,
    pub review: BoundedSpecList,
    pub done: BoundedSpecList,
    pub discarded: BoundedSpecList,
    pub recently_completed: BoundedSpecList,
    pub forced: usize,
    pub malformed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedDiagnosticList {
    pub total: usize,
    pub omitted: usize,
    pub items: Vec<crate::Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactDebt {
    pub id: String,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub category: String,
    pub owner: Option<String>,
    pub milestone: Option<String>,
    pub origin_artifact: Option<String>,
    pub target_specs: Vec<String>,
    pub blocked_by: Vec<String>,
    pub updated: String,
    pub path: String,
    pub relation_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedDebtList {
    pub total: usize,
    pub omitted: usize,
    pub items: Vec<CompactDebt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtDigest {
    pub counts: BTreeMap<String, usize>,
    pub active: BoundedDebtList,
    pub targetless_open: BoundedDebtList,
    pub blocked: BoundedDebtList,
    pub recently_closed: BoundedDebtList,
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
    pub updated: Option<String>,
    pub forced: bool,
    pub parking: Option<SpecParkingSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecParkingSummary {
    pub timestamp: String,
    pub actor: String,
    pub reason: String,
    pub revisit_condition: Option<String>,
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

const DIGEST_SPEC_LIMIT: usize = 20;
const DIGEST_DIAGNOSTIC_LIMIT: usize = 50;
const DIGEST_DEBT_LIMIT: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchingStrategyDigest {
    pub status: String,
    pub topology: Option<String>,
    pub default_branch: Option<String>,
    pub lead_only_push_branches: Vec<String>,
    pub allow_specialist_push: bool,
    pub allowed_prefixes: Vec<String>,
}

pub fn build_branching_strategy_digest(root: &Path) -> BranchingStrategyDigest {
    match crate::load_branching_strategy(root) {
        Ok(Some(strategy)) => BranchingStrategyDigest {
            status: "declared".to_string(),
            topology: Some(format!("{:?}", strategy.topology).to_lowercase()),
            default_branch: Some(strategy.default_branch),
            lead_only_push_branches: strategy.authority.lead_only_push_branches,
            allow_specialist_push: strategy.authority.allow_specialist_push,
            allowed_prefixes: strategy.branch_naming.allowed_prefixes,
        },
        Ok(None) => BranchingStrategyDigest {
            status: "absent".to_string(),
            topology: None,
            default_branch: None,
            lead_only_push_branches: Vec::new(),
            allow_specialist_push: false,
            allowed_prefixes: Vec::new(),
        },
        Err(_) => BranchingStrategyDigest {
            status: "invalid".to_string(),
            topology: None,
            default_branch: None,
            lead_only_push_branches: Vec::new(),
            allow_specialist_push: false,
            allowed_prefixes: Vec::new(),
        },
    }
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
    pub tags: Vec<String>,
    pub capability_tier: Option<String>,
    pub thinking_level: Option<String>,
    /// Why the recommendation applies, in plain language. Never names a model.
    pub effort_rationale: Option<String>,
    pub parking: Option<SpecParkingSummary>,
    pub agent_profile: Option<AgentProfileSummary>,
    pub acceptance_criteria: Vec<Criterion>,
    pub required_verification: Vec<VerificationRequirement>,
    pub required_verification_source: Option<String>,
    pub linked_decisions: Vec<CompactAdr>,
    pub related_reviews: Vec<CompactReview>,
    pub applicable_skills: Vec<SkillSummary>,
    pub explicit_files: Vec<String>,
    pub explicit_areas: Vec<String>,
    pub diagnostics: Vec<String>,
    pub debts: Vec<CompactDebt>,
    pub dependencies: crate::SpecDependencyContext,
    pub branching_strategy: BranchingStrategyDigest,
    pub warnings: Vec<String>,
    pub markdown: String,
}

/// A single acceptance criterion with its check state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub text: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirement {
    pub id: String,
    pub text: String,
    pub checked: bool,
    pub kind: String,
    pub owner: String,
    pub phase: String,
    pub evidence: String,
    pub source: String,
}

/// Compact review reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactReview {
    pub id: String,
    pub title: String,
    pub status: String,
    pub reviewer: Option<String>,
    pub finding_categories: Vec<CategoryNormalization>,
    pub events: Vec<ReviewLifecycleEvent>,
    pub lifecycle_warnings: Vec<String>,
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
    pub domains: Vec<String>,
    pub primary_files: Vec<String>,
    pub review_focus: Vec<String>,
    pub constraints: Vec<String>,
    pub knowledge: Vec<String>,
    pub path: String,
    pub content_digest: String,
    pub operational_guidance: String,
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
    pub content_digest: String,
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
    pub required_verification: Vec<VerificationRequirement>,
    pub required_verification_source: Option<String>,
    pub applicable_skills: Vec<SkillSummary>,
    pub debts: Vec<CompactDebt>,
    pub warnings: Vec<String>,
    pub markdown: String,
}

// ─── Resolution functions ──────────────────────────────────────────

/// Build a project digest from the .lmbrain directory.
pub fn build_project_digest(root: &Path) -> ProjectDigest {
    let lmbrain = root.join(".lmbrain");

    // Read STATUS.md
    let (title, status, milestone) = read_status(&lmbrain);
    let status_present = lmbrain.join("STATUS.md").is_file();
    let version = read_version(&lmbrain);

    // Scan specs
    let mut all_specs = scan_specs(&lmbrain);
    all_specs.sort_by(|left, right| left.id.cmp(&right.id));
    let ready_all: Vec<CompactSpec> = all_specs
        .iter()
        .filter(|s| s.status.as_deref() == Some("ready"))
        .cloned()
        .collect();
    let review_all: Vec<CompactSpec> = all_specs
        .iter()
        .filter(|s| s.status.as_deref() == Some("review"))
        .cloned()
        .collect();
    let ready_specs = ready_all
        .iter()
        .take(DIGEST_SPEC_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let review_specs = review_all
        .iter()
        .take(DIGEST_SPEC_LIMIT)
        .cloned()
        .collect::<Vec<_>>();

    // Scan handoffs
    let ready_handoffs = scan_ready_handoffs(&lmbrain);

    // Scan ADRs
    let active_decisions = scan_active_adrs(&lmbrain);

    // Scan diagnostics
    let all_diagnostics = crate::diagnostics::build_diagnostics(root);
    let diag_summary = DiagnosticsSummary {
        errors: all_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Error)
            .count(),
        warnings: all_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Warning)
            .count(),
        total: all_diagnostics.len(),
    };

    // Collect blockers from diagnostics
    let blocker_messages = all_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    let blockers = blocker_messages
        .iter()
        .take(DIGEST_DIAGNOSTIC_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let blockers_omitted = blocker_messages.len().saturating_sub(blockers.len());
    let warning_messages = all_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Warning)
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    let warnings = warning_messages
        .iter()
        .take(DIGEST_DIAGNOSTIC_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let warnings_omitted = warning_messages.len().saturating_sub(warnings.len());
    let lifecycle = build_lifecycle_view(&all_specs, &all_diagnostics);
    let declared_state = DeclaredProjectState {
        title: title.clone(),
        status: status.clone(),
        current_milestone: milestone.clone(),
        source: "STATUS.md".into(),
        source_present: status_present,
    };
    let derived_state = derive_project_state(&lmbrain, &all_specs);
    let diagnostics = BoundedDiagnosticList {
        total: all_diagnostics.len(),
        omitted: all_diagnostics
            .len()
            .saturating_sub(DIGEST_DIAGNOSTIC_LIMIT),
        items: all_diagnostics
            .iter()
            .take(DIGEST_DIAGNOSTIC_LIMIT)
            .cloned()
            .collect(),
    };
    let debts = build_debt_digest(root);
    let dependency_items = all_specs
        .iter()
        .filter_map(|spec| {
            let context = crate::spec_dependency_context(root, &spec.id).ok()?;
            (!context.blockers.is_empty()).then_some(SpecDependencyDigestItem {
                id: spec.id.clone(),
                status: spec.status.clone().unwrap_or_default(),
                blockers: context.blockers,
            })
        })
        .collect::<Vec<_>>();
    let spec_dependencies = SpecDependencyDigest {
        blocked_total: dependency_items.len(),
        blocked_omitted: dependency_items.len().saturating_sub(DIGEST_SPEC_LIMIT),
        blocked: dependency_items
            .into_iter()
            .take(DIGEST_SPEC_LIMIT)
            .collect(),
    };

    let mut markdown = format_project_digest_md(
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
        warnings_omitted,
        &lifecycle,
        &diagnostics,
        &declared_state,
        &derived_state,
    );
    append_debts_markdown(&mut markdown, "Active debts", &debts.active.items);

    ProjectDigest {
        schema_version: "2".into(),
        title,
        status,
        current_milestone: milestone,
        ready_specs,
        review_specs,
        blockers,
        blockers_omitted,
        ready_handoffs,
        active_decisions,
        diagnostics_summary: diag_summary,
        version,
        warnings,
        warnings_omitted,
        declared_state,
        derived_state,
        lifecycle,
        diagnostics,
        debts,
        spec_dependencies,
        branching_strategy: build_branching_strategy_digest(root),
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
    let capability_tier = document
        .value("capability_tier")
        .and_then(|raw| crate::taxonomy::normalize_capability_tier(&raw));
    let thinking_level = document
        .value("thinking_level")
        .and_then(|raw| crate::taxonomy::normalize_thinking_level(&raw));
    let parking = latest_parking(&document);
    let spec_skills = document.string_array("skills");
    let spec_tags = document.string_array("tags");
    let related_decisions = document.string_array("related_decisions");

    // Parse only the named acceptance section. Other checklists have distinct semantics.
    let criteria = parse_acceptance_criteria(&document.body);
    let verification_refs = document.string_array("verification_gates");
    let (required_verification, required_verification_source, requirement_warnings) =
        parse_verification_requirements(&document.body, &verification_refs);

    // Resolve linked decisions
    let mut linked_decisions = Vec::new();
    let mut warnings = requirement_warnings;
    warnings.extend(verification_reference_warnings(root, &verification_refs));
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
    let debts = debts_for_spec(root, &id);
    let dependencies = crate::spec_dependency_context(root, &id)
        .map_err(|error| format!("Failed to resolve spec dependencies: {error}"))?;

    if recommended_agent.is_some() && agent_profile.is_none() {
        warnings.push(format!(
            "Recommended agent '{}' does not resolve to an existing profile",
            recommended_agent.as_deref().unwrap_or("")
        ));
    }

    let mut markdown = format_spec_context_md(
        &id,
        &title,
        &status,
        &priority,
        &area,
        &milestone,
        &recommended_agent,
        &agent_profile,
        &capability_tier,
        &thinking_level,
        &spec_tags,
        &criteria,
        &required_verification,
        &linked_decisions,
        &related_reviews,
        &applicable_skills,
        &explicit_files,
        &explicit_areas,
        &diagnostics,
        &warnings,
    );
    append_debts_markdown(&mut markdown, "Related debts", &debts);
    append_dependencies_markdown(&mut markdown, &dependencies);
    if let Some(event) = parking.as_ref() {
        markdown.push_str(&format!(
            "\n## Latest parking history\n\n- Parked by {} at {}: {}{}\n",
            event.actor,
            event.timestamp,
            event.reason,
            event
                .revisit_condition
                .as_deref()
                .map(|value| format!("; revisit: {value}"))
                .unwrap_or_default()
        ));
    }

    Ok(SpecContext {
        id,
        title,
        status,
        priority,
        area,
        milestone,
        recommended_agent,
        tags: spec_tags.clone(),
        capability_tier: capability_tier.clone(),
        thinking_level: thinking_level.clone(),
        effort_rationale: effort_rationale(capability_tier.as_deref(), thinking_level.as_deref()),
        parking,
        agent_profile,
        acceptance_criteria: criteria,
        required_verification,
        required_verification_source,
        linked_decisions,
        related_reviews,
        applicable_skills,
        explicit_files,
        explicit_areas,
        diagnostics,
        debts,
        dependencies,
        branching_strategy: build_branching_strategy_digest(root),
        warnings,
        markdown,
    })
}

/// Explains an implementation estimate in plain language for handoff context.
/// It describes capability classes only: naming a provider or model here would
/// bake one vendor's product names into a committed artifact.
fn effort_rationale(tier: Option<&str>, level: Option<&str>) -> Option<String> {
    let tier = tier?;
    let footprint = match tier {
        "luna" => "a small footprint: about two files and a change that is known before starting",
        "terra" => "a moderate footprint: several files in one layer, possibly a new shared helper",
        "sol" => {
            "a large footprint: many files, or work crossing the frontend, core, MCP, and the Markdown contract"
        }
        _ => return None,
    };
    let deliberation = match level.unwrap_or(crate::taxonomy::default_thinking_level(tier)) {
        "minimal" => "mechanical work that needs little deliberation",
        "standard" => "ordinary deliberation",
        "extended" => "careful deliberation: consequences are not local",
        "maximum" => "maximum deliberation: invariants or compatibility are at stake",
        _ => "ordinary deliberation",
    };
    Some(format!(
        "The Project Lead expects {footprint}, and {deliberation}. This is a recommendation, not a guarantee, and it never selects or starts an agent."
    ))
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

    // Parse acceptance criteria and the complete verification contract.
    let criteria = parse_acceptance_criteria(&document.body);
    let verification_refs = document.string_array("verification_gates");
    let (required_verification, required_verification_source, requirement_warnings) =
        parse_verification_requirements(&document.body, &verification_refs);

    // Extract implementation evidence
    let (implementation_evidence, evidence_warnings) =
        extract_implementation_evidence(&document.body);

    // Resolve related reviews
    let linked_reviews = resolve_reviews_for_spec(&lmbrain, &spec_id);

    // Resolve linked decisions
    let related_decisions: Vec<String> = document.string_array("related_decisions");
    let mut relevant_decisions = Vec::new();
    let mut warnings = requirement_warnings;
    warnings.extend(evidence_warnings);
    warnings.extend(verification_reference_warnings(root, &verification_refs));
    for review in &linked_reviews {
        warnings.extend(
            review
                .lifecycle_warnings
                .iter()
                .map(|warning| format!("{}: {warning}", review.id)),
        );
    }
    for adr_id in &related_decisions {
        match resolve_adr(&lmbrain, adr_id) {
            Some(adr) => relevant_decisions.push(adr),
            None => warnings.push(format!("Linked ADR {adr_id} not found")),
        }
    }

    // Extract verification commands
    let verification_commands = required_verification
        .iter()
        .filter(|requirement| {
            matches!(
                requirement.kind.as_str(),
                "command" | "manifest" | "executable" | "unstructured"
            )
        })
        .map(|requirement| requirement.text.clone())
        .collect::<Vec<_>>();
    let applicable_skills = resolve_applicable_skills(
        &lmbrain,
        &spec_skills,
        recommended_agent.as_deref(),
        agent_profile.as_ref(),
        area.as_deref(),
        &spec_tags,
        true,
    );
    let review_ids = linked_reviews
        .iter()
        .map(|review| review.id.as_str())
        .collect::<BTreeSet<_>>();
    let debts = crate::list_debts(root)
        .into_iter()
        .filter_map(|debt| {
            let mut roles = Vec::new();
            if debt
                .origin_artifact
                .as_deref()
                .is_some_and(|id| review_ids.contains(id))
            {
                roles.push("origin".into());
            }
            if debt
                .related_reviews
                .iter()
                .any(|id| review_ids.contains(id.as_str()))
            {
                roles.push("related-review".into());
            }
            if roles.is_empty() {
                None
            } else {
                Some(compact_debt(debt, roles))
            }
        })
        .take(DIGEST_DEBT_LIMIT)
        .collect::<Vec<_>>();

    let mut markdown = format_review_context_md(
        &spec_id,
        &spec_title,
        &criteria,
        &implementation_evidence,
        &linked_reviews,
        &relevant_decisions,
        &verification_commands,
        &required_verification,
        &applicable_skills,
        &warnings,
    );
    append_debts_markdown(&mut markdown, "Promoted debts", &debts);

    Ok(ReviewContext {
        spec_id,
        spec_title,
        acceptance_criteria: criteria,
        implementation_evidence,
        linked_reviews,
        relevant_decisions,
        verification_commands,
        required_verification,
        required_verification_source,
        applicable_skills,
        debts,
        warnings,
        markdown,
    })
}

// ─── Internal helpers ──────────────────────────────────────────────

fn build_debt_digest(root: &Path) -> DebtDigest {
    let mut debts = crate::list_debts(root)
        .into_iter()
        .filter(|debt| !debt.malformed)
        .map(|debt| compact_debt(debt, Vec::new()))
        .collect::<Vec<_>>();
    debts.sort_by(|left, right| {
        debt_severity_rank(&right.severity)
            .cmp(&debt_severity_rank(&left.severity))
            .then_with(|| right.updated.cmp(&left.updated))
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut counts = BTreeMap::new();
    for debt in &debts {
        *counts.entry(debt.status.clone()).or_insert(0) += 1;
    }
    let active = bounded_debts(
        debts
            .iter()
            .filter(|debt| matches!(debt.status.as_str(), "open" | "planned" | "deferred"))
            .cloned()
            .collect(),
    );
    let targetless_open = bounded_debts(
        debts
            .iter()
            .filter(|debt| debt.status == "open" && debt.target_specs.is_empty())
            .cloned()
            .collect(),
    );
    let blocked = bounded_debts(
        debts
            .iter()
            .filter(|debt| !debt.blocked_by.is_empty())
            .cloned()
            .collect(),
    );
    let mut closed = debts
        .into_iter()
        .filter(|debt| matches!(debt.status.as_str(), "resolved" | "accepted-risk"))
        .collect::<Vec<_>>();
    closed.sort_by(|left, right| right.updated.cmp(&left.updated));
    DebtDigest {
        counts,
        active,
        targetless_open,
        blocked,
        recently_closed: bounded_debts(closed),
    }
}

fn debts_for_spec(root: &Path, spec_id: &str) -> Vec<CompactDebt> {
    crate::list_debts(root)
        .into_iter()
        .filter_map(|debt| {
            let mut roles = Vec::new();
            if debt.origin_artifact.as_deref() == Some(spec_id) {
                roles.push("origin".into());
            }
            if debt.related_specs.iter().any(|id| id == spec_id) {
                roles.push("related".into());
            }
            if debt.target_specs.iter().any(|id| id == spec_id) {
                roles.push("target".into());
            }
            if roles.is_empty() {
                None
            } else {
                Some(compact_debt(debt, roles))
            }
        })
        .take(DIGEST_DEBT_LIMIT)
        .collect()
}

fn compact_debt(debt: crate::Debt, relation_roles: Vec<String>) -> CompactDebt {
    CompactDebt {
        id: debt.id,
        title: debt.title,
        status: debt.status,
        severity: debt.severity,
        category: debt.category,
        owner: debt.owner,
        milestone: debt.milestone,
        origin_artifact: debt.origin_artifact,
        target_specs: debt.target_specs,
        blocked_by: debt.blocked_by,
        updated: debt.updated,
        path: debt.path,
        relation_roles,
    }
}

fn bounded_debts(items: Vec<CompactDebt>) -> BoundedDebtList {
    let total = items.len();
    BoundedDebtList {
        total,
        omitted: total.saturating_sub(DIGEST_DEBT_LIMIT),
        items: items.into_iter().take(DIGEST_DEBT_LIMIT).collect(),
    }
}

fn debt_severity_rank(severity: &str) -> usize {
    match severity {
        "critical" => 5,
        "high" => 4,
        "medium" => 3,
        "low" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn append_debts_markdown(markdown: &mut String, heading: &str, debts: &[CompactDebt]) {
    markdown.push_str(&format!("\n## {heading}\n"));
    if debts.is_empty() {
        markdown.push_str("- None\n");
        return;
    }
    for debt in debts {
        let roles = if debt.relation_roles.is_empty() {
            String::new()
        } else {
            format!("; {}", debt.relation_roles.join(", "))
        };
        markdown.push_str(&format!(
            "- {} [{}; {}{}] {}\n",
            debt.id, debt.status, debt.severity, roles, debt.title
        ));
    }
}

fn append_dependencies_markdown(
    markdown: &mut String,
    dependencies: &crate::SpecDependencyContext,
) {
    markdown.push_str("\n## Hard spec dependencies\n");
    if dependencies.direct_prerequisites.is_empty() {
        markdown.push_str("\n- Direct prerequisites: none\n");
    } else {
        markdown.push_str("\n### Direct prerequisites\n");
        for dependency in &dependencies.direct_prerequisites {
            markdown.push_str(&format!(
                "\n- {} — {} ({})",
                dependency.id, dependency.title, dependency.status
            ));
        }
        markdown.push('\n');
    }
    if !dependencies.direct_dependents.is_empty() {
        markdown.push_str("\n### Direct dependents\n");
        for dependency in &dependencies.direct_dependents {
            markdown.push_str(&format!(
                "\n- {} — {} ({})",
                dependency.id, dependency.title, dependency.status
            ));
        }
        markdown.push('\n');
    }
    if !dependencies.blockers.is_empty() {
        markdown.push_str("\n### Blocking chains\n");
        for blocker in &dependencies.blockers {
            markdown.push_str(&format!(
                "\n- {} [{}] via {} — {}",
                blocker.id,
                blocker.status,
                blocker.chain.join(" -> "),
                blocker.cause
            ));
        }
        markdown.push('\n');
    }
    if dependencies.truncated {
        markdown.push_str("\n_Transitive dependency view truncated at its deterministic bound._\n");
    }
}

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
    let status = content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("**status:**") || lower.starts_with("status:") {
                trimmed
                    .split_once(':')
                    .map(|(_, value)| value.trim().trim_matches('*').trim().to_string())
                    .filter(|value| !value.is_empty())
            } else {
                None
            }
        })
        .or_else(|| extract_section(&content, "Status"))
        .unwrap_or_else(|| "unknown".to_string());
    let milestone = crate::diagnostics::extract_declared_milestone(&content);
    (title, status, milestone)
}

fn read_version(lmbrain: &Path) -> Option<String> {
    fs::read_to_string(lmbrain.join("VERSION"))
        .ok()
        .map(|v| v.trim().to_string())
}

fn latest_parking(document: &Document) -> Option<SpecParkingSummary> {
    let event = document.object_array("parking_events").into_iter().last()?;
    Some(SpecParkingSummary {
        timestamp: event.get("timestamp")?.as_str()?.to_string(),
        actor: event.get("actor")?.as_str()?.to_string(),
        reason: event.get("reason")?.as_str()?.to_string(),
        revisit_condition: event
            .get("revisit_condition")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    })
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
                                updated: doc.value("updated"),
                                forced: !doc.object_array("mutation_overrides").is_empty()
                                    || doc.body.contains("## Mutation override"),
                                parking: latest_parking(&doc),
                            });
                        }
                    }
                }
            }
        }
    }
    specs
}

fn build_lifecycle_view(
    specs: &[CompactSpec],
    diagnostics: &[crate::Diagnostic],
) -> SpecLifecycleView {
    let mut counts = BTreeMap::from([
        ("backlog".into(), 0),
        ("ready".into(), 0),
        ("working".into(), 0),
        ("review".into(), 0),
        ("done".into(), 0),
        ("discarded".into(), 0),
        ("malformed".into(), 0),
    ]);
    for spec in specs {
        if let Some(status) = spec.status.as_ref() {
            *counts.entry(status.clone()).or_default() += 1;
        }
    }
    let malformed = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == "frontmatter-malformed"
                && diagnostic
                    .path
                    .as_deref()
                    .is_some_and(|path| path.starts_with("specs/"))
        })
        .count();
    counts.insert("malformed".into(), malformed);
    SpecLifecycleView {
        counts,
        backlog: bounded_specs(specs, "backlog"),
        ready: bounded_specs(specs, "ready"),
        working: bounded_specs(specs, "working"),
        review: bounded_specs(specs, "review"),
        done: bounded_specs(specs, "done"),
        discarded: bounded_specs(specs, "discarded"),
        recently_completed: recently_completed_specs(specs),
        forced: specs.iter().filter(|spec| spec.forced).count(),
        malformed,
    }
}

fn bounded_specs(specs: &[CompactSpec], status: &str) -> BoundedSpecList {
    let matching = specs
        .iter()
        .filter(|spec| spec.status.as_deref() == Some(status))
        .cloned()
        .collect::<Vec<_>>();
    BoundedSpecList {
        total: matching.len(),
        omitted: matching.len().saturating_sub(DIGEST_SPEC_LIMIT),
        items: matching.into_iter().take(DIGEST_SPEC_LIMIT).collect(),
    }
}

fn recently_completed_specs(specs: &[CompactSpec]) -> BoundedSpecList {
    let mut matching = specs
        .iter()
        .filter(|spec| spec.status.as_deref() == Some("done"))
        .cloned()
        .collect::<Vec<_>>();
    matching.sort_by(|left, right| {
        right
            .updated
            .cmp(&left.updated)
            .then_with(|| right.id.cmp(&left.id))
    });
    BoundedSpecList {
        total: matching.len(),
        omitted: matching.len().saturating_sub(DIGEST_SPEC_LIMIT),
        items: matching.into_iter().take(DIGEST_SPEC_LIMIT).collect(),
    }
}

fn derive_project_state(lmbrain: &Path, specs: &[CompactSpec]) -> DerivedProjectState {
    let roadmap = fs::read_to_string(lmbrain.join("ROADMAP.md"))
        .ok()
        .map(|source| crate::diagnostics::parse_roadmap_milestones(&source))
        .unwrap_or_default();
    let active_milestones = roadmap
        .iter()
        .filter(|(_, milestone)| {
            milestone
                .status
                .as_deref()
                .is_some_and(|status| matches!(status, "active" | "in-progress" | "working"))
        })
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let planned_milestones = roadmap
        .iter()
        .filter(|(_, milestone)| {
            milestone
                .status
                .as_deref()
                .is_some_and(|status| matches!(status, "planned" | "proposed"))
        })
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let mut scores: BTreeMap<String, usize> = BTreeMap::new();
    for spec in specs {
        let score = match spec.status.as_deref() {
            Some("working") => 40,
            Some("review") => 30,
            Some("ready") => 20,
            Some("backlog") => 10,
            _ => 0,
        };
        if score > 0 {
            if let Some(milestone) = spec.milestone.as_ref() {
                *scores.entry(milestone.clone()).or_default() += score;
            }
        }
    }
    let maximum = scores.values().copied().max();
    let candidates = maximum
        .map(|maximum| {
            scores
                .into_iter()
                .filter(|(_, score)| *score == maximum)
                .map(|(milestone, _)| milestone)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    DerivedProjectState {
        current_milestone: (candidates.len() == 1).then(|| candidates[0].clone()),
        basis: "weighted open-spec lifecycle: working=40, review=30, ready=20, backlog=10".into(),
        active_milestones,
        planned_milestones,
    }
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
                        let rel_path = path
                            .strip_prefix(lmbrain)
                            .ok()
                            .map(|p| format!(".lmbrain/{}", p.to_string_lossy().replace('\\', "/")))
                            .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"));
                        return Some(AgentProfileSummary {
                            id: agent_id.to_string(),
                            title: doc.value("title").unwrap_or_default(),
                            mnemonic_name: doc.value("mnemonic_name"),
                            role: doc.value("role"),
                            status: doc.value("status").unwrap_or_default(),
                            can_implement: doc.bool("can_implement"),
                            can_review: doc.bool("can_review"),
                            skills: doc.string_array("skills"),
                            domains: doc.string_array("domains"),
                            primary_files: doc.string_array("primary_files"),
                            review_focus: doc.string_array("review_focus"),
                            constraints: doc.string_array("constraints"),
                            knowledge: doc.string_array("knowledge"),
                            path: rel_path,
                            content_digest: crate::content_digest(source.as_bytes()),
                            operational_guidance: bounded_guidance(&doc.body),
                        });
                    }
                }
            }
        }
    }
    None
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

        if (explicit
            || agent_default
            || applies_to_agent
            || domain_match
            || (review_only && review_kind))
            && !result
                .iter()
                .any(|skill: &SkillSummary| skill.id == summary.id)
        {
            result.push(summary);
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
                        let mut commands = doc.string_array("commands");
                        if commands.is_empty() {
                            commands = extract_fenced_commands(&doc.body);
                        }
                        skills.push((
                            SkillSummary {
                                id,
                                title: doc.value("title").unwrap_or_default(),
                                status: doc.value("status").unwrap_or_else(|| status.to_string()),
                                kind: doc.value("kind"),
                                risk: doc.value("risk"),
                                requires_operator_approval: doc.bool("requires_operator_approval"),
                                commands,
                                path: rel_path,
                                content_digest: crate::content_digest(source.as_bytes()),
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
                                let history = parse_review_event_history(&doc);
                                let finding_categories = doc
                                    .string_array("finding_categories")
                                    .iter()
                                    .map(|category| normalize_finding_category(category))
                                    .collect::<Vec<_>>();
                                let mut lifecycle_warnings = history.warnings;
                                lifecycle_warnings.extend(
                                    finding_categories
                                        .iter()
                                        .filter(|category| category.canonical.is_none())
                                        .map(|category| {
                                            format!(
                                                "Unknown debt category '{}'; add an alias or use a canonical category.",
                                                category.raw
                                            )
                                        }),
                                );
                                reviews.push(CompactReview {
                                    id: doc.value("id").unwrap_or_default(),
                                    title: doc.value("title").unwrap_or_default(),
                                    status: doc.value("status").unwrap_or_default(),
                                    reviewer: doc.value("reviewer"),
                                    finding_categories,
                                    events: history.events,
                                    lifecycle_warnings,
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
    let mut criteria = Vec::new();
    let mut current: Option<Criterion> = None;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- [") && trimmed.len() > 5 {
            if let Some(criterion) = current.take() {
                criteria.push(criterion);
            }
            let checked = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            current = Some(Criterion {
                text: trimmed[5..].trim().to_string(),
                checked,
            });
        } else if let Some(criterion) = current.as_mut() {
            if trimmed.is_empty() || trimmed.starts_with("#") || trimmed.starts_with("-") {
                if let Some(criterion) = current.take() {
                    criteria.push(criterion);
                }
            } else {
                criterion.text.push(' ');
                criterion.text.push_str(trimmed);
            }
        }
    }
    if let Some(criterion) = current {
        criteria.push(criterion);
    }
    criteria
}

fn parse_acceptance_criteria(body: &str) -> Vec<Criterion> {
    extract_section(body, "Acceptance criteria")
        .map(|section| parse_criteria(&section))
        .unwrap_or_default()
}

pub fn parse_verification_requirements(
    body: &str,
    manifest_refs: &[String],
) -> (Vec<VerificationRequirement>, Option<String>, Vec<String>) {
    let source = extract_section(body, "Required verification");
    let mut requirements = manifest_refs
        .iter()
        .map(|id| VerificationRequirement {
            id: id.clone(),
            text: format!("Manifest gate `{id}`"),
            checked: false,
            kind: "manifest".into(),
            owner: "kit".into(),
            phase: "before-submit".into(),
            evidence: "kit-transcript".into(),
            source: ".lmbrain/verification.toml".into(),
        })
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();
    let Some(section) = source.as_deref() else {
        if requirements.is_empty() {
            warnings.push(
                "Spec has no Required verification section or verification_gates references".into(),
            );
        }
        return (requirements, source, warnings);
    };

    let mut parsed_any = false;
    for (index, line) in section.lines().enumerate() {
        let trimmed = line.trim();
        let (checked, rest) = if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            (false, rest)
        } else if let Some(rest) = trimmed
            .strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("- [X] "))
        {
            (true, rest)
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            (false, rest)
        } else {
            continue;
        };
        parsed_any = true;
        let parts = rest.split('|').map(str::trim).collect::<Vec<_>>();
        let structured = parts.len() >= 2
            && parts[0]
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if structured {
            let mut fields = BTreeMap::new();
            let mut text_parts = Vec::new();
            for part in parts.iter().skip(1) {
                if let Some((key, value)) = part.split_once('=') {
                    let k = key.trim();
                    if matches!(k, "kind" | "owner" | "phase" | "evidence") {
                        fields.insert(k, value.trim());
                        continue;
                    }
                }
                if !part.is_empty() {
                    text_parts.push(*part);
                }
            }
            let text = text_parts.join(" | ");
            requirements.push(VerificationRequirement {
                id: parts[0].to_string(),
                text: if text.is_empty() {
                    rest.to_string()
                } else {
                    text
                },
                checked,
                kind: fields.get("kind").copied().unwrap_or("manual").to_string(),
                owner: fields.get("owner").copied().unwrap_or("agent").to_string(),
                phase: fields
                    .get("phase")
                    .copied()
                    .unwrap_or("before-submit")
                    .to_string(),
                evidence: fields
                    .get("evidence")
                    .copied()
                    .unwrap_or("transcript")
                    .to_string(),
                source: "spec:Required verification".into(),
            });
        } else {
            requirements.push(VerificationRequirement {
                id: format!("legacy-{:03}", index + 1),
                text: rest.to_string(),
                checked,
                kind: "unstructured".into(),
                owner: "unspecified".into(),
                phase: "unspecified".into(),
                evidence: "unspecified".into(),
                source: "spec:Required verification".into(),
            });
        }
    }
    if !parsed_any && !section.trim().is_empty() {
        requirements.push(VerificationRequirement {
            id: "legacy-unstructured".into(),
            text: section.trim().to_string(),
            checked: false,
            kind: "unstructured".into(),
            owner: "unspecified".into(),
            phase: "unspecified".into(),
            evidence: "unspecified".into(),
            source: "spec:Required verification".into(),
        });
    }
    let mut ids = BTreeSet::new();
    for requirement in &requirements {
        if !ids.insert(requirement.id.clone()) {
            warnings.push(format!(
                "Duplicate verification requirement id {}",
                requirement.id
            ));
        }
        if requirement.owner == "unspecified" || requirement.phase == "unspecified" {
            warnings.push(format!(
                "Verification requirement {} is unstructured or ownerless",
                requirement.id
            ));
        }
    }
    (requirements, source, warnings)
}

fn verification_reference_warnings(root: &Path, references: &[String]) -> Vec<String> {
    if references.is_empty() {
        return Vec::new();
    }
    let mut warnings = Vec::new();
    let mut seen = BTreeSet::new();
    for reference in references {
        if !seen.insert(reference) {
            warnings.push(format!(
                "Duplicate verification_gates reference {reference}"
            ));
        }
    }
    match load_verification_manifest(root) {
        Ok(manifest) => {
            let known = manifest
                .gates
                .into_iter()
                .map(|gate| gate.id)
                .collect::<BTreeSet<_>>();
            for reference in seen {
                if !known.contains(reference) {
                    warnings.push(format!("verification_gates reference {reference} is absent from .lmbrain/verification.toml"));
                }
            }
        }
        Err(error) => warnings.push(format!("verification_gates cannot resolve: {error}")),
    }
    warnings
}

const EVIDENCE_LIMIT: usize = 32 * 1024;

/// Extract `## Implementation evidence` for review context, emitting explicit
/// warnings instead of silently returning incomplete evidence.
fn extract_implementation_evidence(body: &str) -> (Option<String>, Vec<String>) {
    let heading = "Implementation evidence";
    match extract_section(body, heading) {
        None => {
            let warning = if has_section_heading(body, heading) {
                "Implementation evidence section is present but empty".to_string()
            } else {
                "Spec has no Implementation evidence section".to_string()
            };
            (None, vec![warning])
        }
        Some(text) if text.len() > EVIDENCE_LIMIT => {
            let end = (0..=EVIDENCE_LIMIT)
                .rev()
                .find(|index| text.is_char_boundary(*index))
                .unwrap_or(0);
            let bounded = format!(
                "{}\n...[implementation evidence truncated; read the full spec artifact]",
                text[..end].trim_end()
            );
            (
                Some(bounded),
                vec![format!(
                    "Implementation evidence exceeds {EVIDENCE_LIMIT} bytes and was truncated; read the full spec artifact for complete evidence"
                )],
            )
        }
        Some(text) => (Some(text), Vec::new()),
    }
}

fn bounded_guidance(body: &str) -> String {
    const LIMIT: usize = 8 * 1024;
    if body.len() <= LIMIT {
        body.trim().to_string()
    } else {
        let end = (0..=LIMIT)
            .rev()
            .find(|index| body.is_char_boundary(*index))
            .unwrap_or(0);
        format!(
            "{}\n...[profile guidance truncated; read full artifact]",
            &body[..end]
        )
    }
}

fn extract_fenced_commands(body: &str) -> Vec<String> {
    let source = extract_section(body, "Procedure").unwrap_or_else(|| body.to_string());
    let mut commands = Vec::new();
    let mut fenced = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            fenced = !fenced;
        } else if fenced && !trimmed.is_empty() && !trimmed.starts_with('#') {
            commands.push(trimmed.strip_prefix("$ ").unwrap_or(trimmed).to_string());
        }
    }
    commands
}

/// Fence delimiters toggle a state in which heading-like lines are content.
fn is_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Heading level of a Markdown ATX heading line (1-6), if any.
fn heading_level_of(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &trimmed[level..];
    (rest.starts_with(' ') && !rest.trim().is_empty()).then_some(level)
}

/// Heading level of a line whose text equals `heading` exactly, if any.
fn matching_heading_level(line: &str, heading: &str) -> Option<usize> {
    let level = heading_level_of(line)?;
    (line.trim()[level..].trim() == heading).then_some(level)
}

/// Collect the lines of the first non-empty section titled `heading`.
///
/// Heading-level aware: a match at level N includes nested deeper headings
/// and stops only at the next heading of level <= N, so a `##` section keeps
/// its `###` subsections. Heading-like lines inside fenced code blocks are
/// treated as content, never as boundaries.
fn section_body_lines<'a>(body: &'a str, heading: &str) -> Option<Vec<&'a str>> {
    let lines: Vec<&str> = body.lines().collect();
    let mut in_fence = false;
    for (i, line) in lines.iter().enumerate() {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let Some(level) = matching_heading_level(line, heading) else {
            continue;
        };
        let mut content = Vec::new();
        let mut content_fence = false;
        for next in &lines[i + 1..] {
            if is_fence(next) {
                content_fence = !content_fence;
                content.push(*next);
                continue;
            }
            if !content_fence {
                if let Some(next_level) = heading_level_of(next) {
                    if next_level <= level {
                        break;
                    }
                }
            }
            content.push(*next);
        }
        if content.iter().any(|entry| !entry.trim().is_empty()) {
            return Some(content);
        }
        // An empty match keeps searching so a stray duplicate heading cannot
        // shadow the real populated section further down.
    }
    None
}

fn has_section_heading(body: &str, heading: &str) -> bool {
    let mut in_fence = false;
    body.lines().any(|line| {
        if is_fence(line) {
            in_fence = !in_fence;
            return false;
        }
        !in_fence && matching_heading_level(line, heading).is_some()
    })
}

fn extract_section(body: &str, heading: &str) -> Option<String> {
    section_body_lines(body, heading).map(|lines| lines.join("\n").trim().to_string())
}

fn extract_section_list(body: &str, heading: &str) -> Vec<String> {
    section_body_lines(body, heading)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
                .map(|item| item.trim().to_string())
        })
        .collect()
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
    warnings_omitted: usize,
    lifecycle: &SpecLifecycleView,
    diagnostic_items: &BoundedDiagnosticList,
    declared: &DeclaredProjectState,
    derived: &DerivedProjectState,
) -> String {
    let mut md = format!("# Project Digest: {title}\n\n**Status:** {status}\n");
    if let Some(ms) = milestone {
        md.push_str(&format!("**Milestone:** {ms}\n"));
    }
    if let Some(v) = version {
        md.push_str(&format!("**Kit version:** {v}\n"));
    }
    md.push('\n');

    md.push_str("## Reconciled state\n\n");
    md.push_str(&format!(
        "- Declared source: {} ({})\n- Declared milestone: {}\n- Derived milestone: {}\n- Derivation basis: {}\n",
        declared.source,
        if declared.source_present {
            "present"
        } else {
            "missing"
        },
        declared
            .current_milestone
            .as_deref()
            .unwrap_or("not declared"),
        derived
            .current_milestone
            .as_deref()
            .unwrap_or("ambiguous or unavailable"),
        derived.basis
    ));
    md.push_str(&format!(
        "- Lifecycle counts: {}\n- Forced specs: {}\n- Recently completed: {} total, {} omitted\n\n",
        lifecycle
            .counts
            .iter()
            .map(|(status, count)| format!("{status}={count}"))
            .collect::<Vec<_>>()
            .join(", "),
        lifecycle.forced,
        lifecycle.recently_completed.total,
        lifecycle.recently_completed.omitted
    ));

    if lifecycle.backlog.total > 0 {
        md.push_str(&format!(
            "## Planned backlog ({} total, {} omitted)\n\n",
            lifecycle.backlog.total, lifecycle.backlog.omitted
        ));
        for spec in &lifecycle.backlog.items {
            md.push_str(&format!("- **{}**: {}\n", spec.id, spec.title));
        }
        md.push('\n');
    }

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
        "## Diagnostics\n\n- Total: {}\n- Errors: {}\n- Warnings: {}\n- Returned: {}\n- Omitted: {}\n",
        diagnostics.total,
        diagnostics.errors,
        diagnostics.warnings,
        diagnostic_items.items.len(),
        diagnostic_items.omitted
    ));

    if !diagnostic_items.items.is_empty() {
        md.push_str("\n### Actionable debts\n\n");
        for diagnostic in &diagnostic_items.items {
            md.push_str(&format!(
                "- **{}** `{}` {} — Next: {}\n",
                diagnostic.id, diagnostic.code, diagnostic.message, diagnostic.next_action
            ));
        }
    }

    if !warnings.is_empty() {
        md.push_str("\n## Warnings\n\n");
        for w in warnings {
            md.push_str(&format!("- ⚠ {w}\n"));
        }
    }

    if !warnings.is_empty() {
        md.push_str(&format!(
            "\nCompatibility warnings: {} returned, {} omitted. These are derived from the same diagnostic collection.\n",
            warnings.len(),
            warnings_omitted
        ));
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
    capability_tier: &Option<String>,
    thinking_level: &Option<String>,
    tags: &[String],
    criteria: &[Criterion],
    required_verification: &[VerificationRequirement],
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
    if let Some(tier) = capability_tier.as_deref() {
        let level = thinking_level
            .as_deref()
            .unwrap_or_else(|| crate::taxonomy::default_thinking_level(tier));
        md.push_str(&format!(
            "**Implementation estimate:** {tier} · {level} reasoning\n"
        ));
        if let Some(rationale) = effort_rationale(Some(tier), Some(level)) {
            md.push_str(&format!("  - {rationale}\n"));
        }
    }
    if !tags.is_empty() {
        md.push_str(&format!("**Tags:** {}\n", tags.join(", ")));
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

    if !required_verification.is_empty() {
        md.push_str("## Required verification\n\n");
        for requirement in required_verification {
            let marker = if requirement.checked { "[x]" } else { "[ ]" };
            md.push_str(&format!(
                "- {marker} **{}** [{} / {} / {}]: {}\n",
                requirement.id,
                requirement.kind,
                requirement.owner,
                requirement.phase,
                requirement.text
            ));
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
    required_verification: &[VerificationRequirement],
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
            for event in &r.events {
                md.push_str(&format!(
                    "  - `{}` {}: {} -> {} by {}",
                    event.id, event.action, event.from_status, event.to_status, event.actor_role
                ));
                if !event.reason.is_empty() {
                    md.push_str(&format!(" — {}", event.reason));
                }
                md.push('\n');
            }
            if !r.finding_categories.is_empty() {
                let categories = r
                    .finding_categories
                    .iter()
                    .map(|category| {
                        category
                            .canonical
                            .as_deref()
                            .map(|canonical| {
                                if category.is_alias {
                                    format!("{} → {canonical}", category.raw)
                                } else {
                                    canonical.to_string()
                                }
                            })
                            .unwrap_or_else(|| format!("{} (unknown)", category.raw))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                md.push_str(&format!("  - Taxonomy v1: {categories}\n"));
            }
            for warning in &r.lifecycle_warnings {
                md.push_str(&format!("  - ⚠ {warning}\n"));
            }
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

    if !required_verification.is_empty() {
        md.push_str("## Verification requirements\n\n");
        for requirement in required_verification {
            md.push_str(&format!(
                "- **{}** [{} / {} / {}]: {}\n",
                requirement.id,
                requirement.kind,
                requirement.owner,
                requirement.phase,
                requirement.text
            ));
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
        fs::create_dir_all(lmbrain.join("skills/active")).unwrap();

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
skills: [SKILL-VERIFY]
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

## Operating guidance

Never claim a gate that was not executed.
"#,
        )
        .unwrap();

        fs::write(
            lmbrain.join("skills/active/SKILL-VERIFY.md"),
            r#"---
id: SKILL-VERIFY
title: "Verification runbook"
status: active
kind: verification
commands: []
---
# Verification runbook

## Procedure

```bash
cargo test --workspace
```
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
        assert_eq!(ctx.required_verification.len(), 2);
        assert!(!ctx.acceptance_criteria[0].checked);
        assert!(ctx.explicit_files.iter().any(|f| f.contains("main.rs")));
        assert!(ctx.markdown.contains("Spec Context"));
        assert!(ctx.markdown.contains("Sam Stacktrace"));
        assert!(ctx
            .agent_profile
            .as_ref()
            .unwrap()
            .operational_guidance
            .contains("Never claim"));
        assert!(ctx.applicable_skills[0]
            .commands
            .iter()
            .any(|command| command.contains("cargo test --workspace")));
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
    fn structured_verification_is_typed_and_does_not_inflate_acceptance_criteria() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let path = dir
            .path()
            .join(".lmbrain/specs/ready/SPEC-023-v3-context-economy.md");
        let mut source = fs::read_to_string(&path).unwrap();
        source = source.replace(
            "- cargo test\n- pnpm lint",
            "- [ ] rust-tests | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run Rust tests\n- [ ] windows-smoke | kind=operator | owner=operator | phase=before-done | evidence=observation | Smoke packaged Windows app",
        );
        source.push_str("\n## Implementation evidence\n\n### Handoff status\n- [ ] Ready for Project Lead review\n");
        fs::write(&path, source).unwrap();
        let context = build_spec_context(dir.path(), "SPEC-023").unwrap();
        assert_eq!(context.acceptance_criteria.len(), 2);
        assert_eq!(context.required_verification.len(), 2);
        assert_eq!(context.required_verification[0].id, "rust-tests");
        assert_eq!(context.required_verification[1].owner, "operator");
        assert_eq!(context.required_verification[1].phase, "before-done");
    }

    #[test]
    fn astranexus_derived_verification_shapes_never_disappear_from_handoff_context() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        let shapes = [
            "- cargo test --workspace",
            "Run the Rust workspace tests and preserve the complete output.",
            "- [ ] unit | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run unit tests",
            "- [ ] smoke | kind=operator | owner=operator | phase=before-done | evidence=observation | Run packaged smoke",
            "- pnpm lint\n- pnpm test",
            "Database integration must be green before submission.",
            "- [x] audit | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Inspect the audit log",
        ];
        for (index, required) in shapes.iter().enumerate() {
            let id = format!("SPEC-ASTRA-{index}");
            fs::write(
                dir.path().join(format!(".lmbrain/specs/ready/{id}.md")),
                format!(
                    "---\nid: {id}\ntitle: Astra-derived shape\nstatus: ready\n---\n\n## Acceptance criteria\n- [ ] Feature works\n\n## Required verification\n\n{required}\n\n## Implementation evidence\n\n### Handoff status\n- [ ] Ready for Project Lead review\n"
                ),
            ).unwrap();
            let context = build_spec_context(dir.path(), &id).unwrap();
            assert_eq!(context.acceptance_criteria.len(), 1, "shape {index}");
            assert!(!context.required_verification.is_empty(), "shape {index}");
            if required.contains("before-done") {
                assert_eq!(context.required_verification[0].phase, "before-done");
                assert_eq!(context.required_verification[0].owner, "operator");
            }
        }
    }

    fn write_review_spec(dir: &std::path::Path, id: &str, body: &str) {
        fs::write(
            dir.join(format!(".lmbrain/specs/review/{id}.md")),
            format!("---\nid: {id}\ntitle: Evidence shapes\nstatus: review\n---\n\n{body}"),
        )
        .unwrap();
    }

    #[test]
    fn review_context_keeps_nested_evidence_subsections() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        fs::create_dir_all(dir.path().join(".lmbrain/specs/review")).unwrap();
        // The AstraNexus SPEC-049 shape: a template placeholder paragraph
        // followed by the real evidence in nested ### subsections.
        write_review_spec(
            dir.path(),
            "SPEC-049",
            "## Acceptance criteria\n- [x] Works\n\n## Implementation evidence\n\n> Filled in by the specialist after completion.\n\n### Changes made\nRefactored the session module.\n\n### Verification transcript\n\n```text\n$ cargo test\n## not a heading, fenced\nok\n```\n\n#### Notes\nDeep nesting is preserved.\n\n## Next section\nUnrelated.\n",
        );

        let ctx = build_review_context(dir.path(), "SPEC-049").unwrap();
        let evidence = ctx.implementation_evidence.expect("evidence missing");
        assert!(evidence.contains("Changes made"), "{evidence}");
        assert!(evidence.contains("Refactored the session module"));
        assert!(evidence.contains("Verification transcript"));
        assert!(evidence.contains("## not a heading, fenced"));
        assert!(evidence.contains("Deep nesting is preserved"));
        assert!(!evidence.contains("Unrelated"));
        assert!(
            !ctx.warnings.iter().any(|w| w.contains("evidence")),
            "unexpected evidence warnings: {:?}",
            ctx.warnings
        );
    }

    #[test]
    fn review_context_warns_on_missing_or_empty_evidence() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        fs::create_dir_all(dir.path().join(".lmbrain/specs/review")).unwrap();

        write_review_spec(
            dir.path(),
            "SPEC-050",
            "## Acceptance criteria\n- [x] Works\n",
        );
        let ctx = build_review_context(dir.path(), "SPEC-050").unwrap();
        assert!(ctx.implementation_evidence.is_none());
        assert!(ctx
            .warnings
            .iter()
            .any(|w| w.contains("no Implementation evidence section")));

        write_review_spec(
            dir.path(),
            "SPEC-051",
            "## Acceptance criteria\n- [x] Works\n\n## Implementation evidence\n\n## Next\nother\n",
        );
        let ctx = build_review_context(dir.path(), "SPEC-051").unwrap();
        assert!(ctx.implementation_evidence.is_none());
        assert!(ctx.warnings.iter().any(|w| w.contains("present but empty")));
    }

    #[test]
    fn review_context_truncates_oversized_evidence_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        create_test_brain(dir.path());
        fs::create_dir_all(dir.path().join(".lmbrain/specs/review")).unwrap();
        let huge = "evidence line\n".repeat(4000); // ~56 KB
        write_review_spec(
            dir.path(),
            "SPEC-052",
            &format!("## Implementation evidence\n\n{huge}"),
        );
        let ctx = build_review_context(dir.path(), "SPEC-052").unwrap();
        let evidence = ctx.implementation_evidence.unwrap();
        assert!(evidence.len() < huge.len());
        assert!(evidence.contains("[implementation evidence truncated"));
        assert!(ctx.warnings.iter().any(|w| w.contains("truncated")));
    }

    #[test]
    fn extract_section_is_level_aware_and_fence_safe() {
        let body = "## Target\n\n### Nested\ncontent\n\n```sh\n## fenced heading\n```\n\n## After\nother\n";
        let section = super::extract_section(body, "Target").unwrap();
        assert!(section.contains("Nested"));
        assert!(section.contains("## fenced heading"));
        assert!(!section.contains("other"));

        // A heading inside a fence never starts a section.
        let fenced = "```\n## Target\nnot me\n```\n\n## Target\nreal\n";
        assert_eq!(super::extract_section(fenced, "Target").unwrap(), "real");

        // An empty first match does not shadow a later populated one.
        let duplicated = "## Target\n\n## Middle\nx\n\n## Target\nfound\n";
        assert_eq!(
            super::extract_section(duplicated, "Target").unwrap(),
            "found"
        );

        // Lists keep collecting through nested headings, stop at same level.
        let listing = "## Files and areas involved\n- src/a.rs\n\n### More\n- src/b.rs\n\n## Stop\n- src/c.rs\n";
        let items = super::extract_section_list(listing, "Files and areas involved");
        assert_eq!(items, vec!["src/a.rs".to_string(), "src/b.rs".into()]);
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
        let diagnostics = crate::diagnostics::build_diagnostics(dir.path());
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
        let diagnostics = crate::diagnostics::build_diagnostics(dir.path());
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

## Required verification

- [ ] cargo-tests | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run tests
"#,
        )
        .unwrap();
        let diagnostics = crate::diagnostics::build_diagnostics(dir.path());
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("AGENT-NONEXISTENT")),
            "Expected an unresolved agent diagnostic, got: {diagnostics:?}"
        );
    }

    #[test]
    fn diagnostics_report_legacy_done_verification_without_mutating_or_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let done = dir.path().join(".lmbrain/specs/done");
        fs::create_dir_all(&done).unwrap();
        let path = done.join("SPEC-099.md");
        let source = "---\nid: SPEC-099\ntitle: Legacy done\nstatus: done\n---\n## Required verification\n- [ ] LEAD | kind=manual | owner=lead | phase=before-done | evidence=artifact | Independent review\n- [x] WRONG | kind=manual | owner=operator | phase=before-submit | evidence=observation | Invalid policy\n";
        fs::write(&path, source).unwrap();

        let diagnostics = crate::diagnostics::build_diagnostics(dir.path());
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("SPEC-099")
                && diagnostic.message.contains("LEAD")
                && diagnostic.message.contains("owner=lead")
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("Unsupported verification policy")
                && diagnostic.message.contains("WRONG")
        }));
        assert_eq!(fs::read_to_string(path).unwrap(), source);
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

## Required verification

- [ ] cargo-tests | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run tests
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
        assert!(digest
            .diagnostics
            .items
            .iter()
            .any(|diagnostic| diagnostic.code == "executable-gates-without-manifest"));
    }

    #[test]
    fn project_digest_backlog_and_diagnostics_are_reconciled_and_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(lmbrain.join("specs/backlog")).unwrap();
        fs::write(
            lmbrain.join("STATUS.md"),
            "# Project Test\n\n**Status:** active\n**Current milestone:** M-OLD\n",
        )
        .unwrap();
        fs::write(
            lmbrain.join("ROADMAP.md"),
            "# Roadmap\n\n## M-NEW - Planned\n\n- `status`: planned\n- `specs`: []\n",
        )
        .unwrap();
        fs::write(
            lmbrain.join("specs/backlog/SPEC-001.md"),
            "---\nid: SPEC-001\ntitle: Planned work\nstatus: backlog\nmilestone: M-NEW\nrecommended_agent: AGENT-MISSING\n---\n",
        )
        .unwrap();

        let digest = build_project_digest(dir.path());
        assert_eq!(digest.schema_version, "2");
        assert_eq!(digest.lifecycle.backlog.total, 1);
        assert_eq!(digest.lifecycle.backlog.items[0].id, "SPEC-001");
        assert!(digest.markdown.contains("Planned backlog"));
        assert_eq!(
            digest.diagnostics_summary.total,
            digest.diagnostics.items.len() + digest.diagnostics.omitted
        );
        assert_eq!(
            digest.diagnostics_summary.warnings,
            digest
                .diagnostics
                .items
                .iter()
                .filter(|diagnostic| { diagnostic.severity == crate::DiagnosticSeverity::Warning })
                .count()
        );
        assert_eq!(digest.warnings_omitted, 0);
        assert_eq!(digest.warnings.len(), digest.diagnostics_summary.warnings);
        assert!(digest
            .diagnostics
            .items
            .iter()
            .all(|diagnostic| !diagnostic.next_action.is_empty()));
    }

    #[test]
    fn project_digest_states_exact_omissions_for_large_diagnostic_sets() {
        let dir = tempfile::tempdir().unwrap();
        let specs = dir.path().join(".lmbrain/specs/backlog");
        fs::create_dir_all(&specs).unwrap();
        for index in 1..=60 {
            fs::write(
                specs.join(format!("SPEC-{index:03}.md")),
                format!("---\nid: SPEC-{index:03}\n"),
            )
            .unwrap();
        }

        let digest = build_project_digest(dir.path());
        assert_eq!(digest.diagnostics.items.len(), DIGEST_DIAGNOSTIC_LIMIT);
        assert_eq!(
            digest.diagnostics.total,
            digest.diagnostics.items.len() + digest.diagnostics.omitted
        );
        assert!(digest.diagnostics.omitted > 0);
        assert_eq!(
            digest.warnings.len() + digest.warnings_omitted,
            digest.diagnostics_summary.warnings
        );
        assert!(digest
            .markdown
            .contains(&format!("- Omitted: {}", digest.diagnostics.omitted)));
    }
}
