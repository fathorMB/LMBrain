# Agent Hosts

LMBrain supports external coding-agent hosts through local generated configuration and the `lmbrain-mcp` server.

All agent starts remain user-controlled. LMBrain registers tools and instruction pointers; it does not autonomously launch agents.

## Claude Code

When a workspace is opened, LMBrain writes or updates `.mcp.json` in the workspace root. The file registers the `lmbrain` MCP server:

```json
{
  "mcpServers": {
    "lmbrain": {
      "command": "lmbrain-mcp",
      "args": ["--root", "<workspace>"]
    }
  }
}
```

The command is resolved from:

1. `LMBRAIN_MCP_BIN`;
2. the bundled Tauri sidecar next to the running app executable;
3. an `lmbrain-mcp` binary next to the running app executable;
4. Cargo workspace build outputs (`target/debug/lmbrain-mcp` or
   `target/release/lmbrain-mcp`, with `.exe` on Windows);
5. `lmbrain-mcp` on `PATH`.

When a concrete binary is discovered, LMBrain writes that absolute path into
`.mcp.json`. This matters for in-app Claude sessions because they inherit the
desktop app's process environment, which may not include the same `PATH` as an
interactive shell. If LMBrain cannot discover the binary automatically, set
`LMBRAIN_MCP_BIN` before starting the app.

Installer builds bundle `lmbrain-mcp` as a Tauri sidecar, so new workspaces
should receive an absolute command path without requiring a separate PATH setup.
If automatic discovery would otherwise fall back to bare `lmbrain-mcp`, LMBrain
preserves an existing absolute `.mcp.json` command while that file still exists.

The write is best-effort and idempotent.

## Codex

LMBrain also writes project-scoped Codex configuration at `.codex/config.toml`:

```toml
[mcp_servers.lmbrain]
command = "lmbrain-mcp"
args = ["--root", "<workspace>"]
```

Because Codex only loads project config for trusted workspaces, LMBrain ensures a missing trusted-project entry in the user Codex config:

```toml
[projects."<workspace>"]
trust_level = "trusted"
```

The user config is personal. LMBrain preserves existing content and only adds a missing trust entry.

## AGENTS.md

LMBrain scaffolds a concise managed block in root `AGENTS.md` so Codex can discover the project-brain instructions. The block points to `.lmbrain/AGENT.md`, `.lmbrain/CONTRACT.md`, and `.lmbrain/QUALITY.md`.

`AGENTS.md` is local generated host state in this repository and is ignored by Git.

## V3 context-pack tools

All supported agent hosts can use the new context-pack MCP tools:

- `lmbrain_project_digest` — project overview (no parameters)
- `lmbrain_spec_context` — spec handoff context (requires `spec` parameter)
- `lmbrain_review_context` — review context (requires `spec` parameter)

These tools are read-only and registered through the same `lmbrain-mcp` server. Agents should use them for initial orientation before expanding to full artifacts.

## Local Generated Files

These files are machine-specific and should not be committed:

- `.mcp.json`
- `.codex/`
- `.claude/`
- `AGENTS.md`
- root `.lmbrain/` dogfooding state

The reusable distributed kit remains versioned at `kit/.lmbrain/`.
