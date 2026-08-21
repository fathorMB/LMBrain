---
id: ADR-007
title: Codex support through project-scoped MCP registration
status: proposed
decision_date:
decider:
supersedes: []
superseded_by: []
links: [ADR-001, ADR-004, ADR-006]
tags: [architecture, codex, mcp, sessions, trust]
---

# Codex support through project-scoped MCP registration

## Context

LMBrain already exposes its controlled-mutation engine to Claude Code by writing a repository-scoped `.mcp.json` when a workspace is opened. Codex uses a different host configuration shape: MCP servers are read from Codex TOML config, and project-local config is only honored for trusted workspaces.

Supporting Codex extends LMBrain's local-first desktop surface in two ways:

- LMBrain writes another generated file at the repository root: `.codex/config.toml`.
- LMBrain may write a missing trusted-project entry to the user's Codex config at `$CODEX_HOME/config.toml` (default `~/.codex/config.toml`).

The user-level Codex config is personal and may contain models, plugins, MCP servers, and project trust entries unrelated to LMBrain. It must not be rewritten wholesale or treated as an LMBrain-owned file.

## Decision

LMBrain will support Codex as a peer agent host to Claude Code.

- On workspace open, LMBrain writes or merges `.codex/config.toml` with `[mcp_servers.lmbrain]`, using the same `lmbrain-mcp --root <workspace>` server as Claude Code.
- LMBrain ensures the workspace is trusted in the user Codex config by adding only a missing `[projects.<path>] trust_level = "trusted"` entry.
- Existing Codex user config content is preserved with a TOML-preserving merge. Existing trusted entries are detected case-insensitively on Windows to avoid duplicate project trust keys.
- Codex sessions launch native `codex` only. LMBrain does not support `ollama launch codex` in this version.
- Root `AGENTS.md` receives a small LMBrain-managed pointer block so Codex can discover `.lmbrain/AGENT.md`, `CONTRACT.md`, and `QUALITY.md` without duplicating those documents.

## Consequences

### Positive

- Claude Code and Codex use the same controlled-mutation tools and `.lmbrain/` workflow.
- Codex configuration stays project-scoped instead of registering LMBrain globally.
- The user remains in control of starting agent sessions.

### Constraints

- LMBrain now has a narrow, documented write surface outside the repository: adding Codex trust when missing.
- `.codex/config.toml` is generated and machine-specific because it can contain absolute binary and workspace paths.
- Codex binary discovery is part of the supported Windows runtime surface because Desktop Codex installs under a versioned hash directory and may not be on `PATH`.

## Alternatives considered

### `codex mcp add`

Rejected because it writes host configuration through Codex's global registration path, while LMBrain needs a project-scoped registration tied to the opened workspace.

### A separate Codex MCP server

Rejected because the existing `lmbrain-mcp` stdio JSON-RPC server already expresses the required controlled-mutation surface.

### No user config trust write

Rejected because Codex ignores project-local config for untrusted workspaces. Without ensuring trust, the project registration can silently fail to load.

## Review conditions

Revisit if Codex changes project-scoped MCP loading, trust storage, or MCP approval behavior in a way that makes the generated `.codex/config.toml` or trust merge unreliable.
