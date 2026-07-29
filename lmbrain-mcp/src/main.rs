use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
};

use lmbrain_core::context::{build_project_digest, build_review_context, build_spec_context};
use lmbrain_core::transitions::{
    create, record_review_event, review_verdict, set_agent_mnemonic_name, set_recommended_agent,
    transition, ArtifactKind, CreateRequest, MutationOptions,
};
use lmbrain_core::{
    accept_finding_risk, apply_improvement_proposal, approve_verification_manifest,
    attest_spec_requirement, build_agent_improvement_signals, build_review_migration_preview,
    canonical_manifest_digest, canonical_verification_manifest_digest, create_finding,
    create_improvement_proposal, default_verification_approval_path, defer_finding,
    discover_verification_manifest, execute_spec_verification, finding_candidates, finding_context,
    load_harness_manifest, load_verification_manifest, park_spec, parse_harness_manifest,
    plan_finding, read_kit_feedback, record_kit_feedback, reopen_finding, resolve_finding,
    rollback_verification_manifest, set_harness_manifest, set_spec_dependencies,
    set_verification_manifest, spec_dependency_candidates, spec_dependency_context,
    supersede_finding, validate_verification_manifest_source, verification_manifest_status,
    FindingCreateInput, HarnessManifestError, ImprovementProposalRequest, KitFeedbackInput,
    ReviewEventInput, SpecParkingInput, VerificationManifest, VerificationManifestState,
};
use serde_json::{json, Value};

fn main() {
    let root = resolve_root(
        std::env::args().skip(1),
        std::env::var("LMBRAIN_ROOT").ok(),
        std::env::current_dir().ok(),
    );

    for line in io::stdin().lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                reply(Value::Null, Err(format!("invalid JSON: {error}")));
                continue;
            }
        };

        if let Some(id) = request.get("id").cloned() {
            reply(id, handle(&root, &request));
        } else {
            let _ = handle(&root, &request);
        }
    }
}

/// Resolve the workspace root: explicit `--root <path>`/`--root=<path>` wins, then
/// `LMBRAIN_ROOT`, then the launch directory.
fn resolve_root(
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

fn reply(id: Value, result: Result<Value, String>) {
    let response = match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(message) => {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":message}})
        }
    };
    println!("{response}");
    let _ = io::stdout().flush();
}

fn handle(root: &PathBuf, request: &Value) -> Result<Value, String> {
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

fn tools() -> Vec<Value> {
    let mut entries = Vec::new();
    for (name, description) in [
        (
            "spec_ready",
            "Approve a backlog spec to ready (on operator request).",
        ),
        (
            "spec_start",
            "Implementation specialist only: move an assigned ready spec to working when starting implementation.",
        ),
        (
            "spec_submit",
            "Implementation specialist only: move a working spec to review when implementation is complete.",
        ),
        (
            "spec_done",
            "Project Lead on operator/review authority: mark a reviewed spec done after accepted review, checked criteria, and evidence.",
        ),
        (
            "spec_discard",
            "Discard a spec (requires operator approval).",
        ),
        ("adr_accept", "Accept a proposed ADR (on operator request)."),
        ("adr_reject", "Reject a proposed ADR (on operator request)."),
        (
            "agent_activate",
            "Activate a proposed agent profile (on operator request).",
        ),
        (
            "agent_deactivate",
            "Deactivate an agent profile (on operator request).",
        ),
        (
            "agent_proposal_approve",
            "Approve an agent improvement proposal (on operator request).",
        ),
        (
            "agent_proposal_reject",
            "Reject an agent improvement proposal (on operator request).",
        ),
        (
            "skill_activate",
            "Activate a proposed project-scoped skill (on operator request).",
        ),
        (
            "skill_retire",
            "Retire a project-scoped skill that should no longer be recommended.",
        ),
        (
            "handoff_consume",
            "Consume a ready session handoff (Project Lead only, after validation).",
        ),
        (
            "handoff_supersede",
            "Supersede a ready session handoff with a newer one.",
        ),
        (
            "handoff_archive",
            "Archive/retire a session handoff.",
        ),
    ] {
        entries.push(transition_tool(name, description));
    }

    entries.extend([
        review_verdict_tool(
            "review_accept",
            "Accept a review on explicit operator request and record the verdict event.",
            false,
        ),
        review_verdict_tool(
            "review_changes_requested",
            "Project Lead: request changes with a reason and preserve the review verdict history.",
            true,
        ),
        review_verdict_tool(
            "review_block",
            "Project Lead: block a review with a reason and preserve the review verdict history.",
            true,
        ),
        review_verdict_tool(
            "review_supersede",
            "Project Lead: supersede a review with a reason and preserve the review verdict history.",
            true,
        ),
        review_event_tool(
            "review_remediation",
            "Implementation specialist: record one remediation attempt without changing review status.",
            true,
        ),
        review_event_tool(
            "review_escalate",
            "Operator: record an explicitly authorized review escalation without changing review status.",
            false,
        ),
        review_event_tool(
            "review_takeover",
            "Project Lead: record an operator-authorized bounded corrective takeover without changing review status.",
            false,
        ),
        create_tool(),
        lead_attestation_tool(),
        spec_park_tool(),
        setter_tool(
            "lmbrain_set_recommended_agent",
            "Set a spec recommended agent.",
            "agent",
        ),
        setter_tool(
            "lmbrain_set_agent_mnemonic_name",
            "Set an agent profile mnemonic human name.",
            "mnemonic_name",
        ),
        read_tool("lmbrain_get_artifact", "Read a repository artifact."),
        read_tool(
            "lmbrain_validate",
            "Validate controlled-mutation invariants.",
        ),
        read_tool(
            "review_migration_preview",
            "Read-only deterministic report of review lifecycle and finding-taxonomy migration coverage.",
        ),
        read_tool(
            "verification_migration_preview",
            "Read-only conservative preview for legacy operator-owned before-done gates; never rewrites specs.",
        ),
        read_tool("lmbrain_list_ready_handoffs", "List ready handoffs."),
        // V3 context-pack tools
        context_tool(
            "lmbrain_project_digest",
            "Versioned, bounded project orientation: declared and derived state, lifecycle counts and spec lists, roadmap reconciliation, actionable diagnostics with exact omissions, handoffs and decisions. Read-only.",
        ),
        context_tool(
            "lmbrain_spec_context",
            "Spec handoff context: spec metadata, acceptance criteria checklist, linked decisions, recommended agent profile summary, related reviews, referenced milestone, explicit files/areas, and diagnostics affecting the handoff. Returns JSON and Markdown summary. Does not mutate artifacts.",
        ),
        context_tool(
            "lmbrain_review_context",
            "Review context: acceptance criteria, implementation evidence, linked accepted/proposed reviews, relevant decisions, and verification commands claimed by the specialist. Returns JSON and Markdown summary. Does not mutate artifacts.",
        ),
        harness_get_tool(),
        harness_candidate_tool("harness_config_validate", "Validate a complete candidate project harness manifest without writing it."),
        harness_candidate_tool("harness_config_set", "Atomically replace the complete project harness manifest after strict validation and append digest-only audit evidence. This does not approve or materialize native configuration."),
        verification_manifest_tool(),
        verification_manifest_status_tool(),
        verification_manifest_init_tool(),
        verification_manifest_validate_tool(),
        verification_manifest_set_tool(),
        verification_manifest_rollback_tool(),
        verification_approval_tool(),
        spec_verify_tool(),
        improvement_signals_tool(),
        improvement_propose_tool(),
        improvement_apply_tool(),
    ]);
    entries.extend(finding_tools());
    entries.extend(spec_dependency_tools());
    entries.extend(kit_feedback_tools());

    entries
}

fn kit_feedback_tools() -> Vec<Value> {
    vec![
        json!({
            "name":"lmbrain_feedback_record",
            "description":"Project Lead: autonomously append one evidence-backed observation about LMBrain itself to the portable kit feedback report. This never changes project lifecycle state.",
            "inputSchema":{
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
            }
        }),
        json!({
            "name":"lmbrain_feedback_report",
            "description":"Read-only parsed LMBrain kit feedback report with typed notes and category/severity counts. Reading an absent report never creates it.",
            "inputSchema":{"type":"object","properties":{},"additionalProperties":false}
        }),
    ]
}

fn spec_dependency_tools() -> Vec<Value> {
    vec![
        json!({
            "name":"spec_dependency_context",
            "description":"Read-only direct, dependent, transitive, and blocking hard-spec dependency context.",
            "inputSchema":{
                "type":"object","required":["spec"],
                "properties":{"spec":{"type":"string"}},
                "additionalProperties":false
            }
        }),
        json!({
            "name":"spec_dependency_candidates",
            "description":"Read-only conservative inventory of explicit hard-dependency prose. Candidates are never promoted automatically.",
            "inputSchema":{"type":"object","properties":{},"additionalProperties":false}
        }),
        json!({
            "name":"spec_dependencies_set",
            "description":"Project Lead: replace the hard prerequisite set with graph validation, audit history, and optimistic concurrency.",
            "inputSchema":{
                "type":"object","required":["path","depends_on","actor","reason","expected_digest"],
                "properties":{
                    "path":{"type":"string"},
                    "depends_on":{"type":"array","items":{"type":"string"}},
                    "actor":{"type":"string"},
                    "reason":{"type":"string"},
                    "expected_digest":{"type":"string"}
                },
                "additionalProperties":false
            }
        }),
    ]
}

fn finding_tools() -> Vec<Value> {
    vec![
        json!({
            "name":"finding_create",
            "description":"Project Lead: create one evidence-backed open FINDING-* artifact. This does not authorize implementation or rewrite its origin.",
            "inputSchema":{
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
            }
        }),
        finding_transition_tool("finding_plan", "Project Lead: route an unresolved finding to validated target specs.", &[
            ("target_specs", json!({"type":"array","items":{"type":"string"},"minItems":1}))
        ]),
        finding_transition_tool("finding_defer", "Project Lead: retain a finding outside active delivery with a revisit condition.", &[
            ("revisit_condition", json!({"type":"string"}))
        ]),
        finding_transition_tool("finding_resolve", "Project Lead: resolve a finding only with canonical references and explicit resolution evidence.", &[
            ("resolution_refs", json!({"type":"array","items":{"type":"string"},"minItems":1})),
            ("resolution_evidence", json!({"type":"string"}))
        ]),
        finding_transition_tool("finding_accept_risk", "Operator-only: accept remaining behavior with rationale and an explicit revisit policy.", &[
            ("revisit_condition", json!({"type":"string"})),
            ("resolution_refs", json!({"type":"array","items":{"type":"string"}}))
        ]),
        finding_transition_tool("finding_supersede", "Project Lead: supersede a finding with a successor or explicit obsolescence rationale.", &[
            ("successor", json!({"type":["string","null"]}))
        ]),
        finding_transition_tool("finding_reopen", "Operator-only: reopen a resolved or accepted-risk finding with rationale. Superseded history cannot be reopened.", &[]),
        json!({
            "name":"finding_context",
            "description":"Read-only bounded finding detail with canonical source, targets, blockers, decisions, evidence, and event timeline.",
            "inputSchema":{"type":"object","required":["finding"],"properties":{"finding":{"type":"string"}},"additionalProperties":false}
        }),
        json!({
            "name":"finding_candidates",
            "description":"Read-only bounded inventory of stable-form legacy review entries. It never infers disposition or creates findings.",
            "inputSchema":{"type":"object","properties":{},"additionalProperties":false}
        }),
    ]
}

fn finding_transition_tool(name: &str, description: &str, extras: &[(&str, Value)]) -> Value {
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
        "name":name,"description":description,
        "inputSchema":{"type":"object","required":required,"properties":properties,"additionalProperties":false}
    })
}

fn verification_manifest_tool() -> Value {
    json!({
        "name": "verification_manifest_get",
        "description": "Read and validate the versioned verification manifest and return its canonical digest. Does not execute gates.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
    })
}

fn verification_manifest_status_tool() -> Value {
    json!({
        "name": "verification_manifest_status",
        "description": "Read-only typed status for the repository manifest and machine-local digest approval, with the exact next safe action.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
    })
}

fn verification_manifest_init_tool() -> Value {
    json!({
        "name": "verification_manifest_init",
        "description": "Read-only deterministic discovery and exact TOML/diff preview. Never writes, approves, or executes discovered commands.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
    })
}

fn verification_manifest_validate_tool() -> Value {
    json!({
        "name": "verification_manifest_validate",
        "description": "Validate complete verification TOML and return its canonical representation and digest without writing or executing it.",
        "inputSchema": {
            "type": "object",
            "required": ["source"],
            "properties": {"source": {"type": "string", "maxLength": 262144}},
            "additionalProperties": false
        }
    })
}

fn verification_manifest_set_tool() -> Value {
    json!({
        "name": "verification_manifest_set",
        "description": "Project Lead: atomically create or replace a complete validated manifest after preview. This invalidates approval and never executes commands.",
        "inputSchema": {
            "type": "object",
            "required": ["manifest"],
            "properties": {
                "manifest": {"type": "object"},
                "expected_current_digest": {"type": ["string", "null"]}
            },
            "additionalProperties": false
        }
    })
}

fn verification_manifest_rollback_tool() -> Value {
    json!({
        "name": "verification_manifest_rollback",
        "description": "Project Lead: restore the recoverable previous manifest only when the current digest matches the previewed digest. Approval remains separate.",
        "inputSchema": {
            "type": "object",
            "required": ["expected_current_digest"],
            "properties": {"expected_current_digest": {"type": "string"}},
            "additionalProperties": false
        }
    })
}

fn verification_approval_tool() -> Value {
    json!({
        "name": "verification_manifest_approve",
        "description": "Operator-only: approve the current verification manifest digest for this canonical workspace in machine-local state.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
    })
}

fn spec_verify_tool() -> Value {
    json!({
        "name": "spec_verify",
        "description": "Execute only approved named verification gates referenced by a spec and atomically write an honest kit-generated transcript.",
        "inputSchema": {
            "type":"object",
            "required":["path"],
            "properties":{"path":{"type":"string","description":"Spec artifact path relative to repository root."}},
            "additionalProperties":false
        }
    })
}

fn improvement_signals_tool() -> Value {
    json!({
        "name":"agent_improvement_signals",
        "description":"Derive evidence-backed repeated finding signals and per-profile effectiveness metrics without mutating artifacts.",
        "inputSchema":{"type":"object","properties":{},"additionalProperties":false}
    })
}

fn improvement_propose_tool() -> Value {
    json!({
        "name":"agent_improvement_propose",
        "description":"Project Lead only: materialize an explicit evidence-backed improvement proposal; never applies it.",
        "inputSchema":{
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
        }
    })
}

fn improvement_apply_tool() -> Value {
    json!({
        "name":"agent_improvement_apply",
        "description":"Operator-only: atomically apply an approved, non-stale constrained improvement proposal to its target profile.",
        "inputSchema":{"type":"object","required":["path"],"properties":{"path":{"type":"string"}},"additionalProperties":false}
    })
}

fn harness_get_tool() -> Value {
    json!({
        "name": "harness_config_get",
        "description": "Read and validate project harness intent. A missing optional manifest is reported as unconfigured.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
    })
}

fn harness_candidate_tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "required": ["manifest"],
            "properties": {"manifest": {"type": "object"}},
            "additionalProperties": false
        }
    })
}

fn transition_tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {"type": "string", "description": "Artifact path relative to repository root."},
                "force": {"type": "boolean", "default": false},
                "reason": {"type": "string", "description": "Required only when force is true."}
            },
            "additionalProperties": false
        }
    })
}

fn review_verdict_tool(name: &str, description: &str, reason_required: bool) -> Value {
    let mut required = vec![json!("path")];
    if reason_required {
        required.push(json!("reason"));
    }
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
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
        }
    })
}

fn review_event_tool(name: &str, description: &str, remediation_agent_required: bool) -> Value {
    let mut required = vec![json!("path"), json!("reason")];
    if remediation_agent_required {
        required.push(json!("remediation_agent"));
    }
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
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
        }
    })
}

fn lead_attestation_tool() -> Value {
    json!({
        "name": "spec_attest_lead",
        "description": "Project Lead: attest one owner=lead, phase=before-done requirement with evidence. Does not approve or change spec status.",
        "inputSchema": {
            "type": "object",
            "required": ["path", "requirement_id", "actor", "evidence_ref"],
            "properties": {
                "path": {"type": "string"},
                "requirement_id": {"type": "string"},
                "actor": {"type": "string", "description": "Lead profile ID, normally AGENT-LEAD."},
                "evidence_ref": {"type": "string"}
            },
            "additionalProperties": false
        }
    })
}

fn spec_park_tool() -> Value {
    json!({
        "name":"spec_park",
        "description":"Project Lead: park a ready spec back in backlog. This invalidates current readiness without discarding or rejecting the spec.",
        "inputSchema":{
            "type":"object","required":["path","actor","reason"],
            "properties":{
                "path":{"type":"string"},
                "actor":{"type":"string"},
                "reason":{"type":"string"},
                "revisit_condition":{"type":["string","null"]}
            },
            "additionalProperties":false
        }
    })
}

fn create_tool() -> Value {
    json!({
        "name":"lmbrain_create",
        "description":"Create an artifact with an allocated ID.",
        "inputSchema":{
            "type":"object",
            "required":["kind","title"],
            "properties":{
                "kind":{"type":"string","enum":["spec","review","adr","agent","agent-proposal","mcp","mcp-proposal","handoff","skill"]},
                "title":{"type":"string"},
                "status":{"type":"string"},
                "fields":{"type":"array","items":{"type":"array","items":{"type":"string"},"minItems":2,"maxItems":2}}
            },
            "additionalProperties":false
        }
    })
}

fn setter_tool(name: &str, description: &str, field: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "required": ["path", field],
            "properties": {
                "path": {"type":"string"},
                field: {"type":"string"},
                "force": {"type":"boolean","default":false},
                "reason": {"type":"string","description":"Required only when force is true."}
            },
            "additionalProperties": false
        }
    })
}

fn read_tool(name: &str, description: &str) -> Value {
    let schema = if name == "lmbrain_get_artifact" {
        json!({"type":"object","required":["path"],"properties":{"path":{"type":"string"}},"additionalProperties":false})
    } else {
        json!({"type":"object","properties":{},"additionalProperties":false})
    };

    json!({
        "name": name,
        "description": description,
        "inputSchema": schema
    })
}

fn context_tool(name: &str, description: &str) -> Value {
    let schema = if name == "lmbrain_project_digest" {
        json!({"type":"object","properties":{},"additionalProperties":false})
    } else {
        json!({
            "type": "object",
            "required": ["spec"],
            "properties": {
                "spec": {
                    "type": "string",
                    "description": "Spec ID (e.g. SPEC-023) or path relative to .lmbrain/"
                }
            },
            "additionalProperties": false
        })
    };

    json!({
        "name": name,
        "description": description,
        "inputSchema": schema
    })
}

fn opts(args: &Value) -> MutationOptions {
    MutationOptions {
        force: args.get("force").and_then(Value::as_bool).unwrap_or(false),
        reason: args
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_owned),
    }
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{key} missing"))
}

fn string_array(args: &Value, key: &str) -> Result<Vec<String>, String> {
    serde_json::from_value(
        args.get(key)
            .cloned()
            .ok_or_else(|| format!("{key} missing"))?,
    )
    .map_err(|error| format!("{key} must be an array of strings: {error}"))
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
        reason: args
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        evidence_refs,
        remediation_agent: args
            .get("remediation_agent")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn text(value: Value) -> Value {
    json!({"content":[{"type":"text","text":value.to_string()}]})
}

fn call(root: &PathBuf, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name missing")?;
    let args = params.get("arguments").unwrap_or(&Value::Null);

    if let Some((action, actor_role)) = review_event_action(name) {
        return record_review_event(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            action,
            review_event_input(args, actor_role)?,
            opts(args),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string());
    }

    if let Some((status, actor_role)) = review_status(name) {
        return review_verdict(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            status,
            review_event_input(args, actor_role)?,
            opts(args),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string());
    }

    if let Some(status) = specific_status(name) {
        return transition(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            status,
            opts(args),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string());
    }

    match name {
        "spec_attest_lead" => attest_spec_requirement(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            args.get("requirement_id")
                .and_then(Value::as_str)
                .ok_or("requirement_id missing")?,
            "lead",
            args.get("actor")
                .and_then(Value::as_str)
                .ok_or("actor missing")?,
            args.get("evidence_ref")
                .and_then(Value::as_str)
                .ok_or("evidence_ref missing")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "spec_park" => park_spec(
            root,
            required_string(args, "path")?,
            SpecParkingInput {
                actor: required_string(args, "actor")?.into(),
                reason: required_string(args, "reason")?.into(),
                revisit_condition: args
                    .get("revisit_condition")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            },
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "lmbrain_feedback_record" => {
            let input: KitFeedbackInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            record_kit_feedback(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "lmbrain_feedback_report" => read_kit_feedback(root)
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string()),
        "spec_dependency_context" => spec_dependency_context(root, required_string(args, "spec")?)
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string()),
        "spec_dependency_candidates" => spec_dependency_candidates(root)
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string()),
        "spec_dependencies_set" => set_spec_dependencies(
            root,
            PathBuf::from(required_string(args, "path")?).as_path(),
            string_array(args, "depends_on")?,
            required_string(args, "actor")?,
            required_string(args, "reason")?,
            required_string(args, "expected_digest")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "lmbrain_create" => {
            let kind = serde_json::from_value::<ArtifactKind>(
                args.get("kind").cloned().ok_or("kind missing")?,
            )
            .map_err(|error| error.to_string())?;
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .ok_or("title missing")?
                .to_owned();
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
                    status: args
                        .get("status")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    fields,
                },
            )
            .map(|result| text(json!(result)))
            .map_err(|error| error.to_string())
        }
        "lmbrain_set_recommended_agent" => set_recommended_agent(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            args.get("agent")
                .and_then(Value::as_str)
                .ok_or("agent missing")?,
            opts(args),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "lmbrain_set_agent_mnemonic_name" => set_agent_mnemonic_name(
            root,
            args.get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?,
            args.get("mnemonic_name")
                .and_then(Value::as_str)
                .ok_or("mnemonic_name missing")?,
            opts(args),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "lmbrain_get_artifact" => {
            let relative = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?;
            let source =
                lmbrain_core::read_artifact(root, relative).map_err(|error| error.to_string())?;
            Ok(text(json!({ "artifact": source })))
        }
        "lmbrain_list_ready_handoffs" => {
            let paths = std::fs::read_dir(root.join(".lmbrain/handoffs/active"))
                .map_err(|error| error.to_string())?
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let source = std::fs::read_to_string(&path).ok()?;
                    if source.contains("status: ready") {
                        path.file_name()
                            .map(|name| name.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Ok(text(json!({ "handoffs": paths })))
        }
        "lmbrain_validate" => Ok(text(json!({
            "schema_version": "2",
            "unique_ids": lmbrain_core::invariants::unique_ids(root),
            "diagnostics": lmbrain_core::build_diagnostics(root)
        }))),
        "review_migration_preview" => build_review_migration_preview(root)
            .map(|preview| text(json!(preview)))
            .map_err(|error| error.to_string()),
        "verification_migration_preview" => {
            lmbrain_core::build_verification_migration_preview(root)
                .map(|preview| text(json!(preview)))
                .map_err(|error| error.to_string())
        }
        "lmbrain_project_digest" => {
            let digest = build_project_digest(root);
            Ok(text(json!(digest)))
        }
        "lmbrain_spec_context" => {
            let spec = args
                .get("spec")
                .and_then(Value::as_str)
                .ok_or("spec parameter missing")?;
            let ctx = build_spec_context(root, spec)?;
            Ok(text(json!(ctx)))
        }
        "lmbrain_review_context" => {
            let spec = args
                .get("spec")
                .and_then(Value::as_str)
                .ok_or("spec parameter missing")?;
            let ctx = build_review_context(root, spec)?;
            Ok(text(json!(ctx)))
        }
        "finding_create" => {
            let input: FindingCreateInput =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            create_finding(root, input)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "finding_plan" => plan_finding(
            root,
            required_string(args, "path")?,
            string_array(args, "target_specs")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_defer" => defer_finding(
            root,
            required_string(args, "path")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
            required_string(args, "revisit_condition")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_resolve" => resolve_finding(
            root,
            required_string(args, "path")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
            string_array(args, "resolution_refs")?,
            required_string(args, "resolution_evidence")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_accept_risk" => accept_finding_risk(
            root,
            required_string(args, "path")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
            required_string(args, "revisit_condition")?,
            string_array(args, "resolution_refs")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_supersede" => supersede_finding(
            root,
            required_string(args, "path")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
            args.get("successor")
                .and_then(Value::as_str)
                .map(str::to_owned),
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_reopen" => reopen_finding(
            root,
            required_string(args, "path")?,
            required_string(args, "actor")?,
            required_string(args, "rationale")?,
        )
        .map(|result| text(json!(result)))
        .map_err(|error| error.to_string()),
        "finding_context" => finding_context(root, required_string(args, "finding")?)
            .map(|context| text(json!(context)))
            .map_err(|error| error.to_string()),
        "finding_candidates" => Ok(text(json!(finding_candidates(root)))),
        "harness_config_get" => match load_harness_manifest(root) {
            Ok(manifest) => Ok(text(json!({
                "configured": true,
                "digest": canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?,
                "manifest": manifest
            }))),
            Err(HarnessManifestError::Missing(_)) => Ok(text(json!({"configured": false}))),
            Err(error) => Err(error.to_string()),
        },
        "harness_config_validate" => {
            let manifest = candidate_manifest(args)?;
            Ok(text(json!({
                "valid": true,
                "digest": canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?,
                "manifest": manifest
            })))
        }
        "harness_config_set" => {
            let manifest = candidate_manifest(args)?;
            set_harness_manifest(root, &manifest)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "verification_manifest_get" => {
            let manifest = load_verification_manifest(root).map_err(|error| error.to_string())?;
            let digest = canonical_verification_manifest_digest(&manifest)
                .map_err(|error| error.to_string())?;
            Ok(text(json!({"manifest": manifest, "digest": digest})))
        }
        "verification_manifest_status" => {
            let status =
                verification_manifest_status(root, &default_verification_approval_path(root))
                    .map_err(|error| error.to_string())?;
            Ok(text(json!(status)))
        }
        "verification_manifest_init" => {
            let preview =
                discover_verification_manifest(root, &default_verification_approval_path(root))
                    .map_err(|error| error.to_string())?;
            Ok(text(json!(preview)))
        }
        "verification_manifest_validate" => {
            let source = args
                .get("source")
                .and_then(Value::as_str)
                .ok_or("source missing")?;
            validate_verification_manifest_source(source)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "verification_manifest_set" => {
            let manifest: VerificationManifest =
                serde_json::from_value(args.get("manifest").cloned().ok_or("manifest missing")?)
                    .map_err(|error| format!("invalid manifest payload: {error}"))?;
            let expected = args.get("expected_current_digest").and_then(Value::as_str);
            set_verification_manifest(root, &manifest, expected)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "verification_manifest_rollback" => {
            let expected = args
                .get("expected_current_digest")
                .and_then(Value::as_str)
                .ok_or("expected_current_digest missing")?;
            rollback_verification_manifest(root, expected)
                .map(|result| text(json!(result)))
                .map_err(|error| error.to_string())
        }
        "verification_manifest_approve" => {
            let approval =
                approve_verification_manifest(root, &default_verification_approval_path(root))
                    .map_err(|error| error.to_string())?;
            Ok(text(json!(approval)))
        }
        "spec_verify" => {
            let relative = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?;
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
        }
        "agent_improvement_signals" => {
            let (signals, metrics) =
                build_agent_improvement_signals(root).map_err(|error| error.to_string())?;
            Ok(text(json!({"signals": signals, "metrics": metrics})))
        }
        "agent_improvement_propose" => {
            let request: ImprovementProposalRequest =
                serde_json::from_value(args.clone()).map_err(|error| error.to_string())?;
            let path =
                create_improvement_proposal(root, &request).map_err(|error| error.to_string())?;
            Ok(text(json!({"path": path})))
        }
        "agent_improvement_apply" => {
            let path = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?;
            let result = apply_improvement_proposal(root, &root.join(path))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(result)))
        }
        _ => Err("unknown tool".into()),
    }
}

fn candidate_manifest(args: &Value) -> Result<lmbrain_core::HarnessManifest, String> {
    let candidate = args.get("manifest").ok_or("manifest missing")?;
    parse_harness_manifest(&candidate.to_string()).map_err(|error| error.to_string())
}

fn specific_status(name: &str) -> Option<&'static str> {
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

fn review_status(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "review_accept" => Some(("accepted", "operator")),
        "review_changes_requested" => Some(("changes-requested", "project-lead")),
        "review_block" => Some(("blocked", "project-lead")),
        "review_supersede" => Some(("superseded", "project-lead")),
        _ => None,
    }
}

fn review_event_action(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "review_remediation" => Some(("remediation", "implementation-specialist")),
        "review_escalate" => Some(("escalation", "operator")),
        "review_takeover" => Some(("takeover", "project-lead")),
        _ => None,
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
            "finding_create",
            "finding_plan",
            "finding_defer",
            "finding_resolve",
            "finding_accept_risk",
            "finding_supersede",
            "finding_reopen",
            "finding_context",
            "finding_candidates",
        ] {
            assert!(names.contains(&name.to_string()), "{name} missing");
        }
        assert!(names.contains(&"lmbrain_set_agent_mnemonic_name".to_string()));
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
            &dir.path().to_path_buf(),
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
            &dir.path().to_path_buf(),
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
            &dir.path().to_path_buf(),
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
    fn finding_protocol_creates_plans_and_contextualizes_without_rewriting_origin() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/reviews/accepted")).unwrap();
        std::fs::create_dir_all(dir.path().join(".lmbrain/specs/backlog")).unwrap();
        let review = dir.path().join(".lmbrain/reviews/accepted/REVIEW-054.md");
        let review_source =
            "---\nid: REVIEW-054\ntitle: Review\nstatus: accepted\n---\n- FINDING-07 debt\n";
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
                "name":"finding_create",
                "arguments":{
                    "title":"Routed debt","category":"correctness","severity":"high",
                    "origin_artifact":"REVIEW-054","origin_ref":"FINDING-07",
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
                "name":"finding_plan",
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
                "name":"finding_context","arguments":{"finding":"FINDING-001"}
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
                .pointer("/finding/status")
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
            &serde_json::json!({"name":"finding_candidates","arguments":{}}),
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
            "---\nid: SPEC-002\ntitle: Dependent\nstatus: backlog\nrecommended_agent: AGENT-IMPL\ndepends_on: []\ndependency_events: []\nparking_events: []\nupdated: 2026-07-29\n---\n",
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
    fn harness_set_validates_and_writes_without_materializing_host_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        let args = serde_json::json!({"manifest":{"schema_version":1,"hosts":{}}});
        let response = super::call(
            &dir.path().to_path_buf(),
            &serde_json::json!({"name":"harness_config_set","arguments":args}),
        )
        .unwrap();
        assert!(
            response.to_string().contains("manifest_digest")
                || response.to_string().contains("digest")
        );
        assert!(dir.path().join(".lmbrain/HARNESSES.json").exists());
        assert!(!dir.path().join("opencode.json").exists());
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
}
