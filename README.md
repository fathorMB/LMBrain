# LMBrain

[![Latest release](https://img.shields.io/github/v/release/fathorMB/LMBrain?display_name=tag)](https://github.com/fathorMB/LMBrain/releases/latest)
[![Build installers](https://github.com/fathorMB/LMBrain/actions/workflows/build-installers.yml/badge.svg)](https://github.com/fathorMB/LMBrain/actions/workflows/build-installers.yml)

LMBrain is a local-first desktop workspace for planning, implementing, and reviewing software with coding agents. Project state lives in readable Markdown under `.lmbrain/`; the desktop app turns that state into a project pulse, specification board, roadmap, review system, knowledge browser, diagnostics, and controlled agent tooling.

No hosted LMBrain account or remote database is required. You choose the repository, you start every agent session, and your files remain the source of truth.

[Download the latest release](https://github.com/fathorMB/LMBrain/releases/latest) · [Read the documentation](docs/README.md) · [Report a problem](https://github.com/fathorMB/LMBrain/issues)

## What LMBrain provides

- A reusable Markdown project brain that can be initialized inside an existing repository.
- A desktop view of project status, specs, reviews, decisions, milestones, knowledge, and design artifacts.
- A governed spec lifecycle: `backlog → ready → working → review → done`.
- Repository-scoped MCP tools for controlled artifact creation, transitions, context packs, verification, and agent-profile improvements.
- Manual interactive sessions for Claude Code, Codex, Pi, and OpenCode when their CLIs are installed locally.
- Local diagnostics, review-quality metrics, verification transcripts, and migration guidance.
- No automatic agent launch, profile activation, command execution, or project migration.

## Install LMBrain

Prebuilt packages are published on the [GitHub Releases page](https://github.com/fathorMB/LMBrain/releases/latest). Current release builds target x64 Windows and x64 Linux.

### Windows

Download one of these assets from the latest release:

- `LMBrain_<version>_x64-setup.exe` — recommended interactive installer;
- `LMBrain_<version>_x64_en-US.msi` — MSI package for managed installation.

Run the installer, then launch **LMBrain** from the Start menu. Installed builds include the `lmbrain-mcp.exe` sidecar; you do not need to download the standalone MCP executable for normal desktop use.

### Debian or Ubuntu

Download `LMBrain_<version>_amd64.deb`, then install it with:

```bash
sudo apt install ./LMBrain_<version>_amd64.deb
```

Launch LMBrain from the application menu or run `lmbrain` if your desktop package exposes it on `PATH`.

### Other x64 Linux distributions

Download `LMBrain_<version>_amd64.AppImage`, make it executable, and run it:

```bash
chmod +x LMBrain_<version>_amd64.AppImage
./LMBrain_<version>_amd64.AppImage
```

If your distribution cannot run AppImages, install its FUSE compatibility package or use the source-build instructions below.

### macOS and unsupported platforms

There is currently no prebuilt macOS installer. Build from source using the platform prerequisites documented by Tauri, or use a supported Windows/Linux package.

## First run

1. Launch LMBrain and select **Choose repository folder…**.
2. Choose the local project directory you want to manage.
3. If it has no `.lmbrain/` directory, review the preview and select **Initialize LMBrain kit**.
4. Open **Pulse** and resolve any diagnostics before starting project work.
5. Commit the project-owned `.lmbrain/` Markdown artifacts when you want the brain to travel with the repository.

Initialization copies the bundled kit into a new `.lmbrain/` directory. It refuses to overwrite an existing brain and does not copy or upload your repository.

LMBrain may also create or update machine-oriented integration files such as `AGENTS.md`, `.mcp.json`, `.codex/config.toml`, `.pi/mcp.json`, and `opencode.json`. Review these files and your repository ignore policy before committing them; credentials never belong in LMBrain project manifests.

## Using LMBrain

A typical workflow is:

1. Use **Pulse**, **Roadmap**, and **Wiki** to understand the current project state.
2. Ask the Project Lead agent to turn a request into an implementation-ready `SPEC-*` artifact.
3. Approve the spec, then manually start the recommended specialist from **Sessions** or from your preferred external agent host.
4. Let the specialist implement the spec and submit real verification evidence.
5. Request a review, address findings without losing lifecycle history, and mark the spec done only after acceptance.
6. Use **Insights** and **Agents & MCP** to inspect repeated failures and propose governed profile improvements.

Markdown remains authoritative. Context packs and dashboards are derived views and never replace the underlying project artifacts.

## Optional agent hosts

The desktop app works as a project browser without an agent CLI. Interactive agent sessions require the relevant tools to already be installed and authenticated on the same machine:

| Host | Supported route | Additional requirement |
| --- | --- | --- |
| Claude Code | Native or Ollama | `claude`; Ollama for the local route |
| Codex | Native | `codex` |
| Pi | Ollama | `pi`, Ollama, and a tool-capable model |
| OpenCode | Ollama | `opencode`, Ollama, and a tool-capable model |

LMBrain detects these executables and reports their exact paths and versions under **Settings → Harnesses**. It does not install agent CLIs, models, or credentials. Updates are always explicitly initiated by the operator.

Installed LMBrain packages bundle the repository-scoped MCP sidecar and register it for supported hosts when a workspace is opened. See [Agent Hosts](docs/agent-hosts.md) and [Sessions](docs/sessions.md) for host-specific configuration and trust behavior.

## Safety model

- Agents are started manually; LMBrain does not run them in the background.
- Repository harness intent is inert until the operator approves its exact manifest digest locally.
- Named verification gates execute project code with the current user's permissions only after explicit local approval. They are governed execution, not an operating-system sandbox.
- Profile-learning signals are read-only. Applying a profile improvement requires an evidence-linked proposal, operator approval, and a non-stale target digest.
- Existing project files and customized brains are not silently migrated or overwritten.
- External agent authentication, network access, billing, and provider policies remain the responsibility of each agent host.

Use version control, inspect proposed changes, and never approve a repository manifest you do not trust.

## Troubleshooting

### An agent CLI is not detected

Open **Settings → Harnesses** and confirm the resolved executable path. Restart LMBrain after changing your user `PATH`. Codex also supports a machine-local executable override in Settings.

### The MCP server is unavailable

Use an installed desktop package whenever possible; it bundles the correct sidecar. Source builds must prepare the sidecar before packaging. External/manual setups can point `LMBRAIN_MCP_BIN` to a compatible `lmbrain-mcp` executable before starting LMBrain.

### A workspace reports a kit migration

LMBrain does not rewrite customized project brains automatically. Copy the migration prompt from Pulse, let the Project Lead prepare an additive plan from `.lmbrain/MIGRATIONS.md`, review it, and approve the changes explicitly.

### An AppImage does not start

Confirm that the file is executable and that your distribution provides AppImage/FUSE compatibility. The Debian package is preferred on Debian-derived systems.

For unresolved problems, include the LMBrain version, operating system, relevant diagnostic text, and reproduction steps in a [GitHub issue](https://github.com/fathorMB/LMBrain/issues).

## Build from source

Development requires Node.js 22, pnpm 10, the Rust toolchain declared by the workspace, and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

```bash
git clone https://github.com/fathorMB/LMBrain.git
cd LMBrain
pnpm install --frozen-lockfile
pnpm tauri dev
```

Useful verification commands:

```bash
pnpm lint
pnpm test
pnpm build
cargo test --workspace
node scripts/check-version.mjs
```

`pnpm tauri dev` opens the desktop application. Do not run it when another development instance is using the same build outputs.

## Documentation

| Guide | Contents |
| --- | --- |
| [Product](docs/product.md) | Product model, views, and local-first boundaries |
| [Kit](docs/kit.md) | `.lmbrain/` structure, lifecycle, migration, and agent taxonomy |
| [Agent Hosts](docs/agent-hosts.md) | Claude Code, Codex, Pi, OpenCode, and MCP registration |
| [Sessions](docs/sessions.md) | Interactive terminal routes and prerequisites |
| [Architecture](docs/architecture.md) | React, Tauri, Rust core, and MCP internals |
| [Development](docs/development.md) | Local development and repository layout |
| [Release](docs/release.md) | Version alignment, CI, installers, and release assets |

## Repository layout

```text
kit/.lmbrain/      Reusable project-brain kit bundled with the app
lmbrain-core/      Tauri-free artifact, verification, and governance engine
lmbrain-mcp/       Repository-scoped MCP stdio server
src/               React frontend
src-tauri/         Tauri backend and desktop shell
scripts/           Build and release maintenance scripts
docs/              Product and technical documentation
```
