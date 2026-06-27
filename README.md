# LMBrain

LMBrain is a local-first project brain and desktop workspace for agent-assisted software work.

The repository contains three product surfaces:

- a reusable Markdown kit in `kit/.lmbrain/`;
- a Tauri desktop app in `src/` and `src-tauri/`;
- shared Rust services in `lmbrain-core/` and `lmbrain-mcp/`.

The kit is copied into a target repository as `.lmbrain/`. The desktop app reads that directory, shows project state, registers local agent tooling, and can launch interactive agent sessions. Project-specific dogfooding state for this repository is intentionally not versioned.

## Quick Start

Install dependencies:

```bash
pnpm install --frozen-lockfile
```

Run the desktop app in development mode:

```bash
pnpm tauri dev
```

Run frontend checks:

```bash
pnpm lint
pnpm test
pnpm build
```

Run Rust checks:

```bash
cargo test
```

Verify release version alignment:

```bash
node scripts/check-version.mjs
```

## Documentation

Current solution documentation lives in [`docs/`](docs/README.md):

- [Product](docs/product.md)
- [Architecture](docs/architecture.md)
- [Kit](docs/kit.md)
- [Agent Hosts](docs/agent-hosts.md)
- [Sessions](docs/sessions.md)
- [Development](docs/development.md)
- [Release](docs/release.md)

## Repository Layout

```text
kit/.lmbrain/      Reusable Markdown project-brain template bundled with the app
lmbrain-core/      Tauri-free artifact mutation and invariant engine
lmbrain-mcp/       MCP stdio server exposing controlled mutation tools to agents
src/               React frontend
src-tauri/         Tauri backend and desktop application shell
scripts/           Repository maintenance scripts
docs/              Current solution documentation
```

## Local Generated Files

LMBrain and agent hosts generate workspace-local files that are useful on a developer machine but must not be committed:

- `.lmbrain/` at the repository root, when dogfooding LMBrain on itself;
- `AGENTS.md`, generated as a Codex instruction pointer;
- `.mcp.json`, generated for Claude Code MCP registration;
- `.codex/`, generated for Codex project configuration;
- `.claude/`, host-local Claude settings.

Those paths are ignored by Git. The versioned kit remains under `kit/.lmbrain/`.
