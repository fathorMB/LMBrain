# Development

## Requirements

- Node.js compatible with the project dependencies.
- pnpm.
- Rust toolchain matching the workspace MSRV in `Cargo.toml`.
- Tauri platform prerequisites for the target OS.

## Install

```bash
pnpm install --frozen-lockfile
```

## Run

```bash
pnpm tauri dev
```

The Tauri dev command starts Vite with the configured dev URL and opens the desktop shell.

## Frontend Commands

```bash
pnpm lint
pnpm test
pnpm build
```

`pnpm build` runs `tsc -b` and then `vite build`.

## Rust Commands

```bash
cargo test
cargo build --release -p lmbrain-mcp
```

The Cargo workspace contains:

- `src-tauri`: desktop backend;
- `lmbrain-core`: shared mutation engine;
- `lmbrain-mcp`: MCP stdio server.

## Verification gate environment diagnostics

Verification gates start from a minimal inherited environment. Their generated transcript and structured run result include `removed_environment_variables`, a deterministic list of variable names excluded by the `minimal-inherited-allowlist` policy. Values are never recorded. On Windows names are compared case-insensitively and reported in uppercase; on other platforms their case is preserved.

When a tool reports a missing executable, SDK, or system installation, inspect this list before treating the failure as a machine-provisioning defect. A removed name identifies the minimal-environment policy as a possible cause; it does not expose the removed value or change the environment used by the gate.

## Repository Map

```text
docs/             Current solution documentation
kit/.lmbrain/     Bundled reusable kit
lmbrain-core/     Shared controlled-mutation engine
lmbrain-mcp/      Agent-facing MCP server
public/           Static web assets
scripts/          Maintenance scripts
src/              React frontend
src-tauri/        Tauri backend and app configuration
```

## Ignored Local State

The repository ignores generated local state such as `dist/`, `target/`, `node_modules/`, `.lmbrain/`, `.mcp.json`, the exact Pi MCP files `.pi/mcp.json*`, project-local Pi package cache `.pi/npm/`, `.codex/`, `.claude/`, and `AGENTS.md`. Existing `opencode.json` files are no longer ignored or modified by LMBrain and may therefore appear as untracked. `.pi/settings.json` remains visible/versionable because it may contain user-owned project configuration alongside LMBrain's approved pin.
