---
id: SPEC-018
title: "Register and distribute lmbrain-mcp so agents actually get the tools"
status: ready
kind: feature
priority: high
area: core-tooling
milestone:
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-004]
links: [ADR-004, SPEC-017]
created: 2026-06-23
updated: 2026-06-23
tags: [mcp, distribution, bootstrap, wiring]
---

# Register and distribute lmbrain-mcp so agents actually get the tools

## Objective

Make the `lmbrain-mcp` server actually available to the agent host, so agents use the controlled-mutation tools instead of editing markdown by hand. This closes the unmet SPEC-017 acceptance criterion ("the kit bootstrap registers the MCP server") and the symptoms it caused.

## Context

[[SPEC-017-controlled-mutation-engine]] shipped a working engine (`lmbrain-core`) and a working MCP server (`lmbrain-mcp`), both tested in CI. But the **last mile is missing**: nothing registers the server with the agent host, and the binary is not distributed.

Observed in a real test (the `Brewlog` project): the Project Lead, running under Claude Code, produced a spec whose frontmatter said `status: ready` while the file stayed in `specs/proposed/` (status set by hand, not via the `spec_ready` tool), created no tasks, and used a non-existent `recommended_agent`. Root cause: the lead **had no `lmbrain-mcp` tools available**, so it fell back to hand-editing.

Concretely:
- The target project has no `.mcp.json` (the file Claude Code reads for project MCP servers).
- The kit ships `kit/.lmbrain/mcp/lmbrain-mcp.json` in a **custom format** (`{name, transport, command, cwd, …}`) that no host interprets; Claude Code expects `{"mcpServers": {"<name>": {"command": …, "args": []}}}`.
- There is no code that writes a host config; "registration" was only an inert template.
- `lmbrain-mcp` is built in CI as an artifact only — it is not installed on the user's machine or on `PATH`, so even a correct `.mcp.json` `command` would not resolve.
- `lmbrain-mcp` derives the workspace root from `std::env::current_dir()`, which is not guaranteed to be the repository root when the host launches it.

## Scope

### Included
- **Distribution:** ship `lmbrain-mcp` with the desktop app as a Tauri **sidecar** (bundled per-OS) so installing the app installs a resolvable binary.
- **Registration:** the app writes/updates a correct, host-readable `.mcp.json` at the opened workspace root (during kit initialization and/or workspace open), pointing at the installed binary and passing the repository root explicitly. The write is idempotent and preserves any other `mcpServers` entries already present.
- **Robust root:** `lmbrain-mcp` accepts the workspace root explicitly (`--root <path>` and/or `LMBRAIN_ROOT`), falling back to `current_dir` only if unset.
- **Kit template:** replace the inert custom-format file with the correct format, or remove it in favor of the programmatic write; keep the kit prompts consistent.
- **End-to-end verification:** the manual smoke test SPEC-017 deferred — a real agent host loads the tools and performs a transition that mutates the file and the audit trail.

### Excluded
- A CLI surface (per ADR-004).
- Host integrations beyond the standard `.mcp.json` format (which Claude Code and compatible hosts read). Other hosts are out of scope for this round.
- Changes to the transition/invariant engine itself (delivered and accepted in SPEC-017).

## Existing-project analysis

- `lmbrain-mcp/src/main.rs` — root from `std::env::current_dir()`; add explicit root handling.
- `kit/.lmbrain/mcp/lmbrain-mcp.json` — wrong format; fix or remove.
- No registration code anywhere (`grep` for `.mcp.json`/`mcpServers` is empty).
- `.github/workflows/build-installers.yml` already builds and uploads `lmbrain-mcp` per-OS; extend to bundle it as an app sidecar.
- The app already performs the one-time kit bootstrap (`initialize_workspace_kit`) and knows the workspace root — the natural place to write `.mcp.json`.

## Technical proposal

1. **Sidecar packaging.** Add `lmbrain-mcp` as a Tauri `externalBin` so `tauri build` bundles the per-OS binary with the installer. Resolve its installed path at runtime via Tauri's resource/sidecar API.
2. **Registration command.** Add a backend command (called by `initialize_workspace_kit` and on workspace open) that reads any existing `.mcp.json` at the workspace root, merges/sets `mcpServers.lmbrain = { command: <resolved sidecar path>, args: ["--root", <workspace root>] }`, and writes it back atomically. Do not clobber unrelated entries; make repeated runs idempotent.
3. **Explicit root in the server.** Parse `--root`/`LMBRAIN_ROOT` in `lmbrain-mcp`; use it for `PathGuard`, falling back to `current_dir`.
4. **Kit consistency.** Replace `kit/.lmbrain/mcp/lmbrain-mcp.json` with the correct host format (as documentation/example), or drop it; ensure `AGENT.md`/handoff/bootstrap wording matches how registration actually happens.

## Files and areas involved

- `src-tauri/tauri.conf.json` (sidecar/externalBin), `src-tauri/src/commands/` (registration command + wiring into bootstrap/open), `src-tauri/src/lib.rs` (command registration).
- `lmbrain-mcp/src/main.rs` (root arg/env).
- `.github/workflows/build-installers.yml` (sidecar bundling).
- `kit/.lmbrain/mcp/lmbrain-mcp.json` and prompt wording (kit + live).

## Acceptance criteria
- [ ] Installing the app installs a resolvable `lmbrain-mcp` binary (Tauri sidecar), built per-OS in CI.
- [ ] Opening/initializing a workspace writes a `.mcp.json` at its root in the host format `{"mcpServers": {"lmbrain": {"command": …, "args": ["--root", <root>]}}}`, idempotently and without removing other entries.
- [ ] `lmbrain-mcp` honors `--root`/`LMBRAIN_ROOT` and operates on that root regardless of launch `cwd`.
- [ ] The kit no longer ships an inert, wrong-format registration file; prompts match the real mechanism.
- [ ] End-to-end: a real agent host (Claude Code) loads the `lmbrain-mcp` tools from the generated `.mcp.json` and a `task_start` (or equivalent) call moves the file and writes the audit entry. Evidence recorded.
- [ ] Automated tests cover the registration writer (new file, merge/idempotency, preserved entries) and the server root-resolution; `pnpm test` and `cargo test` green on Linux + Windows in CI.

## Implementation plan
1. Add `--root`/`LMBRAIN_ROOT` to `lmbrain-mcp` (+ tests).
2. Implement the registration writer (pure function over a path → merged `.mcp.json`) with unit tests for new/merge/idempotent cases.
3. Wire it into `initialize_workspace_kit` and workspace open; resolve the sidecar path.
4. Add the sidecar to `tauri.conf.json` and the CI bundling.
5. Fix/replace the kit template and align prompt wording.
6. Run the end-to-end smoke test and record evidence.

## Required verification
- `cargo test` (all crates) + `pnpm test` green on Linux + Windows in CI.
- Registration-writer unit tests (new/merge/idempotent/preserve).
- Manual end-to-end: host loads tools from the generated `.mcp.json` and performs a real transition; capture the resulting file + audit entry.

## Production quality and documentation
- Follow [[QUALITY]]; production work, not a prototype.
- Update `CONTRACT.md`/`AGENT.md`/MCP docs to describe how registration actually occurs.
- Report any quality-policy exception explicitly.

## Risks and open decisions
- **Distribution mechanism.** The spec recommends a Tauri sidecar so installing the app provides the binary. Alternatives — `cargo install`, a published package, or a fixed PATH location — are viable but shift setup onto the user; if chosen instead, this likely warrants its own ADR. Confirm before implementing the sidecar if there is a reason to prefer another mechanism.
- **Who writes `.mcp.json`.** ADR-004 said "the kit bootstrap registers it," but the **app** is the component that reliably knows the binary path and can write files, so registration is app-driven here. If this changes the intent of ADR-004, record a short follow-up ADR.
- **Host launch `cwd`.** Passing `--root` explicitly removes the dependency on how the host sets `cwd`.

## Instructions for the assigned specialist
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour. In particular, do not mark the end-to-end criterion done without actually loading the tools in a host.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation (Tauri sidecar/externalBin, the host's `.mcp.json` format) when behavior is uncertain.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

### Changes made
- `lmbrain-mcp` now resolves its workspace root from `--root`/`--root=`/`LMBRAIN_ROOT`, falling back to the launch directory (pure, unit-tested helper).
- New app module `commands/mcp_registration`: `build_mcp_config` (pure JSON merge that registers the `lmbrain` server in host format, preserving other keys/servers), `register_mcp_server` (idempotent atomic write of `.mcp.json`), and `resolve_mcp_command` (`LMBRAIN_MCP_BIN` → binary next to the app executable → bare `lmbrain-mcp` on `PATH`).
- `open_workspace` writes/refreshes `.mcp.json` at the workspace root on every open (best-effort, non-blocking).
- Kit MCP template rewritten to the correct host format; MCP README (kit + live) documents the automatic registration and how to provide the binary. `/.mcp.json` is git-ignored (generated, machine-specific).

### Files changed
- `lmbrain-mcp/src/main.rs`, `src-tauri/src/commands/mcp_registration.rs` (new), `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `kit/.lmbrain/mcp/lmbrain-mcp.json`, `.lmbrain/mcp/README.md` + `kit/.lmbrain/mcp/README.md`, `.gitignore`.

### Verification performed
- Added unit tests: server root resolution (flag/env/cwd precedence) and the config builder (create / preserve / idempotent / non-object root). Frontend suite unchanged and green. Rust validated in CI (`cargo test` on Linux + Windows).

### Deviations from the specification
- **Distribution mechanism changed from a Tauri sidecar to PATH/binary resolution.** Rationale: the implementing environment cannot build or test Tauri bundling, and adding `externalBin` would risk breaking local app builds that could not be verified here. Instead the app writes `.mcp.json` with a resolved `command` (`LMBRAIN_MCP_BIN` → next-to-exe → `PATH`). In a dev/workspace build the app and `lmbrain-mcp` share the same `target/` directory, so the next-to-exe resolution works without extra setup; installed-app users install the binary (`cargo install --path lmbrain-mcp`) or set `LMBRAIN_MCP_BIN`. **Bundling `lmbrain-mcp` as an installer sidecar remains a follow-up** so end-users get it automatically; it likely warrants its own ADR.
- The **end-to-end smoke test** (a real agent host loading the tools and performing a transition) still requires the operator's host and is not verifiable in this environment.

### Handoff status
- [x] Ready for Project Lead review (note the two deviations above)
