---
id: SPEC-034
# Note: Quote the title if it contains a colon
title: "Manage local agent harness installations and updates"
status: review
kind: feature
priority: medium
area: local-tooling
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-11
updated: 2026-07-11
tags: [harnesses, updates, claude, codex, pi, security]
activity:
  - date: 2026-07-11
    action: "created"
activity:
  - date: 2026-07-11
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-11
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-11
    action: "transitioned working -> review"
---
# Manage local agent harness installations and updates

## Objective

Add a local-machine Harnesses page that shows whether Claude Code, Codex, and Pi are installed, identifies the exact executable and current version, and lets the operator explicitly run each harness's supported self-update workflow with safe progress, logs, and post-update verification.

## Context

LMBrain can launch three agent hosts but exposes their local configuration only indirectly. Operators cannot see which binary will run, distinguish missing/stale installations, or trigger supported updates from the app. Update operations mutate user-level software outside the repository and therefore require a stricter boundary than normal workspace reads.

Verified on 2026-07-11 in the operator environment:

- Claude Code `2.1.206`, `C:\Users\moren\.local\bin\claude.exe`, supports `claude update`.
- Codex CLI `0.144.1`, Desktop-managed path under `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe`, supports `codex update`.
- Pi `0.79.10`, resolved through the user's npm shim, supports `pi update --self`.

## Scope
### Included

- New `Local Harnesses` navigation page, clearly separated from project `Agents & MCP` artifacts.
- Cards for Claude Code, Codex, and Pi with installed/missing/error state, exact resolved path, parsed version, and last probe time.
- Resolution parity with session launch, including the configured Codex binary override.
- Explicit per-harness `Check & update` action using fixed executable/argv pairs: `claude update`, `codex update`, `pi update --self`.
- Confirmation explaining that the operation modifies user-level software and may use the network.
- Backend serialization: at most one harness update at a time.
- Block update while an LMBrain session using that harness is running; explain which sessions must be closed.
- Run without shell interpolation, elevation, `sudo`, arbitrary package-manager commands, or workspace mutation.
- Capture exit status and bounded stdout/stderr; show progress, success, already-current, failure, and logs.
- Re-probe executable path/version after every completed update before declaring success.
- Missing installations show official installation guidance/copyable commands but are not installed automatically in this scope.
- Tests, documentation, and patch/minor release decision at implementation time.

### Excluded

- Silent or automatic background updates.
- Updating project Pi extensions/packages (`pi update --extensions` / `--all`).
- Installing missing harnesses, running downloaded scripts, or elevating privileges.
- Managing Ollama itself, models, LMBrain, IDE extensions, authentication, or global MCP servers.
- Updating a harness while its sessions are active.

## Existing-project analysis

- Codex already has custom path resolution in `sessions.rs`; Claude and Pi use command lookup helpers.
- SessionManager tracks host/status and can enforce the active-session safety gate.
- Tauri commands and typed TypeScript DTOs are the established backend/frontend boundary.
- `SettingsView` is too narrow and `Agents & MCP` represents repository governance; local executable lifecycle deserves its own page.
- Update commands may take time and emit useful diagnostics, so they must run off the Tauri command thread and return bounded structured output.

## Technical proposal

Introduce a backend `harnesses` module and typed `HarnessStatus`, `HarnessUpdateRequest`, and `HarnessUpdateResult` DTOs. Probing resolves the same executable that sessions would launch, runs only `<binary> --version`, applies a timeout, parses a host-specific version string, and never mutates state.

Updating requires an explicit Tauri invocation after frontend confirmation. The backend re-resolves/re-probes the selected harness, rejects active matching sessions or another in-flight update, validates the harness enum into a compile-time fixed argv list, and executes directly without a shell. Run the child on a blocking worker with a bounded runtime/output policy; expose cancellation only if process ownership can be implemented reliably across supported platforms. Always re-probe after a zero exit code and report before/after versions plus sanitized stdout/stderr.

The frontend page loads all statuses in parallel, supports per-card reprobe, and shows one update confirmation/dialog at a time. Do not label a harness updated solely from command exit status; the verified post-update state is authoritative.

## Files and areas involved

- `src-tauri/src/commands/harnesses.rs` (new)
- `src-tauri/src/commands/sessions.rs` / session-manager read API
- `src-tauri/src/models/` and `src-tauri/src/lib.rs`
- `src/types/index.ts`, `src/lib/commands.ts`
- `src/components/Harnesses/HarnessesView.tsx` (new)
- `src/components/Layout/Sidebar.tsx`, `src/components/Layout/AppShell.tsx`
- frontend and Rust tests
- `docs/agent-hosts.md`, `docs/product.md`, `docs/architecture.md`

## Acceptance criteria
- [x] Local Harnesses page lists Claude Code, Codex, and Pi independently of the current workspace's agent profiles.
- [x] Each card accurately reports installed/missing/probe-error, exact resolved executable, current version, and probe time.
- [x] Codex status honors the configured binary override and matches session launch resolution.
- [x] Update actions use only fixed direct argv: Claude `update`, Codex `update`, Pi `update --self --no-approve`.
- [x] Every update requires explicit confirmation that user-level software and network state may change.
- [x] No shell command string, elevation, global package-manager guess, workspace write, or automatic update is used.
- [x] Matching running sessions block the update with actionable guidance; different-host sessions may remain running.
- [x] Concurrent update attempts are rejected/disabled across frontend and backend.
- [x] UI shows in-progress state and bounded stdout/stderr result detail without freezing other views.
- [x] Success requires a zero updater exit plus a successful post-update probe; before/after versions and path are shown.
- [x] Failure and timeout preserve the previous known status, expose logs, and permit retry/reprobe.
- [x] Missing harnesses are never auto-installed and instead show official guidance.
- [x] Tests cover resolution, version parsing, fixed argv, active-session/concurrency gates, success, no-change, failure, timeout/output bounds, and frontend states.
- [x] Full LMBrain quality/release gates pass and documentation explains security/ownership boundaries.

## Implementation plan
1. Add shared harness resolver/probe DTOs and tests, reusing session launch resolution.
2. Add serialized safe update execution and active-session gates with structured results.
3. Add Local Harnesses navigation/page, confirmation, progress, logs, reprobe, and missing-state guidance.
4. Add focused frontend tests and update architecture/product/host documentation.
5. Run full gates and perform manual installed/missing/update checks in a disposable or explicitly operator-approved environment.

## Required verification

- Rust unit/integration tests for all probe/update policies without updating real operator tools.
- Frontend component tests with mocked commands; no real updater invocation in automated tests.
- Manual real update only after a separate explicit operator confirmation, one harness at a time, with no matching sessions running.
- Full `pnpm test`, lint/build, Rust workspace tests, version alignment, and diff checks.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Self-updaters may replace the executable currently running on Windows; matching sessions must be closed and post-update resolution may select a different path.
- Claude auto-updates already; manual update can legitimately report no version change.
- Codex supports native, npm, Homebrew, and release-binary installs. Its own `codex update` is preferred over guessing a package manager because current Codex update logic derives an action from install context.
- Pi self-update may migrate npm package scope; `pi update --self` owns that transition and must not be replaced with hard-coded npm package names.
- Cancellation/timeout must not leave an orphan updater. If cross-platform child termination cannot be made reliable, implementation must block on a documented bounded self-updater contract or return to the operator for a scope decision rather than ship unsafe cancellation.
- Version-availability checks are not uniformly exposed without running the updater. Initial UI should say `Check & update`, not claim an update is available before the harness reports it.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

- Added typed backend probe/update models and commands for Claude Code, Codex, and Pi.
- Reused session command resolution, including configured/Desktop Codex resolution, and verified versions using bounded `--version` probes.
- Added fixed self-update argv, user-home working directory, explicit active-session gate, global update lease, ten-minute timeout, process-tree termination, bounded non-blocking stdout/stderr capture, and authoritative post-update probing.
- Added Local Harnesses navigation/page with installed/missing/error cards, exact path/version/time, reprobe, official missing-install guidance, explicit confirmation, progress, results/logs, and actionable failures.
- Preserved project boundaries: no auto-install, elevation, arbitrary package-manager selection, project Pi extension update, or workspace mutation.
- Bumped the shared app/kit version to `2.7.0` and documented the no-artifact migration.

### Files changed

- `src-tauri/src/commands/harnesses.rs`, `src-tauri/src/models/harness.rs`
- `src-tauri/src/commands/sessions.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/lib.rs`
- `src/components/Harnesses/HarnessesView.tsx`
- `src/components/Layout/Sidebar.tsx`, `src/components/Layout/AppShell.tsx`
- `src/types/index.ts`, `src/lib/commands.ts`
- `src/__tests__/HarnessesView.test.tsx`
- `docs/agent-hosts.md`, `docs/product.md`, `docs/architecture.md`
- `package.json`, `src-tauri/Cargo.toml`, `Cargo.lock`, `kit/.lmbrain/VERSION`, `kit/.lmbrain/CHANGELOG.md`, `kit/.lmbrain/MIGRATIONS.md`

### Verification performed

- `cargo test --workspace` - passed across app, core, MCP, protocol, and integration suites; 43 app unit tests passed with three intentional ignored helper/manual tests.
- Focused harness runner tests passed for fixed argv, parsing, serialization, active-session message, post-probe outcomes, timeout/process termination, and 64 KiB output bounds.
- Manual read-only `probes_operator_installed_harnesses` smoke test passed: Claude Code `2.1.206`, Codex `0.144.0-alpha.4` at the Desktop-resolved path, and Pi `0.79.10` via `pi.cmd`.
- `pnpm test` - passed, 21 files / 118 tests.
- `pnpm lint` - passed.
- `pnpm build` - passed; existing main-bundle warning remains (approximately 824 kB).
- `cargo check --workspace --tests` - passed.
- `node scripts/check-version.mjs` - passed at `2.7.0`.
- `git diff --check` - passed.
- No real Claude, Codex, or Pi updater was invoked.

### Deviations from the specification

- Pi update argv adds documented `--no-approve` to `update --self`, ensuring project-local files are ignored while updating the user-level CLI.
- No cancel button is exposed. Timeout termination is process-tree-aware on Windows and Unix and log readers cannot block command completion; normal UI waits for the supported self-updater to finish.
- Real mutating update smoke tests remain operator-controlled and were intentionally not run during implementation.

### Handoff status
- [x] Ready for Project Lead review
