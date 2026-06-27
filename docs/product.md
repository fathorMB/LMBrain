# Product

LMBrain is a local Markdown project brain paired with a desktop visualizer and agent tooling.

The main product idea is simple: a repository owns its project state as Markdown files under `.lmbrain/`, and LMBrain provides a local desktop app plus controlled mutation tools so humans and coding agents can work against that state without introducing a database or hosted service.

## What Ships

- `kit/.lmbrain/`: the reusable project-brain template copied into target repositories.
- Desktop app: a Tauri 2 application with a React 19 frontend.
- `lmbrain-core`: a Rust crate for controlled artifact creation, transitions, path safety, frontmatter editing, and invariants.
- `lmbrain-mcp`: an MCP stdio server that exposes safe mutation verbs to agent hosts.
- CI release pipeline: version alignment, tests, installer builds, MCP binary artifacts, and GitHub Release publishing.

## Core Workflow

1. A user copies or initializes the kit into a target repository.
2. The user opens that repository in LMBrain.
3. LMBrain reads `.lmbrain/` and shows project pulse, wiki, board, roadmap, reviews, decisions, agents, and MCP state.
4. LMBrain registers local agent tooling for supported hosts.
5. The user manually starts agents or sessions when needed.

LMBrain does not automatically start agents and does not require a remote service.

## Main Views

- Pulse: current project state, diagnostics, and recommended actions.
- Wiki: file tree and Markdown rendering for the `.lmbrain/` workspace.
- Board: specifications grouped by status.
- Roadmap: milestones parsed from `.lmbrain/ROADMAP.md`.
- Reviews and Decisions: project governance artifacts.
- Agents & MCP: agent profiles, proposals, MCP records, and built-in MCP tools.
- Sessions: floating interactive terminals for supported agent CLIs.
- Settings: local preferences and agent binary paths.

## Local-First Boundaries

LMBrain reads and writes local files selected by the user. Repository state remains versionable Markdown. Generated host configuration such as `.mcp.json`, `.codex/`, and `AGENTS.md` is workspace-local and machine-specific, so it is ignored in this repository.
