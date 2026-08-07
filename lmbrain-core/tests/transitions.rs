use lmbrain_core::{
    context::{build_review_context, build_spec_context},
    frontmatter::Document,
    invariants, park_spec,
    transitions::{
        create, record_review_event, review_verdict, set_agent_mnemonic_name, supersede_adr,
        transition, ArtifactKind, CreateRequest, MutationOptions,
    },
    build_diagnostics,
    parse_review_event_history, ReviewEventInput, SpecParkingInput,
};
use std::fs;
use tempfile::tempdir;

fn write(root: &std::path::Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn spec_parking_is_semantic_audited_and_requires_normal_reapproval() {
    let dir = tempdir().unwrap();
    let ready = ".lmbrain/specs/ready/SPEC-077.md";
    write(
        dir.path(),
        ready,
        "---\nid: SPEC-077\ntitle: Park me\nstatus: ready\nrecommended_agent: AGENT-IMPL\ncapability_tier: terra\nthinking_level: standard\ndepends_on: []\nparking_events: []\nactivity: []\nupdated: 2026-07-29\n---\n# Park me\n",
    );
    write(
        dir.path(),
        ".lmbrain/agents/profiles/AGENT-IMPL.md",
        "---\nid: AGENT-IMPL\nstatus: active\n---\n",
    );
    let parked = park_spec(
        dir.path(),
        ready,
        SpecParkingInput {
            actor: "AGENT-LEAD".into(),
            reason: "Milestone order changed".into(),
            revisit_condition: Some("After SPEC-080".into()),
        },
    )
    .unwrap();
    assert_eq!(parked.status, "backlog");
    assert!(!dir.path().join(ready).exists());
    let document = Document::parse(&fs::read_to_string(&parked.path).unwrap()).unwrap();
    assert_eq!(document.object_array("parking_events").len(), 1);
    assert!(transition(
        dir.path(),
        &parked.path,
        "working",
        MutationOptions::default()
    )
    .is_err());
    let reapproved = transition(
        dir.path(),
        &parked.path,
        "ready",
        MutationOptions::default(),
    )
    .unwrap();
    let reapproved_document =
        Document::parse(&fs::read_to_string(&reapproved.path).unwrap()).unwrap();
    assert_eq!(reapproved_document.object_array("parking_events").len(), 1);
}

#[test]
fn parking_rejects_wrong_state_reason_and_collision_without_partial_mutation() {
    let dir = tempdir().unwrap();
    let ready = ".lmbrain/specs/ready/SPEC-078.md";
    let source = "---\nid: SPEC-078\nstatus: ready\nparking_events: []\n---\n";
    write(dir.path(), ready, source);
    assert!(park_spec(
        dir.path(),
        ready,
        SpecParkingInput {
            actor: "AGENT-LEAD".into(),
            reason: " ".into(),
            revisit_condition: None,
        },
    )
    .is_err());
    write(
        dir.path(),
        ".lmbrain/specs/backlog/SPEC-078.md",
        "---\nid: SPEC-999\nstatus: backlog\n---\n",
    );
    assert!(park_spec(
        dir.path(),
        ready,
        SpecParkingInput {
            actor: "AGENT-LEAD".into(),
            reason: "Collision fixture".into(),
            revisit_condition: None,
        },
    )
    .is_err());
    assert_eq!(fs::read_to_string(dir.path().join(ready)).unwrap(), source);
}

#[test]
fn hard_dependencies_block_readiness_and_force_records_exact_chain() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        ".lmbrain/agents/profiles/AGENT-IMPL.md",
        "---\nid: AGENT-IMPL\nstatus: active\n---\n",
    );
    write(
        dir.path(),
        ".lmbrain/specs/backlog/SPEC-090.md",
        "---\nid: SPEC-090\ntitle: Prerequisite\nstatus: backlog\ndepends_on: []\n---\n",
    );
    let dependent = ".lmbrain/specs/backlog/SPEC-091.md";
    write(
        dir.path(),
        dependent,
        "---\nid: SPEC-091\ntitle: Dependent\nstatus: backlog\nrecommended_agent: AGENT-IMPL\ndepends_on: [SPEC-090]\nmutation_overrides: []\n---\n",
    );
    let error = transition(dir.path(), dependent, "ready", MutationOptions::default()).unwrap_err();
    assert!(error.to_string().contains("SPEC-090 [backlog]"));

    let forced = transition(
        dir.path(),
        dependent,
        "ready",
        MutationOptions {
            force: true,
            reason: Some("Emergency sequencing exception".into()),
        },
    )
    .unwrap();
    let document = Document::parse(&fs::read_to_string(forced.path).unwrap()).unwrap();
    let overrides = document.object_array("mutation_overrides");
    assert_eq!(overrides.len(), 1);
    assert!(overrides[0]
        .get("unmet_invariant")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .contains("SPEC-090 [backlog]"));
}
fn artifact(kind: ArtifactKind, status: &str) -> (&'static str, String) {
    let (id, base) = match kind {
        ArtifactKind::Spec => ("SPEC-001", "specs"),
        ArtifactKind::Review => ("REVIEW-001", "reviews"),
        ArtifactKind::Adr => ("ADR-001", "decisions"),
        ArtifactKind::Agent => ("AGENT-001", "agents/profiles"),
        ArtifactKind::AgentProposal => ("AGENT-PROP-001", "agents/proposals"),
        ArtifactKind::Mcp => ("MCP-001", "mcp/specs"),
        ArtifactKind::McpProposal => ("MCP-PROP-001", "mcp/proposals"),
        ArtifactKind::Handoff => ("HANDOFF-001", "handoffs/active"),
        ArtifactKind::Skill => ("SKILL-001", "skills"),
        ArtifactKind::Finding => ("FINDING-001", "findings"),
    };
    let relative = if matches!(
        kind,
        ArtifactKind::Spec | ArtifactKind::Review | ArtifactKind::Skill | ArtifactKind::Finding
    ) {
        format!(".lmbrain/{base}/{status}/{id}.md")
    } else {
        format!(".lmbrain/{base}/{id}.md")
    };
    (id, relative)
}
fn source(id: &str, status: &str) -> String {
    format!("---\nid: {id}\nstatus: {status}\n---\n\n## Acceptance criteria\n- [x] Complete\n\n## Evidence\nproof\n")
}

#[test]
fn every_declared_transition_has_valid_and_illegal_coverage() {
    let cases = &[
        (ArtifactKind::Spec, "backlog", "ready"),
        (ArtifactKind::Spec, "ready", "working"),
        (ArtifactKind::Spec, "working", "review"),
        (ArtifactKind::Spec, "review", "done"),
        (ArtifactKind::Spec, "backlog", "discarded"),
        (ArtifactKind::Review, "pending", "accepted"),
        (ArtifactKind::Review, "pending", "changes-requested"),
        (ArtifactKind::Review, "pending", "blocked"),
        (ArtifactKind::Review, "pending", "superseded"),
        (
            ArtifactKind::Review,
            "changes-requested",
            "changes-requested",
        ),
        (ArtifactKind::Review, "changes-requested", "accepted"),
        (ArtifactKind::Review, "changes-requested", "blocked"),
        (ArtifactKind::Review, "blocked", "blocked"),
        (ArtifactKind::Review, "blocked", "changes-requested"),
        (ArtifactKind::Review, "blocked", "accepted"),
        (ArtifactKind::Adr, "proposed", "accepted"),
        (ArtifactKind::Adr, "proposed", "rejected"),
        (ArtifactKind::Adr, "accepted", "superseded"),
        (ArtifactKind::Adr, "accepted", "deprecated"),
        (ArtifactKind::Agent, "proposed", "active"),
        (ArtifactKind::Agent, "proposed", "inactive"),
        (ArtifactKind::Agent, "active", "inactive"),
        (ArtifactKind::Agent, "inactive", "active"),
        (ArtifactKind::Agent, "active", "retired"),
        (ArtifactKind::AgentProposal, "proposed", "approved"),
        (ArtifactKind::AgentProposal, "proposed", "rejected"),
        (ArtifactKind::Mcp, "specified", "active"),
        (ArtifactKind::Mcp, "active", "inactive"),
        (ArtifactKind::Mcp, "inactive", "active"),
        (ArtifactKind::Mcp, "active", "deprecated"),
        (ArtifactKind::McpProposal, "proposed", "approved"),
        (ArtifactKind::McpProposal, "proposed", "rejected"),
        (ArtifactKind::McpProposal, "approved", "implemented"),
        (ArtifactKind::McpProposal, "approved", "blocked"),
        (ArtifactKind::Handoff, "ready", "consumed"),
        (ArtifactKind::Handoff, "ready", "superseded"),
        (ArtifactKind::Handoff, "consumed", "archived"),
        (ArtifactKind::Skill, "proposed", "active"),
        (ArtifactKind::Skill, "proposed", "retired"),
        (ArtifactKind::Skill, "active", "retired"),
    ];
    for &(kind, from, to) in cases {
        let d = tempdir().unwrap();
        let (id, path) = artifact(kind, from);
        write(d.path(), &path, &source(id, from));
        let options = MutationOptions {
            force: true,
            reason: Some("matrix fixture bypasses cross-artifact setup".into()),
        };
        let valid = if kind == ArtifactKind::Review {
            review_verdict(
                d.path(),
                &path,
                to,
                ReviewEventInput {
                    actor_role: if to == "accepted" {
                        "operator".into()
                    } else {
                        "project-lead".into()
                    },
                    reason: "matrix verdict".into(),
                    evidence_refs: vec![],
                    remediation_agent: None,
                },
                options,
            )
        } else {
            transition(d.path(), &path, to, options)
        };
        assert!(valid.is_ok(), "{kind:?} {from}->{to}: {valid:?}");
        let d = tempdir().unwrap();
        let (id, path) = artifact(kind, from);
        write(d.path(), &path, &source(id, from));
        let illegal = transition(d.path(), &path, "not-a-status", MutationOptions::default());
        assert!(
            illegal.is_err(),
            "{kind:?} {from} illegal transition accepted"
        );
    }
}

#[test]
fn review_verdicts_are_typed_audited_and_repeatable() {
    let d = tempdir().unwrap();
    let pending = ".lmbrain/reviews/pending/REVIEW-001.md";
    write(d.path(), pending, &source("REVIEW-001", "pending"));

    let first = review_verdict(
        d.path(),
        pending,
        "changes-requested",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: "Missing regression coverage".into(),
            evidence_refs: vec!["SPEC-001".into(), "tests/review.rs".into()],
            remediation_agent: Some("AGENT-002".into()),
        },
        MutationOptions::default(),
    )
    .unwrap();
    let second = review_verdict(
        d.path(),
        &first.path,
        "changes-requested",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: "Regression still reproduces".into(),
            evidence_refs: vec!["REVIEW-001-EVIDENCE-002".into()],
            remediation_agent: None,
        },
        MutationOptions::default(),
    )
    .unwrap();
    let accepted = review_verdict(
        d.path(),
        &second.path,
        "accepted",
        ReviewEventInput {
            actor_role: "operator".into(),
            reason: String::new(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    )
    .unwrap();

    let document = Document::parse(&fs::read_to_string(accepted.path).unwrap()).unwrap();
    let events = document.object_array("review_events");
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0].get("id").and_then(serde_json::Value::as_str),
        Some("REVIEW-001-EVENT-001")
    );
    assert_eq!(
        events[1].get("id").and_then(serde_json::Value::as_str),
        Some("REVIEW-001-EVENT-002")
    );
    assert_eq!(
        events[2]
            .get("to_status")
            .and_then(serde_json::Value::as_str),
        Some("accepted")
    );
    assert_eq!(
        events[0]
            .get("evidence_refs")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        events[0]
            .get("remediation_agent")
            .and_then(serde_json::Value::as_str),
        Some("AGENT-002")
    );
}

#[test]
fn review_verdict_validation_fails_without_mutating_the_artifact() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/pending/REVIEW-001.md";
    write(d.path(), path, &source("REVIEW-001", "pending"));
    let before = fs::read_to_string(d.path().join(path)).unwrap();

    let missing_reason = review_verdict(
        d.path(),
        path,
        "blocked",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: " ".into(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(missing_reason.is_err());
    assert_eq!(fs::read_to_string(d.path().join(path)).unwrap(), before);

    let wrong_authority = review_verdict(
        d.path(),
        path,
        "accepted",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: String::new(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(wrong_authority.is_err());
    assert_eq!(fs::read_to_string(d.path().join(path)).unwrap(), before);

    let generic_bypass = transition(d.path(), path, "accepted", MutationOptions::default());
    assert!(generic_bypass.is_err());
    assert_eq!(fs::read_to_string(d.path().join(path)).unwrap(), before);
}

#[test]
fn review_destination_collision_preserves_both_files() {
    let d = tempdir().unwrap();
    let pending = ".lmbrain/reviews/pending/REVIEW-001.md";
    let accepted = ".lmbrain/reviews/accepted/REVIEW-001.md";
    write(d.path(), pending, &source("REVIEW-001", "pending"));
    write(
        d.path(),
        accepted,
        "---\nid: REVIEW-COLLISION\nstatus: accepted\n---\nExisting\n",
    );
    let pending_before = fs::read_to_string(d.path().join(pending)).unwrap();
    let accepted_before = fs::read_to_string(d.path().join(accepted)).unwrap();

    let result = review_verdict(
        d.path(),
        pending,
        "accepted",
        ReviewEventInput {
            actor_role: "operator".into(),
            reason: String::new(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(d.path().join(pending)).unwrap(),
        pending_before
    );
    assert_eq!(
        fs::read_to_string(d.path().join(accepted)).unwrap(),
        accepted_before
    );
}

#[test]
fn review_non_verdict_events_are_append_only_and_attributable() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/changes-requested/REVIEW-001.md";
    write(
        d.path(),
        path,
        "---\nid: REVIEW-001\nstatus: changes-requested\nimplementation_agent: AGENT-001\n---\n",
    );
    let event =
        |actor_role: &str, reason: &str, remediation_agent: Option<&str>| ReviewEventInput {
            actor_role: actor_role.into(),
            reason: reason.into(),
            evidence_refs: vec!["SPEC-001".into()],
            remediation_agent: remediation_agent.map(str::to_owned),
        };
    record_review_event(
        d.path(),
        path,
        "remediation",
        event(
            "implementation-specialist",
            "Addressed the requested changes",
            Some("AGENT-002"),
        ),
        MutationOptions::default(),
    )
    .unwrap();
    record_review_event(
        d.path(),
        path,
        "escalation",
        event(
            "operator",
            "Two remediation attempts were insufficient",
            None,
        ),
        MutationOptions::default(),
    )
    .unwrap();
    record_review_event(
        d.path(),
        path,
        "takeover",
        event(
            "project-lead",
            "Operator-authorized bounded corrective takeover",
            None,
        ),
        MutationOptions::default(),
    )
    .unwrap();

    let document = Document::parse(&fs::read_to_string(d.path().join(path)).unwrap()).unwrap();
    assert_eq!(
        document.value("status").as_deref(),
        Some("changes-requested")
    );
    let history = lmbrain_core::parse_review_event_history(&document);
    assert_eq!(history.events.len(), 3);
    assert_eq!(history.events[0].action, "remediation");
    assert_eq!(
        history.events[0].remediation_agent.as_deref(),
        Some("AGENT-002")
    );
    let analysis = lmbrain_core::analyze_review_lifecycle(&document);
    assert_eq!(analysis.escalation_count, 1);
    assert_eq!(analysis.takeover_count, 1);
    assert_eq!(analysis.remediation_agents, vec!["AGENT-002"]);
}

#[test]
fn review_lifecycle_reasons_stay_typed_and_preserve_final_decision_body() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/changes-requested/REVIEW-001.md";
    let long_reason = "independent verification ".repeat(80);
    write(
        d.path(),
        path,
        "---\nid: REVIEW-001\nstatus: changes-requested\n---\n\n## Analysis\nHuman-authored analysis\n\n## Final decision\nKeep this conclusion last.\n",
    );
    record_review_event(
        d.path(),
        path,
        "remediation",
        ReviewEventInput {
            actor_role: "implementation-specialist".into(),
            reason: "Remediation completed".into(),
            evidence_refs: vec!["tests/review.rs".into()],
            remediation_agent: Some("AGENT-002".into()),
        },
        MutationOptions::default(),
    )
    .unwrap();
    record_review_event(
        d.path(),
        path,
        "remediation-verification",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: long_reason.clone(),
            evidence_refs: vec!["REVIEW-001".into()],
            remediation_agent: None,
        },
        MutationOptions::default(),
    )
    .unwrap();

    let output = fs::read_to_string(d.path().join(path)).unwrap();
    assert!(!output.contains("## Mutation override"));
    assert!(output.ends_with("## Final decision\nKeep this conclusion last.\n"));
    let document = Document::parse(&output).unwrap();
    let events = document.object_array("review_events");
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[1].get("reason").and_then(serde_json::Value::as_str),
        Some(long_reason.trim())
    );
}

#[test]
fn review_non_verdict_events_enforce_authority_and_required_attribution() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/pending/REVIEW-001.md";
    write(d.path(), path, &source("REVIEW-001", "pending"));
    let before = fs::read_to_string(d.path().join(path)).unwrap();

    for (action, input) in [
        (
            "escalation",
            ReviewEventInput {
                actor_role: "project-lead".into(),
                reason: "Not operator-owned".into(),
                evidence_refs: vec![],
                remediation_agent: None,
            },
        ),
        (
            "remediation",
            ReviewEventInput {
                actor_role: "implementation-specialist".into(),
                reason: "Missing agent".into(),
                evidence_refs: vec![],
                remediation_agent: None,
            },
        ),
    ] {
        assert!(
            record_review_event(d.path(), path, action, input, MutationOptions::default()).is_err()
        );
        assert_eq!(fs::read_to_string(d.path().join(path)).unwrap(), before);
    }
}

#[test]
fn review_remediation_verification_requires_order_and_evidence() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/changes-requested/REVIEW-001.md";
    write(d.path(), path, &source("REVIEW-001", "changes-requested"));
    let verification = || ReviewEventInput {
        actor_role: "project-lead".into(),
        reason: "Verified the remediation independently".into(),
        evidence_refs: vec!["REVIEW-001".into()],
        remediation_agent: None,
    };

    let before_remediation = record_review_event(
        d.path(),
        path,
        "remediation-verification",
        verification(),
        MutationOptions::default(),
    );
    assert!(before_remediation.is_err());

    record_review_event(
        d.path(),
        path,
        "remediation",
        ReviewEventInput {
            actor_role: "implementation-specialist".into(),
            reason: "Implemented the requested change".into(),
            evidence_refs: vec!["tests/review.rs".into()],
            remediation_agent: Some("AGENT-002".into()),
        },
        MutationOptions::default(),
    )
    .unwrap();
    record_review_event(
        d.path(),
        path,
        "remediation-verification",
        verification(),
        MutationOptions::default(),
    )
    .unwrap();

    let repeated = record_review_event(
        d.path(),
        path,
        "remediation-verification",
        verification(),
        MutationOptions::default(),
    );
    assert!(repeated.is_err());
    assert_eq!(
        parse_review_event_history(
            &Document::parse(&fs::read_to_string(d.path().join(path)).unwrap()).unwrap()
        )
        .events
        .len(),
        2
    );
}

#[test]
fn review_remediation_verification_requires_evidence_and_lead_authority() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/changes-requested/REVIEW-001.md";
    write(d.path(), path, &source("REVIEW-001", "changes-requested"));
    record_review_event(
        d.path(),
        path,
        "remediation",
        ReviewEventInput {
            actor_role: "implementation-specialist".into(),
            reason: "Implemented the requested change".into(),
            evidence_refs: vec![],
            remediation_agent: Some("AGENT-002".into()),
        },
        MutationOptions::default(),
    )
    .unwrap();

    let no_evidence = record_review_event(
        d.path(),
        path,
        "remediation-verification",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: "Checked the remediation".into(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(no_evidence.is_err());

    let wrong_actor = record_review_event(
        d.path(),
        path,
        "remediation-verification",
        ReviewEventInput {
            actor_role: "implementation-specialist".into(),
            reason: "Checked the remediation".into(),
            evidence_refs: vec!["tests/review.rs".into()],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(wrong_actor.is_err());
}

#[test]
fn malformed_review_event_history_fails_closed() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/reviews/pending/REVIEW-001.md";
    let malformed =
        "---\nid: REVIEW-001\nstatus: pending\nreview_events: not-a-list\n---\nReview\n";
    write(d.path(), path, malformed);

    let result = review_verdict(
        d.path(),
        path,
        "changes-requested",
        ReviewEventInput {
            actor_role: "project-lead".into(),
            reason: "Needs work".into(),
            evidence_refs: vec![],
            remediation_agent: None,
        },
        MutationOptions::default(),
    );
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(d.path().join(path)).unwrap(), malformed);
}

#[test]
fn invariants_cover_reviews_handoffs_specs_criteria_and_agents() {
    let d = tempdir().unwrap();
    let r = d.path();
    write(
        r,
        ".lmbrain/reviews/accepted/REVIEW-001.md",
        "---\nid: REVIEW-001\nspec: SPEC-001\nstatus: accepted\n---\n",
    );
    assert!(invariants::spec_has_accepted_review(r, "SPEC-001"));
    assert!(!invariants::spec_has_accepted_review(r, "SPEC-404"));
    write(
        r,
        ".lmbrain/handoffs/active/HANDOFF-001.md",
        "---\nid: HANDOFF-001\nstatus: ready\n---\n",
    );
    assert!(!invariants::single_ready_handoff(r, None));
    assert!(invariants::single_ready_handoff(
        r,
        Some(&r.join(".lmbrain/handoffs/active/HANDOFF-001.md"))
    ));
    assert!(invariants::criteria_complete_with_evidence(
        "## Acceptance criteria\n- [x] Done\n## Evidence\nproof"
    ));
    assert!(invariants::criteria_complete_with_evidence(
        "## Acceptance criteria\n- [x] Done\n\n## Implementation evidence\n### Changes made\nImplemented.\n\n### Handoff status\n- [ ] Ready for Project Lead review"
    ));
    assert!(!invariants::criteria_complete_with_evidence(
        "## Evidence\nproof"
    ));
    assert!(!invariants::criteria_complete_with_evidence(
        "## Acceptance criteria\n- [ ] Pending\n## Evidence\nproof"
    ));
    assert!(!invariants::criteria_complete_with_evidence(
        "## Acceptance criteria\n- [x] Done\n\n## Implementation evidence\n### Changes made\n### Verification performed"
    ));
    write(
        r,
        ".lmbrain/agents/profiles/AGENT-001.md",
        "---\nid: AGENT-001\nstatus: active\n---\n",
    );
    assert!(invariants::recommended_agent_resolves(r, Some("AGENT-001")));
    assert!(!invariants::recommended_agent_resolves(
        r,
        Some("AGENT-XXX")
    ));
}

#[test]
fn spec_done_accepts_checked_criteria_with_implementation_evidence_and_other_unchecked_lists() {
    let d = tempdir().unwrap();
    let r = d.path();
    write(
        r,
        ".lmbrain/specs/review/SPEC-001-real-shape.md",
        "---\nid: SPEC-001\nstatus: review\n---\n\n## Acceptance criteria\n- [x] The actual acceptance criterion is met.\n\n## Implementation evidence\n### Changes made\nImplemented the requested behavior.\n\n### Handoff status\n- [ ] Ready for Project Lead review\n",
    );
    write(
        r,
        ".lmbrain/reviews/accepted/REVIEW-001.md",
        "---\nid: REVIEW-001\nspec: SPEC-001\nstatus: accepted\n---\n",
    );

    let result = transition(
        r,
        ".lmbrain/specs/review/SPEC-001-real-shape.md",
        "done",
        MutationOptions::default(),
    )
    .unwrap();

    assert_eq!(result.status, "done");
    assert!(
        result
            .path
            .ends_with(".lmbrain/specs/done/SPEC-001-real-shape.md"),
        "unexpected path {:?}",
        result.path
    );
}

#[test]
fn skill_creation_and_lifecycle_use_status_directories() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain/templates")).unwrap();
    write(
        r,
        ".lmbrain/templates/skill.md",
        "---\nid: SKILL-XXX\ntitle: Skill\nstatus: proposed\ncreated: YYYY-MM-DD\nupdated: YYYY-MM-DD\ntags: []\nlinks: []\n---\n# Skill\n",
    );

    let created = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Skill,
            title: "Build and Test".into(),
            status: None,
            fields: vec![("kind".into(), "verification".into())],
        },
    )
    .unwrap();
    assert_eq!(created.id, "SKILL-001");
    assert!(
        created
            .path
            .ends_with("skills/proposed/SKILL-001-build-and-test.md"),
        "unexpected path {:?}",
        created.path
    );

    let activated = transition(r, &created.path, "active", MutationOptions::default()).unwrap();
    assert!(activated
        .path
        .ends_with("skills/active/SKILL-001-build-and-test.md"));

    let retired = transition(r, &activated.path, "retired", MutationOptions::default()).unwrap();
    assert!(retired
        .path
        .ends_with("skills/retired/SKILL-001-build-and-test.md"));
}

#[test]
fn spec_context_includes_applicable_active_skills() {
    let d = tempdir().unwrap();
    let r = d.path();
    write(
        r,
        ".lmbrain/specs/ready/SPEC-101.md",
        "---\nid: SPEC-101\ntitle: Skill Context\nstatus: ready\nrecommended_agent: AGENT-IMPL\nskills: [SKILL-001]\ntags: [test]\n---\n\n## Acceptance criteria\n- [ ] Works\n",
    );
    write(
        r,
        ".lmbrain/agents/profiles/AGENT-IMPL.md",
        "---\nid: AGENT-IMPL\ntitle: Implementer\nstatus: active\nskills: [SKILL-002]\n---\nBody",
    );
    write(
        r,
        ".lmbrain/skills/active/SKILL-001.md",
        "---\nid: SKILL-001\ntitle: Build and test\nstatus: active\nkind: verification\nrisk: medium\ncommands: [cargo test --workspace]\nrequires_operator_approval: true\n---\nBody",
    );
    write(
        r,
        ".lmbrain/skills/proposed/SKILL-002.md",
        "---\nid: SKILL-002\ntitle: Proposed only\nstatus: proposed\nkind: test\n---\nBody",
    );

    let ctx = build_spec_context(r, "SPEC-101").unwrap();
    assert_eq!(ctx.applicable_skills.len(), 1);
    assert_eq!(ctx.applicable_skills[0].id, "SKILL-001");
    assert!(ctx.markdown.contains("Build and test"));
    assert!(ctx.markdown.contains("operator approval required"));
    assert!(!ctx.markdown.contains("Proposed only"));
}

#[test]
fn creation_allocates_progressive_ids_and_keeps_flat_artifacts_flat() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain")).unwrap();
    write(
        r,
        ".lmbrain/agents/profiles/legacy.md",
        "---\nid: AGENT-ALPHA\nstatus: active\n---\n",
    );
    write(
        r,
        ".lmbrain/agents/profiles/AGENT-007.md",
        "---\nid: AGENT-007\nstatus: active\n---\n",
    );
    let result = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Agent,
            title: "New Agent".into(),
            status: None,
            fields: vec![],
        },
    )
    .unwrap();
    assert_eq!(result.id, "AGENT-008");
    assert!(
        result
            .path
            .ends_with("agents/profiles/AGENT-008-new-agent.md"),
        "unexpected path {:?}",
        result.path
    );
}

#[test]
fn agent_mnemonic_name_setter_is_agent_only_and_audited() {
    let d = tempdir().unwrap();
    let r = d.path();
    write(
        r,
        ".lmbrain/agents/profiles/AGENT-001.md",
        "---\nid: AGENT-001\ntitle: Specialist\nstatus: active\n---\nBody",
    );
    write(
        r,
        ".lmbrain/specs/ready/SPEC-001.md",
        "---\nid: SPEC-001\ntitle: Spec\nstatus: ready\n---\nBody",
    );

    let result = set_agent_mnemonic_name(
        r,
        ".lmbrain/agents/profiles/AGENT-001.md",
        "Ada Checklist",
        MutationOptions::default(),
    )
    .unwrap();
    let out = fs::read_to_string(result.path).unwrap();
    assert!(out.contains("mnemonic_name: \"Ada Checklist\""));
    assert!(out.contains("action: \"set mnemonic_name\""));

    let not_agent = set_agent_mnemonic_name(
        r,
        ".lmbrain/specs/ready/SPEC-001.md",
        "Spec Wrangler",
        MutationOptions::default(),
    );
    assert!(not_agent.is_err());
}

#[test]
fn spec_create_defaults_to_backlog() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain")).unwrap();
    let result = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Spec,
            title: "New Spec".into(),
            status: None,
            fields: vec![],
        },
    )
    .unwrap();
    assert_eq!(result.status, "backlog");
    assert!(
        result.path.ends_with("specs/backlog/SPEC-001-new-spec.md"),
        "unexpected path {:?}",
        result.path
    );
}

#[test]
fn review_create_starts_pending_with_one_submitted_event() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain")).unwrap();
    let result = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Review,
            title: "Review of SPEC-001".into(),
            status: None,
            fields: vec![
                ("spec".into(), "SPEC-001".into()),
                ("implementation_agent".into(), "AGENT-002".into()),
            ],
        },
    )
    .unwrap();
    let document = Document::parse(&fs::read_to_string(result.path).unwrap()).unwrap();
    let history = lmbrain_core::parse_review_event_history(&document);

    assert_eq!(history.warnings, Vec::<String>::new());
    assert_eq!(history.events.len(), 1);
    assert_eq!(history.events[0].action, "submitted");
    assert_eq!(history.events[0].from_status, "none");
    assert_eq!(history.events[0].to_status, "pending");
    assert_eq!(
        history.events[0].implementation_agent.as_deref(),
        Some("AGENT-002")
    );
    assert_eq!(
        document.value("finding_taxonomy_version").as_deref(),
        Some("1")
    );
}

#[test]
fn new_reviews_require_canonical_finding_categories() {
    let create_review = |root: &std::path::Path, category: &str| {
        create(
            root,
            CreateRequest {
                kind: ArtifactKind::Review,
                title: format!("Review with {category}"),
                status: None,
                fields: vec![("finding_categories".into(), format!("[{category}]"))],
            },
        )
    };

    let canonical = tempdir().unwrap();
    fs::create_dir_all(canonical.path().join(".lmbrain")).unwrap();
    assert!(create_review(canonical.path(), "verification-integrity").is_ok());

    for category in ["evidence-integrity", "project-specific"] {
        let invalid = tempdir().unwrap();
        fs::create_dir_all(invalid.path().join(".lmbrain")).unwrap();
        let result = create_review(invalid.path(), category);
        assert!(
            result.is_err(),
            "{category} should fail new-write validation"
        );
        assert!(!invalid.path().join(".lmbrain/reviews").exists());
    }
}

#[test]
fn review_context_exposes_event_history_and_legacy_uncertainty() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain")).unwrap();
    write(
        r,
        ".lmbrain/specs/review/SPEC-001.md",
        "---\nid: SPEC-001\ntitle: Test spec\nstatus: review\n---\n\n## Acceptance criteria\n- [x] Complete\n\n## Implementation evidence\nImplemented and tested.\n",
    );
    create(
        r,
        CreateRequest {
            kind: ArtifactKind::Review,
            title: "Typed review".into(),
            status: None,
            fields: vec![("spec".into(), "SPEC-001".into())],
        },
    )
    .unwrap();
    write(
        r,
        ".lmbrain/reviews/accepted/REVIEW-LEGACY.md",
        "---\nid: REVIEW-LEGACY\ntitle: Legacy review\nstatus: accepted\nspec: SPEC-001\n---\n",
    );

    let context = build_review_context(r, "SPEC-001").unwrap();
    let typed = context
        .linked_reviews
        .iter()
        .find(|review| review.id == "REVIEW-001")
        .unwrap();
    assert_eq!(typed.events.len(), 1);
    assert!(typed.lifecycle_warnings.is_empty());
    assert!(context.markdown.contains("REVIEW-001-EVENT-001"));
    assert!(context
        .warnings
        .iter()
        .any(|warning| warning.contains("REVIEW-LEGACY") && warning.contains("unknown")));
}

#[test]
fn force_reason_is_required_and_audited() {
    let d = tempdir().unwrap();
    let r = d.path();
    let path = ".lmbrain/specs/review/SPEC-001.md";
    write(r,path,"---\nid: SPEC-001\nstatus: review\n---\n\n## Acceptance criteria\n- [x] Done\n\n## Evidence\nproof\n");
    // 'done' needs an accepted review; without force it fails, and force needs a reason.
    assert!(transition(
        r,
        path,
        "done",
        MutationOptions {
            force: false,
            reason: None
        }
    )
    .is_err());
    assert!(transition(
        r,
        path,
        "done",
        MutationOptions {
            force: true,
            reason: None
        }
    )
    .is_err());
    let result = transition(
        r,
        path,
        "done",
        MutationOptions {
            force: true,
            reason: Some("operator accepted without a formal review".into()),
        },
    )
    .unwrap();
    let out = fs::read_to_string(result.path).unwrap();
    assert!(out.contains("activity:"));
    assert!(!out.contains("## Mutation override"));
    assert!(out.contains("operator accepted without a formal review"));
    assert_eq!(
        Document::parse(&out)
            .unwrap()
            .object_array("mutation_overrides")
            .len(),
        1
    );
}

#[test]
fn spec_submit_requires_scoped_nonempty_verification_transcript() {
    let d = tempdir().unwrap();
    let root = d.path();
    let cases = [
        ("missing", "## Implementation evidence\n\n### Changes made\nDone\n"),
        ("misplaced", "## Other\n\n### Verification transcript\n\n```text\nok\n```\n\n## Implementation evidence\nDone\n"),
        ("empty", "## Implementation evidence\n\n### Verification transcript\n\n```text\n\n```\n"),
    ];
    for (name, body) in cases {
        let path = format!(".lmbrain/specs/working/SPEC-{name}.md");
        write(
            root,
            &path,
            &format!("---\nid: SPEC-{name}\nstatus: working\n---\n\n{body}"),
        );
        let error = transition(root, &path, "review", MutationOptions::default()).unwrap_err();
        assert!(
            error.to_string().contains("Verification transcript"),
            "{name}: {error}"
        );
    }

    let path = ".lmbrain/specs/working/SPEC-valid.md";
    write(root, path, "---\nid: SPEC-valid\nstatus: working\n---\n\n## Implementation evidence\n\n### Verification transcript\n\n```text\n$ cargo test\npassed\n```\n");
    let result = transition(root, path, "review", MutationOptions::default()).unwrap();
    assert_eq!(result.status, "review");
}

#[test]
fn spec_submit_force_bypass_requires_reason_and_is_audited() {
    let d = tempdir().unwrap();
    let path = ".lmbrain/specs/working/SPEC-override.md";
    write(
        d.path(),
        path,
        "---\nid: SPEC-override\nstatus: working\n---\n\n## Implementation evidence\n",
    );
    assert!(transition(
        d.path(),
        path,
        "review",
        MutationOptions {
            force: true,
            reason: None
        }
    )
    .is_err());
    let result = transition(
        d.path(),
        path,
        "review",
        MutationOptions {
            force: true,
            reason: Some("operator accepts unavailable platform gate".into()),
        },
    )
    .unwrap();
    let output = fs::read_to_string(result.path).unwrap();
    assert!(!output.contains("## Mutation override"));
    assert!(output.contains("operator accepts unavailable platform gate"));
    assert_eq!(
        Document::parse(&output)
            .unwrap()
            .object_array("mutation_overrides")
            .len(),
        1
    );
}

fn snapshot(dir: &std::path::Path) -> Vec<String> {
    let mut entries = Vec::new();
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        if let Ok(read) = fs::read_dir(dir) {
            for entry in read.flatten() {
                let path = entry.path();
                out.push(path.strip_prefix(base).unwrap().display().to_string());
                if path.is_dir() {
                    walk(&path, base, out);
                }
            }
        }
    }
    walk(dir, dir, &mut entries);
    entries.sort();
    entries
}

#[test]
fn creation_status_allowlist_is_enforced_per_kind() {
    let accepted = &[
        (ArtifactKind::Spec, "backlog"),
        (ArtifactKind::Review, "pending"),
        (ArtifactKind::Adr, "proposed"),
        (ArtifactKind::Agent, "proposed"),
        (ArtifactKind::AgentProposal, "proposed"),
        (ArtifactKind::Mcp, "specified"),
        (ArtifactKind::McpProposal, "proposed"),
        (ArtifactKind::Handoff, "ready"),
        (ArtifactKind::Skill, "proposed"),
    ];
    for &(kind, status) in accepted {
        let d = tempdir().unwrap();
        fs::create_dir_all(d.path().join(".lmbrain")).unwrap();
        let result = create(
            d.path(),
            CreateRequest {
                kind,
                title: "Allowed".into(),
                status: Some(status.into()),
                fields: vec![],
            },
        );
        assert!(result.is_ok(), "{kind:?} '{status}' rejected: {result:?}");
    }

    let rejected = &[
        (ArtifactKind::Spec, "ready"),
        (ArtifactKind::Spec, "done"),
        (ArtifactKind::Review, "accepted"),
        (ArtifactKind::Adr, "accepted"),
        (ArtifactKind::Agent, "active"),
        (ArtifactKind::Skill, "active"),
        (ArtifactKind::Handoff, "consumed"),
        (ArtifactKind::Spec, "../escape"),
        (ArtifactKind::Spec, "a/b"),
        (ArtifactKind::Spec, r"a\b"),
        (ArtifactKind::Spec, "C:/tmp"),
        (ArtifactKind::Spec, ""),
        (ArtifactKind::Spec, "unknown-status"),
    ];
    for &(kind, status) in rejected {
        let d = tempdir().unwrap();
        fs::create_dir_all(d.path().join(".lmbrain")).unwrap();
        let before = snapshot(d.path());
        let error = create(
            d.path(),
            CreateRequest {
                kind,
                title: "Rejected".into(),
                status: Some(status.into()),
                fields: vec![],
            },
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                lmbrain_core::TransitionError::InvalidCreationStatus { .. }
            ),
            "{kind:?} '{status}': unexpected error {error}"
        );
        assert_eq!(
            before,
            snapshot(d.path()),
            "{kind:?} '{status}' left filesystem residue"
        );
    }
}

#[test]
fn create_rejects_reserved_fields_and_injection_without_residue() {
    let reserved = [
        "id", "Id", " status", "created", "updated", "title", "activity",
    ];
    for key in reserved {
        let d = tempdir().unwrap();
        fs::create_dir_all(d.path().join(".lmbrain")).unwrap();
        let before = snapshot(d.path());
        let error = create(
            d.path(),
            CreateRequest {
                kind: ArtifactKind::Spec,
                title: "Reserved".into(),
                status: None,
                fields: vec![(key.into(), "SPEC-999".into())],
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, lmbrain_core::TransitionError::ReservedField(_)),
            "'{key}': unexpected error {error}"
        );
        assert_eq!(before, snapshot(d.path()), "'{key}' left residue");
    }

    let invalid = [
        ("bad key", "value"),
        ("1leading", "value"),
        ("key:colon", "value"),
        ("spec", "value\nid: SPEC-999"),
        ("spec", "value\r\nstatus: done"),
    ];
    for (key, value) in invalid {
        let d = tempdir().unwrap();
        fs::create_dir_all(d.path().join(".lmbrain")).unwrap();
        let error = create(
            d.path(),
            CreateRequest {
                kind: ArtifactKind::Spec,
                title: "Injection".into(),
                status: None,
                fields: vec![(key.into(), value.into())],
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, lmbrain_core::TransitionError::InvalidField(_)),
            "('{key}', {value:?}): unexpected error {error}"
        );
    }

    // Legitimate domain fields still work, including 'kind' (a skill field,
    // distinct from the artifact kind carried by the ID prefix).
    let d = tempdir().unwrap();
    fs::create_dir_all(d.path().join(".lmbrain")).unwrap();
    let result = create(
        d.path(),
        CreateRequest {
            kind: ArtifactKind::Skill,
            title: "Valid".into(),
            status: None,
            fields: vec![
                ("kind".into(), "verification".into()),
                ("recommended_agent".into(), "AGENT-001".into()),
            ],
        },
    )
    .unwrap();
    let out = fs::read_to_string(result.path).unwrap();
    assert!(out.contains("kind: verification"));
    assert!(out.contains("recommended_agent: AGENT-001"));
}

#[test]
fn create_cannot_produce_a_second_ready_handoff() {
    let d = tempdir().unwrap();
    let r = d.path();
    fs::create_dir_all(r.join(".lmbrain")).unwrap();
    let first = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Handoff,
            title: "First".into(),
            status: None,
            fields: vec![],
        },
    )
    .unwrap();
    assert_eq!(first.status, "ready");

    let error = create(
        r,
        CreateRequest {
            kind: ArtifactKind::Handoff,
            title: "Second".into(),
            status: None,
            fields: vec![],
        },
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("only one ready handoff"),
        "unexpected error: {error}"
    );
    let remaining = fs::read_dir(r.join(".lmbrain/handoffs/active"))
        .unwrap()
        .count();
    assert_eq!(remaining, 1, "the failed create left an artifact behind");
}

#[test]
fn frontmatter_round_trip_keeps_comments_and_order() {
    let mut document = Document::parse(
        "---\n# comment\nid: SPEC-1\nstatus: backlog\nunknown: value # inline\n---\nbody\n",
    )
    .unwrap();
    document.set("status", "working");
    let out = document.render();
    assert!(out.contains("# comment"));
    assert!(out.contains("unknown: value # inline"));
}

#[test]
fn before_done_reports_every_authority_blocker_and_force_audits_them() {
    let directory = tempdir().unwrap();
    let relative = ".lmbrain/specs/review/SPEC-001.md";
    let source = "---\nid: SPEC-001\nstatus: review\n---\n\n## Acceptance criteria\n- [x] Complete\n\n## Implementation evidence\nproof\n\n## Required verification\n- [ ] LEAD-CHECK | kind=manual | owner=lead | phase=before-done | evidence=artifact | Independent review\n- [x] HUMAN-PLAY | kind=operator | owner=operator | phase=before-done | evidence=observation | Exercise the app\n";
    write(directory.path(), relative, source);

    let error = transition(
        directory.path(),
        relative,
        "done",
        MutationOptions::default(),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("LEAD-CHECK (owner=lead)"));
    assert!(error.contains("HUMAN-PLAY (owner=operator)"));
    assert_eq!(
        fs::read_to_string(directory.path().join(relative)).unwrap(),
        source
    );

    transition(
        directory.path(),
        relative,
        "done",
        MutationOptions {
            force: true,
            reason: Some("Operator-authorized emergency closeout".into()),
        },
    )
    .unwrap();
    let done =
        fs::read_to_string(directory.path().join(".lmbrain/specs/done/SPEC-001.md")).unwrap();
    assert!(done.contains("Operator-authorized emergency closeout"));
    assert!(done.contains("LEAD-CHECK (owner=lead)"));
    assert!(done.contains("HUMAN-PLAY (owner=operator)"));
    let document = Document::parse(&done).unwrap();
    let overrides = document.object_array("mutation_overrides");
    assert_eq!(overrides.len(), 1);
    assert_eq!(
        overrides[0]
            .get("actor_role")
            .and_then(serde_json::Value::as_str),
        Some("project-lead")
    );
    assert!(overrides[0]
        .get("unmet_invariant")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|invariant| {
            invariant.contains("LEAD-CHECK") && invariant.contains("HUMAN-PLAY")
        }));
}

// ─── Governed spec metadata (issues #49 and #64) ──────────────────

fn spec_fixture(root: &std::path::Path, relative: &str, extra: &str) {
    write(
        root,
        relative,
        &format!(
            "---\nid: SPEC-200\ntitle: Metadata subject\nstatus: backlog\nmilestone: 4.0.0\narea: rust\npriority: high\ntags: []\nlinks: []\nactivity: []\ncreated: 2026-07-01\nupdated: 2026-07-01\n{extra}---\n# Metadata subject\n"
        ),
    );
}

#[test]
fn setting_tags_normalizes_and_rejects_field_restating_values() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/backlog/SPEC-200.md";
    spec_fixture(dir.path(), spec, "");

    let error = lmbrain_core::set_spec_tags(
        dir.path(),
        spec,
        &["4.0.0".into(), "wiki".into()],
        MutationOptions::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("milestone"));
    // The rejected mutation leaves the artifact untouched.
    let untouched = Document::parse(&fs::read_to_string(dir.path().join(spec)).unwrap()).unwrap();
    assert!(untouched.string_array("tags").is_empty());

    let result = lmbrain_core::set_spec_tags(
        dir.path(),
        spec,
        &["  Kit_Feedback ".into(), "#Docs".into(), "wiki".into()],
        MutationOptions::default(),
    )
    .unwrap();
    let document = Document::parse(&fs::read_to_string(&result.path).unwrap()).unwrap();
    assert_eq!(
        document.string_array("tags"),
        vec![
            "kit-feedback".to_string(),
            "documentation".to_string(),
            "wiki".to_string()
        ]
    );
}

#[test]
fn forcing_invalid_tags_records_an_audited_reason() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/backlog/SPEC-200.md";
    spec_fixture(dir.path(), spec, "");

    let result = lmbrain_core::set_spec_tags(
        dir.path(),
        spec,
        &["4.0.0".into(), "wiki".into()],
        MutationOptions {
            force: true,
            reason: Some("importing a legacy planning tag".into()),
        },
    )
    .unwrap();
    assert!(result.forced);
    let document = Document::parse(&fs::read_to_string(&result.path).unwrap()).unwrap();
    // The offending value is dropped, not written, but the override is recorded.
    assert_eq!(document.string_array("tags"), vec!["wiki".to_string()]);
    assert!(!document.object_array("mutation_overrides").is_empty());
}

#[test]
fn effort_defaults_the_thinking_level_from_the_tier() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/backlog/SPEC-200.md";
    spec_fixture(dir.path(), spec, "");

    let result =
        lmbrain_core::set_spec_effort(dir.path(), spec, "Sol", None, MutationOptions::default())
            .unwrap();
    let document = Document::parse(&fs::read_to_string(&result.path).unwrap()).unwrap();
    assert_eq!(document.value("capability_tier").as_deref(), Some("sol"));
    assert_eq!(
        document.value("thinking_level").as_deref(),
        Some("extended")
    );
}

#[test]
fn effort_rejects_unknown_and_constrained_combinations() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/backlog/SPEC-200.md";
    spec_fixture(dir.path(), spec, "");

    let unknown =
        lmbrain_core::set_spec_effort(dir.path(), spec, "jupiter", None, MutationOptions::default())
            .unwrap_err();
    assert!(unknown.to_string().contains("unknown capability tier"));

    let constrained = lmbrain_core::set_spec_effort(
        dir.path(),
        spec,
        "sol",
        Some("minimal"),
        MutationOptions::default(),
    )
    .unwrap_err();
    assert!(constrained.to_string().contains("Sol"));

    // Forcing it keeps the value but records why the invariant was crossed.
    let forced = lmbrain_core::set_spec_effort(
        dir.path(),
        spec,
        "sol",
        Some("minimal"),
        MutationOptions {
            force: true,
            reason: Some("mechanical rename across layers".into()),
        },
    )
    .unwrap();
    let document = Document::parse(&fs::read_to_string(&forced.path).unwrap()).unwrap();
    assert_eq!(
        document.value("thinking_level").as_deref(),
        Some("minimal")
    );
    assert!(!document.object_array("mutation_overrides").is_empty());
}

#[test]
fn a_spec_cannot_become_ready_without_a_valid_estimate() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/backlog/SPEC-200.md";
    spec_fixture(dir.path(), spec, "depends_on: []\n");

    let blocked = transition(dir.path(), spec, "ready", MutationOptions::default()).unwrap_err();
    assert!(blocked.to_string().contains("capability_tier"));

    lmbrain_core::set_spec_effort(dir.path(), spec, "terra", None, MutationOptions::default())
        .unwrap();
    let ready = transition(dir.path(), spec, "ready", MutationOptions::default()).unwrap();
    assert_eq!(ready.status, "ready");
}

#[test]
fn effort_observations_are_append_only_and_never_rewrite_the_recommendation() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/working/SPEC-200.md";
    spec_fixture(
        dir.path(),
        spec,
        "capability_tier: luna\nthinking_level: minimal\neffort_observations: []\n",
    );

    lmbrain_core::record_effort_observation(
        dir.path(),
        spec,
        "sol",
        "AGENT-IMPL",
        "Needed contract changes the estimate did not anticipate",
        MutationOptions::default(),
    )
    .unwrap();
    let result = lmbrain_core::record_effort_observation(
        dir.path(),
        spec,
        "terra",
        "AGENT-IMPL",
        "Second pass was smaller",
        MutationOptions::default(),
    )
    .unwrap();

    let document = Document::parse(&fs::read_to_string(&result.path).unwrap()).unwrap();
    let observations = document.object_array("effort_observations");
    assert_eq!(observations.len(), 2);
    assert_eq!(
        observations[0].get("observed_tier").and_then(|v| v.as_str()),
        Some("sol")
    );
    assert_eq!(
        observations[0]
            .get("recommended_tier")
            .and_then(|v| v.as_str()),
        Some("luna")
    );
    // The Lead-owned recommendation is untouched by specialist feedback.
    assert_eq!(document.value("capability_tier").as_deref(), Some("luna"));
    assert_eq!(document.value("thinking_level").as_deref(), Some("minimal"));
}

#[test]
fn effort_observations_require_an_actor_and_a_note() {
    let dir = tempdir().unwrap();
    let spec = ".lmbrain/specs/working/SPEC-200.md";
    spec_fixture(dir.path(), spec, "effort_observations: []\n");

    assert!(lmbrain_core::record_effort_observation(
        dir.path(),
        spec,
        "terra",
        "   ",
        "note",
        MutationOptions::default()
    )
    .is_err());
    assert!(lmbrain_core::record_effort_observation(
        dir.path(),
        spec,
        "terra",
        "AGENT-IMPL",
        "  ",
        MutationOptions::default()
    )
    .is_err());
}

#[test]
fn governed_metadata_verbs_reject_non_spec_artifacts() {
    let dir = tempdir().unwrap();
    let adr = ".lmbrain/decisions/ADR-010.md";
    write(
        dir.path(),
        adr,
        "---\nid: ADR-010\ntitle: A decision\nstatus: accepted\ntags: []\n---\n# A decision\n",
    );
    assert!(lmbrain_core::set_spec_tags(
        dir.path(),
        adr,
        &["wiki".into()],
        MutationOptions::default()
    )
    .is_err());
}

fn decision(id: &str, status: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: Decision {id}\nstatus: {status}\ndecision_date: 2026-07-01\ndecider: user\nsupersedes: []\nsuperseded_by: []\nlinks: []\ntags: []\nactivity: []\nupdated: 2026-07-01\n---\n# Decision {id}\n"
    )
}

/// The two known workspace inconsistencies (ADR-010/009 and ADR-014/013) exist
/// because the relationship was written on one side only. The verb writes both.
#[test]
fn supersession_writes_both_sides() {
    let dir = tempdir().unwrap();
    let successor = ".lmbrain/decisions/ADR-010.md";
    write(dir.path(), successor, &decision("ADR-010", "accepted"));
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-009.md",
        &decision("ADR-009", "accepted"),
    );

    let result = supersede_adr(
        dir.path(),
        successor,
        "ADR-009",
        MutationOptions::default(),
    )
    .unwrap();
    assert_eq!(result.id, "ADR-010");

    let new_side = Document::parse(&fs::read_to_string(dir.path().join(successor)).unwrap()).unwrap();
    assert_eq!(new_side.string_array("supersedes"), vec!["ADR-009"]);

    let old_side = Document::parse(
        &fs::read_to_string(dir.path().join(".lmbrain/decisions/ADR-009.md")).unwrap(),
    )
    .unwrap();
    assert_eq!(old_side.value("status").unwrap(), "superseded");
    assert_eq!(old_side.string_array("superseded_by"), vec!["ADR-010"]);
}

/// Re-running the verb is the repair path for a half-written relationship, so
/// it must be safe to run against a pair that is already consistent.
#[test]
fn supersession_is_idempotent() {
    let dir = tempdir().unwrap();
    let successor = ".lmbrain/decisions/ADR-010.md";
    write(dir.path(), successor, &decision("ADR-010", "accepted"));
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-009.md",
        &decision("ADR-009", "accepted"),
    );

    supersede_adr(dir.path(), successor, "ADR-009", MutationOptions::default()).unwrap();
    let after_first = fs::read_to_string(dir.path().join(successor)).unwrap();
    supersede_adr(dir.path(), successor, "ADR-009", MutationOptions::default()).unwrap();
    assert_eq!(fs::read_to_string(dir.path().join(successor)).unwrap(), after_first);
}

/// ADR-014 is `proposed` yet declares `supersedes: [ADR-013]`. Declaring the
/// intent is fine; enacting it before acceptance is not.
#[test]
fn a_proposal_cannot_retire_a_decision() {
    let dir = tempdir().unwrap();
    let successor = ".lmbrain/decisions/ADR-014.md";
    write(dir.path(), successor, &decision("ADR-014", "proposed"));
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-013.md",
        &decision("ADR-013", "accepted"),
    );

    let error = supersede_adr(dir.path(), successor, "ADR-013", MutationOptions::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("must be accepted"), "{error}");

    let untouched = Document::parse(
        &fs::read_to_string(dir.path().join(".lmbrain/decisions/ADR-013.md")).unwrap(),
    )
    .unwrap();
    assert_eq!(untouched.value("status").unwrap(), "accepted");
}

#[test]
fn supersession_rejects_self_reference_and_unknown_targets() {
    let dir = tempdir().unwrap();
    let successor = ".lmbrain/decisions/ADR-010.md";
    write(dir.path(), successor, &decision("ADR-010", "accepted"));

    let self_error = supersede_adr(dir.path(), successor, "ADR-010", MutationOptions::default())
        .unwrap_err()
        .to_string();
    assert!(self_error.contains("cannot supersede itself"), "{self_error}");

    let missing = supersede_adr(dir.path(), successor, "ADR-404", MutationOptions::default())
        .unwrap_err()
        .to_string();
    assert!(missing.contains("does not exist"), "{missing}");

    let wrong_kind = supersede_adr(dir.path(), successor, "SPEC-001", MutationOptions::default())
        .unwrap_err()
        .to_string();
    assert!(wrong_kind.contains("not a decision ID"), "{wrong_kind}");
}

#[test]
fn forced_supersession_records_the_invariant_it_broke() {
    let dir = tempdir().unwrap();
    let successor = ".lmbrain/decisions/ADR-014.md";
    write(dir.path(), successor, &decision("ADR-014", "proposed"));
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-013.md",
        &decision("ADR-013", "accepted"),
    );

    supersede_adr(
        dir.path(),
        successor,
        "ADR-013",
        MutationOptions {
            force: true,
            reason: Some("Operator accepted the risk".into()),
        },
    )
    .unwrap();

    let document = Document::parse(&fs::read_to_string(dir.path().join(successor)).unwrap()).unwrap();
    let overrides = document.object_array("mutation_overrides");
    assert_eq!(overrides.len(), 1);
    assert!(
        overrides[0]
            .get("unmet_invariant")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value.contains("must be accepted")),
        "{overrides:?}"
    );
}

/// A one-sided claim is exactly what a crash between the two writes leaves
/// behind, and what the workspace contains today. It must be visible.
#[test]
fn diagnostics_report_a_one_sided_supersession() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-010.md",
        &decision("ADR-010", "accepted").replace("supersedes: []", "supersedes: [ADR-009]"),
    );
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-009.md",
        &decision("ADR-009", "accepted"),
    );

    let diagnostics = build_diagnostics(dir.path());
    let dangling: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "dangling-supersession")
        .collect();
    assert_eq!(dangling.len(), 1, "{diagnostics:?}");
    assert_eq!(dangling[0].artifact_id.as_deref(), Some("ADR-009"));

    // Running the verb clears it.
    supersede_adr(
        dir.path(),
        ".lmbrain/decisions/ADR-010.md",
        "ADR-009",
        MutationOptions::default(),
    )
    .unwrap();
    assert!(build_diagnostics(dir.path())
        .iter()
        .all(|diagnostic| diagnostic.code != "dangling-supersession"));
}

#[test]
fn diagnostics_stay_quiet_on_a_proposals_pending_claim() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-014.md",
        &decision("ADR-014", "proposed").replace("supersedes: []", "supersedes: [ADR-013]"),
    );
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-013.md",
        &decision("ADR-013", "accepted"),
    );

    assert!(build_diagnostics(dir.path())
        .iter()
        .all(|diagnostic| diagnostic.code != "dangling-supersession"));
}

#[test]
fn diagnostics_report_an_unresolvable_successor() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        ".lmbrain/decisions/ADR-009.md",
        &decision("ADR-009", "superseded").replace("superseded_by: []", "superseded_by: [ADR-404]"),
    );

    let diagnostics = build_diagnostics(dir.path());
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "supersession-not-mutual"),
        "{diagnostics:?}"
    );
}

#[test]
fn governed_repair_merges_duplicate_activity_and_audits_the_reason() {
    let dir = tempdir().unwrap();
    let relative = ".lmbrain/specs/backlog/SPEC-005.md";
    // Field corruption written by 4.0.1: duplicate top-level activity blocks.
    write(
        dir.path(),
        relative,
        "---\nid: SPEC-005\ntitle: Corrupted\nstatus: backlog\ntags: []\nactivity:\n  - date: 2026-08-06\n    action: \"created\"\nactivity:\n  - date: 2026-08-06\n    action: \"set effort\"\n---\n# Corrupted\n",
    );

    // Empty reason is refused before any read or write.
    assert!(lmbrain_core::repair_artifact_frontmatter(dir.path(), relative, "  ").is_err());

    let result =
        lmbrain_core::repair_artifact_frontmatter(dir.path(), relative, "operator-authorized")
            .unwrap();
    assert_eq!(result.id, "SPEC-005");
    assert_eq!(result.merged_keys, vec!["activity".to_string()]);

    let source = fs::read_to_string(dir.path().join(relative)).unwrap();
    assert_eq!(source.matches("\nactivity:").count(), 1);
    let document = Document::parse(&source).unwrap();
    let activity = document.object_array("activity");
    assert_eq!(activity.len(), 3);
    let last = activity.last().unwrap();
    let action = last.get("action").and_then(|value| value.as_str()).unwrap();
    assert!(action.contains("repaired duplicate frontmatter keys"));
    assert!(action.contains("operator-authorized"));

    // A healthy artifact reports nothing to repair.
    assert!(
        lmbrain_core::repair_artifact_frontmatter(dir.path(), relative, "second pass").is_err()
    );
}

#[test]
fn concurrent_governed_setters_serialize_and_keep_one_activity_key() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        ".lmbrain/agents/profiles/AGENT-IMPL.md",
        "---\nid: AGENT-IMPL\nstatus: active\n---\n",
    );
    let created = create(
        dir.path(),
        CreateRequest {
            kind: ArtifactKind::Spec,
            title: "Concurrency fixture".into(),
            status: None,
            fields: Vec::new(),
        },
    )
    .unwrap();
    let relative = created
        .path
        .strip_prefix(dir.path())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");

    // The 4.0.1 field failure mode: concurrent governed setters against one
    // artifact. Each mutation must serialize under the artifact lock and the
    // artifact must stay parseable with a single top-level activity key.
    let root = dir.path().to_path_buf();
    let handles: Vec<std::thread::JoinHandle<()>> = vec![
        {
            let root = root.clone();
            let relative = relative.clone();
            std::thread::spawn(move || {
                lmbrain_core::set_spec_effort(
                    &root,
                    &relative,
                    "terra",
                    Some("standard"),
                    MutationOptions::default(),
                )
                .unwrap();
            })
        },
        {
            let root = root.clone();
            let relative = relative.clone();
            std::thread::spawn(move || {
                lmbrain_core::transitions::set_recommended_agent(
                    &root,
                    &relative,
                    "AGENT-IMPL",
                    MutationOptions::default(),
                )
                .unwrap();
            })
        },
        {
            let root = root.clone();
            let relative = relative.clone();
            std::thread::spawn(move || {
                lmbrain_core::record_effort_observation(
                    &root,
                    &relative,
                    "terra",
                    "AGENT-IMPL",
                    "matched the estimate",
                    MutationOptions::default(),
                )
                .unwrap();
            })
        },
    ];
    for handle in handles {
        handle.join().unwrap();
    }

    let source = fs::read_to_string(created.path).unwrap();
    assert_eq!(source.matches("\nactivity:").count(), 1, "{source}");
    let document = Document::parse(&source).unwrap();
    // created + three governed mutations, all present in one list.
    assert_eq!(document.object_array("activity").len(), 4);
    assert_eq!(document.value("capability_tier").as_deref(), Some("terra"));
    assert_eq!(
        document.value("recommended_agent").as_deref(),
        Some("AGENT-IMPL")
    );
    assert_eq!(document.object_array("effort_observations").len(), 1);
}
