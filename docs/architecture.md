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
- `src/components/Sessions/`: floating session windows and xterm terminal integration.
- `src/context/WorkspaceContext.tsx`: workspace state, navigation, session window state, and data refresh.

## Tauri Backend

`src-tauri/src/lib.rs` wires Tauri commands, application state, and services:

- `WorkspaceService`: validates workspaces, tracks recent repositories, initializes the bundled kit.
- `PathGuard`: scopes file access to the selected workspace.
- `FileWatcherService`: watches the workspace and emits refresh events.
- `SessionManager`: owns PTY-backed interactive sessions.

The backend reads `.lmbrain/` artifacts, parses Markdown/frontmatter, builds diagnostics, starts and stops watchers, registers MCP host files, and exposes session commands.

## Artifact Model

The app treats `.lmbrain/` as the project source of truth. Key artifact families include specs, reviews, decisions, agent profiles and proposals, MCP records/proposals, handoffs, roadmap, status, and knowledge pages.

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

It intentionally does not expose task tools or operator-only ADR acceptance tools.
