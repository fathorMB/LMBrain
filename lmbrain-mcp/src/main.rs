use std::{io::{self, BufRead, Write}, path::PathBuf};
use lmbrain_core::transitions::{create, set_recommended_agent, transition, ArtifactKind, CreateRequest, MutationOptions};
use serde_json::{json, Value};

fn main() { let root=resolve_root(std::env::args().skip(1), std::env::var("LMBRAIN_ROOT").ok(), std::env::current_dir().ok()); for line in io::stdin().lock().lines().map_while(Result::ok) { if line.trim().is_empty(){continue}; let request:Value=match serde_json::from_str(&line){Ok(v)=>v,Err(e)=>{reply(Value::Null,Err(format!("invalid JSON: {e}")));continue}}; let id=request.get("id").cloned().unwrap_or(Value::Null); reply(id,handle(&root,&request)); } }

/// Resolve the workspace root: explicit `--root <path>`/`--root=<path>` wins, then
/// `LMBRAIN_ROOT`, then the launch directory. This removes the dependency on how the
/// agent host sets the server's working directory.
fn resolve_root(args: impl Iterator<Item = String>, env: Option<String>, cwd: Option<PathBuf>) -> PathBuf {
    let mut args = args;
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--root=") { return PathBuf::from(value); }
        if arg == "--root" { if let Some(value) = args.next() { return PathBuf::from(value); } }
    }
    if let Some(value) = env.filter(|v| !v.trim().is_empty()) { return PathBuf::from(value); }
    cwd.unwrap_or_else(|| PathBuf::from("."))
}
fn reply(id:Value,result:Result<Value,String>){let response=match result{Ok(value)=>json!({"jsonrpc":"2.0","id":id,"result":value}),Err(message)=>json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":message}})};println!("{response}");let _=io::stdout().flush();}
fn handle(root:&PathBuf,request:&Value)->Result<Value,String>{match request.get("method").and_then(Value::as_str){Some("initialize")=>Ok(json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"lmbrain-mcp","version":env!("CARGO_PKG_VERSION")}})),Some("tools/list")=>Ok(json!({"tools":tools()})),Some("tools/call")=>call(root,request.get("params").unwrap_or(&Value::Null)),_=>Err("method not found".into())}}
fn tools()->Vec<Value>{let mut entries=vec![];for(name,description)in[("spec_ready","Approve a backlog spec to ready (on operator request)."),("spec_start","Start a ready spec (begin implementation)."),("spec_submit","Submit a working spec for review."),("spec_done","Mark a reviewed spec done."),("spec_discard","Discard a spec (requires operator approval)."),("review_accept","Accept a review.")]{entries.push(transition_tool(name,description));}entries.extend([create_tool(),setter_tool("lmbrain_set_recommended_agent","Set a spec recommended agent.","agent"),read_tool("lmbrain_get_artifact","Read a repository artifact."),read_tool("lmbrain_validate","Validate controlled-mutation invariants."),read_tool("lmbrain_list_ready_handoffs","List ready handoffs.")]);entries}
fn transition_tool(name:&str,description:&str)->Value{json!({"name":name,"description":description,"inputSchema":{"type":"object","required":["path"],"properties":{"path":{"type":"string","description":"Artifact path relative to repository root."},"force":{"type":"boolean","default":false},"reason":{"type":"string","description":"Required only when force is true."}},"additionalProperties":false}})}
fn create_tool()->Value{json!({"name":"lmbrain_create","description":"Create an artifact with an allocated ID.","inputSchema":{"type":"object","required":["kind","title"],"properties":{"kind":{"type":"string","enum":["spec","review","adr","agent","agent-proposal","mcp","mcp-proposal","handoff"]},"title":{"type":"string"},"status":{"type":"string"},"fields":{"type":"array","items":{"type":"array","items":{"type":"string"},"minItems":2,"maxItems":2}}},"additionalProperties":false}})}
fn setter_tool(name:&str,description:&str,field:&str)->Value{json!({"name":name,"description":description,"inputSchema":{"type":"object","required":["path",field],"properties":{"path":{"type":"string"},field:{"type":"string"},"force":{"type":"boolean","default":false},"reason":{"type":"string","description":"Required only when force is true."}},"additionalProperties":false}})}
fn read_tool(name:&str,description:&str)->Value{let schema=if name=="lmbrain_get_artifact"{json!({"type":"object","required":["path"],"properties":{"path":{"type":"string"}},"additionalProperties":false})}else{json!({"type":"object","properties":{},"additionalProperties":false})};json!({"name":name,"description":description,"inputSchema":schema})}
fn opts(args:&Value)->MutationOptions{MutationOptions{force:args.get("force").and_then(Value::as_bool).unwrap_or(false),reason:args.get("reason").and_then(Value::as_str).map(str::to_owned)}}
fn text(value:Value)->Value{json!({"content":[{"type":"text","text":value.to_string()}]})}
fn call(root:&PathBuf,params:&Value)->Result<Value,String>{let name=params.get("name").and_then(Value::as_str).ok_or("tool name missing")?;let args=params.get("arguments").unwrap_or(&Value::Null);if let Some(status)=specific_status(name){return transition(root,args.get("path").and_then(Value::as_str).ok_or("path missing")?,status,opts(args)).map(|result|text(json!(result))).map_err(|error|error.to_string())}match name{"lmbrain_create"=>{let kind=serde_json::from_value::<ArtifactKind>(args.get("kind").cloned().ok_or("kind missing")?).map_err(|e|e.to_string())?;let title=args.get("title").and_then(Value::as_str).ok_or("title missing")?.to_owned();let fields=args.get("fields").and_then(Value::as_array).map(|items|items.iter().filter_map(|pair|Some((pair.get(0)?.as_str()?.to_owned(),pair.get(1)?.as_str()?.to_owned()))).collect()).unwrap_or_default();create(root,CreateRequest{kind,title,status:args.get("status").and_then(Value::as_str).map(str::to_owned),fields}).map(|result|text(json!(result))).map_err(|error|error.to_string())},"lmbrain_set_recommended_agent"=>set_recommended_agent(root,args.get("path").and_then(Value::as_str).ok_or("path missing")?,args.get("agent").and_then(Value::as_str).ok_or("agent missing")?,opts(args)).map(|result|text(json!(result))).map_err(|error|error.to_string()),"lmbrain_get_artifact"=>{let source=std::fs::read_to_string(root.join(args.get("path").and_then(Value::as_str).ok_or("path missing")?)).map_err(|e|e.to_string())?;Ok(text(json!({"artifact":source})))},"lmbrain_list_ready_handoffs"=>{let paths=std::fs::read_dir(root.join(".lmbrain/handoffs/active")).map_err(|e|e.to_string())?.flatten().filter_map(|entry|{let path=entry.path();let source=std::fs::read_to_string(&path).ok()?;if source.contains("status: ready"){path.file_name().map(|name|name.to_string_lossy().to_string())}else{None}}).collect::<Vec<_>>();Ok(text(json!({"handoffs":paths})))},"lmbrain_validate"=>Ok(text(json!({"unique_ids":lmbrain_core::invariants::unique_ids(root)}))),_=>Err("unknown tool".into() )}}
fn specific_status(name:&str)->Option<&'static str>{match name{"spec_ready"=>Some("ready"),"spec_start"=>Some("working"),"spec_submit"=>Some("review"),"spec_done"=>Some("done"),"spec_discard"=>Some("discarded"),"review_accept"=>Some("accepted"),_=>None}}

#[cfg(test)]
mod tests {
    use super::resolve_root;
    use std::path::PathBuf;
    fn args(items: &[&str]) -> std::vec::IntoIter<String> { items.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter() }

    #[test]
    fn flag_space_form_wins() {
        let root = resolve_root(args(&["--root", "/ws"]), Some("/env".into()), Some(PathBuf::from("/cwd")));
        assert_eq!(root, PathBuf::from("/ws"));
    }
    #[test]
    fn flag_equals_form_wins() {
        let root = resolve_root(args(&["--root=/ws"]), Some("/env".into()), Some(PathBuf::from("/cwd")));
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
    fn agent_tools_exclude_operator_only_and_tasks() {
        // Accepting ADRs is operator-only; tasks no longer exist.
        assert!(super::specific_status("adr_accept").is_none());
        assert!(super::specific_status("task_start").is_none());
        let names: Vec<String> = super::tools()
            .iter()
            .filter_map(|tool| tool.get("name").and_then(|name| name.as_str()).map(str::to_string))
            .collect();
        assert!(!names.contains(&"adr_accept".to_string()), "adr_accept must not be exposed");
        assert!(!names.iter().any(|n| n.starts_with("task_")), "no task tools must remain");
        // The spec board verbs are present.
        assert!(names.contains(&"spec_done".to_string()));
        assert!(names.contains(&"spec_discard".to_string()));
        assert!(names.contains(&"review_accept".to_string()));
    }
}
