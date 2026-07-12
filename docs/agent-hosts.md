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

## Pi through Ollama

Pi sessions are launched only through the operator-selected local Ollama
daemon using `ollama launch pi --model <model>`. LMBrain does not install or
upgrade Pi, Ollama, or models. Pi reads the same root `AGENTS.md` instruction
pointer as other hosts.

Pi's core distribution does not include an MCP client. During visible workspace
preparation, LMBrain therefore checks and, only when missing, installs the
operator-approved exact project-local package pin:

```text
pi install npm:pi-mcp-extension@1.5.0 -l --approve
```

The command never targets global settings and never selects an unpinned version.

## OpenCode through Ollama

OpenCode sessions run `opencode <workspace> --model ollama/<model>` with a
session-scoped inline provider pointing to `http://localhost:11434/v1`. LMBrain requires
the `opencode` executable to already be present, preventing session startup from
becoming an implicit installation flow. OpenCode supports MCP natively; LMBrain
safely merges only `mcp.lmbrain` into project-local `opencode.json`, preserving
unrelated provider, permission, agent, and MCP configuration. No OpenCode
package, credential, global config, or permission policy is installed or changed.

LMBrain passes the absolute workspace as OpenCode's project positional and also
sets the child cwd, avoiding nested-launcher process-state ambiguity. Generated
configuration also adds `lsp: true` when the key is absent. Explicit `lsp: false`
and custom LSP objects remain operator-owned and are never overwritten. Built-in
OpenCode LSPs may download supported servers into the OpenCode user cache;
operators can disable that upstream behavior with `OPENCODE_DISABLE_LSP_DOWNLOAD`.
LMBrain also adds a non-destructive `references.workspace` entry when absent, so
`@workspace/` provides deterministic project-file autocomplete even when the
OpenCode TUI prioritizes agent mentions in its bare `@` popup.

## User-level harness lifecycle

The Local Harnesses page manages only the agent CLI itself, not project packages or authentication. It probes the exact resolved executable with `--version` and exposes these explicitly confirmed self-update commands:

- Claude Code: `claude update`
- Codex: `codex update`
- Pi: `pi update --self --no-approve`
- OpenCode: `opencode upgrade`

LMBrain passes fixed argv directly to the resolved executable, runs outside the workspace, never elevates privileges, and never guesses npm/Homebrew/native ownership. Only one update may run at a time, and sessions using the selected host must be closed first. A zero updater exit is not sufficient: LMBrain probes the executable again and reports the verified before/after version and path. Missing harnesses receive official installation guidance but are not installed automatically.

Pi's project-local pinned MCP extension is a separate integration dependency. Updating Pi itself never updates project extensions or changes `.pi/settings.json`.
LMBrain first verifies both project `.pi/settings.json` and an offline `pi list`,
so an already-ready project is not reinstalled. Installation failure does not
block workspace access: Pulse opens with a persistent Pi warning. LMBrain also
safely merges only `mcpServers.lmbrain` into generated `.pi/mcp.json`, preserving
unrelated servers and settings. Immediately before Pi PTY creation it repeats a
defensive offline readiness check.

Pi sessions run with `PI_OFFLINE=1`, `PI_SKIP_VERSION_CHECK=1`, and
`PI_TELEMETRY=0` so session startup cannot install/update Pi packages or perform
Pi update/telemetry network operations. Model traffic still goes through the
operator's Ollama daemon; cloud-backed Ollama models remain remote inference.

Troubleshooting starts with the non-mutating checks: confirm `ollama` and `pi` are on
the desktop app's `PATH`, confirm `http://localhost:11434/api/tags` lists the
selected model with `tools`, and run `pi list` in the workspace to inspect the
exact package pin. To roll back Pi support for a project, close Pi sessions and
run `pi remove npm:pi-mcp-extension -l`; the next workspace open will reinstall
the approved dependency unless automatic preparation is removed by policy. The
generated `.pi/mcp.json` contains no credential and may be deleted while LMBrain
is closed, although workspace open will recreate it.

## AGENTS.md

LMBrain scaffolds a concise managed block in root `AGENTS.md` so Codex can discover the project-brain instructions. The block points to `.lmbrain/AGENT.md`, `.lmbrain/CONTRACT.md`, and `.lmbrain/QUALITY.md`.

`AGENTS.md` is local generated host state in this repository and is ignored by Git.

## V3 context-pack tools

All MCP-enabled agent hosts (Claude Code, Codex, OpenCode, and a correctly
provisioned Pi session) can use the new context-pack MCP tools:

- `lmbrain_project_digest` — project overview (no parameters)
- `lmbrain_spec_context` — spec handoff context (requires `spec` parameter)
- `lmbrain_review_context` — review context (requires `spec` parameter)

These tools are read-only and registered through the same `lmbrain-mcp` server. Spec and review context include applicable active `SKILL-*` procedures when the project defines them. Agents should use them for initial orientation before expanding to full artifacts.

`lmbrain-mcp` also exposes skill lifecycle tools:

- `skill_activate`
- `skill_retire`

Skills are documented procedures, not executable MCP tools. The MCP server does not run skill commands automatically.

## Local Generated Files

These files are machine-specific and should not be committed:

- `.mcp.json`
- `.codex/`
- `.claude/`
- `opencode.json`
- `AGENTS.md`
- root `.lmbrain/` dogfooding state

The reusable distributed kit remains versioned at `kit/.lmbrain/`.
