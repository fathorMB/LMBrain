use lmbrain_core::{
    context::build_spec_context,
    frontmatter::Document,
    invariants,
    transitions::{
        create, set_agent_mnemonic_name, transition, ArtifactKind, CreateRequest, MutationOptions,
    },
};
use std::fs;
use tempfile::tempdir;

fn write(root: &std::path::Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
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
    };
    let relative = if matches!(
        kind,
        ArtifactKind::Spec | ArtifactKind::Review | ArtifactKind::Skill
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
        let valid = transition(
            d.path(),
            &path,
            to,
            MutationOptions {
                force: true,
                reason: Some("matrix fixture bypasses cross-artifact setup".into()),
            },
        );
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
    assert!(out.contains("Mutation override"));
    assert!(out.contains("operator accepted without a formal review"));
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
    assert!(output.contains("Mutation override"));
    assert!(output.contains("operator accepts unavailable platform gate"));
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
    let reserved = ["id", "Id", " status", "created", "updated", "title", "activity"];
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
