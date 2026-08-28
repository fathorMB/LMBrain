pub mod registry;

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

pub use registry::{call, specific_status, tools, ToolSpec, TOOLS};

/// Resolve the workspace root: explicit `--root <path>`/`--root=<path>` wins, then
/// `LMBRAIN_ROOT`, then the launch directory.
pub fn resolve_root(
    args: impl Iterator<Item = String>,
    env: Option<String>,
    cwd: Option<PathBuf>,
) -> PathBuf {
    let mut args = args;
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--root=") {
            return PathBuf::from(value);
        }
        if arg == "--root" {
            if let Some(value) = args.next() {
                return PathBuf::from(value);
            }
        }
    }

    if let Some(value) = env.filter(|value| !value.trim().is_empty()) {
        return PathBuf::from(value);
    }

    cwd.unwrap_or_else(|| PathBuf::from("."))
}

pub fn reply(id: Value, result: Result<Value, String>) {
    let response = match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(message) => {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":message}})
        }
    };
    println!("{response}");
    let _ = io::stdout().flush();
}

pub fn handle(root: &Path, request: &Value) -> Result<Value, String> {
    match request.get("method").and_then(Value::as_str) {
        Some("initialize") => Ok(json!({
            "protocolVersion":"2024-11-05",
            "capabilities":{"tools":{}},
            "serverInfo":{"name":"lmbrain-mcp","version":env!("CARGO_PKG_VERSION")}
        })),
        Some("tools/list") => Ok(json!({ "tools": tools() })),
        Some("tools/call") => call(root, request.get("params").unwrap_or(&Value::Null)),
        _ => Err("method not found".into()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use super::resolve_root;

    fn args(items: &[&str]) -> std::vec::IntoIter<String> {
        items
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn legacy_debt_workspace(root: &std::path::Path) {
        std::fs::create_dir_all(root.join(".lmbrain/findings/open")).unwrap();
        std::fs::create_dir_all(root.join(".lmbrain/reviews/accepted")).unwrap();
        std::fs::write(root.join(".lmbrain/VERSION"), "4.1.0\n").unwrap();
        std::fs::write(
            root.join(".lmbrain/findings/open/FINDING-001-sample.md"),
            "---\nid: FINDING-001\ntitle: Sample\nstatus: open\ncategory: correctness\nseverity: medium\ncreated: 2026-08-13\nupdated: 2026-08-13\norigin_artifact: REVIEW-001\norigin_ref: FINDING-07\nrelated_specs: []\nrelated_reviews: [REVIEW-001]\nrelated_decisions: []\ntarget_specs: []\nblocked_by: []\nresolution_refs: []\nfinding_events: []\n---\n## Statement\nSample\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".lmbrain/reviews/accepted/REVIEW-001.md"),
            "---\nid: REVIEW-001\ntitle: Review\nstatus: accepted\ncreated: 2026-08-13\nupdated: 2026-08-13\ntags: []\nlinks: []\n---\n## Findings\n- FINDING-07 local issue\n",
        )
        .unwrap();
    }

    #[test]
    fn flag_space_form_wins() {
        let root = resolve_root(
            args(&["--root", "/ws"]),
            Some("/env".into()),
            Some(PathBuf::from("/cwd")),
        );
        assert_eq!(root, PathBuf::from("/ws"));
    }

    #[test]
    fn flag_equals_form_wins() {
        let root = resolve_root(
            args(&["--root=/ws"]),
            Some("/env".into()),
            Some(PathBuf::from("/cwd")),
        );
        assert_eq!(root, PathBuf::from("/ws"));
    }

    #[test]
    fn env_used_when_no_flag() {
        let root = resolve_root(args(&[]), Some("/env".into()), Some(PathBuf::from("/cwd")));
        assert_eq!(root, PathBuf::from("/env"));
    }

    #[test]
    fn cwd_is_last_resort() {
        let root = resolve_root(args(&[]), Some("   ".into()), Some(PathBuf::from("/cwd")));
        assert_eq!(root, PathBuf::from("/cwd"));
    }

    #[test]
    fn agent_tools_include_operator_governed_transitions_and_exclude_tasks() {
        assert_eq!(super::specific_status("adr_accept"), Some("accepted"));
        assert_eq!(super::specific_status("adr_reject"), Some("rejected"));
        assert_eq!(super::specific_status("agent_activate"), Some("active"));
        assert!(super::specific_status("task_start").is_none());

        let names: Vec<String> = super::tools()
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();

        assert!(names.contains(&"adr_accept".to_string()));
        assert!(names.contains(&"adr_reject".to_string()));
        assert!(names.contains(&"agent_activate".to_string()));
        assert!(names.contains(&"agent_deactivate".to_string()));
        assert!(names.contains(&"skill_activate".to_string()));
        assert!(names.contains(&"skill_retire".to_string()));
        assert!(!names.iter().any(|name| name.starts_with("task_")));
        assert!(names.contains(&"spec_done".to_string()));
        assert!(names.contains(&"spec_discard".to_string()));
        assert!(names.contains(&"spec_park".to_string()));
        assert!(names.contains(&"spec_dependency_context".to_string()));
        assert!(names.contains(&"spec_dependency_candidates".to_string()));
        assert!(names.contains(&"spec_dependencies_set".to_string()));
        assert!(names.contains(&"spec_set_verification_gates".to_string()));
        assert!(names.contains(&"lmbrain_feedback_record".to_string()));
        assert!(names.contains(&"lmbrain_feedback_report".to_string()));
        assert!(names.contains(&"review_accept".to_string()));
        assert!(names.contains(&"review_changes_requested".to_string()));
        assert!(names.contains(&"review_block".to_string()));
        assert!(names.contains(&"review_supersede".to_string()));
        assert!(names.contains(&"review_remediation".to_string()));
        assert!(names.contains(&"review_escalate".to_string()));
        assert!(names.contains(&"review_takeover".to_string()));
        assert!(names.contains(&"review_migration_preview".to_string()));
        assert!(names.contains(&"spec_attest_lead".to_string()));
        assert!(names.contains(&"verification_migration_preview".to_string()));
        assert!(names.contains(&"verification_manifest_status".to_string()));
        assert!(names.contains(&"verification_manifest_init".to_string()));
        assert!(names.contains(&"verification_manifest_validate".to_string()));
        assert!(names.contains(&"verification_manifest_set".to_string()));
        assert!(names.contains(&"verification_manifest_rollback".to_string()));
        for name in [
            "debt_create",
            "debt_plan",
            "debt_defer",
            "debt_resolve",
            "debt_accept_risk",
            "debt_supersede",
            "debt_reopen",
            "debt_context",
            "debt_candidates",
            "debt_migration_preview",
            "debt_migrate",
        ] {
            assert!(names.contains(&name.to_string()), "{name} missing");
        }
        assert!(names.contains(&"lmbrain_set_agent_mnemonic_name".to_string()));
    }

    #[test]
    fn debt_migration_confirmation_is_typed_and_dispatches_true() {
        let tool = super::tools()
            .into_iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some("debt_migrate"))
            .unwrap();
        assert_eq!(
            tool.pointer("/inputSchema/properties/confirmed/type")
                .and_then(Value::as_str),
            Some("boolean")
        );
        assert_eq!(
            tool.pointer("/inputSchema/properties/confirmed/const")
                .and_then(Value::as_bool),
            Some(true)
        );

        let directory = tempfile::tempdir().unwrap();
        legacy_debt_workspace(directory.path());
        let root = directory.path().to_path_buf();
        let preview = super::call(
            &root,
            &serde_json::json!({"name":"debt_migration_preview","arguments":{}}),
        )
        .unwrap();
        let preview: Value = serde_json::from_str(
            preview
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            preview.get("schema_version").and_then(Value::as_str),
            Some("3")
        );
        assert!(preview
            .get("reference_mappings")
            .and_then(Value::as_array)
            .is_some_and(|mappings| !mappings.is_empty()
                && mappings.iter().all(|mapping| {
                    mapping
                        .get("occurrences")
                        .and_then(Value::as_u64)
                        .is_some_and(|occurrences| occurrences > 0)
                        && mapping.get("replacement").and_then(Value::as_str).is_some()
                        && mapping
                            .get("classification")
                            .and_then(Value::as_str)
                            .is_some()
                })));
        assert!(preview
            .get("scaffolding_items")
            .and_then(Value::as_array)
            .is_some());
        let digest = preview.get("digest").and_then(Value::as_str).unwrap();

        let migrated = super::call(
            &root,
            &serde_json::json!({
                "name":"debt_migrate",
                "arguments":{"expected_preview_digest":digest,"confirmed":true}
            }),
        )
        .unwrap();
        let migrated: Value = serde_json::from_str(
            migrated
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            migrated.get("version").and_then(Value::as_str),
            Some("4.2.0")
        );
        assert_eq!(
            std::fs::read_to_string(root.join(".lmbrain/VERSION")).unwrap(),
            "4.2.0\n"
        );
    }

    #[test]
    fn negative_review_verdict_tools_require_a_reason() {
        let tools = super::tools();
        for name in [
            "review_changes_requested",
            "review_block",
            "review_supersede",
        ] {
            let tool = tools
                .iter()
                .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
                .unwrap();
            let required = tool
                .pointer("/inputSchema/required")
                .and_then(Value::as_array)
                .unwrap();
            assert!(required.iter().any(|value| value.as_str() == Some("path")));
            assert!(required
                .iter()
                .any(|value| value.as_str() == Some("reason")));
        }

        let remediation = tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some("review_remediation"))
            .unwrap();
        let required = remediation
            .pointer("/inputSchema/required")
            .and_then(Value::as_array)
            .unwrap();
        assert!(required
            .iter()
            .any(|value| value.as_str() == Some("remediation_agent")));
    }

    #[test]
    fn review_verdict_dispatch_moves_and_audits_without_invalid_partial_writes() {
        let dir = tempfile::tempdir().unwrap();
        let pending_dir = dir.path().join(".lmbrain/reviews/pending");
        std::fs::create_dir_all(&pending_dir).unwrap();
        let profiles = dir.path().join(".lmbrain/agents/profiles");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::write(
            profiles.join("AGENT-002.md"),
            "---\nid: AGENT-002\nstatus: active\n---\n",
        )
        .unwrap();
        let pending = pending_dir.join("REVIEW-001.md");
        let source = "---\nid: REVIEW-001\nstatus: pending\n---\nReview\n";
        std::fs::write(&pending, source).unwrap();
        let root = dir.path().to_path_buf();

        let invalid = super::call(
            &root,
            &serde_json::json!({
                "name": "review_block",
                "arguments": {"path": ".lmbrain/reviews/pending/REVIEW-001.md"}
            }),
        );
        assert!(invalid.is_err());
        assert_eq!(std::fs::read_to_string(&pending).unwrap(), source);

        super::call(
            &root,
            &serde_json::json!({
                "name": "review_changes_requested",
                "arguments": {
                    "path": ".lmbrain/reviews/pending/REVIEW-001.md",
                    "reason": "Regression is missing",
                    "evidence_refs": ["SPEC-001", "tests/review.rs"],
                    "remediation_agent": "AGENT-002"
                }
            }),
        )
        .unwrap();
        let changed = dir
            .path()
            .join(".lmbrain/reviews/changes-requested/REVIEW-001.md");
        assert!(changed.exists());
        assert!(!pending.exists());

        for (name, arguments) in [
            (
                "review_remediation",
                serde_json::json!({
                    "path": ".lmbrain/reviews/changes-requested/REVIEW-001.md",
                    "reason": "Implemented the requested regression coverage",
                    "remediation_agent": "AGENT-002",
                    "evidence_refs": ["tests/review.rs"]
                }),
            ),
            (
                "review_escalate",
                serde_json::json!({
                    "path": ".lmbrain/reviews/changes-requested/REVIEW-001.md",
                    "reason": "Operator escalated after repeated remediation"
                }),
            ),
            (
                "review_takeover",
                serde_json::json!({
                    "path": ".lmbrain/reviews/changes-requested/REVIEW-001.md",
                    "reason": "Bounded corrective takeover authorized by operator"
                }),
            ),
        ] {
            super::call(
                &root,
                &serde_json::json!({"name": name, "arguments": arguments}),
            )
            .unwrap();
        }

        super::call(
            &root,
            &serde_json::json!({
                "name": "review_accept",
                "arguments": {
                    "path": ".lmbrain/reviews/changes-requested/REVIEW-001.md",
                    "evidence_refs": ["tests/review.rs"]
                }
            }),
        )
        .unwrap();
        let accepted = dir.path().join(".lmbrain/reviews/accepted/REVIEW-001.md");
        let document =
            lmbrain_core::frontmatter::Document::parse(&std::fs::read_to_string(accepted).unwrap())
                .unwrap();
        let events = document.object_array("review_events");
        assert_eq!(events.len(), 5);
        assert_eq!(
            events[0].get("actor_role").and_then(Value::as_str),
            Some("project-lead")
        );
        assert_eq!(
            events[0]
                .get("evidence_refs")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            events[4].get("actor_role").and_then(Value::as_str),
            Some("operator")
        );
        let analysis = lmbrain_core::analyze_review_lifecycle(&document);
        assert_eq!(analysis.escalation_count, 1);
        assert_eq!(analysis.takeover_count, 1);
        assert_eq!(analysis.remediation_agents, vec!["AGENT-002"]);
    }

    #[test]
    fn review_migration_preview_dispatch_is_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let reviews = dir.path().join(".lmbrain/reviews/accepted");
        std::fs::create_dir_all(&reviews).unwrap();
        let review = reviews.join("REVIEW-047.md");
        let source = "---\nid: REVIEW-047\nstatus: accepted\nreview_cycles: 3\nfinding_categories: [metric-cannot-fail]\n---\n";
        std::fs::write(&review, source).unwrap();

        let response = super::call(
            dir.path(),
            &serde_json::json!({
                "name": "review_migration_preview",
                "arguments": {}
            }),
        )
        .unwrap();
        assert!(response.to_string().contains("legacy_explicit_reviews"));
        assert!(response.to_string().contains("metrics-integrity"));
        assert_eq!(std::fs::read_to_string(review).unwrap(), source);
    }

    #[test]
    fn verification_migration_preview_dispatch_is_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let specs = dir.path().join(".lmbrain/specs/done");
        std::fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("SPEC-001.md");
        let source = "---\nid: SPEC-001\nstatus: done\n---\n## Required verification\n- [x] LEAD | kind=manual | owner=operator | phase=before-done | evidence=artifact | Project Lead review\n";
        std::fs::write(&spec, source).unwrap();

        let response = super::call(
            dir.path(),
            &serde_json::json!({
                "name": "verification_migration_preview",
                "arguments": {}
            }),
        )
        .unwrap();
        let payload: Value = serde_json::from_str(
            response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            payload.get("proposed_lead_count").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(payload.get("mutated").and_then(Value::as_bool), Some(false));
        assert_eq!(std::fs::read_to_string(spec).unwrap(), source);
    }

    #[test]
    fn verification_onboarding_dispatch_previews_sets_and_reports_unapproved_without_execution() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain")).unwrap();
        let root = dir.path().to_path_buf();

        let preview_response = super::call(
            &root,
            &serde_json::json!({
                "name": "verification_manifest_init",
                "arguments": {}
            }),
        )
        .unwrap();
        let preview: Value = serde_json::from_str(
            preview_response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            preview.pointer("/status/state").and_then(Value::as_str),
            Some("absent")
        );
        assert_eq!(
            preview
                .pointer("/candidates/0/gate/program")
                .and_then(Value::as_str),
            Some("cargo")
        );
        assert!(!dir.path().join(".lmbrain/verification.toml").exists());

        let manifest = preview.get("proposed_manifest").cloned().unwrap();
        super::call(
            &root,
            &serde_json::json!({
                "name": "verification_manifest_set",
                "arguments": {
                    "manifest": manifest,
                    "expected_current_digest": null
                }
            }),
        )
        .unwrap();
        let status_response = super::call(
            &root,
            &serde_json::json!({
                "name": "verification_manifest_status",
                "arguments": {}
            }),
        )
        .unwrap();
        let status: Value = serde_json::from_str(
            status_response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            status.get("state").and_then(Value::as_str),
            Some("unapproved")
        );
        assert_eq!(status.get("gate_count").and_then(Value::as_u64), Some(1));
        assert!(!dir.path().join("target").exists());
    }

    #[test]
    fn lead_attestation_dispatch_records_evidence_without_a_status_transition() {
        let dir = tempfile::tempdir().unwrap();
        let specs = dir.path().join(".lmbrain/specs/review");
        std::fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("SPEC-001.md");
        std::fs::write(
            &spec,
            "---\nid: SPEC-001\nstatus: review\n---\n## Required verification\n- [x] LEAD | kind=manual | owner=lead | phase=before-done | evidence=artifact | Independent review\n",
        )
        .unwrap();

        super::call(
            dir.path(),
            &serde_json::json!({
                "name": "spec_attest_lead",
                "arguments": {
                    "path": ".lmbrain/specs/review/SPEC-001.md",
                    "requirement_id": "LEAD",
                    "actor": "AGENT-LEAD",
                    "evidence_ref": "lead-review:SPEC-001"
                }
            }),
        )
        .unwrap();
        let document =
            lmbrain_core::frontmatter::Document::parse(&std::fs::read_to_string(spec).unwrap())
                .unwrap();
        assert_eq!(document.value("status").as_deref(), Some("review"));
        assert_eq!(lmbrain_core::verification_attestations(&document).len(), 1);
        assert_eq!(
            lmbrain_core::verification_attestations(&document)[0].actor_role,
            "lead"
        );
    }

    #[test]
    fn debt_protocol_creates_plans_and_contextualizes_without_rewriting_origin() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/reviews/accepted")).unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        let review = dir.path().join(".lmbrain/reviews/accepted/REVIEW-054.md");
        let review_source =
            "---\nid: REVIEW-054\ntitle: Review\nstatus: accepted\n---\n## Review findings\n- RF-007 debt\n";
        std::fs::write(&review, review_source).unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/specs/backlog/SPEC-059.md"),
            "---\nid: SPEC-059\ntitle: Target\nstatus: backlog\n---\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let created = super::call(
            &root,
            &serde_json::json!({
                "name":"debt_create",
                "arguments":{
                    "title":"Routed debt","category":"correctness","severity":"high",
                    "origin_artifact":"REVIEW-054","origin_ref":"RF-007",
                    "related_specs":[],"related_reviews":["REVIEW-054"],
                    "related_decisions":[],"blocked_by":[],"tags":[],
                    "statement":"Defect remains","evidence":"Review evidence",
                    "impact":"Incorrect behavior","resolution_criteria":"Regression passes",
                    "actor":"AGENT-LEAD","rationale":"Routed beyond origin"
                }
            }),
        )
        .unwrap();
        let created_payload: Value = serde_json::from_str(
            created
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let path = created_payload.get("path").and_then(Value::as_str).unwrap();
        super::call(
            &root,
            &serde_json::json!({
                "name":"debt_plan",
                "arguments":{
                    "path":path,"target_specs":["SPEC-059"],
                    "actor":"AGENT-LEAD","rationale":"Delivery routed"
                }
            }),
        )
        .unwrap();
        let context = super::call(
            &root,
            &serde_json::json!({
                "name":"debt_context","arguments":{"debt":"DEBT-001"}
            }),
        )
        .unwrap();
        let context_payload: Value = serde_json::from_str(
            context
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            context_payload
                .pointer("/debt/status")
                .and_then(Value::as_str),
            Some("planned")
        );
        assert_eq!(
            context_payload
                .pointer("/target_specs/0/id")
                .and_then(Value::as_str),
            Some("SPEC-059")
        );
        let candidates = super::call(
            &root,
            &serde_json::json!({"name":"debt_candidates","arguments":{}}),
        )
        .unwrap();
        let candidate_payload: Value = serde_json::from_str(
            candidates
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            candidate_payload.get("mutated").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(std::fs::read_to_string(review).unwrap(), review_source);
    }

    #[test]
    fn dependency_and_parking_protocol_is_governed_and_audited() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/done")).unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/agents/profiles")).unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/agents/profiles/AGENT-IMPL.md"),
            "---\nid: AGENT-IMPL\nstatus: active\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/specs/done/SPEC-001.md"),
            "---\nid: SPEC-001\ntitle: Prerequisite\nstatus: done\ndepends_on: []\n---\n",
        )
        .unwrap();
        let dependent = dir.path().join(".lmbrain/specs/backlog/SPEC-002.md");
        std::fs::write(
            &dependent,
            "---\nid: SPEC-002\ntitle: Dependent\nstatus: backlog\nrecommended_agent: AGENT-IMPL\ncapability_tier: terra\nthinking_level: standard\ndepends_on: []\ndependency_events: []\nparking_events: []\nupdated: 2026-07-29\n---\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let context = super::call(
            &root,
            &serde_json::json!({
                "name":"spec_dependency_context","arguments":{"spec":"SPEC-002"}
            }),
        )
        .unwrap();
        let context: Value = serde_json::from_str(
            context
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let digest = context
            .get("source_digest")
            .and_then(Value::as_str)
            .unwrap();
        super::call(
            &root,
            &serde_json::json!({
                "name":"spec_dependencies_set",
                "arguments":{
                    "path":".lmbrain/specs/backlog/SPEC-002.md",
                    "depends_on":["SPEC-001"],"actor":"AGENT-LEAD",
                    "reason":"Explicit delivery order","expected_digest":digest
                }
            }),
        )
        .unwrap();
        super::call(
            &root,
            &serde_json::json!({
                "name":"spec_ready",
                "arguments":{"path":".lmbrain/specs/backlog/SPEC-002.md"}
            }),
        )
        .unwrap();
        super::call(
            &root,
            &serde_json::json!({
                "name":"spec_park",
                "arguments":{
                    "path":".lmbrain/specs/ready/SPEC-002.md",
                    "actor":"AGENT-LEAD","reason":"Milestone order changed",
                    "revisit_condition":"After M-08"
                }
            }),
        )
        .unwrap();
        let document = lmbrain_core::frontmatter::Document::parse(
            &std::fs::read_to_string(dir.path().join(".lmbrain/specs/backlog/SPEC-002.md"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(document.string_array("depends_on"), vec!["SPEC-001"]);
        assert_eq!(document.object_array("dependency_events").len(), 1);
        assert_eq!(document.object_array("parking_events").len(), 1);
        assert_eq!(document.value("status").as_deref(), Some("backlog"));
    }

    #[test]
    fn skill_activation_moves_the_artifact_and_writes_its_registry_row() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/skills/proposed")).unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/skills/active")).unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/skills/proposed/SKILL-001-negative-gate-proof.md"),
            "---\nid: SKILL-001\ntitle: \"Negative gate proof\"\nstatus: proposed\nkind: verification\nrisk: low\napplies_to: [AGENT-001]\ncreated: 2026-08-26\nupdated: 2026-08-26\ntags: [verification]\nlinks: []\n---\n# Negative gate proof\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/skills/registry.md"),
            "---\ntitle: Skill registry\nupdated: 2026-07-07\n---\n\n# Skill Registry\n\n| ID | Skill | Status | Kind | Risk | Applies to | Definition |\n| --- | --- | --- | --- | --- | --- | --- |\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let response = super::call(
            &root,
            &serde_json::json!({
                "name":"skill_activate",
                "arguments":{"path":".lmbrain/skills/proposed/SKILL-001-negative-gate-proof.md"}
            }),
        )
        .unwrap();
        let payload: Value = serde_json::from_str(
            response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            payload.pointer("/registry_sync/action").and_then(Value::as_str),
            Some("inserted")
        );
        let registry =
            std::fs::read_to_string(dir.path().join(".lmbrain/skills/registry.md")).unwrap();
        assert!(registry.contains("| SKILL-001 | Negative gate proof | active | verification | low | AGENT-001 | `skills/active/SKILL-001-negative-gate-proof.md` |"));
        // The registry row and the moved artifact agree, so validate reports
        // no skill-registry divergence.
        let validation = super::call(
            &root,
            &serde_json::json!({"name":"lmbrain_validate","arguments":{}}),
        )
        .unwrap();
        let validation: Value = serde_json::from_str(
            validation
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert!(!validation["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"]
                .as_str()
                .is_some_and(|code| code.starts_with("skill-registry"))));
    }

    #[test]
    fn an_active_skill_missing_its_registry_row_fails_validation() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/skills/active")).unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/skills/active/SKILL-004-converge-derivations.md"),
            "---\nid: SKILL-004\ntitle: \"Converge derivations\"\nstatus: active\nkind: verification\nrisk: low\napplies_to: [AGENT-001]\ncreated: 2026-08-26\nupdated: 2026-08-26\ntags: [verification]\nlinks: []\n---\n# Converge derivations\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/skills/registry.md"),
            "---\ntitle: Skill registry\nupdated: 2026-07-07\n---\n\n# Skill Registry\n\n| ID | Skill | Status | Kind | Risk | Applies to | Definition |\n| --- | --- | --- | --- | --- | --- | --- |\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let validation = super::call(
            &root,
            &serde_json::json!({"name":"lmbrain_validate","arguments":{}}),
        )
        .unwrap();
        let validation: Value = serde_json::from_str(
            validation
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let diagnostic = validation["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["code"].as_str() == Some("skill-registry-row-missing"))
            .expect("missing registry row must be diagnosed");
        assert_eq!(diagnostic["severity"].as_str(), Some("error"));
        assert_eq!(diagnostic["artifact_id"].as_str(), Some("SKILL-004"));
    }

    #[test]
    fn project_lead_can_record_and_read_kit_feedback_without_project_lifecycle_changes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        std::fs::write(dir.path().join(".lmbrain/VERSION"), "3.1.0\n").unwrap();
        let spec = dir.path().join(".lmbrain/specs/backlog/SPEC-001.md");
        let spec_source = "---\nid: SPEC-001\nstatus: backlog\n---\n# Project work\n";
        std::fs::write(&spec, spec_source).unwrap();
        let root = dir.path().to_path_buf();
        super::call(
            &root,
            &serde_json::json!({
                "name":"lmbrain_feedback_record",
                "arguments":{
                    "category":"usability","severity":"medium",
                    "summary":"Operator language needs a second explanation",
                    "observed_behavior":"The Lead used unexplained internal shorthand.",
                    "expected_behavior":"The Lead should use concise plain language with the operator.",
                    "impact":"The operator had to request clarification.",
                    "evidence":"Observed in the operator-facing handoff.",
                    "workaround":"Ask for a human-readable explanation.",
                    "suggested_improvement":"Separate operator-facing and agent-facing language rules.",
                    "related_note":null,"actor":"AGENT-LEAD"
                }
            }),
        )
        .unwrap();
        let report = super::call(
            &root,
            &serde_json::json!({"name":"lmbrain_feedback_report","arguments":{}}),
        )
        .unwrap();
        let report: Value = serde_json::from_str(
            report
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(report.get("total").and_then(Value::as_u64), Some(1));
        assert_eq!(
            report.pointer("/notes/0/id").and_then(Value::as_str),
            Some("KIT-NOTE-001")
        );
        assert_eq!(std::fs::read_to_string(spec).unwrap(), spec_source);

        // A note resolves through the governed verb and the report derives
        // its status without the note being edited (#95).
        super::call(
            &root,
            &serde_json::json!({
                "name":"lmbrain_feedback_resolve",
                "arguments":{"note_id":"KIT-NOTE-001","kind":"resolved","version":"4.0.3","reference":null,"actor":"AGENT-LEAD"}
            }),
        )
        .unwrap();
        let report = super::call(
            &root,
            &serde_json::json!({"name":"lmbrain_feedback_report","arguments":{}}),
        )
        .unwrap();
        let report: Value = serde_json::from_str(
            report
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(report.get("resolved").and_then(Value::as_u64), Some(1));
        assert_eq!(
            report
                .pointer("/note_statuses/0/status")
                .and_then(Value::as_str),
            Some("resolved")
        );
        assert_eq!(
            report
                .pointer("/note_statuses/0/resolved_in")
                .and_then(Value::as_str),
            Some("4.0.3")
        );
    }

    #[test]
    fn create_tool_accepts_skill_kind() {
        let tools = super::tools();
        let tool = tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some("lmbrain_create"))
            .expect("lmbrain_create tool not found");
        let enum_values = tool
            .pointer("/inputSchema/properties/kind/enum")
            .and_then(Value::as_array)
            .expect("kind enum missing");
        assert!(enum_values
            .iter()
            .any(|value| value.as_str() == Some("skill")));
    }

    #[test]
    fn context_pack_tools_are_listed() {
        let names: Vec<String> = super::tools()
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        assert!(names.contains(&"lmbrain_project_digest".to_string()));
        assert!(names.contains(&"lmbrain_spec_context".to_string()));
        assert!(names.contains(&"lmbrain_review_context".to_string()));
        assert!(names.contains(&"harness_config_get".to_string()));
        assert!(names.contains(&"harness_config_validate".to_string()));
        assert!(names.contains(&"harness_config_set".to_string()));
        assert!(names.contains(&"verification_manifest_get".to_string()));
        assert!(names.contains(&"verification_manifest_approve".to_string()));
        assert!(names.contains(&"spec_verify".to_string()));
        assert!(names.contains(&"agent_improvement_signals".to_string()));
        assert!(names.contains(&"agent_improvement_propose".to_string()));
        assert!(names.contains(&"agent_improvement_apply".to_string()));
        assert!(names.contains(&"agent_proposal_approve".to_string()));
        assert!(names.contains(&"agent_proposal_reject".to_string()));
        assert!(names.contains(&"lmbrain_repair_frontmatter".to_string()));
        assert!(names.contains(&"harness_approval_status".to_string()));
        assert!(names.contains(&"harness_manifest_approve".to_string()));
        assert!(names.contains(&"harness_approval_revoke".to_string()));
        assert!(names.contains(&"harness_plan_preview".to_string()));
        assert!(names.contains(&"harness_config_apply".to_string()));
        assert!(names.contains(&"harness_drift_status".to_string()));
    }

    #[test]
    fn mcp_page_static_tool_list_matches_the_server_catalog() {
        // Guard for #88: the MCP page renders a static list of built-in tools;
        // this test fails whenever that list and the server catalog diverge,
        // so a new verb cannot silently stay invisible in the app.
        let catalog: std::collections::BTreeSet<String> = super::tools()
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .map(str::to_string)
            .collect();
        let frontend = include_str!("../../src/lib/mcpCatalog.ts");
        let array = frontend
            .split("export const LMBRAIN_MCP_TOOLS")
            .nth(1)
            .and_then(|rest| rest.split("];").next())
            .expect("LMBRAIN_MCP_TOOLS array not found in McpView.tsx");
        let listed: std::collections::BTreeSet<String> = array
            .split("name: \"")
            .skip(1)
            .filter_map(|chunk| chunk.split('"').next())
            .map(str::to_string)
            .collect();
        let missing: Vec<_> = catalog.difference(&listed).collect();
        let stale: Vec<_> = listed.difference(&catalog).collect();
        assert!(
            missing.is_empty() && stale.is_empty(),
            "MCP page tool list drifted from the server catalog.\nMissing from page: {missing:?}\nListed but not served: {stale:?}"
        );
    }

    #[test]
    fn harness_mutating_verbs_are_digest_bound_and_schema_tight() {
        let tools = super::tools();
        for name in ["harness_manifest_approve", "harness_config_apply"] {
            let tool = tools
                .iter()
                .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
                .unwrap_or_else(|| panic!("{name} tool not found"));
            assert!(tool
                .pointer("/inputSchema/properties/expected_digest")
                .is_some());
            assert!(tool.pointer("/inputSchema/properties/command").is_none());
            assert_eq!(
                tool.pointer("/inputSchema/additionalProperties"),
                Some(&Value::Bool(false))
            );
        }
    }

    #[test]
    fn governed_execution_tools_do_not_accept_ad_hoc_commands() {
        let tools = super::tools();
        let verify = tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some("spec_verify"))
            .expect("spec_verify tool not found");
        assert!(verify.pointer("/inputSchema/properties/path").is_some());
        assert!(verify.pointer("/inputSchema/properties/command").is_none());

        let apply = tools
            .iter()
            .find(|tool| {
                tool.get("name").and_then(Value::as_str) == Some("agent_improvement_apply")
            })
            .expect("agent_improvement_apply tool not found");
        assert!(apply.pointer("/inputSchema/properties/path").is_some());
        assert!(apply
            .pointer("/inputSchema/properties/raw_markdown")
            .is_none());
    }

    #[test]
    fn spec_set_verification_gates_binds_manifest_gates_and_rejects_unknown_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir_all(root.join(".lmbrain/specs/working")).unwrap();
        std::fs::write(
            root.join(".lmbrain/verification.toml"),
            "schema_version = 1

[[gates]]
id = \"unit\"
program = \"cargo\"
args = [\"test\"]
",
        )
        .unwrap();
        let spec = root.join(".lmbrain/specs/working/SPEC-001-demo.md");
        std::fs::write(
            &spec,
            "---
id: SPEC-001
title: Demo
status: working
verification_gates: []
---

## Acceptance criteria

- [ ] Works
",
        )
        .unwrap();
        let digest = lmbrain_core::content_digest(&std::fs::read(&spec).unwrap());

        let unknown = super::call(
            &root,
            &serde_json::json!({
                "name": "spec_set_verification_gates",
                "arguments": {
                    "path": ".lmbrain/specs/working/SPEC-001-demo.md",
                    "verification_gates": ["missing"],
                    "actor": "AGENT-LEAD",
                    "reason": "bind the approved manifest",
                    "expected_digest": digest
                }
            }),
        )
        .unwrap_err();
        assert!(
            unknown.contains("absent from the current manifest"),
            "{unknown}"
        );

        let ok = super::call(
            &root,
            &serde_json::json!({
                "name": "spec_set_verification_gates",
                "arguments": {
                    "path": ".lmbrain/specs/working/SPEC-001-demo.md",
                    "verification_gates": ["unit"],
                    "actor": "AGENT-LEAD",
                    "reason": "bind the approved manifest",
                    "expected_digest": digest
                }
            }),
        )
        .unwrap();
        assert!(ok.to_string().contains("unit"));
        let written = std::fs::read_to_string(&spec).unwrap();
        assert!(
            written.contains("verification_gates: [\"unit\"]"),
            "{written}"
        );
        assert!(written.contains("verification_gate_events"), "{written}");
    }

    #[test]
    fn harness_set_validates_and_writes_without_materializing_host_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        let args = serde_json::json!({"manifest":{"schema_version":1,"hosts":{}}});
        let response = super::call(
            dir.path(),
            &serde_json::json!({"name":"harness_config_set","arguments":args}),
        )
        .unwrap();
        assert!(
            response.to_string().contains("manifest_digest")
                || response.to_string().contains("digest")
        );
        assert!(dir.path().join(".lmbrain/HARNESSES.json").exists());
        assert!(!dir.path().join(".mcp.json").exists());
    }

    #[test]
    fn get_artifact_reads_workspace_files_and_rejects_escapes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        std::fs::write(
            dir.path().join(".lmbrain/specs/backlog/SPEC-001-demo.md"),
            "---\nid: SPEC-001\n---\n\n# Demo\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();

        let ok = super::call(
            &root,
            &serde_json::json!({
                "name": "lmbrain_get_artifact",
                "arguments": {"path": ".lmbrain/specs/backlog/SPEC-001-demo.md"}
            }),
        )
        .unwrap();
        assert!(ok.to_string().contains("SPEC-001"));

        for escape in ["../outside.md", "/etc/passwd", r"..\outside.md"] {
            let error = super::call(
                &root,
                &serde_json::json!({
                    "name": "lmbrain_get_artifact",
                    "arguments": {"path": escape}
                }),
            )
            .unwrap_err();
            let canonical = root.canonicalize().unwrap();
            assert!(
                !error.contains(
                    &lmbrain_core::path::clean_path(&canonical)
                        .display()
                        .to_string()
                ),
                "error leaks host workspace path: {error}"
            );
        }
    }

    #[test]
    fn create_dispatch_fails_closed_on_invalid_status_reserved_and_malformed_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain")).unwrap();
        let root = dir.path().to_path_buf();
        let call = |arguments: serde_json::Value| {
            super::call(
                &root,
                &serde_json::json!({"name":"lmbrain_create","arguments":arguments}),
            )
        };

        let ok = call(serde_json::json!({"kind":"spec","title":"Valid"})).unwrap();
        assert!(ok.to_string().contains("SPEC-001"));

        let status = call(serde_json::json!({"kind":"spec","title":"Bad","status":"../escape"}))
            .unwrap_err();
        assert!(status.contains("invalid creation status"), "{status}");

        let reserved = call(serde_json::json!({
            "kind":"spec","title":"Bad","fields":[["id","SPEC-999"]]
        }))
        .unwrap_err();
        assert!(reserved.contains("core-owned"), "{reserved}");

        let malformed = call(serde_json::json!({
            "kind":"spec","title":"Bad","fields":[["only-key"]]
        }))
        .unwrap_err();
        assert!(malformed.contains("[key, value]"), "{malformed}");

        let nonstring = call(serde_json::json!({
            "kind":"spec","title":"Bad","fields":[["key", 42]]
        }))
        .unwrap_err();
        assert!(nonstring.contains("[key, value]"), "{nonstring}");
    }

    #[test]
    fn context_tools_have_input_schemas() {
        let tools = super::tools();
        for tool in &tools {
            let name = tool.get("name").and_then(Value::as_str).unwrap_or("");
            if !name.starts_with("lmbrain_") {
                continue;
            }
            let schema = tool.get("inputSchema").and_then(Value::as_object);
            assert!(schema.is_some(), "Tool {name} is missing inputSchema");
        }
    }

    #[test]
    fn spec_context_tool_requires_spec_param() {
        let tools = super::tools();
        let tool = tools
            .iter()
            .find(|t| t.get("name").and_then(Value::as_str) == Some("lmbrain_spec_context"))
            .expect("lmbrain_spec_context tool not found");
        let schema = tool.get("inputSchema").and_then(Value::as_object).unwrap();
        let required = schema.get("required").and_then(Value::as_array).unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("spec")));
    }

    #[test]
    fn review_context_tool_requires_spec_param() {
        let tools = super::tools();
        let tool = tools
            .iter()
            .find(|t| t.get("name").and_then(Value::as_str) == Some("lmbrain_review_context"))
            .expect("lmbrain_review_context tool not found");
        let schema = tool.get("inputSchema").and_then(Value::as_object).unwrap();
        let required = schema.get("required").and_then(Value::as_array).unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("spec")));
    }

    #[test]
    fn project_digest_tool_has_no_required_params() {
        let tools = super::tools();
        let tool = tools
            .iter()
            .find(|t| t.get("name").and_then(Value::as_str) == Some("lmbrain_project_digest"))
            .expect("lmbrain_project_digest tool not found");
        let schema = tool.get("inputSchema").and_then(Value::as_object).unwrap();
        // Should have no required params
        let required = schema.get("required");
        assert!(required.is_none());
    }

    #[test]
    fn project_digest_and_validate_share_bounded_actionable_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let specs = dir.path().join(".lmbrain/specs/backlog");
        std::fs::create_dir_all(&specs).unwrap();
        for index in 1..=55 {
            std::fs::write(
                specs.join(format!("SPEC-{index:03}.md")),
                format!("---\nid: SPEC-{index:03}\n"),
            )
            .unwrap();
        }
        let root = dir.path().to_path_buf();
        let digest_response = super::call(
            &root,
            &serde_json::json!({
                "name": "lmbrain_project_digest",
                "arguments": {}
            }),
        )
        .unwrap();
        let digest: Value = serde_json::from_str(
            digest_response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let validation_response = super::call(
            &root,
            &serde_json::json!({
                "name": "lmbrain_validate",
                "arguments": {}
            }),
        )
        .unwrap();
        let validation: Value = serde_json::from_str(
            validation_response
                .pointer("/content/0/text")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let digest_diagnostics = digest
            .pointer("/diagnostics/items")
            .and_then(Value::as_array)
            .unwrap();
        let validation_diagnostics = validation
            .get("diagnostics")
            .and_then(Value::as_array)
            .unwrap();
        assert_eq!(digest_diagnostics.len(), 50);
        assert!(
            digest
                .pointer("/diagnostics/omitted")
                .and_then(Value::as_u64)
                .unwrap()
                > 0
        );
        assert_eq!(
            digest_diagnostics[0].get("id"),
            validation_diagnostics[0].get("id")
        );
        assert!(digest_diagnostics.iter().all(|diagnostic| {
            diagnostic
                .get("next_action")
                .and_then(Value::as_str)
                .is_some_and(|action| !action.is_empty())
        }));
    }

    #[test]
    fn branching_strategy_get_and_set_verbs_work_and_enforce_operator_authority() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let get_resp = super::call(
            &root,
            &serde_json::json!({
                "name": "branching_strategy_get",
                "arguments": {}
            }),
        )
        .unwrap();
        let get_val: Value = serde_json::from_str(
            get_resp
                .pointer("/content/0/text")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(get_val["digest"]["status"], "absent");

        let default_strat = lmbrain_core::BranchingStrategy::default_scaffolded();
        let set_lead_resp = super::call(
            &root,
            &serde_json::json!({
                "name": "branching_strategy_set",
                "arguments": {
                    "strategy": default_strat,
                    "actor": "project-lead",
                    "reason": "attempt lead mutation"
                }
            }),
        );
        assert!(set_lead_resp.is_err());

        let set_op_resp = super::call(
            &root,
            &serde_json::json!({
                "name": "branching_strategy_set",
                "arguments": {
                    "strategy": default_strat,
                    "actor": "operator",
                    "reason": "initialize strategy"
                }
            }),
        )
        .unwrap();
        assert!(set_op_resp
            .pointer("/content/0/text")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("\"success\":true"));

        let get_after_resp = super::call(
            &root,
            &serde_json::json!({
                "name": "branching_strategy_get",
                "arguments": {}
            }),
        )
        .unwrap();
        let get_after_val: Value = serde_json::from_str(
            get_after_resp
                .pointer("/content/0/text")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(get_after_val["digest"]["status"], "declared");
    }
}
