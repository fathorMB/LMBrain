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

## Antigravity

Antigravity discovers MCP servers only through a user-global `mcp_config.json`;
it has no project-level MCP configuration. When a workspace is opened, LMBrain
merges the `lmbrain` entry into every Antigravity config location that already
exists:

- `~/.gemini/antigravity/mcp_config.json` — the Antigravity IDE (1.x) layout
  (`%USERPROFILE%\.gemini\antigravity\mcp_config.json` on Windows);
- `~/.gemini/config/mcp_config.json` — the Antigravity 2.0 unified CLI+IDE
  layout.

The entry uses the same stdio shape as the other hosts:

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

Behavior and limitations:

- The write is best-effort, idempotent, and merge-preserving: unrelated servers
  (including `serverUrl`-based remote entries) and unknown keys are never
  touched. When both layouts exist, both are updated so IDE and CLI stay
  consistent.
- LMBrain only writes where an Antigravity installation is already detectable
  (the config file or its parent directory exists). It never creates
  `~/.gemini` for users without Antigravity. `LMBRAIN_ANTIGRAVITY_HOME`
  overrides the home directory for tests and non-standard installs.
- Because the configuration is user-global, the single `lmbrain` entry points
  at the **most recently opened LMBrain workspace**. Opening another workspace
  re-targets the entry; only one workspace at a time is registered with
  Antigravity.
- Antigravity sessions are launched only from the Antigravity IDE. LMBrain
  adds no Antigravity session entry, does not probe or update the IDE, and the
  governed `.lmbrain/HARNESSES.json` environment does not yet include an
  Antigravity host adapter.
- Project orientation works through the root `AGENTS.md` pointer block that
  LMBrain already scaffolds; Antigravity reads `AGENTS.md` natively, so
  sessions can reach `.lmbrain/AGENT.md`, `CONTRACT.md`, `QUALITY.md`, and the
  context-pack MCP tools without further setup.
- No project-local file is generated for Antigravity, so there is nothing new
  to gitignore.

## User-level harness lifecycle

The Settings → Harnesses tab manages only the agent CLI itself, not project packages or authentication. It probes the exact resolved executable with `--version` and exposes these explicitly confirmed self-update commands:

- Claude Code: `claude update`
- Codex: `codex update`
- Pi: `pi update --self --no-approve`

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

## Governed project harness environment

LMBrain 2.8 adds the optional `.lmbrain/HARNESSES.json` source of project intent. Its strict schema permits enabled hosts, portable required-tool identifiers, non-secret environment values, and supported LSP requirements. It rejects unknown fields, secret-like keys, commands, scripts, hooks, absolute paths, traversal, oversized input, and host-incompatible capabilities.

The Environment page in the sidebar shows the effective configuration, deterministic native-file plan, approval state, and drift — strictly read-only. Since 4.0.2 (#87) the Project Lead manages the whole lifecycle through the MCP server: `harness_config_set` proposes the manifest, `harness_plan_preview` shows the exact native-file plan, `harness_manifest_approve` approves the previewed canonical digest for this machine/workspace identity, `harness_config_apply` materializes it, and `harness_approval_revoke` withdraws the approval. Apply uses a shared mutation lock, staged multi-file replacement, structural ownership, rollback, and machine-local applied-content hashes for drift detection (`harness_drift_status`). Approve and apply are digest-bound — a manifest that changed since the preview is refused — and every action is audited with its actor in `.lmbrain/HARNESSES.audit.jsonl`.

## Governed browser capability (phase 1)

`HARNESSES.json` host configurations for Claude Code may declare a typed, allow-listed browser capability:

```json
{ "browser_mcp": { "provider": "playwright", "mode": "isolated", "headed": true } }
```

The schema accepts only the `playwright` provider and `isolated` mode; commands, arguments, URLs, environment variables, and browser-profile paths are rejected as unknown fields. Host adapters derive a fixed `.mcp.json` entry (`mcpServers.lmbrain-browser`) from the profile — `node node_modules/@playwright/mcp/cli.js --isolated --browser chromium` (`--headless` appended when `headed` is `false`) — and never serialize agent-supplied strings.

**Operator provisioning is a prerequisite; LMBrain never installs anything.** Before approving a manifest that declares the capability:

```bash
npm install --save-dev --save-exact @playwright/mcp
```

```bash
npx playwright install chromium
```

The plan preview reports discovery-only readiness (package presence and version under the project-local `node_modules`, and a best-effort Chromium runtime probe honoring `PLAYWRIGHT_BROWSERS_PATH`). A missing prerequisite marks the capability `failed` and the host not ready; absence is never reported as active. The same digest approval, atomic apply, preservation, rollback, and drift rules that govern the rest of the manifest apply to the browser entry, and dropping the capability from an approved manifest removes only the LMBrain-owned `lmbrain-browser` entry.

Privacy boundary: the first profile always runs an isolated browser context. It never attaches to the operator's personal browser or profile, never exposes a remote-debugging endpoint, and never injects secrets. Chrome DevTools MCP and built-in-browser attachment remain out of scope for this phase. Hosts other than Claude Code reject the capability in phase 1.

## AGENTS.md

LMBrain scaffolds a concise managed block in root `AGENTS.md` so Codex can discover the project-brain instructions. The block points to `.lmbrain/AGENT.md`, `.lmbrain/CONTRACT.md`, and `.lmbrain/QUALITY.md`.

`AGENTS.md` is local generated host state in this repository and is ignored by Git.

## V3 context-pack tools

All MCP-enabled agent hosts (Claude Code, Codex, Antigravity, and a
correctly provisioned Pi session) can use the new context-pack MCP tools:

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
- `AGENTS.md`
- root `.lmbrain/` dogfooding state

The reusable distributed kit remains versioned at `kit/.lmbrain/`.
