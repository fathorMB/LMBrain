use lmbrain_core::{
    frontmatter::Document,
    invariants,
    transitions::{create, transition, ArtifactKind, CreateRequest, MutationOptions},
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
    };
    let relative = if matches!(kind, ArtifactKind::Spec | ArtifactKind::Review) {
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
    assert!(!invariants::criteria_complete_with_evidence(
        "## Evidence\nproof"
    ));
    assert!(!invariants::criteria_complete_with_evidence(
        "## Acceptance criteria\n- [ ] Pending\n## Evidence\nproof"
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
