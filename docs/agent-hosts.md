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
2. an `lmbrain-mcp` binary next to the running app executable;
3. `lmbrain-mcp` on `PATH`.

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

## Local Generated Files

These files are machine-specific and should not be committed:

- `.mcp.json`
- `.codex/`
- `.claude/`
- `AGENTS.md`
- root `.lmbrain/` dogfooding state

The reusable distributed kit remains versioned at `kit/.lmbrain/`.
