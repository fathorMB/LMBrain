use std::path::{Path, PathBuf};
use serde_json::{json, Value};

use lmbrain_core::context::{
    build_branching_strategy_digest, build_project_digest, build_review_context, build_spec_context,
};
use lmbrain_core::harness_environment::{
    apply_approved_harness_configuration, approve_harness_manifest,
    default_harness_approval_store_path, harness_approval_status, plan_harness_configuration,
    revoke_harness_approval,
};
use lmbrain_core::transitions::{
    create, record_effort_observation, record_review_event, repair_artifact_frontmatter,
    review_verdict, set_agent_mnemonic_name, set_recommended_agent,
    set_review_implementation_agent, set_spec_effort, set_spec_tags, supersede_adr, transition,
    ArtifactKind, CreateRequest, MutationOptions,
};
use lmbrain_core::{
    accept_debt_risk, apply_improvement_proposal, approve_verification_manifest,
    attest_spec_requirement, attest_spec_requirement_delegated, build_agent_improvement_signals,
    build_review_migration_preview, canonical_manifest_digest,
    canonical_verification_manifest_digest, capture_dream, create_debt,
    create_improvement_proposal, debt_candidates, debt_context, debt_migrate,
    debt_migration_preview, default_verification_approval_path, defer_debt,
    discover_verification_manifest, execute_spec_verification, kit_migrate, kit_migration_preview,
    load_branching_strategy, load_harness_manifest, load_verification_manifest, park_spec, parse_harness_manifest,
    plan_debt, read_kit_feedback, record_kit_feedback, record_kit_feedback_resolution, reopen_debt,
    resolve_debt, rollback_verification_manifest, set_branching_strategy, set_harness_manifest,
    set_spec_dependencies, set_spec_verification_gates, set_verification_manifest,
    spec_dependency_candidates, spec_dependency_context, supersede_debt,
    validate_verification_manifest_source, verification_manifest_status, AttestationDelegation,
    BranchingStrategy, DebtCreateInput, DreamCreateInput, HarnessManifestError,
    ImprovementProposalRequest, KitFeedbackInput, KitFeedbackResolutionInput, ReviewEventInput,
    SpecParkingInput, VerificationManifest, VerificationManifestState,
};

pub struct ToolSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub schema_fn: fn() -> Value,
    pub handler: fn(&Path, &Value) -> Result<Value, String>,
}

// Parameter extraction helpers
pub fn req_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{key} missing"))
}

pub fn opt_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn resolve_bundled_kit_path(root: &Path, args: &Value) -> Result<PathBuf, String> {
    if let Some(path) = opt_str(args, "bundled_kit_path") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = std::env::var("LMBRAIN_BUNDLED_KIT") {
        return Ok(PathBuf::from(path));
    }
    let adjacent_kit = root.join("kit");
    if adjacent_kit.exists() {
        return Ok(adjacent_kit);
    }
    Err("no bundled kit is configured; supply bundled_kit_path or set LMBRAIN_BUNDLED_KIT".into())
}

pub fn opt_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}

pub fn req_str_vec(args: &Value, key: &str) -> Result<Vec<String>, String> {
    serde_json::from_value(
        args.get(key)
            .cloned()
            .ok_or_else(|| format!("{key} missing"))?,
    )
    .map_err(|error| format!("{key} must be an array of strings: {error}"))
}

pub fn opts(args: &Value) -> MutationOptions {
    MutationOptions {
        force: opt_bool(args, "force").unwrap_or(false),
        reason: opt_str(args, "reason").map(str::to_owned),
    }
}

pub fn text(value: Value) -> Value {
    json!({"content":[{"type":"text","text":value.to_string()}]})
}

fn review_event_input(args: &Value, actor_role: &str) -> Result<ReviewEventInput, String> {
    let evidence_refs = match args.get("evidence_refs") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "evidence_refs must contain only strings".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err("evidence_refs must be an array of strings".into()),
    };
    Ok(ReviewEventInput {
        actor_role: actor_role.into(),
        reason: opt_str(args, "reason").unwrap_or_default().into(),
        evidence_refs,
        remediation_agent: opt_str(args, "remediation_agent").map(str::to_owned),
    })
}

fn candidate_manifest(args: &Value) -> Result<lmbrain_core::HarnessManifest, String> {
    let candidate = args.get("manifest").ok_or("manifest missing")?;
    parse_harness_manifest(&candidate.to_string()).map_err(|error| error.to_string())
}

fn mcp_server_command() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| format!("cannot resolve the MCP server executable: {error}"))
}

// Schemas
fn transition_schema() -> Value {
    json!({
        "type": "object",
        "required": ["path"],
        "properties": {
            "path": {"type": "string", "description": "Artifact path relative to repository root."},
            "force": {"type": "boolean", "default": false},
            "reason": {"type": "string", "description": "Required only when force is true."}
        },
        "additionalProperties": false
    })
}

fn review_verdict_schema(reason_required: bool) -> Value {
    let mut required = vec![json!("path")];
    if reason_required {
        required.push(json!("reason"));
    }
    json!({
        "type": "object",
        "required": required,
        "properties": {
            "path": {"type": "string", "description": "Review path relative to repository root."},
            "reason": {"type": "string", "description": "Verdict rationale; required for non-accepting verdicts."},
            "evidence_refs": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Artifact IDs or paths supporting the verdict."
            },
            "remediation_agent": {
                "type": "string",
                "description": "Optional agent expected to remediate the verdict."
            },
            "force": {"type": "boolean", "default": false}
        },
        "additionalProperties": false
    })
}

fn review_event_schema(remediation_agent_required: bool) -> Value {
    let mut required = vec![json!("path"), json!("reason")];
    if remediation_agent_required {
        required.push(json!("remediation_agent"));
    }
    json!({
        "type": "object",
        "required": required,
        "properties": {
            "path": {"type": "string", "description": "Review path relative to repository root."},
            "reason": {"type": "string", "description": "Required lifecycle-event rationale."},
            "evidence_refs": {"type": "array", "items": {"type": "string"}},
            "remediation_agent": {"type": "string"},
            "force": {"type": "boolean", "default": false}
        },
        "additionalProperties": false
    })
}

fn setter_schema(field: &str) -> Value {
    json!({
        "type": "object",
        "required": ["path", field],
        "properties": {
            "path": {"type":"string"},
            field: {"type":"string"},
            "force": {"type":"boolean","default":false},
            "reason": {"type":"string","description":"Required only when force is true."}
        },
        "additionalProperties": false
    })
}

fn debt_transition_schema(extras: &[(&str, Value)]) -> Value {
    let mut properties = serde_json::Map::from_iter([
        ("path".into(), json!({"type":"string"})),
        ("actor".into(), json!({"type":"string"})),
        ("rationale".into(), json!({"type":"string"})),
    ]);
    let mut required = vec![json!("path"), json!("actor"), json!("rationale")];
    for (name, schema) in extras {
        properties.insert((*name).into(), schema.clone());
        if schema.get("type") != Some(&json!(["string", "null"])) {
            required.push(json!(*name));
        }
    }
    json!({
        "type": "object",
        "required": required,
        "properties": properties,
        "additionalProperties": false
    })
}

macro_rules! transition_fn {
    ($fn_name:ident, $target_status:literal) => {
        fn $fn_name(root: &Path, args: &Value) -> Result<Value, String> {
            transition(root, req_str(args, "path")?, $target_status, opts(args))
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
    };
}

transition_fn!(handle_spec_ready, "ready");
transition_fn!(handle_spec_start, "working");
transition_fn!(handle_spec_submit, "review");
transition_fn!(handle_spec_done, "done");
transition_fn!(handle_spec_discard, "discarded");
transition_fn!(handle_adr_accept, "accepted");
transition_fn!(handle_adr_reject, "rejected");
transition_fn!(handle_agent_activate, "active");
transition_fn!(handle_agent_deactivate, "inactive");
transition_fn!(handle_agent_proposal_approve, "approved");
transition_fn!(handle_agent_proposal_reject, "rejected");
transition_fn!(handle_skill_activate, "active");
transition_fn!(handle_skill_retire, "retired");
transition_fn!(handle_handoff_consume, "consumed");
transition_fn!(handle_handoff_supersede, "superseded");
transition_fn!(handle_handoff_archive, "archived");

pub static TOOLS: &[ToolSpec] = &[
    // Spec transitions
    ToolSpec {
        name: "spec_ready",
        category: "Spec",
        description: "Approve a backlog spec to ready (on operator request).",
        schema_fn: transition_schema,
        handler: handle_spec_ready,
    },
    ToolSpec {
        name: "spec_start",
        category: "Spec",
        description: "Implementation specialist only: move an assigned ready spec to working when starting implementation.",
        schema_fn: transition_schema,
        handler: handle_spec_start,
    },
    ToolSpec {
        name: "spec_submit",
        category: "Spec",
        description: "Implementation specialist only: move a working spec to review when implementation is complete.",
        schema_fn: transition_schema,
        handler: handle_spec_submit,
    },
    ToolSpec {
        name: "spec_done",
        category: "Spec",
        description: "Project Lead on operator/review authority: mark a reviewed spec done after accepted review, checked criteria, and evidence.",
        schema_fn: transition_schema,
        handler: handle_spec_done,
    },
    ToolSpec {
        name: "spec_discard",
        category: "Spec",
        description: "Discard a spec (requires operator approval).",
        schema_fn: transition_schema,
        handler: handle_spec_discard,
    },
    ToolSpec {
        name: "adr_accept",
        category: "ADR",
        description: "Accept a proposed ADR (on operator request).",
        schema_fn: transition_schema,
        handler: handle_adr_accept,
    },
    ToolSpec {
        name: "adr_reject",
        category: "ADR",
        description: "Reject a proposed ADR (on operator request).",
        schema_fn: transition_schema,
        handler: handle_adr_reject,
    },
    ToolSpec {
        name: "agent_activate",
        category: "Agent",
        description: "Activate a proposed agent profile (on operator request).",
        schema_fn: transition_schema,
        handler: handle_agent_activate,
    },
    ToolSpec {
        name: "agent_deactivate",
        category: "Agent",
        description: "Deactivate an agent profile (on operator request).",
        schema_fn: transition_schema,
        handler: handle_agent_deactivate,
    },
    ToolSpec {
        name: "agent_proposal_approve",
        category: "Agent",
        description: "Approve an agent improvement proposal (on operator request).",
        schema_fn: transition_schema,
        handler: handle_agent_proposal_approve,
    },
    ToolSpec {
        name: "agent_proposal_reject",
        category: "Agent",
        description: "Reject an agent improvement proposal (on operator request).",
        schema_fn: transition_schema,
        handler: handle_agent_proposal_reject,
    },
    ToolSpec {
        name: "skill_activate",
        category: "Skill",
        description: "Activate a proposed project-scoped skill (on operator request).",
        schema_fn: transition_schema,
        handler: handle_skill_activate,
    },
    ToolSpec {
        name: "skill_retire",
        category: "Skill",
        description: "Retire a project-scoped skill that should no longer be recommended.",
        schema_fn: transition_schema,
        handler: handle_skill_retire,
    },
    ToolSpec {
        name: "handoff_consume",
        category: "Handoff",
        description: "Consume a ready session handoff (Project Lead only, after validation).",
        schema_fn: transition_schema,
        handler: handle_handoff_consume,
    },
    ToolSpec {
        name: "handoff_supersede",
        category: "Handoff",
        description: "Supersede a ready session handoff with a newer one.",
        schema_fn: transition_schema,
        handler: handle_handoff_supersede,
    },
    ToolSpec {
        name: "handoff_archive",
        category: "Handoff",
        description: "Archive/retire a session handoff.",
        schema_fn: transition_schema,
        handler: handle_handoff_archive,
    },

    // Review verdict tools
    ToolSpec {
        name: "review_accept",
        category: "Review",
        description: "Accept a review on explicit operator request and record the verdict event.",
        schema_fn: || review_verdict_schema(false),
        handler: |root, args| {
            review_verdict(
                root,
                req_str(args, "path")?,
                "accepted",
                review_event_input(args, "operator")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_changes_requested",
        category: "Review",
        description: "Project Lead: request changes with a reason and preserve the review verdict history.",
        schema_fn: || review_verdict_schema(true),
        handler: |root, args| {
            review_verdict(
                root,
                req_str(args, "path")?,
                "changes-requested",
                review_event_input(args, "project-lead")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_block",
        category: "Review",
        description: "Project Lead: block a review with a reason and preserve the review verdict history.",
        schema_fn: || review_verdict_schema(true),
        handler: |root, args| {
            review_verdict(
                root,
                req_str(args, "path")?,
                "blocked",
                review_event_input(args, "project-lead")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_supersede",
        category: "Review",
        description: "Project Lead: supersede a review with a reason and preserve the review verdict history.",
        schema_fn: || review_verdict_schema(true),
        handler: |root, args| {
            review_verdict(
                root,
                req_str(args, "path")?,
                "superseded",
                review_event_input(args, "project-lead")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },

    // Review event tools
    ToolSpec {
        name: "review_remediation",
        category: "Review",
        description: "Implementation specialist: record one remediation attempt without changing review status.",
        schema_fn: || review_event_schema(true),
        handler: |root, args| {
            record_review_event(
                root,
                req_str(args, "path")?,
                "remediation",
                review_event_input(args, "implementation-specialist")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_remediation_verified",
        category: "Review",
        description: "Project Lead: record one evidence-backed verification immediately after a remediation attempt without changing review status.",
        schema_fn: || review_event_schema(true),
        handler: |root, args| {
            record_review_event(
                root,
                req_str(args, "path")?,
                "remediation-verification",
                review_event_input(args, "project-lead")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_escalate",
        category: "Review",
        description: "Operator: record an explicitly authorized review escalation without changing review status.",
        schema_fn: || review_event_schema(false),
        handler: |root, args| {
            record_review_event(
                root,
                req_str(args, "path")?,
                "escalation",
                review_event_input(args, "operator")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_takeover",
        category: "Review",
        description: "Project Lead: record an operator-authorized bounded corrective takeover without changing review status.",
        schema_fn: || review_event_schema(false),
        handler: |root, args| {
            record_review_event(
                root,
                req_str(args, "path")?,
                "takeover",
                review_event_input(args, "project-lead")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "review_set_implementation_agent",
        category: "Review",
        description: "Project Lead: correct a provably wrong implementation_agent attribution on a review. The value must resolve to an existing AGENT-* profile; the append-only history gains an attribution-correction event recording the previous value. Not applicable to superseded reviews.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "agent", "reason"],
            "properties": {
                "path": {"type":"string"},
                "agent": {"type":"string","description":"The AGENT-* profile that actually implemented the spec."},
                "actor": {"type":"string","default":"project-lead"},
                "reason": {"type":"string","description":"Why the recorded attribution is wrong and how the correct agent was established."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            set_review_implementation_agent(
                root,
                req_str(args, "path")?,
                req_str(args, "agent")?,
                opt_str(args, "actor").unwrap_or("project-lead"),
                req_str(args, "reason")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },

    // Creation and setters
    ToolSpec {
        name: "lmbrain_create",
        category: "Create",
        description: "Create an artifact with an allocated ID.",
        schema_fn: || json!({
            "type":"object",
            "required":["kind","title"],
            "properties":{
                "kind":{"type":"string","enum":["spec","review","adr","agent","agent-proposal","mcp","mcp-proposal","handoff","skill"]},
                "title":{"type":"string"},
                "status":{"type":"string"},
                "fields":{"type":"array","items":{"type":"array","items":{"type":"string"},"minItems":2,"maxItems":2}}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let kind = serde_json::from_value::<ArtifactKind>(
                args.get("kind").cloned().ok_or("kind missing")?,
            )
            .map_err(|error| error.to_string())?;
            let title = req_str(args, "title")?.to_owned();
            let fields = match args.get("fields") {
                None | Some(Value::Null) => Vec::new(),
                Some(Value::Array(items)) => items
                    .iter()
                    .map(|pair| {
                        let key = pair.get(0).and_then(Value::as_str);
                        let value = pair.get(1).and_then(Value::as_str);
                        match (key, value, pair.as_array().map(Vec::len)) {
                            (Some(key), Some(value), Some(2)) => {
                                Ok((key.to_owned(), value.to_owned()))
                            }
                            _ => Err(format!(
                                "each field must be a [key, value] pair of strings, got {pair}"
                            )),
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                Some(other) => return Err(format!("fields must be an array, got {other}")),
            };

            create(
                root,
                CreateRequest {
                    kind,
                    title,
                    status: opt_str(args, "status").map(str::to_owned),
                    fields,
                },
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_attest_lead",
        category: "Spec",
        description: "Project Lead: attest one owner=lead, phase=before-done requirement with evidence. Does not approve or change spec status.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "requirement_id", "actor", "evidence_ref"],
            "properties": {
                "path": {"type": "string"},
                "requirement_id": {"type": "string"},
                "actor": {"type": "string", "description": "Lead profile ID, normally AGENT-LEAD."},
                "evidence_ref": {"type": "string"}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            attest_spec_requirement(
                root,
                req_str(args, "path")?,
                req_str(args, "requirement_id")?,
                "lead",
                req_str(args, "actor")?,
                req_str(args, "evidence_ref")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_attest_operator_delegated",
        category: "Verification",
        description: "Project Lead: record an operator attestation for one owner=operator, phase=before-done requirement when the operator granted approval out of band (e.g. in conversation) instead of through the desktop Operations page. Requires the operator's name, the channel, and the quoted authorization; the gate is satisfied without force and the attestation is auditable as delegated. Never a substitute for the operator's judgement — only for its recording channel.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "requirement_id", "operator", "channel", "authorization", "evidence_ref"],
            "properties": {
                "path": {"type": "string"},
                "requirement_id": {"type": "string"},
                "operator": {"type": "string", "description": "The human operator who granted the approval."},
                "recorded_by": {"type": "string", "default": "AGENT-LEAD", "description": "Lead profile recording the attestation."},
                "channel": {"type": "string", "description": "Where consent was given, e.g. 'conversation'."},
                "authorization": {"type": "string", "description": "The operator's approval, quoted or closely paraphrased (min 20 chars)."},
                "evidence_ref": {"type": "string"}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            attest_spec_requirement_delegated(
                root,
                req_str(args, "path")?,
                req_str(args, "requirement_id")?,
                req_str(args, "operator")?,
                req_str(args, "evidence_ref")?,
                AttestationDelegation {
                    recorded_by: opt_str(args, "recorded_by").unwrap_or("AGENT-LEAD").to_string(),
                    channel: req_str(args, "channel")?.to_string(),
                    authorization: req_str(args, "authorization")?.to_string(),
                },
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_park",
        category: "Spec",
        description: "Project Lead: park a ready spec back in backlog. This invalidates current readiness without discarding or rejecting the spec.",
        schema_fn: || json!({
            "type":"object","required":["path","actor","reason"],
            "properties":{
                "path":{"type":"string"},
                "actor":{"type":"string"},
                "reason":{"type":"string"},
                "revisit_condition":{"type":["string","null"]}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            park_spec(
                root,
                req_str(args, "path")?,
                SpecParkingInput {
                    actor: req_str(args, "actor")?.into(),
                    reason: req_str(args, "reason")?.into(),
                    revisit_condition: opt_str(args, "revisit_condition").map(str::to_string),
                },
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_set_recommended_agent",
        category: "Setter",
        description: "Set a spec recommended agent.",
        schema_fn: || setter_schema("agent"),
        handler: |root, args| {
            set_recommended_agent(
                root,
                req_str(args, "path")?,
                req_str(args, "agent")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_set_agent_mnemonic_name",
        category: "Setter",
        description: "Set an agent profile mnemonic human name.",
        schema_fn: || setter_schema("mnemonic_name"),
        handler: |root, args| {
            set_agent_mnemonic_name(
                root,
                req_str(args, "path")?,
                req_str(args, "mnemonic_name")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_set_tags",
        category: "Spec",
        description: "Project Lead: replace a spec's descriptive tags. Values are normalized; tags that restate `milestone`, `area`, or `priority` are rejected.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "tags"],
            "properties": {
                "path": {"type":"string"},
                "tags": {"type":"array","items":{"type":"string"}},
                "force": {"type":"boolean","default":false},
                "reason": {"type":"string","description":"Required only when force is true."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            let tags = args
                .get("tags")
                .and_then(Value::as_array)
                .ok_or("tags missing")?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or("tags must be strings")
                })
                .collect::<Result<Vec<_>, _>>()?;
            set_spec_tags(root, req_str(args, "path")?, &tags, opts(args))
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_set_effort",
        category: "Spec",
        description: "Project Lead: set the implementation estimate. `capability_tier` is the expected change footprint; `thinking_level` defaults from the tier when omitted. Required before a spec can become ready.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "capability_tier"],
            "properties": {
                "path": {"type":"string"},
                "capability_tier": {"type":"string","enum":["luna","terra","sol"]},
                "thinking_level": {"type":"string","enum":["minimal","standard","extended","maximum"]},
                "force": {"type":"boolean","default":false},
                "reason": {"type":"string","description":"Required only when force is true."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            set_spec_effort(
                root,
                req_str(args, "path")?,
                req_str(args, "capability_tier")?,
                opt_str(args, "thinking_level"),
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_record_effort_observation",
        category: "Spec",
        description: "Implementation specialist: append the effort the work actually required. Evidence only — it never changes the Lead's recommendation.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "observed_tier", "actor", "note"],
            "properties": {
                "path": {"type":"string"},
                "observed_tier": {"type":"string","enum":["luna","terra","sol"]},
                "actor": {"type":"string"},
                "note": {"type":"string"},
                "force": {"type":"boolean","default":false},
                "reason": {"type":"string","description":"Required only when force is true."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            record_effort_observation(
                root,
                req_str(args, "path")?,
                req_str(args, "observed_tier")?,
                req_str(args, "actor")?,
                req_str(args, "note")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "adr_supersede",
        category: "ADR",
        description: "Project Lead: retire a decision in favour of an accepted one, writing both sides of the relationship. The superseding ADR must already be accepted. Idempotent.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "superseded_id"],
            "properties": {
                "path": {"type":"string","description":"Path to the superseding (new) decision."},
                "superseded_id": {"type":"string","description":"ID of the decision being retired, e.g. ADR-009."},
                "force": {"type":"boolean","default":false},
                "reason": {"type":"string","description":"Required only when force is true."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            supersede_adr(
                root,
                req_str(args, "path")?,
                req_str(args, "superseded_id")?,
                opts(args),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_repair_frontmatter",
        category: "Repair",
        description: "Requires explicit operator authorization: repair managed frontmatter corrupted by failed mutations by merging duplicate top-level keys (e.g. duplicate `activity:` blocks). Refuses ambiguous shapes; records the repair and its reason in the activity log. Call when instructed by operator.",
        schema_fn: || json!({
            "type": "object",
            "required": ["path", "reason"],
            "properties": {
                "path": {"type":"string","description":"Artifact path relative to repository root."},
                "reason": {"type":"string","description":"Operator-authorized justification recorded in the activity log."}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            repair_artifact_frontmatter(root, req_str(args, "path")?, req_str(args, "reason")?)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_get_artifact",
        category: "Read",
        description: "Read a repository artifact.",
        schema_fn: || json!({
            "type":"object",
            "required":["path"],
            "properties":{"path":{"type":"string"}},
            "additionalProperties":false
        }),
        handler: |root, args| {
            let relative = req_str(args, "path")?;
            let source = lmbrain_core::read_artifact(root, relative).map_err(|error| error.to_string())?;
            Ok(text(json!({ "artifact": source })))
        },
    },
    ToolSpec {
        name: "lmbrain_validate",
        category: "Read",
        description: "Validate controlled-mutation invariants.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            Ok(text(json!({
                "schema_version": "2",
                "unique_ids": lmbrain_core::invariants::unique_ids(root),
                "diagnostics": lmbrain_core::build_diagnostics(root)
            })))
        },
    },
    ToolSpec {
        name: "review_migration_preview",
        category: "Review",
        description: "Read-only deterministic report of review lifecycle and debt-taxonomy migration coverage.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            build_review_migration_preview(root)
                .map(|preview| text(json!(preview)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "verification_migration_preview",
        category: "Verification",
        description: "Read-only conservative preview for legacy operator-owned before-done gates; never rewrites specs.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            lmbrain_core::build_verification_migration_preview(root)
                .map(|preview| text(json!(preview)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_list_ready_handoffs",
        category: "Read",
        description: "List ready handoffs.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let paths = std::fs::read_dir(root.join(".lmbrain/handoffs/active"))
                .map_err(|error| error.to_string())?
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let source = std::fs::read_to_string(&path).ok()?;
                    if source.contains("status: ready") {
                        path.file_name().map(|name| name.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Ok(text(json!({ "handoffs": paths })))
        },
    },
    ToolSpec {
        name: "lmbrain_project_digest",
        category: "Context",
        description: "Versioned, bounded project orientation: declared and derived state, lifecycle counts and spec lists, roadmap reconciliation, actionable diagnostics with exact omissions, handoffs and decisions. Read-only.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let digest = build_project_digest(root);
            Ok(text(json!(digest)))
        },
    },
    ToolSpec {
        name: "lmbrain_spec_context",
        category: "Context",
        description: "Spec handoff context: spec metadata, acceptance criteria checklist, linked decisions, recommended agent profile summary, related reviews, referenced milestone, explicit files/areas, and diagnostics affecting the handoff. Returns JSON and Markdown summary. Does not mutate artifacts.",
        schema_fn: || json!({
            "type": "object",
            "required": ["spec"],
            "properties": {
                "spec": {
                    "type": "string",
                    "description": "Spec ID (e.g. SPEC-023) or path relative to .lmbrain/"
                }
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            let spec = req_str(args, "spec")?;
            let ctx = build_spec_context(root, spec)?;
            Ok(text(json!(ctx)))
        },
    },
    ToolSpec {
        name: "lmbrain_review_context",
        category: "Context",
        description: "Review context: acceptance criteria, implementation evidence, linked accepted/proposed reviews, relevant decisions, and verification commands claimed by the specialist. Returns JSON and Markdown summary. Does not mutate artifacts.",
        schema_fn: || json!({
            "type": "object",
            "required": ["spec"],
            "properties": {
                "spec": {
                    "type": "string",
                    "description": "Spec ID (e.g. SPEC-023) or path relative to .lmbrain/"
                }
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            let spec = req_str(args, "spec")?;
            let ctx = build_review_context(root, spec)?;
            Ok(text(json!(ctx)))
        },
    },

    // Harness tools
    ToolSpec {
        name: "harness_config_get",
        category: "Environment",
        description: "Read and validate project harness intent. A missing optional manifest is reported as unconfigured.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            match load_harness_manifest(root) {
                Ok(manifest) => Ok(text(json!({
                    "configured": true,
                    "digest": canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?,
                    "manifest": manifest
                }))),
                Err(HarnessManifestError::Missing(_)) => Ok(text(json!({"configured": false}))),
                Err(error) => Err(error.to_string()),
            }
        },
    },
    ToolSpec {
        name: "harness_config_validate",
        category: "Environment",
        description: "Validate a complete candidate project harness manifest without writing it.",
        schema_fn: || json!({
            "type": "object",
            "required": ["manifest"],
            "properties": {"manifest": {"type": "object"}},
            "additionalProperties": false
        }),
        handler: |_root, args| {
            let manifest = candidate_manifest(args)?;
            Ok(text(json!({
                "valid": true,
                "digest": canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?,
                "manifest": manifest
            })))
        },
    },
    ToolSpec {
        name: "harness_config_set",
        category: "Environment",
        description: "Atomically replace the complete project harness manifest after strict validation and append digest-only audit evidence. Approval and materialization are separate digest-bound Lead actions (harness_manifest_approve, harness_config_apply).",
        schema_fn: || json!({
            "type": "object",
            "required": ["manifest"],
            "properties": {"manifest": {"type": "object"}},
            "additionalProperties": false
        }),
        handler: |root, args| {
            let manifest = candidate_manifest(args)?;
            set_harness_manifest(root, &manifest)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "harness_approval_status",
        category: "Environment",
        description: "Read the machine-local approval state of the project harness manifest: unconfigured, approval-required, approved, or stale. Read-only.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let store = default_harness_approval_store_path()?;
            harness_approval_status(root, &store).map(|status| text(json!(status)))
        },
    },
    ToolSpec {
        name: "harness_manifest_approve",
        category: "Environment",
        description: "Project Lead: approve the exact previewed harness manifest digest for this workspace. A manifest that changed since the preview is refused; every approval is audited with its actor.",
        schema_fn: || json!({
            "type": "object",
            "required": ["expected_digest"],
            "properties": {"expected_digest": {"type":"string","description":"The canonical manifest digest from harness_config_get or harness_plan_preview."}},
            "additionalProperties": false
        }),
        handler: |root, args| {
            let expected = req_str(args, "expected_digest")?;
            let store = default_harness_approval_store_path()?;
            approve_harness_manifest(root, &store, expected, "project-lead")
                .map(|status| text(json!(status)))
        },
    },
    ToolSpec {
        name: "harness_approval_revoke",
        category: "Environment",
        description: "Project Lead: revoke this workspace's harness manifest approval. Idempotent and audited.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let store = default_harness_approval_store_path()?;
            revoke_harness_approval(root, &store, "project-lead").map(|status| text(json!(status)))
        },
    },
    ToolSpec {
        name: "harness_plan_preview",
        category: "Environment",
        description: "Deterministic preview of the native host files the approved manifest would materialize: per-host readiness, capability prerequisites, exact file actions, and conflicts. Discovery never executes commands. Read-only.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let command = mcp_server_command()?;
            plan_harness_configuration(root, &command).map(|plan| text(json!(plan)))
        },
    },
    ToolSpec {
        name: "harness_config_apply",
        category: "Environment",
        description: "Project Lead: materialize the approved harness manifest into native host files. Requires the approved digest, refuses conflicts and non-ready hosts, applies atomically with rollback, records applied-content hashes for drift detection, and audits the action.",
        schema_fn: || json!({
            "type": "object",
            "required": ["expected_digest"],
            "properties": {"expected_digest": {"type":"string","description":"The approved canonical manifest digest."}},
            "additionalProperties": false
        }),
        handler: |root, args| {
            let expected = req_str(args, "expected_digest")?;
            let store = default_harness_approval_store_path()?;
            let command = mcp_server_command()?;
            apply_approved_harness_configuration(root, &store, &command, expected, "project-lead")
                .map(|result| text(json!(result)))
        },
    },
    ToolSpec {
        name: "harness_drift_status",
        category: "Environment",
        description: "Compare applied native-file content hashes against the files on disk and report drift. Read-only.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let store = default_harness_approval_store_path()?;
            let applied = lmbrain_core::harness_environment::applied_files(root, &store)?;
            let drift = lmbrain_core::harness_environment::detect_drift(root, &applied);
            Ok(text(json!({"applied_files": applied, "drift": drift})))
        },
    },

    // Verification manifest & execution
    ToolSpec {
        name: "verification_manifest_get",
        category: "Verification",
        description: "Read and validate the versioned verification manifest and return its canonical digest. Does not execute gates.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let manifest = load_verification_manifest(root).map_err(|error| error.to_string())?;
            let digest = canonical_verification_manifest_digest(&manifest)
                .map_err(|error| error.to_string())?;
            Ok(text(json!({"manifest": manifest, "digest": digest})))
        },
    },
    ToolSpec {
        name: "verification_manifest_status",
        category: "Verification",
        description: "Read-only typed status for the repository manifest and machine-local digest approval, with the exact next safe action.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let status = verification_manifest_status(root, &default_verification_approval_path(root))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(status)))
        },
    },
    ToolSpec {
        name: "verification_manifest_init",
        category: "Verification",
        description: "Read-only deterministic discovery and exact TOML/diff preview. Never writes, approves, or executes discovered commands.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let preview = discover_verification_manifest(root, &default_verification_approval_path(root))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(preview)))
        },
    },
    ToolSpec {
        name: "verification_manifest_validate",
        category: "Verification",
        description: "Validate complete verification TOML and return its canonical representation and digest without writing or executing it.",
        schema_fn: || json!({
            "type": "object",
            "required": ["source"],
            "properties": {"source": {"type": "string", "maxLength": 262144}},
            "additionalProperties": false
        }),
        handler: |_root, args| {
            let source = req_str(args, "source")?;
            validate_verification_manifest_source(source)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "verification_manifest_set",
        category: "Verification",
        description: "Project Lead: atomically create or replace a complete validated manifest after preview. This invalidates approval and never executes commands.",
        schema_fn: || json!({
            "type": "object",
            "required": ["manifest"],
            "properties": {
                "manifest": {"type": "object"},
                "expected_current_digest": {"type": ["string", "null"]}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            let manifest: VerificationManifest =
                serde_json::from_value(args.get("manifest").cloned().ok_or("manifest missing")?)
                    .map_err(|error| format!("invalid manifest payload: {error}"))?;
            let expected = opt_str(args, "expected_current_digest");
            set_verification_manifest(root, &manifest, expected)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "verification_manifest_rollback",
        category: "Verification",
        description: "Project Lead: restore the recoverable previous manifest only when the current digest matches the previewed digest. Approval remains separate.",
        schema_fn: || json!({
            "type": "object",
            "required": ["expected_current_digest"],
            "properties": {"expected_current_digest": {"type": "string"}},
            "additionalProperties": false
        }),
        handler: |root, args| {
            let expected = req_str(args, "expected_current_digest")?;
            rollback_verification_manifest(root, expected)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "verification_manifest_approve",
        category: "Verification",
        description: "Project Lead: approve the current verification manifest digest for this canonical workspace in machine-local state. Digest-bound and audited; spec_verify executes only gates referenced by the approved manifest.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let approval = approve_verification_manifest(root, &default_verification_approval_path(root))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(approval)))
        },
    },
    ToolSpec {
        name: "spec_verify",
        category: "Verification",
        description: "Execute only approved named verification gates referenced by a spec and atomically write an honest kit-generated transcript.",
        schema_fn: || json!({
            "type":"object",
            "required":["path"],
            "properties":{"path":{"type":"string","description":"Spec artifact path relative to repository root."}},
            "additionalProperties":false
        }),
        handler: |root, args| {
            let relative = req_str(args, "path")?;
            let approval_path = default_verification_approval_path(root);
            let status = verification_manifest_status(root, &approval_path)
                .map_err(|error| error.to_string())?;
            if status.state != VerificationManifestState::Approved {
                return Err(format!(
                    "spec_verify blocked: verification manifest state is {:?}. {}",
                    status.state, status.next_action
                ));
            }
            let report = execute_spec_verification(root, &root.join(relative), &approval_path)
                .map_err(|error| match error {
                    lmbrain_core::VerificationError::UnknownGate(gate) => format!(
                        "unknown verification gate '{gate}'; run verification_manifest_init and reconcile the spec reference before retrying"
                    ),
                    other => other.to_string(),
                })?;
            Ok(text(json!(report)))
        },
    },

    // Learning
    ToolSpec {
        name: "agent_improvement_signals",
        category: "Learning",
        description: "Derive evidence-backed repeated debt signals and per-profile effectiveness metrics without mutating artifacts.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            let (signals, metrics) =
                build_agent_improvement_signals(root).map_err(|error| error.to_string())?;
            Ok(text(json!({"signals": signals, "metrics": metrics})))
        },
    },
    ToolSpec {
        name: "agent_improvement_propose",
        category: "Learning",
        description: "Project Lead only: materialize an explicit evidence-backed improvement proposal; never applies it.",
        schema_fn: || json!({
            "type":"object","required":["target_profile","category","evidence_reviews","evidence_specs"],
            "properties":{
                "target_profile":{"type":"string"},"category":{"type":"string"},
                "evidence_reviews":{"type":"array","items":{"type":"string"}},
                "evidence_specs":{"type":"array","items":{"type":"string"}},
                "add_review_focus":{"type":"array","items":{"type":"string"}},
                "add_skills":{"type":"array","items":{"type":"string"}},
                "add_constraints":{"type":"array","items":{"type":"string"}},
                "add_primary_files":{"type":"array","items":{"type":"string"}},
                "guidance":{"type":"string"}
            },"additionalProperties":false
        }),
        handler: |root, args| {
            let request: ImprovementProposalRequest =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            let path =
                create_improvement_proposal(root, &request).map_err(|error| error.to_string())?;
            Ok(text(json!({"path": path})))
        },
    },
    ToolSpec {
        name: "agent_improvement_apply",
        category: "Learning",
        description: "Requires explicit operator authorization: atomically apply an approved, non-stale constrained improvement proposal to its target profile. Call when instructed by operator.",
        schema_fn: || json!({"type":"object","required":["path"],"properties":{"path":{"type":"string"}},"additionalProperties":false}),
        handler: |root, args| {
            let path = req_str(args, "path")?;
            let result = apply_improvement_proposal(root, &root.join(path))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(result)))
        },
    },

    // Debts
    ToolSpec {
        name: "debt_create",
        category: "Debt",
        description: "Project Lead: create one evidence-backed open DEBT-* artifact. This does not authorize implementation or rewrite its origin.",
        schema_fn: || json!({
            "type":"object",
            "required":["title","category","severity","statement","evidence","impact","resolution_criteria","actor","rationale"],
            "properties":{
                "title":{"type":"string"},"category":{"type":"string"},
                "severity":{"enum":["critical","high","medium","low","info"]},
                "origin_severity":{"type":["string","null"]},
                "area":{"type":["string","null"]},"milestone":{"type":["string","null"]},
                "owner":{"type":["string","null"]},"origin_artifact":{"type":["string","null"]},
                "origin_ref":{"type":["string","null"]},
                "related_specs":{"type":"array","items":{"type":"string"}},
                "related_reviews":{"type":"array","items":{"type":"string"}},
                "related_decisions":{"type":"array","items":{"type":"string"}},
                "blocked_by":{"type":"array","items":{"type":"string"}},
                "tags":{"type":"array","items":{"type":"string"}},
                "statement":{"type":"string"},"evidence":{"type":"string"},
                "impact":{"type":"string"},"resolution_criteria":{"type":"string"},
                "actor":{"type":"string"},"rationale":{"type":"string"}
            },"additionalProperties":false
        }),
        handler: |root, args| {
            let input: DebtCreateInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            create_debt(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_plan",
        category: "Debt",
        description: "Project Lead: route an unresolved debt to validated target specs.",
        schema_fn: || debt_transition_schema(&[
            ("target_specs", json!({"type":"array","items":{"type":"string"},"minItems":1}))
        ]),
        handler: |root, args| {
            plan_debt(
                root,
                req_str(args, "path")?,
                req_str_vec(args, "target_specs")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_defer",
        category: "Debt",
        description: "Project Lead: retain a debt outside active delivery with a revisit condition.",
        schema_fn: || debt_transition_schema(&[
            ("revisit_condition", json!({"type":"string"}))
        ]),
        handler: |root, args| {
            defer_debt(
                root,
                req_str(args, "path")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
                req_str(args, "revisit_condition")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_resolve",
        category: "Debt",
        description: "Project Lead: resolve a debt only with canonical references and explicit resolution evidence.",
        schema_fn: || debt_transition_schema(&[
            ("resolution_refs", json!({"type":"array","items":{"type":"string"},"minItems":1})),
            ("resolution_evidence", json!({"type":"string"}))
        ]),
        handler: |root, args| {
            resolve_debt(
                root,
                req_str(args, "path")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
                req_str_vec(args, "resolution_refs")?,
                req_str(args, "resolution_evidence")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_accept_risk",
        category: "Debt",
        description: "Requires explicit operator authorization: accept remaining behavior with rationale and an explicit revisit policy. Call when instructed by operator.",
        schema_fn: || debt_transition_schema(&[
            ("revisit_condition", json!({"type":"string"})),
            ("resolution_refs", json!({"type":"array","items":{"type":"string"}}))
        ]),
        handler: |root, args| {
            accept_debt_risk(
                root,
                req_str(args, "path")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
                req_str(args, "revisit_condition")?,
                req_str_vec(args, "resolution_refs")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_supersede",
        category: "Debt",
        description: "Project Lead: supersede a debt with a successor or explicit obsolescence rationale.",
        schema_fn: || debt_transition_schema(&[
            ("successor", json!({"type":["string","null"]}))
        ]),
        handler: |root, args| {
            supersede_debt(
                root,
                req_str(args, "path")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
                opt_str(args, "successor").map(str::to_owned),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_reopen",
        category: "Debt",
        description: "Requires explicit operator authorization: reopen a resolved or accepted-risk debt with rationale. Superseded history cannot be reopened. Call when instructed by operator.",
        schema_fn: || debt_transition_schema(&[]),
        handler: |root, args| {
            reopen_debt(
                root,
                req_str(args, "path")?,
                req_str(args, "actor")?,
                req_str(args, "rationale")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_context",
        category: "Debt",
        description: "Read-only bounded debt detail with canonical source, targets, blockers, decisions, evidence, and event timeline.",
        schema_fn: || json!({
            "type":"object","required":["debt"],"properties":{"debt":{"type":"string"}},"additionalProperties":false
        }),
        handler: |root, args| {
            debt_context(root, req_str(args, "debt")?)
                .map(|context| text(json!(context)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_candidates",
        category: "Debt",
        description: "Read-only bounded inventory of stable-form legacy review entries. It never infers disposition or creates debts.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            Ok(text(json!(debt_candidates(root))))
        },
    },
    ToolSpec {
        name: "debt_migration_preview",
        category: "Debt",
        description: "Read-only deterministic preview of the breaking legacy-durable-ID to DEBT workspace migration. Fails closed on malformed or ambiguous input.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            debt_migration_preview(root)
                .map(|preview| text(json!(preview)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "debt_migrate",
        category: "Debt",
        description: "Operator-confirmed, digest-bound atomic migration from legacy durable findings to debts and review-local RF identifiers.",
        schema_fn: || json!({
            "type":"object",
            "required":["expected_preview_digest","confirmed"],
            "properties":{
                "expected_preview_digest":{"type":"string"},
                "confirmed":{"type":"boolean","const":true,"description":"Set to true only after the operator explicitly confirms the reviewed preview."}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            debt_migrate(
                root,
                req_str(args, "expected_preview_digest")?,
                opt_bool(args, "confirmed").unwrap_or(false),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },

    // Kit Migration
    ToolSpec {
        name: "kit_migration_preview",
        category: "Kit",
        description: "Read-only preview and classification of kit-owned vs project-owned files for upgrading the workspace kit to 5.0.",
        schema_fn: || json!({
            "type":"object",
            "properties":{
                "bundled_kit_path":{"type":["string","null"],"description":"Optional explicit path to the bundled kit template directory."}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let bundled_path = resolve_bundled_kit_path(root, args)?;
            kit_migration_preview(root, &bundled_path)
                .map(|preview| text(json!(preview)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "kit_migrate",
        category: "Kit",
        description: "Operator-confirmed, digest-bound atomic migration updating kit-owned files and templates while strictly preserving project-owned artifacts.",
        schema_fn: || json!({
            "type":"object",
            "required":["expected_preview_digest","confirmed"],
            "properties":{
                "expected_preview_digest":{"type":"string"},
                "confirmed":{"type":"boolean","const":true,"description":"Set to true only after the operator explicitly confirms the migration preview."},
                "bundled_kit_path":{"type":["string","null"],"description":"Optional explicit path to the bundled kit template directory."}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let bundled_path = resolve_bundled_kit_path(root, args)?;
            kit_migrate(
                root,
                &bundled_path,
                req_str(args, "expected_preview_digest")?,
                opt_bool(args, "confirmed").unwrap_or(false),
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },

    // Dreams
    ToolSpec {
        name: "dream_capture",
        category: "Dream",
        description: "Project Lead, only after an explicit operator invitation to a bounded dreaming session: capture one grounded, tentative technical- or design-debt observation. It never creates a debt, spec, roadmap item, or decision.",
        schema_fn: || json!({
            "type":"object",
            "required":["title","classification","confidence","related_artifacts","context_digest","rationale","suggested_disposition","actor"],
            "properties":{
                "title":{"type":"string"},
                "classification":{"enum":["technical-debt","design-debt"]},
                "confidence":{"enum":["low","medium","high"]},
                "area":{"type":["string","null"]},
                "related_artifacts":{"type":"array","items":{"type":"string"},"minItems":1},
                "context_digest":{"type":"string","description":"Digest/timestamp of the bounded project context examined."},
                "rationale":{"type":"string","description":"Tentative, evidence-grounded observation; do not state unsupported facts."},
                "suggested_disposition":{"type":"string"},
                "actor":{"type":"string"}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let input: DreamCreateInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            capture_dream(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },

    // Dependencies
    ToolSpec {
        name: "spec_dependency_context",
        category: "Context",
        description: "Read-only direct, dependent, transitive, and blocking hard-spec dependency context.",
        schema_fn: || json!({
            "type":"object","required":["spec"],
            "properties":{"spec":{"type":"string"}},
            "additionalProperties":false
        }),
        handler: |root, args| {
            spec_dependency_context(root, req_str(args, "spec")?)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_dependency_candidates",
        category: "Context",
        description: "Read-only conservative inventory of explicit hard-dependency prose. Candidates are never promoted automatically.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            spec_dependency_candidates(root)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_set_verification_gates",
        category: "Spec",
        description: "Project Lead: replace the executable gate contract a spec declares, validated against the current verification manifest, with audit history and optimistic concurrency. Allowed in backlog, ready, and working only.",
        schema_fn: || json!({
            "type":"object","required":["path","verification_gates","actor","reason","expected_digest"],
            "properties":{
                "path":{"type":"string"},
                "verification_gates":{"type":"array","items":{"type":"string"},"description":"Gate IDs from .lmbrain/verification.toml. An empty array clears the contract."},
                "actor":{"type":"string"},
                "reason":{"type":"string"},
                "expected_digest":{"type":"string"}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            set_spec_verification_gates(
                root,
                PathBuf::from(req_str(args, "path")?).as_path(),
                req_str_vec(args, "verification_gates")?,
                req_str(args, "actor")?,
                req_str(args, "reason")?,
                req_str(args, "expected_digest")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "spec_dependencies_set",
        category: "Spec",
        description: "Project Lead: replace the hard prerequisite set with graph validation, audit history, and optimistic concurrency.",
        schema_fn: || json!({
            "type":"object","required":["path","depends_on","actor","reason","expected_digest"],
            "properties":{
                "path":{"type":"string"},
                "depends_on":{"type":"array","items":{"type":"string"}},
                "actor":{"type":"string"},
                "reason":{"type":"string"},
                "expected_digest":{"type":"string"}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            set_spec_dependencies(
                root,
                PathBuf::from(req_str(args, "path")?).as_path(),
                req_str_vec(args, "depends_on")?,
                req_str(args, "actor")?,
                req_str(args, "reason")?,
                req_str(args, "expected_digest")?,
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        },
    },

    // Kit feedback
    ToolSpec {
        name: "lmbrain_feedback_record",
        category: "Feedback",
        description: "Project Lead: autonomously append one evidence-backed observation about LMBrain itself to the portable kit feedback report. This never changes project lifecycle state.",
        schema_fn: || json!({
            "type":"object",
            "required":["category","severity","summary","observed_behavior","expected_behavior","impact","evidence","actor"],
            "properties":{
                "category":{"enum":["bug","usability","workflow","documentation","compatibility","performance","improvement"]},
                "severity":{"enum":["blocking","high","medium","low","info"]},
                "summary":{"type":"string"},
                "observed_behavior":{"type":"string"},
                "expected_behavior":{"type":"string"},
                "impact":{"type":"string"},
                "evidence":{"type":"string"},
                "workaround":{"type":["string","null"]},
                "suggested_improvement":{"type":["string","null"]},
                "related_note":{"type":["string","null"]},
                "actor":{"type":"string","description":"Project Lead profile ID, normally AGENT-LEAD."}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let input: KitFeedbackInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            record_kit_feedback(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_feedback_report",
        category: "Feedback",
        description: "Read-only parsed LMBrain kit feedback report with typed notes, append-only resolution events, derived per-note status, and category/severity counts. Reading an absent report never creates it.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            read_kit_feedback(root)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },
    ToolSpec {
        name: "lmbrain_feedback_resolve",
        category: "Feedback",
        description: "Project Lead: append one lifecycle event to an existing kit feedback note. kind=resolved retires the note against the named LMBrain release; kind=reconfirmed records that a still-open note reproduces on a later version without minting a new note ID. The note content is never edited and a resolved note accepts no further events.",
        schema_fn: || json!({
            "type":"object",
            "required":["note_id","kind","version","actor"],
            "properties":{
                "note_id":{"type":"string","description":"Existing KIT-NOTE-* ID."},
                "kind":{"enum":["resolved","reconfirmed"]},
                "version":{"type":"string","description":"The LMBrain release that resolves the note, or the version it was reconfirmed against."},
                "reference":{"type":["string","null"],"description":"Optional upstream issue/PR/URL."},
                "actor":{"type":"string","description":"Project Lead profile ID, normally AGENT-LEAD."}
            },
            "additionalProperties":false
        }),
        handler: |root, args| {
            let input: KitFeedbackResolutionInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            record_kit_feedback_resolution(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        },
    },

    // Branching strategy
    ToolSpec {
        name: "branching_strategy_get",
        category: "Branching",
        description: "Read declared project branching strategy (.lmbrain/BRANCHING.json) and summary digest. An absent manifest is reported as unconfigured.",
        schema_fn: || json!({"type":"object","properties":{},"additionalProperties":false}),
        handler: |root, _args| {
            match load_branching_strategy(root) {
                Ok(strategy) => Ok(text(json!({
                    "digest": build_branching_strategy_digest(root),
                    "strategy": strategy
                }))),
                Err(error) => Err(error.to_string()),
            }
        },
    },
    ToolSpec {
        name: "branching_strategy_set",
        category: "Branching",
        description: "Operator-only: set and atomically write declared branching strategy to .lmbrain/BRANCHING.json with audit trail.",
        schema_fn: || json!({
            "type": "object",
            "required": ["strategy", "actor", "reason"],
            "properties": {
                "strategy": {"type": "object"},
                "actor": {"type": "string"},
                "reason": {"type": "string"}
            },
            "additionalProperties": false
        }),
        handler: |root, args| {
            let strategy_val = args
                .get("strategy")
                .ok_or_else(|| "Missing required parameter 'strategy'".to_string())?;
            let strategy: BranchingStrategy = serde_json::from_value(strategy_val.clone())
                .map_err(|e| format!("Invalid strategy format: {e}"))?;
            let actor = req_str(args, "actor")?;
            let reason = req_str(args, "reason")?;
            set_branching_strategy(root, &strategy, actor, reason)
                .map(|_| text(json!({"success": true, "strategy": strategy})))
                .map_err(|error| error.to_string())
        },
    },
];

pub fn tools() -> Vec<Value> {
    TOOLS
        .iter()
        .map(|spec| {
            json!({
                "name": spec.name,
                "description": spec.description,
                "inputSchema": (spec.schema_fn)()
            })
        })
        .collect()
}

pub fn call(root: &Path, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name missing")?;
    let args = params.get("arguments").unwrap_or(&Value::Null);

    for spec in TOOLS {
        if spec.name == name {
            return (spec.handler)(root, args);
        }
    }

    Err("unknown tool".into())
}

pub fn specific_status(name: &str) -> Option<&'static str> {
    match name {
        "spec_ready" => Some("ready"),
        "spec_start" => Some("working"),
        "spec_submit" => Some("review"),
        "spec_done" => Some("done"),
        "spec_discard" => Some("discarded"),
        "adr_accept" => Some("accepted"),
        "adr_reject" => Some("rejected"),
        "agent_activate" => Some("active"),
        "agent_deactivate" => Some("inactive"),
        "agent_proposal_approve" => Some("approved"),
        "agent_proposal_reject" => Some("rejected"),
        "skill_activate" => Some("active"),
        "skill_retire" => Some("retired"),
        "handoff_consume" => Some("consumed"),
        "handoff_supersede" => Some("superseded"),
        "handoff_archive" => Some("archived"),
        _ => None,
    }
}
