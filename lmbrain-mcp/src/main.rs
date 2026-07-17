use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
};

use lmbrain_core::context::{build_project_digest, build_review_context, build_spec_context};
use lmbrain_core::transitions::{
    create, set_agent_mnemonic_name, set_recommended_agent, transition, ArtifactKind,
    CreateRequest, MutationOptions,
};
use lmbrain_core::{
    apply_improvement_proposal, approve_verification_manifest, build_agent_improvement_signals,
    canonical_manifest_digest, canonical_verification_manifest_digest, create_improvement_proposal,
    execute_spec_verification, load_harness_manifest, load_verification_manifest,
    parse_harness_manifest, set_harness_manifest, HarnessManifestError, ImprovementProposalRequest,
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
        ("review_accept", "Accept a review on explicit operator request."),
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
        create_tool(),
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
        read_tool("lmbrain_list_ready_handoffs", "List ready handoffs."),
        // V3 context-pack tools
        context_tool(
            "lmbrain_project_digest",
            "Compact project overview: title/status, current milestone, ready/review specs, blockers, ready handoffs, active decisions, diagnostics summary, and version/health warnings. Returns JSON and Markdown summary. Does not mutate artifacts.",
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
        verification_approval_tool(),
        spec_verify_tool(),
        improvement_signals_tool(),
        improvement_propose_tool(),
        improvement_apply_tool(),
    ]);

    entries
}

fn verification_manifest_tool() -> Value {
    json!({
        "name": "verification_manifest_get",
        "description": "Read and validate the versioned verification manifest and return its canonical digest. Does not execute gates.",
        "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
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

fn text(value: Value) -> Value {
    json!({"content":[{"type":"text","text":value.to_string()}]})
}

fn call(root: &PathBuf, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name missing")?;
    let args = params.get("arguments").unwrap_or(&Value::Null);

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
            "unique_ids": lmbrain_core::invariants::unique_ids(root)
        }))),
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
        "verification_manifest_approve" => {
            let approval = approve_verification_manifest(root, &verification_approval_path(root))
                .map_err(|error| error.to_string())?;
            Ok(text(json!(approval)))
        }
        "spec_verify" => {
            let relative = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or("path missing")?;
            let report = execute_spec_verification(
                root,
                &root.join(relative),
                &verification_approval_path(root),
            )
            .map_err(|error| error.to_string())?;
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

fn verification_approval_path(root: &std::path::Path) -> PathBuf {
    if let Some(path) = std::env::var_os("LMBRAIN_VERIFICATION_APPROVAL_STORE") {
        return PathBuf::from(path);
    }
    let base = std::env::var_os(if cfg!(windows) {
        "LOCALAPPDATA"
    } else {
        "XDG_DATA_HOME"
    })
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    .unwrap_or_else(std::env::temp_dir);
    let identity = lmbrain_core::workspace_identity(root)
        .map(|identity| identity.fingerprint)
        .unwrap_or_else(|_| "unknown-workspace".into());
    base.join("lmbrain/verification-approvals")
        .join(format!("{identity}.json"))
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
        "review_accept" => Some("accepted"),
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
        assert!(names.contains(&"review_accept".to_string()));
        assert!(names.contains(&"lmbrain_set_agent_mnemonic_name".to_string()));
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
                !error.contains(&lmbrain_core::path::clean_path(&canonical).display().to_string()),
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
}
