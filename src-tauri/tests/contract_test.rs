use std::fs;
use std::path::Path;

use lmbrain_lib::commands::contract;
use lmbrain_lib::commands::workspace::WorkspaceService;

fn setup_test_kit(dir: &Path) {
    // Create .lmbrain directory structure
    let lmbrain = dir.join(".lmbrain");
    fs::create_dir_all(&lmbrain).unwrap();

    // Create VERSION
    fs::write(lmbrain.join("VERSION"), "1.0.0").unwrap();

    // Create STATUS.md
    fs::write(
        lmbrain.join("STATUS.md"),
        "## Current focus\n\nTest focus\n\n## Current milestone\n\nM-01 — Test\n",
    )
    .unwrap();

    // Create spec directories (board statuses)
    fs::create_dir_all(lmbrain.join("specs").join("backlog")).unwrap();
    fs::create_dir_all(lmbrain.join("specs").join("ready")).unwrap();

    // Create review directories
    fs::create_dir_all(lmbrain.join("reviews").join("pending")).unwrap();
}

#[test]
fn test_build_diagnostics_no_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // A spec whose folder matches its frontmatter status — no mismatch.
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-050.md"),
        "---\nid: SPEC-050\ntitle: Test Spec\nstatus: ready\n---\nBody",
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let status_mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Status mismatch"))
        .collect();
    assert!(
        status_mismatches.is_empty(),
        "Expected no status mismatches, got: {:?}",
        status_mismatches
    );
}

#[test]
fn test_build_diagnostics_flags_unresolved_recommended_agent() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Spec recommends AGENT-XXX (template placeholder, no such profile exists).
    let spec_content = r#"---
id: SPEC-050
title: Needs an agent
status: ready
recommended_agent: AGENT-XXX
---
Body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-050.md"),
        spec_content,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let missing: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Missing reference") && d.message.contains("AGENT-XXX"))
        .collect();
    assert!(
        !missing.is_empty(),
        "Expected a missing-reference diagnostic for the unresolved recommended_agent"
    );
}

#[test]
fn test_build_diagnostics_accepts_resolved_recommended_agent() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // A real agent profile exists...
    fs::create_dir_all(dir.path().join(".lmbrain").join("agents").join("profiles")).unwrap();
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("agents")
            .join("profiles")
            .join("AGENT-IMPL.md"),
        "---\nid: AGENT-IMPL\ntitle: Implementer\nstatus: active\n---\nBody",
    )
    .unwrap();

    // ...and the spec recommends it.
    let spec_content = r#"---
id: SPEC-051
title: Has a valid agent
status: ready
recommended_agent: AGENT-IMPL
---
Body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-051.md"),
        spec_content,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let missing: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Missing reference") && d.message.contains("SPEC-051"))
        .collect();
    assert!(
        missing.is_empty(),
        "Did not expect a missing-reference diagnostic for a resolved recommended_agent, got: {:?}",
        missing
    );
}

#[test]
fn test_build_roadmap_parses_h3_milestones() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    // Milestones use h3 (`### M-01 — …`) per the kit template; section headers
    // (`# Roadmap`) must be ignored.
    let roadmap = r#"---
title: Roadmap
---

# Roadmap

### M-01 — Running scaffold

- `status`: planned
- `outcome`: The stack is wired end to end.
- `specs`: [SPEC-001]
- `risks`: [Tauri 2.x API stability]

### M-02 — Core brew logging

- `status`: proposed
- `specs`: []
"#;
    fs::write(dir.path().join(".lmbrain").join("ROADMAP.md"), roadmap).unwrap();

    let result = contract::build_roadmap(dir.path()).unwrap();
    assert_eq!(
        result.milestones.len(),
        2,
        "expected 2 milestones, got {:?}",
        result.milestones
    );
    let m1 = &result.milestones[0];
    assert_eq!(m1.id, "M-01");
    assert_eq!(m1.title, "Running scaffold");
    assert_eq!(m1.status, "planned");
    assert_eq!(m1.specs, vec!["SPEC-001".to_string()]);
    assert_eq!(result.milestones[1].id, "M-02");
}

#[test]
fn test_build_diagnostics_spec_status_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a spec in backlog/ with status: ready — should produce a diagnostic
    let spec_content = r#"---
id: SPEC-001
title: Mismatched Spec
status: ready
---
Spec body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("backlog")
            .join("SPEC-001.md"),
        spec_content,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Status mismatch"))
        .collect();
    assert!(
        !mismatches.is_empty(),
        "Expected at least one status mismatch diagnostic for spec"
    );
}

#[test]
fn test_build_diagnostics_malformed_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a file with malformed YAML
    let bad_content = r#"---
id: SPEC-002
title: [unclosed list
---
Body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("backlog")
            .join("SPEC-002.md"),
        bad_content,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let parse_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Malformed"))
        .collect();
    assert!(
        !parse_errors.is_empty(),
        "Expected malformed YAML diagnostic"
    );
}

// ─── V3 agent metadata tests ──────────────────────────────────────

#[test]
fn test_build_agents_parses_v3_metadata_fields() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let profiles_dir = dir.path().join(".lmbrain").join("agents").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // Profile with all v3 metadata fields
    fs::write(
        profiles_dir.join("AGENT-FRONTEND-UI.md"),
        r#"---
id: AGENT-FRONTEND-UI
title: "Frontend UI Specialist"
mnemonic_name: "Marta Pixelperfetta"
status: proposed
role: frontend-ui-specialist
activation: manual
can_implement: true
can_review: false
domains: [frontend, ui, react]
primary_files: [src/components, src/lib]
review_focus: [accessibility, state-management]
context_pack: spec
constraints: []
links: []
created: 2026-07-02
updated: 2026-07-02
tags: [v3, frontend]
---
# Frontend UI Specialist
"#,
    )
    .unwrap();

    let agents = contract::build_agents(dir.path()).unwrap();
    let agent = agents.iter().find(|a| a.id == "AGENT-FRONTEND-UI").unwrap();

    assert_eq!(agent.mnemonic_name, Some("Marta Pixelperfetta".into()));
    assert_eq!(
        agent.domains,
        Some(vec!["frontend".into(), "ui".into(), "react".into()])
    );
    assert_eq!(
        agent.primary_files,
        Some(vec!["src/components".into(), "src/lib".into()])
    );
    assert_eq!(
        agent.review_focus,
        Some(vec!["accessibility".into(), "state-management".into()])
    );
    assert_eq!(agent.context_pack, Some("spec".into()));
    assert_eq!(agent.constraints, Some(Vec::new()));
}

#[test]
fn test_build_agents_backward_compatible_without_v3_fields() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let profiles_dir = dir.path().join(".lmbrain").join("agents").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // Legacy v2 profile without any v3 metadata fields
    fs::write(
        profiles_dir.join("AGENT-LEGACY.md"),
        r#"---
id: AGENT-LEGACY
title: "Legacy Agent"
status: active
role: specialist
activation: manual
can_implement: true
can_review: false
links: []
created: 2026-01-01
updated: 2026-01-01
tags: []
---
# Legacy
"#,
    )
    .unwrap();

    let agents = contract::build_agents(dir.path()).unwrap();
    let agent = agents.iter().find(|a| a.id == "AGENT-LEGACY").unwrap();

    assert_eq!(agent.domains, None);
    assert_eq!(agent.mnemonic_name, None);
    assert_eq!(agent.primary_files, None);
    assert_eq!(agent.review_focus, None);
    assert_eq!(agent.context_pack, None);
    assert_eq!(agent.constraints, None);
    assert_eq!(agent.status.as_str(), "active");
    assert_eq!(agent.can_implement, Some(true));
}

#[test]
fn test_build_agent_proposals_parses_v3_fields() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let proposals_dir = dir.path().join(".lmbrain").join("agents").join("proposals");
    fs::create_dir_all(&proposals_dir).unwrap();

    // Improvement proposal with v3 fields
    fs::write(
        proposals_dir.join("AGENT-PROP-IMPROVE-001.md"),
        r#"---
id: AGENT-PROP-IMPROVE-001
title: "Improve frontend specialist"
status: proposed
proposed_mnemonic_name: "Marta Pixelperfetta"
requested_by: AGENT-LEAD
reason: repeated-review-finding
proposal_type: improvement
target_profile: AGENT-FRONTEND-UI
recommended_for: [SPEC-001]
links: [REVIEW-001]
created: 2026-07-02
updated: 2026-07-02
tags: [proposal, improvement]
---
# Improvement proposal
"#,
    )
    .unwrap();

    let proposals = contract::build_agent_proposals(dir.path()).unwrap();
    let prop = proposals
        .iter()
        .find(|p| p.id == "AGENT-PROP-IMPROVE-001")
        .unwrap();

    assert_eq!(
        prop.proposed_mnemonic_name,
        Some("Marta Pixelperfetta".into())
    );
    assert_eq!(prop.proposal_type, Some("improvement".into()));
    assert_eq!(prop.target_profile, Some("AGENT-FRONTEND-UI".into()));
}

#[test]
fn test_build_agent_proposals_backward_compatible_without_v3_fields() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let proposals_dir = dir.path().join(".lmbrain").join("agents").join("proposals");
    fs::create_dir_all(&proposals_dir).unwrap();

    // Legacy proposal without v3 fields
    fs::write(
        proposals_dir.join("AGENT-PROP-LEGACY-001.md"),
        r#"---
id: AGENT-PROP-LEGACY-001
title: "Legacy proposal"
status: proposed
requested_by: AGENT-LEAD
reason: recurring-specialized-work
recommended_for: []
links: []
created: 2026-01-01
updated: 2026-01-01
tags: [proposal]
---
# Legacy
"#,
    )
    .unwrap();

    let proposals = contract::build_agent_proposals(dir.path()).unwrap();
    let prop = proposals
        .iter()
        .find(|p| p.id == "AGENT-PROP-LEGACY-001")
        .unwrap();

    assert_eq!(prop.proposal_type, None);
    assert_eq!(prop.proposed_mnemonic_name, None);
    assert_eq!(prop.target_profile, None);
}

#[test]
fn test_build_skills_parses_project_scoped_skill() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let skill_dir = dir.path().join(".lmbrain").join("skills").join("active");
    fs::create_dir_all(&skill_dir).unwrap();

    fs::write(
        skill_dir.join("SKILL-001-build-and-test.md"),
        r#"---
id: SKILL-001
title: "Build and test"
status: active
scope: project
kind: verification
risk: medium
applies_to: [AGENT-FULLSTACK-DESKTOP]
domains: [build, test]
commands: [pnpm test, cargo test --workspace]
requires_operator_approval: false
links: []
created: 2026-07-07
updated: 2026-07-07
tags: [verification]
---
# Build and test
"#,
    )
    .unwrap();

    let skills = contract::build_skills(dir.path()).unwrap();
    assert_eq!(skills.len(), 1);
    let skill = &skills[0];
    assert_eq!(skill.id, "SKILL-001");
    assert_eq!(skill.status.as_str(), "active");
    assert_eq!(skill.kind.as_deref(), Some("verification"));
    assert_eq!(skill.risk.as_deref(), Some("medium"));
    assert_eq!(skill.commands, vec!["pnpm test", "cargo test --workspace"]);
}

#[test]
fn test_build_diagnostics_flags_unresolved_skill_references() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");
    fs::create_dir_all(lmbrain.join("skills/active")).unwrap();
    fs::create_dir_all(lmbrain.join("agents/profiles")).unwrap();

    fs::write(
        lmbrain.join("specs/ready/SPEC-SKILL.md"),
        r#"---
id: SPEC-SKILL
title: Skill ref
status: ready
skills: ["SKILL-MISSING"]
links: []
created: 2026-07-07
updated: 2026-07-07
tags: []
---
# Skill ref
"#,
    )
    .unwrap();
    fs::write(
        lmbrain.join("agents/profiles/AGENT-SKILL.md"),
        r#"---
id: AGENT-SKILL
title: Agent Skill
status: active
skills: ["SKILL-MISSING"]
links: []
created: 2026-07-07
updated: 2026-07-07
tags: []
---
# Agent Skill
"#,
    )
    .unwrap();
    fs::write(
        lmbrain.join("skills/active/SKILL-001.md"),
        r#"---
id: SKILL-001
title: Diagnostic Skill
status: active
risk: spicy
applies_to: ["AGENT-MISSING"]
links: []
created: 2026-07-07
updated: 2026-07-07
tags: []
---
# Diagnostic Skill
"#,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let messages = diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|m| m.contains("spec SPEC-SKILL references skill 'SKILL-MISSING'")));
    assert!(messages
        .iter()
        .any(|m| m.contains("agent AGENT-SKILL references skill 'SKILL-MISSING'")));
    assert!(messages.iter().any(|m| m.contains("Invalid skill risk")));
    assert!(messages
        .iter()
        .any(|m| m.contains("skill SKILL-001 applies to 'AGENT-MISSING'")));
}

#[test]
fn test_build_diagnostics_area_domain_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create an agent profile with domains
    let profiles_dir = dir.path().join(".lmbrain").join("agents").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();
    fs::write(
        profiles_dir.join("AGENT-FRONTEND.md"),
        r#"---
id: AGENT-FRONTEND
title: "Frontend Specialist"
status: active
role: frontend
activation: manual
can_implement: true
can_review: false
domains: [frontend, ui, react]
links: []
created: 2026-07-02
updated: 2026-07-02
tags: []
---
# Frontend
"#,
    )
    .unwrap();

    // Spec with area "backend" that doesn't match agent domains
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-MISMATCH.md"),
        r#"---
id: SPEC-MISMATCH
title: "Backend work"
status: ready
area: backend
recommended_agent: AGENT-FRONTEND
links: []
created: 2026-07-02
updated: 2026-07-02
tags: []
---
# Backend work
"#,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let area_mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Area mismatch"))
        .collect();
    assert!(
        !area_mismatches.is_empty(),
        "Expected an area/domain mismatch diagnostic, got: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_build_diagnostics_area_domain_match_stays_quiet() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create an agent profile with domains
    let profiles_dir = dir.path().join(".lmbrain").join("agents").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();
    fs::write(
        profiles_dir.join("AGENT-BACKEND.md"),
        r#"---
id: AGENT-BACKEND
title: "Backend Specialist"
status: active
role: backend
activation: manual
can_implement: true
can_review: false
domains: [backend, tauri, rust]
links: []
created: 2026-07-02
updated: 2026-07-02
tags: []
---
# Backend
"#,
    )
    .unwrap();

    // Spec with area "backend" that matches agent domains
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-MATCH.md"),
        r#"---
id: SPEC-MATCH
title: "Backend work"
status: ready
area: backend
recommended_agent: AGENT-BACKEND
links: []
created: 2026-07-02
updated: 2026-07-02
tags: []
---
# Backend work
"#,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let area_mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Area mismatch"))
        .collect();
    assert!(
        area_mismatches.is_empty(),
        "Expected no area/domain mismatch for matching domains, got: {:?}",
        area_mismatches
    );
}

#[test]
fn test_wikilink_index() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a file that links to another
    let source_content = r#"---
id: SPEC-003
title: Source
---
See [[TARGET-001]] for details."#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("specs")
            .join("ready")
            .join("SPEC-003.md"),
        source_content,
    )
    .unwrap();

    let index = contract::build_wikilink_index(dir.path());
    assert!(
        index.contains_key("target-001"),
        "Wikilink index should contain 'target-001', got keys: {:?}",
        index.keys()
    );
}

#[test]
fn test_wiki_tree_lists_only_operator_content_directories() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    fs::create_dir_all(lmbrain.join("decisions")).unwrap();
    fs::create_dir_all(lmbrain.join("knowledge/deep")).unwrap();
    fs::create_dir_all(lmbrain.join("specs/ready")).unwrap();
    fs::create_dir_all(lmbrain.join("agents/proposals")).unwrap();
    fs::create_dir_all(lmbrain.join("reviews/pending")).unwrap();

    fs::write(lmbrain.join("decisions/ADR-001.md"), "# Decision").unwrap();
    fs::write(lmbrain.join("knowledge/deep/Topic.md"), "# Topic").unwrap();
    fs::write(lmbrain.join("specs/ready/SPEC-001.md"), "# Spec").unwrap();
    fs::write(
        lmbrain.join("agents/proposals/AGENT-PROP-001.md"),
        "# Proposal",
    )
    .unwrap();
    fs::write(lmbrain.join("reviews/pending/REVIEW-001.md"), "# Review").unwrap();
    fs::write(lmbrain.join("STATUS.md"), "# Status").unwrap();

    let tree = contract::build_wiki_tree(dir.path()).unwrap();
    let names: Vec<_> = tree
        .root
        .children
        .iter()
        .map(|node| node.name.as_str())
        .collect();

    assert_eq!(names, vec!["decisions", "knowledge", "specs"]);
    assert_eq!(tree.root.count, Some(3));
}

#[test]
fn test_wikilink_index_uses_only_operator_content_directories() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    fs::create_dir_all(lmbrain.join("knowledge")).unwrap();
    fs::create_dir_all(lmbrain.join("agents/proposals")).unwrap();
    fs::write(lmbrain.join("knowledge/Topic.md"), "See [[VISIBLE]].").unwrap();
    fs::write(
        lmbrain.join("agents/proposals/AGENT-PROP-001.md"),
        "See [[HIDDEN]].",
    )
    .unwrap();

    let index = contract::build_wikilink_index(dir.path());

    assert!(index.contains_key("visible"));
    assert!(!index.contains_key("hidden"));
}

#[test]
fn test_status_md_heading_parsing() {
    let content = "## Current focus\n\nTest focus content\n\n## Current milestone\n\nM-01 — Test\n";
    let focus = contract::extract_focus_for_test(content);
    assert_eq!(focus, Some("Test focus content".to_string()));

    let milestone = contract::extract_milestone_for_test(content);
    assert_eq!(milestone, Some("M-01 — Test".to_string()));
}

#[test]
fn test_status_md_heading_parsing_fallback() {
    let content = "# Just a heading\n\nNo focus or milestone here.";
    let focus = contract::extract_focus_for_test(content);
    assert!(focus.is_none());

    let milestone = contract::extract_milestone_for_test(content);
    assert!(milestone.is_none());
}

#[test]
fn test_initialize_kit_copies_template_and_refuses_overwrite() {
    let repository = tempfile::tempdir().unwrap();
    let template_root = tempfile::tempdir().unwrap();
    setup_test_kit(template_root.path());

    let service = WorkspaceService::new();
    let info = service
        .initialize_kit(repository.path(), &template_root.path().join(".lmbrain"))
        .unwrap();

    assert_eq!(info.kit_version, "1.0.0");
    assert!(repository.path().join(".lmbrain/STATUS.md").is_file());

    let overwrite =
        service.initialize_kit(repository.path(), &template_root.path().join(".lmbrain"));
    assert!(overwrite.is_err());
}

#[test]
fn test_build_adrs_excludes_readme_and_non_genuine_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let lmbrain = dir.path().join(".lmbrain");
    let decisions_dir = lmbrain.join("decisions");
    fs::create_dir_all(&decisions_dir).unwrap();

    // Write README.md (no frontmatter)
    fs::write(
        decisions_dir.join("README.md"),
        "# Decisions\nThis is a README.",
    )
    .unwrap();

    // Write a stray file with non-ADR ID or invalid format
    fs::write(
        decisions_dir.join("STRAY.md"),
        "---\nid: STRAY-001\ntitle: Stray\nstatus: proposed\n---\nBody",
    )
    .unwrap();

    // Write a valid ADR
    fs::write(decisions_dir.join("ADR-001.md"), "---\nid: ADR-001\ntitle: Valid ADR\nstatus: accepted\ncreated: 2026-06-22\nupdated: 2026-06-22\ntags: []\nlinks: []\n---\nBody").unwrap();

    let adrs = contract::build_adrs(dir.path()).unwrap();
    assert_eq!(adrs.len(), 1);
    assert_eq!(adrs[0].id, "ADR-001");
}

#[test]
fn test_build_roadmap_success() {
    let dir = tempfile::tempdir().unwrap();
    let lmbrain = dir.path().join(".lmbrain");
    fs::create_dir_all(&lmbrain).unwrap();

    let roadmap_content = r#"---
title: Custom Roadmap
updated: 2026-06-22
---

# Roadmap

## M-01 — Read-only desktop workspace

- `status`: active
- `target`: obsolete legacy schedule
- `outcome`: Operators can select an LMBrain repository.
- `specs`: [SPEC-001, SPEC-009]
- `risks`: [filesystem boundaries, watcher reliability]

## M-02 — Operator workflow

- `status`: planned
- `target`: obsolete legacy schedule
- `outcome`: Write support.
- `decisions`: [ADR-002]
- `depends_on`: M-01
"#;

    fs::write(lmbrain.join("ROADMAP.md"), roadmap_content).unwrap();

    let roadmap = contract::build_roadmap(dir.path()).unwrap();
    assert_eq!(roadmap.title, "Custom Roadmap");
    assert_eq!(roadmap.milestones.len(), 2);

    let m1 = &roadmap.milestones[0];
    assert_eq!(m1.id, "M-01");
    assert_eq!(m1.title, "Read-only desktop workspace");
    assert_eq!(m1.status, "active");
    assert_eq!(m1.outcome, "Operators can select an LMBrain repository.");
    assert_eq!(m1.specs, vec!["SPEC-001", "SPEC-009"]);
    assert_eq!(
        m1.risks,
        vec!["filesystem boundaries", "watcher reliability"]
    );

    let m2 = &roadmap.milestones[1];
    assert_eq!(m2.id, "M-02");
    assert_eq!(m2.title, "Operator workflow");
    assert_eq!(m2.status, "planned");
    assert_eq!(m2.outcome, "Write support.");
    assert_eq!(m2.decisions, vec!["ADR-002"]);
    assert_eq!(m2.depends_on, Some("M-01".to_string()));
}

#[test]
fn test_build_milestone_overview_produces_derived_data() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    // Create a ROADMAP.md
    fs::write(
        lmbrain.join("ROADMAP.md"),
        "---\ntitle: Test Roadmap\n---\n\n# Roadmap\n\n### M-01 — First milestone\n\n- `status`: active\n- `outcome`: Deliver the core.\n- `specs`: [SPEC-001, SPEC-002]\n- `decisions`: [ADR-001]\n- `risks`: [API stability]\n",
    )
    .unwrap();

    // Create specs
    fs::write(
        lmbrain.join("specs/ready/SPEC-001.md"),
        "---\nid: SPEC-001\ntitle: Setup\nstatus: ready\nmilestone: M-01\npriority: high\narea: core\nrecommended_agent: AGENT-IMPL\n---\nBody",
    )
    .unwrap();
    fs::write(
        lmbrain.join("specs/backlog/SPEC-002.md"),
        "---\nid: SPEC-002\ntitle: Integration\nstatus: backlog\nmilestone: M-01\n---\nBody",
    )
    .unwrap();

    // Create a decisions directory and matching ADR
    fs::create_dir_all(lmbrain.join("decisions")).unwrap();
    fs::write(
        lmbrain.join("decisions/ADR-001.md"),
        "---\nid: ADR-001\ntitle: Architecture choice\nstatus: accepted\ncreated: 2026-07-01\nupdated: 2026-07-01\ntags: []\nlinks: []\n---\nBody",
    )
    .unwrap();

    // Create a matching agent profile so the spec's recommended_agent resolves
    fs::create_dir_all(lmbrain.join("agents/profiles")).unwrap();
    fs::write(
        lmbrain.join("agents/profiles/AGENT-IMPL.md"),
        "---\nid: AGENT-IMPL\ntitle: Implementer\nstatus: active\nrole: specialist\nactivation: manual\ncan_implement: true\ncan_review: false\nlinks: []\ncreated: 2026-07-01\nupdated: 2026-07-01\ntags: []\n---\nBody",
    )
    .unwrap();

    let overview = contract::build_milestone_overview(dir.path()).unwrap();
    assert_eq!(overview.title, "Test Roadmap");
    assert_eq!(overview.milestones.len(), 1);

    let m = &overview.milestones[0];
    assert_eq!(m.id, "M-01");
    assert_eq!(m.spec_count, 2);
    assert_eq!(m.specs.len(), 2);
    assert_eq!(m.decisions.len(), 1);
    assert_eq!(m.decisions[0].id, "ADR-001");
    // progress is 0/2 since nothing is done (both are ready/backlog)
    assert!(m.spec_counts_by_status.get("done").copied().unwrap_or(0) == 0);
    assert!(m.next_action.is_some());
    assert!(m.next_action.as_ref().unwrap().contains("ready"));
}

#[test]
fn test_build_milestone_overview_handles_missing_references() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    fs::write(
        lmbrain.join("ROADMAP.md"),
        "---\ntitle: Roadmap\n---\n\n# Roadmap\n\n### M-01 — Test\n\n- `status`: planned\n- `specs`: []\n- `decisions`: [ADR-999]\n- `depends_on`: M-99\n",
    )
    .unwrap();

    let overview = contract::build_milestone_overview(dir.path()).unwrap();
    assert_eq!(overview.milestones.len(), 1);
    assert!(
        !overview.milestones[0].unresolved_refs.is_empty(),
        "Expected unresolved refs for missing ADR and dependency"
    );
}

#[test]
fn test_build_roadmap_accepts_nucleus_style_milestones_and_inline_refs() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    fs::write(
        lmbrain.join("ROADMAP.md"),
        "---\ntitle: Roadmap\n---\n\n# Roadmap\n\n## Milestones\n\n### M0 — Foundations & architecture\n- `status`: done\n- `outcome`: Delivered.\n- `specs`: [SPEC-001]\n- `decisions`: [ADR-001]\n\n### M4 — Persistence & data-driven scenarios\n- `status`: in progress\n- `outcome`: Versioned save/load and hardening.\n- `specs`: [SPEC-010] **delivered**. Split work: [SPEC-012], [SPEC-013], [SPEC-014].\n- `risks`: Save schema evolution.\n\n### Future — Extensibility\n- `status`: proposed\n- `specs`: (backlog)\n",
    )
    .unwrap();

    let roadmap = contract::build_roadmap(dir.path()).unwrap();
    assert_eq!(roadmap.milestones.len(), 2);
    assert_eq!(roadmap.milestones[0].id, "M0");
    assert_eq!(roadmap.milestones[0].title, "Foundations & architecture");
    assert_eq!(roadmap.milestones[1].id, "M4");
    assert_eq!(roadmap.milestones[1].status, "in progress");
    assert_eq!(
        roadmap.milestones[1].specs,
        vec!["SPEC-010", "SPEC-012", "SPEC-013", "SPEC-014"]
    );
}

#[test]
fn test_build_milestone_overview_maps_specs_to_nucleus_style_milestone_ids() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let lmbrain = dir.path().join(".lmbrain");

    fs::write(
        lmbrain.join("ROADMAP.md"),
        "---\ntitle: Roadmap\n---\n\n# Roadmap\n\n### M4 — Persistence & data-driven scenarios\n- `status`: in progress\n- `outcome`: Versioned save/load and hardening.\n- `specs`: [SPEC-010] **delivered**. Split work: [SPEC-012].\n",
    )
    .unwrap();
    fs::create_dir_all(lmbrain.join("specs/done")).unwrap();
    fs::create_dir_all(lmbrain.join("specs/backlog")).unwrap();
    fs::write(
        lmbrain.join("specs/done/SPEC-010.md"),
        "---\nid: SPEC-010\ntitle: Persistence\nstatus: done\nmilestone: M4\ncreated: 2026-07-03\nupdated: 2026-07-03\ntags: []\nlinks: []\n---\nBody",
    )
    .unwrap();
    fs::write(
        lmbrain.join("specs/backlog/SPEC-012.md"),
        "---\nid: SPEC-012\ntitle: Balance\nstatus: backlog\nmilestone: M4\ncreated: 2026-07-03\nupdated: 2026-07-03\ntags: []\nlinks: []\n---\nBody",
    )
    .unwrap();

    let overview = contract::build_milestone_overview(dir.path()).unwrap();
    assert_eq!(overview.milestones.len(), 1);
    assert_eq!(overview.milestones[0].id, "M4");
    assert_eq!(overview.milestones[0].spec_count, 2);
    let spec_ids = overview.milestones[0]
        .specs
        .iter()
        .map(|spec| spec.id.as_str())
        .collect::<Vec<_>>();
    assert!(spec_ids.contains(&"SPEC-010"));
    assert!(spec_ids.contains(&"SPEC-012"));
    assert!(overview.unmapped_specs.is_empty());
}

#[test]
fn test_set_artifact_status_and_rejected_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());
    let path_guard = lmbrain_lib::commands::filesystem::PathGuard::new();
    path_guard.set_root(dir.path());

    // On Windows the temp dir is exposed via its 8.3 short path (RUNNER~1) while
    // `set_artifact_status` returns a canonicalized (long) path, so compare
    // canonicalized forms to keep these assertions platform-independent.
    fn canon(p: impl AsRef<std::path::Path>) -> std::path::PathBuf {
        std::fs::canonicalize(p).unwrap()
    }

    // Create directories
    let specs_backlog_dir = dir.path().join(".lmbrain").join("specs").join("backlog");
    let specs_ready_dir = dir.path().join(".lmbrain").join("specs").join("ready");
    let specs_discarded_dir = dir.path().join(".lmbrain").join("specs").join("discarded");
    let decisions_dir = dir.path().join(".lmbrain").join("decisions");
    let agent_prop_dir = dir.path().join(".lmbrain").join("agents").join("proposals");
    let mcp_prop_dir = dir.path().join(".lmbrain").join("mcp").join("proposals");
    let agent_profiles_dir = dir.path().join(".lmbrain").join("agents").join("profiles");

    fs::create_dir_all(&specs_backlog_dir).unwrap();
    fs::create_dir_all(&specs_ready_dir).unwrap();
    fs::create_dir_all(&specs_discarded_dir).unwrap();
    fs::create_dir_all(&decisions_dir).unwrap();
    fs::create_dir_all(&agent_prop_dir).unwrap();
    fs::create_dir_all(&mcp_prop_dir).unwrap();
    fs::create_dir_all(&agent_profiles_dir).unwrap();

    // 1. SPEC - backlog -> ready (operator approval)
    let spec_path = specs_backlog_dir.join("SPEC-001.md");
    let spec_content = r#"---
id: SPEC-001
title: Test Spec
status: backlog
created: 2026-06-22
updated: 2026-06-22
---
Spec Body"#;
    fs::write(&spec_path, spec_content).unwrap();

    let new_path =
        contract::set_artifact_status(&path_guard, &spec_path.to_string_lossy(), "ready").unwrap();
    assert_eq!(canon(&new_path), canon(specs_ready_dir.join("SPEC-001.md")));
    assert!(!spec_path.exists());
    assert!(new_path.exists());

    let updated_content = fs::read_to_string(&new_path).unwrap();
    assert!(updated_content.contains("status: ready"));
    assert!(updated_content.contains("Spec Body"));

    // 2. SPEC - backlog -> done (illegal: not a legal transition) should fail
    let spec_path2 = specs_backlog_dir.join("SPEC-002.md");
    let spec_content2 = r#"---
id: SPEC-002
title: Test Spec 2
status: backlog
created: 2026-06-22
updated: 2026-06-22
---
Spec Body 2"#;
    fs::write(&spec_path2, spec_content2).unwrap();

    let res = contract::set_artifact_status(&path_guard, &spec_path2.to_string_lossy(), "done");
    assert!(res.is_err());
    assert!(spec_path2.exists()); // Original still exists

    // 3. SPEC - backlog -> discarded
    let new_path2 =
        contract::set_artifact_status(&path_guard, &spec_path2.to_string_lossy(), "discarded")
            .unwrap();
    assert_eq!(
        canon(&new_path2),
        canon(specs_discarded_dir.join("SPEC-002.md"))
    );
    assert!(!spec_path2.exists());
    assert!(new_path2.exists());

    let updated_content2 = fs::read_to_string(&new_path2).unwrap();
    assert!(updated_content2.contains("status: discarded"));

    // 4. ADR - proposed -> accepted (Approve)
    let adr_path = decisions_dir.join("ADR-001.md");
    let adr_content = r#"---
id: ADR-001
title: Test ADR
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
ADR Body"#;
    fs::write(&adr_path, adr_content).unwrap();

    let new_adr_path =
        contract::set_artifact_status(&path_guard, &adr_path.to_string_lossy(), "accepted")
            .unwrap();
    assert_eq!(canon(&new_adr_path), canon(&adr_path));
    assert!(new_adr_path.exists());
    let updated_adr = fs::read_to_string(&new_adr_path).unwrap();
    assert!(updated_adr.contains("status: accepted"));

    // 5. ADR - proposed -> rejected (Reject)
    let adr_path2 = decisions_dir.join("ADR-002.md");
    let adr_content2 = r#"---
id: ADR-002
title: Test ADR 2
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
ADR Body 2"#;
    fs::write(&adr_path2, adr_content2).unwrap();

    let new_adr_path2 =
        contract::set_artifact_status(&path_guard, &adr_path2.to_string_lossy(), "rejected")
            .unwrap();
    assert_eq!(canon(&new_adr_path2), canon(&adr_path2));
    assert!(new_adr_path2.exists());
    let updated_adr2 = fs::read_to_string(&new_adr_path2).unwrap();
    assert!(updated_adr2.contains("status: rejected"));

    // 6. Agent proposal - proposed -> approved (Approve)
    let ap_path = agent_prop_dir.join("AGENT-PROP-001.md");
    let ap_content = r#"---
id: AGENT-PROP-001
title: Test Agent Proposal
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
Agent Proposal Body"#;
    fs::write(&ap_path, ap_content).unwrap();

    let new_ap_path =
        contract::set_artifact_status(&path_guard, &ap_path.to_string_lossy(), "approved").unwrap();
    assert_eq!(canon(&new_ap_path), canon(&ap_path));
    assert!(new_ap_path.exists());
    let updated_ap = fs::read_to_string(&new_ap_path).unwrap();
    assert!(updated_ap.contains("status: approved"));

    // 7. MCP proposal - proposed -> rejected (Reject)
    let mp_path = mcp_prop_dir.join("MCP-PROP-001.md");
    let mp_content = r#"---
id: MCP-PROP-001
title: Test MCP Proposal
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
MCP Proposal Body"#;
    fs::write(&mp_path, mp_content).unwrap();

    let new_mp_path =
        contract::set_artifact_status(&path_guard, &mp_path.to_string_lossy(), "rejected").unwrap();
    assert_eq!(canon(&new_mp_path), canon(&mp_path));
    assert!(new_mp_path.exists());
    let updated_mp = fs::read_to_string(&new_mp_path).unwrap();
    assert!(updated_mp.contains("status: rejected"));

    // 8. Agent profile - proposed -> active (Approve)
    let profile_path = agent_profiles_dir.join("AGENT-001.md");
    let profile_content = r#"---
id: AGENT-001
title: Test Agent Profile
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
Agent Profile Body"#;
    fs::write(&profile_path, profile_content).unwrap();

    let new_profile_path =
        contract::set_artifact_status(&path_guard, &profile_path.to_string_lossy(), "active")
            .unwrap();
    assert_eq!(canon(&new_profile_path), canon(&profile_path));
    assert!(new_profile_path.exists());
    let updated_profile = fs::read_to_string(&new_profile_path).unwrap();
    assert!(updated_profile.contains("status: active"));

    // 9. Illegal transition for the source status (active -> proposed is not legal)
    let res =
        contract::set_artifact_status(&path_guard, &new_profile_path.to_string_lossy(), "proposed");
    assert!(res.is_err());

    // 10. Illegal target status (Should fail)
    let adr_path3 = decisions_dir.join("ADR-003.md");
    let adr_content3 = r#"---
id: ADR-003
title: Test ADR 3
status: proposed
created: 2026-06-22
updated: 2026-06-22
---
ADR Body 3"#;
    fs::write(&adr_path3, adr_content3).unwrap();

    let res = contract::set_artifact_status(&path_guard, &adr_path3.to_string_lossy(), "active");
    assert!(res.is_err());

    // 11. Check diagnostics for rejected Spec (no status mismatch diagnostic should exist)
    let diags = contract::build_diagnostics(dir.path());
    let mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Status mismatch"))
        .collect();
    assert!(
        mismatches.is_empty(),
        "Expected no status mismatches (e.g. for rejected spec), got: {:?}",
        mismatches
    );
}
