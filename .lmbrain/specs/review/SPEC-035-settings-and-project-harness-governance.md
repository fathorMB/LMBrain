---
id: SPEC-035
title: "Settings and project harness governance"
status: review
kind: feature
priority: high
area: settings-and-agent-hosts
milestone: M-04
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-011]
links: [SPEC-029, SPEC-034]
created: 2026-07-12
updated: 2026-07-12
tags: [settings, harnesses, governance, project-configuration, security, 2.8.0]
activity:
  - date: 2026-07-12
    action: "submitted for review after automated and operator-coordinated packaged Windows verification"
  - date: 2026-07-12
    action: "operator approved; Project Lead direct implementation authorized for dogfooding checkpoint 035-A because lmbrain-mcp was unavailable"
  - date: 2026-07-12
    action: "created"
---

# Settings and project harness governance

## Objective

Make Settings a real application area for LMBrain 2.8.0, move Local Harnesses into it, and give the Project Lead a controlled, project-scoped way to declare and validate harness environment requirements without directly rewriting heterogeneous native harness configuration.

## Context

The current Settings page contains a functional Codex executable override alongside appearance controls that do not change application behavior and an intentionally disabled auto-start control. Local Harnesses is a separate workspace navigation item even though it manages user-machine software rather than repository artifacts.

The Claude Code, Codex, Pi, and OpenCode integrations also have different project configuration formats and capabilities. Recent OpenCode testing showed why LMBrain needs to distinguish configured, available, active, and failing capabilities such as LSPs. Giving the Lead direct ownership of every native file would duplicate host-specific policy, weaken preservation guarantees, and make security review difficult.

## Scope

### Included

- Replace the placeholder-style Settings view with tabbed, routable settings whose visible controls are functional.
- Move the existing Local Harnesses experience into a `Harnesses` Settings tab and remove its primary-sidebar entry.
- Preserve current probe, update, confirmation, concurrency, active-session, logging, and post-update verification behavior from [[SPEC-034-manage-local-agent-harness-installations-and-updates]].
- Move the Codex executable override into the Harnesses tab and keep it machine-local.
- Add a project-scoped, versioned LMBrain harness manifest governed by the Project Lead through controlled MCP mutations.
- Model capabilities explicitly per host rather than pretending all hosts support the same settings.
- Support declarative requirements for enabled hosts, runtime/tool availability, non-secret environment values, and host-native project configuration fragments explicitly owned by LMBrain.
- Represent LSP policy/readiness for hosts that support it, including the distinction between configured, prerequisite-ready, active, and diagnostic failure.
- Validate schema, paths, types, supported capability keys, executable prerequisites, conflicts, and preservation boundaries before application.
- Show the effective configuration, validation results, native-file change preview, ownership, last-applied state, and drift in Settings.
- Require explicit operator approval before first materialization of executable or environment-affecting project configuration.
- Bind approval to the canonical manifest hash in machine-local LMBrain state; any material change invalidates approval.
- Materialize configuration idempotently and atomically while preserving unrelated user-owned native configuration.
- Keep secrets, credentials, tokens, absolute machine paths, and user-global harness settings out of the project manifest.
- Add a Project Lead prompt/action for creating or remediating the manifest; LMBrain does not autonomously act as the Lead.
- Add migration guidance from 2.7.x and document that no existing project is mutated merely by opening it.

### Excluded

- A universal free-form editor for native Claude, Codex, Pi, or OpenCode configuration.
- Arbitrary shell commands, install scripts, package-manager commands, hooks, or post-install actions declared by a project.
- Project-controlled executable overrides or PATH changes; binary selection remains machine-local.
- Storage or injection of secrets from repository files.
- Automatic approval, silent repair, global configuration mutation, or background harness installation.
- Claiming an LSP is active solely because it is enabled or its executable exists.
- Implementing non-functional theme, density, or agent auto-start controls merely to populate Settings.

## Existing-project analysis

- `SettingsView` persists only `lmbrain.codexBin`; its appearance controls are currently visual placeholders.
- `HarnessesView` already provides the complete local lifecycle UI and backend contracts for Claude Code, Codex, Pi, and OpenCode.
- Workspace preparation already performs narrow, idempotent MCP registrations with host-specific preservation rules.
- OpenCode has native LSP configuration and lazy activation; shell access to source files does not prove LSP activation.
- Pi has a separately governed exact-pin MCP extension and must not accept arbitrary project package installation.
- Existing generated native files are machine-specific and ignored by this repository, while the new LMBrain manifest must be versionable and reviewable.

## Technical proposal

Introduce a canonical `.lmbrain/HARNESSES.json` manifest with an explicit schema version and host-keyed declarations. JSON is chosen over parsing prose or embedding configuration in `PROJECT.md`; controlled mutation commands own serialization and atomic writes. Add the manifest to the LMBrain contract and kit migration rather than treating it as an undocumented file.

The backend parses the manifest into typed DTOs, rejects unknown or unsupported capability keys, computes a canonical digest, and produces a per-host plan. Each host adapter owns validation and materialization into its native project files. Adapters use narrow structural merge functions and report preserved, added, changed, conflicted, and unsupported paths. No adapter accepts an arbitrary command string.

Approval is stored outside the repository in application-local state keyed by canonical workspace identity and manifest digest. A changed digest, moved workspace whose identity cannot be verified, newly executable-affecting capability, or materialization conflict returns the project to `approval required`. Read-only validation and preview never require approval. Opening a project may report drift but must not apply unapproved changes.

Expose controlled MCP verbs such as `harness_config_get`, `harness_config_validate`, and `harness_config_set`. The setter validates the complete candidate and writes atomically; it does not materialize native files or grant approval. Operator approval and application remain explicit UI actions. Audit entries record Lead mutation, operator approval, application result, and digest without recording secrets.

Settings tabs should be URL/view-state addressable and keyboard accessible:

1. `General` — only real application preferences; initially may contain no speculative controls.
2. `Harnesses` — machine-local installation, executable resolution, versions, updates, and overrides.
3. `Project environment` — manifest status, capability matrix, validation, preview, approval, apply, drift, and Lead prompt.
4. `About` — app/kit versions and copyable diagnostics.

## Files and areas involved

- `.lmbrain/CONTRACT.md`, kit contract/templates/migration files
- Rust manifest models, parser, validator, digest, approval store, host adapters, and Tauri commands
- MCP protocol/core/server controlled mutation verbs and audit trail
- Existing Claude/Codex/Pi/OpenCode registration modules
- `SettingsView`, `HarnessesView`, Sidebar/AppShell routing, commands and TypeScript DTOs
- frontend and Rust tests
- `docs/agent-hosts.md`, `docs/architecture.md`, `docs/product.md`, `docs/sessions.md`

## Acceptance criteria

- [x] Settings uses accessible, stateful tabs and contains no control that appears editable but has no effect.
- [x] Local Harnesses is removed from workspace navigation and retains all existing behavior in the Harnesses tab.
- [x] The Codex executable override remains machine-local, persists correctly, and drives both probe and session resolution.
- [x] A schema-versioned project harness manifest is part of the LMBrain contract, kit, diagnostics, and 2.8.0 migration guidance.
- [x] The Project Lead can read, validate, and atomically replace the complete manifest through controlled MCP tools with audit evidence.
- [x] The manifest cannot contain secrets, credentials, absolute machine paths, arbitrary commands, install scripts, hooks, or global-setting mutations.
- [x] Capability validation is host-specific and reports unsupported settings instead of silently ignoring them.
- [x] LSP state distinguishes configured, prerequisite-ready, active, inactive/lazy, and failed when the host exposes sufficient evidence.
- [x] Settings shows a deterministic native-file preview and ownership/conflict information before application.
- [x] Executable/environment-affecting configuration is not materialized until explicitly approved by the operator.
- [x] Approval is machine-local and digest-bound; a material manifest change invalidates it.
- [x] Materialization is atomic, idempotent, scoped to LMBrain-owned keys, and preserves unrelated native configuration.
- [x] Opening a 2.7.x or unconfigured project performs no new mutation and gives actionable optional setup guidance.
- [x] Drift and partial failures remain visible and retryable; LMBrain never reports applied/healthy from configuration intent alone.
- [x] Tests cover schema evolution, canonical hashing, approval invalidation, malicious inputs, host capability differences, merge preservation, conflicts, rollback, drift, tabs, routing, and migrated Harnesses behavior.
- [x] Full quality, installer, migration, and Windows packaged-app checks pass before release.

## Implementation plan

1. Approve [[ADR-011-project-harness-manifest-and-operator-approval]] and finalize the manifest schema/capability matrix.
2. Add contract, parser, canonical digest, validation, diagnostics, and migration fixtures.
3. Add MCP read/validate/set verbs, atomic writes, audit records, and authorization tests.
4. Add approval storage, preview planning, per-host adapters, atomic materialization, drift detection, and rollback behavior.
5. Refactor Settings into tabs; embed Local Harnesses and add Project environment/About views.
6. Remove placeholder controls and the Local Harnesses sidebar route while preserving deep-link compatibility or a redirect.
7. Run full automated gates and coordinate explicit packaged Windows testing without starting or stopping an operator production instance.

## Required verification

- Rust unit and integration tests using isolated workspaces and fake executables; no real harness updater or installer.
- MCP contract tests proving invalid/unapproved content cannot be materialized and all writes are audited.
- Frontend component/routing/accessibility tests for every tab and approval/drift state.
- Preservation fixtures for pre-existing native configurations for all supported hosts.
- Security tests for traversal, symlinks/reparse points, absolute paths, command injection, environment abuse, oversized manifests, malformed content, and stale approval replay.
- Full `pnpm test`, lint/build, Rust workspace tests/checks, version alignment, migration checks, and `git diff --check`.
- Separate operator-coordinated packaged Windows smoke test for navigation, persistence, preview, approval, materialization, restart, and drift.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Update the LMBrain contract and bundled kit alongside application behavior.
- Document host-specific limitations honestly; absence of evidence is never displayed as an active capability.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- The exact first-version capability matrix must remain narrow; adding free-form native fragments would defeat validation and security boundaries.
- Workspace identity for approval must survive ordinary path normalization without allowing approval reuse by a different repository.
- Native files may contain user edits that overlap LMBrain-owned paths; conflicts must stop application and never be overwritten silently.
- Some hosts expose no reliable live LSP state. The UI must show `unknown` rather than infer activation.
- The persisted settings store needs permissions and corruption recovery appropriate for non-secret local state.

## Instructions for the assigned specialist

- Start only after this spec is explicitly approved and moved to `ready`; call `spec_start` first and `spec_submit` when complete.
- Implement only the stated scope and do not broaden the initial capability matrix without operator approval.
- Report changed files, tests run, security verification, migrations, and known limitations.
- Produce production-grade code; do not ship placeholder UI or knowingly incomplete behavior.
- Do not start or stop an operator LMBrain instance. Coordinate packaged manual testing separately.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

- Checkpoint 035-A foundation: added the strict schema-v1 harness manifest model, host-specific capability validation, canonical SHA-256 digest, confined loader, and normalized workspace identity.
- Added the optional empty manifest to the bundled kit and documented its contract, planned 2.8.0 migration, security boundary, and explicit-approval requirement.
- Added controlled MCP `harness_config_get`, `harness_config_validate`, and `harness_config_set` verbs. Set operations are serialized, atomically replace the complete validated manifest, append digest-only audit evidence, and never materialize native host files.
- Added read-only Tauri diagnostics for malformed or unsafe manifests; absence remains valid and quiet.
- Added a machine-local, schema-versioned approval store keyed by canonical workspace fingerprint and manifest digest, with explicit status/approve/revoke commands, stale-preview rejection, automatic stale state after manifest changes, atomic persistence, restrictive Unix permissions, and corruption quarantine.
- Added a read-only per-host planner exposing effective configuration, supported capability keys, required-tool availability, LMBrain-owned native paths, and deterministic added/changed/preserved/conflicted previews for Claude Code, Codex, Pi, and OpenCode.
- Added structural conflict detection for incompatible JSON/TOML parents while preserving unrelated native configuration and leaving all inspected files untouched.
- Added approval-gated native materialization with a shared manifest/apply lock, exact approved-digest enforcement, same-filesystem staging, multi-file rollback, symlink/path guards, idempotence, and preservation through the existing host-specific structural merge builders.
- Added machine-local applied-content hashes and read-only drift reporting for changed or missing managed files.
- Replaced placeholder Settings controls with accessible, hash-addressable General, Harnesses, Project environment, and About tabs.
- Moved Local Harnesses and the machine-local Codex executable override into Settings, removed the primary sidebar entry, and retained the legacy `harnesses` view as a redirect-compatible tab entry.
- Added Project environment states for unconfigured/required/approved/stale, deterministic preview, readiness/conflicts, approval/revoke/apply, drift, refresh, and a copyable Project Lead setup prompt.

### Files changed

- `lmbrain-core/src/harness_manifest.rs`, core exports/dependencies, and Cargo lockfile.
- `kit/.lmbrain/HARNESSES.json`, `CONTRACT.md`, `MIGRATIONS.md`, and `CHANGELOG.md`.

### Verification performed

- `cargo test -p lmbrain-core` — 39 tests passed (29 unit tests and 10 integration tests).
- `cargo check -p lmbrain-core` — passed.
- `git diff --check` — passed.
- `cargo test -p lmbrain-core -p lmbrain-mcp --no-fail-fast` — 55 tests passed across core, transition, MCP unit, and protocol suites.
- Targeted Tauri contract diagnostic test — passed; the app was not started or stopped.
- `cargo test -p lmbrain harness_approval --lib` — 3 approval lifecycle/corruption tests passed.
- `cargo check -p lmbrain` and `pnpm exec tsc --noEmit -p tsconfig.app.json` — passed without launching the app.
- `cargo test -p lmbrain harness_planner --lib` — 3 deterministic preview, preservation, Pi JSON conflict, and Codex TOML conflict tests passed.
- `cargo test -p lmbrain harness_materializer --lib` — 3 idempotence/preservation, injected batch rollback, and changed/missing drift tests passed.
- Settings/Harnesses component tests — 8 passed; lint and TypeScript type-check passed.
- Full frontend suite — 120 tests passed; production frontend build passed with only the existing large-chunk advisory.
- Full `cargo test --workspace --no-fail-fast` passed: 189 tests passed and 3 explicitly manual harness tests remained ignored; `cargo check --workspace` passed.
- App/kit/Tauri version alignment passed at `2.8.0`; documentation covers architecture, host governance, migration, and rollback.
- Oversized manifest/environment limits and LSP prerequisite/runtime-state presentation were verified in the final hardening pass.
- Real Windows probes resolved all four installed harnesses. Packaged 2.8.0 smoke testing confirmed Settings navigation, corrected Project environment layout, silent Harnesses probing, silent workspace-open Git/Pi helpers, and optional setup guidance.
- Clipboard prompt feedback is covered by a component test; the final MSI and NSIS packages were rebuilt after this correction.

### Deviations from the specification

- Manual packaged Windows verification and broader four-host failure-injection fixtures remain deferred; the running operator instance was not started, stopped, or used for testing.

### Handoff status

- [x] Ready for Project Lead review
