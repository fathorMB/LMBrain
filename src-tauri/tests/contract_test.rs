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
