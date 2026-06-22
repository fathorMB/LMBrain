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

    // Create task directories
    fs::create_dir_all(lmbrain.join("tasks").join("planned")).unwrap();
    fs::create_dir_all(lmbrain.join("tasks").join("in-progress")).unwrap();
    fs::create_dir_all(lmbrain.join("tasks").join("done")).unwrap();

    // Create spec directories
    fs::create_dir_all(lmbrain.join("specs").join("ready")).unwrap();
    fs::create_dir_all(lmbrain.join("specs").join("proposed")).unwrap();

    // Create review directories
    fs::create_dir_all(lmbrain.join("reviews").join("pending")).unwrap();
}

#[test]
fn test_build_diagnostics_no_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a task with matching status
    let task_content = r#"---
id: TASK-001
title: Test Task
status: in-progress
---
Task body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("tasks")
            .join("in-progress")
            .join("TASK-001.md"),
        task_content,
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
fn test_build_diagnostics_task_status_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a task in planned/ with status: done — should produce a diagnostic
    let task_content = r#"---
id: TASK-002
title: Mismatched Task
status: done
---
Task body"#;
    fs::write(
        dir.path()
            .join(".lmbrain")
            .join("tasks")
            .join("planned")
            .join("TASK-002.md"),
        task_content,
    )
    .unwrap();

    let diags = contract::build_diagnostics(dir.path());
    let mismatches: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("Status mismatch"))
        .collect();
    assert!(
        !mismatches.is_empty(),
        "Expected at least one status mismatch diagnostic"
    );
    assert!(mismatches[0].message.contains("planned"));
    assert!(mismatches[0].message.contains("done"));
}

#[test]
fn test_build_diagnostics_spec_status_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_kit(dir.path());

    // Create a spec in proposed/ with status: ready — should produce a diagnostic
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
            .join("proposed")
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
            .join("proposed")
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
