use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
};

use lmbrain_core::context::{build_project_digest, build_review_context, build_spec_context};
use lmbrain_core::transitions::{
    create, set_recommended_agent, transition, ArtifactKind, CreateRequest, MutationOptions,
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
        ("spec_start", "Start a ready spec (begin implementation)."),
        ("spec_submit", "Submit a working spec for review."),
        ("spec_done", "Mark a reviewed spec done."),
        (
            "spec_discard",
            "Discard a spec (requires operator approval).",
        ),
        ("review_accept", "Accept a review."),
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
    ]);

    entries
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
                "kind":{"type":"string","enum":["spec","review","adr","agent","agent-proposal","mcp","mcp-proposal","handoff"]},
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
            let fields = args
                .get("fields")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|pair| {
                            Some((
                                pair.get(0)?.as_str()?.to_owned(),
                                pair.get(1)?.as_str()?.to_owned(),
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();

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
        "lmbrain_get_artifact" => {
            let source = std::fs::read_to_string(
                root.join(
                    args.get("path")
                        .and_then(Value::as_str)
                        .ok_or("path missing")?,
                ),
            )
            .map_err(|error| error.to_string())?;
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
        _ => Err("unknown tool".into()),
    }
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
        assert!(!names.iter().any(|name| name.starts_with("task_")));
        assert!(names.contains(&"spec_done".to_string()));
        assert!(names.contains(&"spec_discard".to_string()));
        assert!(names.contains(&"review_accept".to_string()));
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
