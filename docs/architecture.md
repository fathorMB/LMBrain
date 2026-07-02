# Architecture

LMBrain is split into a TypeScript frontend, a Tauri backend, and two Rust workspace crates.

## Workspace Crates

```text
src/           React frontend
src-tauri/     Tauri application backend
lmbrain-core/  Shared artifact mutation engine
lmbrain-mcp/   MCP stdio server for agents
```

The root `Cargo.toml` defines a workspace containing `src-tauri`, `lmbrain-core`, and `lmbrain-mcp`.

## Frontend

The frontend is React 19 with Vite and TypeScript. It uses inline component styling and a central `WorkspaceContext` for app state. Tauri commands are wrapped in `src/lib/commands.ts`.

Important areas:

- `src/components/Layout/`: shell, sidebar, top bar, modal surfaces.
- `src/components/Pulse/`: project pulse and recommended actions.
- `src/components/Wiki/`: Markdown tree and page viewer.
- `src/components/Design/`: design mockup browser and isolated HTML preview.
- `src/components/Taskboard/`: spec board.
- `src/components/Sessions/`: tab-based session workspace with xterm terminal integration.
- `src/context/WorkspaceContext.tsx`: workspace state, navigation, session tab state (`SessionInfo[]`, `activeSessionId`), and data refresh.

## Tauri Backend

`src-tauri/src/lib.rs` wires Tauri commands, application state, and services:

- `WorkspaceService`: validates workspaces, tracks recent repositories, initializes the bundled kit, and compares project kit and bundled kit versions using semver logic to derive the workspace migration status (`KitMigrationStatus`).
- `PathGuard`: scopes file access to the selected workspace.
- `FileWatcherService`: watches the workspace and emits refresh events.
- `SessionManager`: owns PTY-backed interactive sessions.

The backend reads `.lmbrain/` artifacts, parses Markdown/frontmatter, builds diagnostics, starts and stops watchers, registers MCP host files, and exposes session commands.

## Artifact Model

The app treats `.lmbrain/` as the project source of truth. Key artifact families include specs, reviews, decisions, agent profiles and proposals, MCP records/proposals, handoffs, roadmap, status, and knowledge pages.

### Agent profiles (v3 specialization metadata)

Agent profiles now support optional specialization fields for granular role targeting:

- `domains` — list of domain tags (e.g. `[frontend, ui, react]`)
- `primary_files` — list of file globs the agent typically works with
- `review_focus` — list of review focus areas (e.g. `[accessibility, path-safety]`)
- `context_pack` — preferred context-pack tool (`spec` or `review`)
- `constraints` — list of operational constraints

All fields are optional. Existing v2 profiles without these fields continue to parse correctly.

### Agent proposals (v3 improvement loop)

Agent proposals now support a `proposal_type` field (`new-profile` or `improvement`) and a `target_profile` field for improvement proposals targeting existing profiles. Improvement proposals follow the same lifecycle as new-profile proposals but require operator approval before behavior-affecting changes become active.

### Milestone intelligence (v3)

The milestone intelligence view is backed by `get_milestone_overview`, a Tauri command that joins data from `ROADMAP.md`, `build_specs`, `build_reviews`, and `build_adrs` to produce a derived `MilestoneOverview`. The overview includes per-milestone spec counts by status, linked specs with metadata, linked reviews, linked decisions, unresolved reference warnings, dependency status, and a recommended next action.

The derived data is produced by `contract::build_milestone_overview` and returned as a new `MilestoneOverview` model (not an extension of the existing `Roadmap` model, preserving backward compatibility). The frontend renders it as a sidebar/detail layout with click-through artifact navigation.

Design mockups under `.lmbrain/design/` are regular files rather than managed lifecycle artifacts. The backend scans that subtree with the same workspace path guard used for other local reads, and the frontend previews HTML with an isolated iframe surface.

Specs are the board unit. Current spec statuses are:

```text
backlog -> ready -> working -> review -> done
discarded
```

Tasks are not a first-class board artifact in the current product.

## Controlled Mutation Core

`lmbrain-core` is Tauri-free. It contains the controlled mutation logic shared by the app and the MCP server:

- artifact kind and path mapping;
- frontmatter parsing and surgical updates;
- atomic file writes and moves;
- state transitions;
- field setters;
- creation with allocated IDs;
- invariant checks.

Agents should use the MCP tools backed by this core instead of editing managed frontmatter by hand.

## MCP Server

`lmbrain-mcp` is a JSON-RPC stdio server. It supports initialization, `tools/list`, and `tools/call`. It accepts a workspace root from `--root`, `--root=<path>`, `LMBRAIN_ROOT`, or current working directory.

The server exposes specific tools such as:

- `spec_ready`, `spec_start`, `spec_submit`, `spec_done`, `spec_discard`;
- `review_accept`;
- `lmbrain_create`;
- `lmbrain_set_recommended_agent`;
- `lmbrain_get_artifact`;
- `lmbrain_validate`;
- `lmbrain_list_ready_handoffs`.

### V3 context-pack tools (added in kit 2.2.7)

- `lmbrain_project_digest` — compact project overview: title/status, current milestone, ready/review specs, blockers, ready handoffs, active decisions, diagnostics summary, and version/health warnings. Returns JSON and Markdown summary. No required parameters.
- `lmbrain_spec_context` — spec handoff context: spec metadata, acceptance criteria checklist, linked decisions, recommended agent profile summary, related reviews, referenced milestone, explicit files/areas, and diagnostics affecting the handoff. Returns JSON and Markdown summary. Requires `spec` parameter (ID or path).
- `lmbrain_review_context` — review context: acceptance criteria, implementation evidence, linked accepted/proposed reviews, relevant decisions, and verification commands claimed by the specialist. Returns JSON and Markdown summary. Requires `spec` parameter (ID or path).

All context-pack tools are read-only. They resolve references through existing ID/path logic and report missing links as structured warnings. They are backed by `lmbrain-core/src/context.rs`.

It intentionally does not expose task tools or operator-only ADR acceptance tools.
