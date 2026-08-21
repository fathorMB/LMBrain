use std::io::{self, BufRead};
use serde_json::Value;

fn main() {
    let root = lmbrain_mcp::resolve_root(
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
                lmbrain_mcp::reply(Value::Null, Err(format!("invalid JSON: {error}")));
                continue;
            }
        };

        if let Some(id) = request.get("id").cloned() {
            lmbrain_mcp::reply(id, lmbrain_mcp::handle(&root, &request));
        } else {
            let _ = lmbrain_mcp::handle(&root, &request);
        }
    }
}
