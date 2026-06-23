---
id: SPEC-017
title: "Controlled-mutation engine (lmbrain-core) with MCP delivery"
status: accepted
kind: feature
priority: high
area: core-tooling
milestone:
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-004, ADR-001, ADR-002]
links: [ADR-004, REVIEW-014]
created: 2026-06-23
updated: 2026-06-23
tags: [mcp, transitions, core, refactor, rust]
---

# Controlled-mutation engine (lmbrain-core) with MCP delivery

## Objective

Provide a single, unit-tested engine that performs **controlled mutations** of `.lmbrain/` markdown artifacts (status transitions, creation with automatic ID allocation, and a small set of targeted field setters), and expose it to agents as **MCP tools**, so agents stop editing markdown by hand. See [[ADR-004-controlled-mutation-engine-mcp]] for the decision and rationale.

## Context

State rules (allowed transitions + invariants) currently live only as prose in `CONTRACT.md` and as a partial, fragile implementation (`set_artifact_status` in `src-tauri/src/commands/contract.rs`): spec/adr/agent/mcp only, naive line-based frontmatter edits, folder move for specs only, no task support. Agents run outside the app and mutate markdown by hand, which causes recurring drift (tasks never set `in-progress`, `planned` without a ready spec, unresolved `recommended_agent`, the Lead implementing scaffolding). This spec replaces the ad-hoc approach with one shared, tested core reused by the app, the agents (via MCP), and the diagnostics.

## Scope

### Included
- A new tauri-free `lmbrain-core` crate (cargo workspace) holding: domain models, markdown/frontmatter parser, `transitions` (per-artifact state machines + invariants + surgical frontmatter mutator), and path-safety/atomic-write primitives.
- Controlled operations for every artifact type: **state transitions**, **creation** (atomic progressive ID allocation), and **targeted setters** (at minimum `recommended_agent` on specs and evidence on tasks).
- Hard-blocking invariant enforcement with a recorded `force` + `reason` override.
- A structured audit trail (frontmatter `activity` entry + `updated` bump; override justification in the body).
- An `lmbrain-mcp` server exposing specific per-verb tools plus a few read tools; built as a per-OS binary in CI.
- Kit bootstrap registration of the MCP server, scoped to the repo root.
- Migration of the existing app `set_artifact_status` to a thin wrapper over the core (no behavior change for the operator).
- Kit prompt updates (`AGENT.md`, handoff, bootstrap) instructing agents to use the MCP tools instead of editing markdown.

### Excluded
- A CLI surface (explicitly out per ADR-004; the core keeps it cheap to add later).
- Any change to the product's read-only Taskboard UI beyond reusing the unified core.
- Git operations (commits/branches) performed by the tools — the agent/operator handles version control.
- Auto-cascading side effects between artifacts beyond what an invariant strictly requires (e.g. accepting a review does not silently accept the spec; the spec `accept` tool checks the invariant).

## Existing-project analysis

- Mutation today: `set_artifact_status` — [`src-tauri/src/commands/contract.rs`](../../../src-tauri/src/commands/contract.rs) (≈ line 1200). Reuse its atomic temp+rename and spec folder-move patterns; replace its naive string frontmatter rewrite with the surgical mutator.
- Path safety: `PathGuard` — [`src-tauri/src/commands/filesystem.rs`](../../../src-tauri/src/commands/filesystem.rs). Reuse as-is in the core.
- Readers/state enums already exist: `build_tasks/specs/reviews/...` and the status enums in `src-tauri/src/models/`. Move these into `lmbrain-core`.
- Invariants already partially encoded as diagnostics in `build_diagnostics` (status mismatch, unresolved `recommended_agent`, `planned`-without-ready-spec). Refactor these to call the shared invariant functions so UI/MCP/tests agree.
- Crate is currently a single Tauri crate (`lmbrain_lib`); `serde_yaml`, `chrono`, `regex`, `uuid`, `thiserror`, `tempfile` (dev) are available.

## Technical proposal

1. **Workspace refactor (no behavior change).** Convert the repo into a cargo workspace. Create `lmbrain-core` (no `tauri` dependency) and move `models`, `parser`, `filesystem` (PathGuard), and the artifact readers into it. `src-tauri` depends on `lmbrain-core`. All existing tests must still pass.
2. **Transitions module.** For each artifact type declare an explicit state machine `(from, verb) -> to`. Implement transition functions that: validate the transition, evaluate the relevant invariants, apply the surgical frontmatter edit (`status`, `updated`, appended `activity`), move the file to the matching status directory when applicable, and write atomically (temp + rename). Illegal transitions and failed invariants return typed errors with human-readable messages.
3. **Invariant library.** Pure, reusable predicates: `spec_has_accepted_review`, `task_criteria_complete_with_evidence`, `single_ready_handoff`, `task_planned_requires_ready_spec`, `recommended_agent_resolves`, `unique_ids`, `folder_matches_status`. Used by transitions (hard-block) and by `build_diagnostics` (warnings).
4. **Creation + ID allocation.** `create_*` operations scan the artifact tree for the highest numeric ID and allocate the next one atomically, scaffolding from the existing templates.
5. **Override + audit.** Transition/setter APIs accept `force: bool` and `reason: Option<String>` (required when `force=true`). The audit entry is appended to frontmatter `activity`; override justifications are written to a body section.
6. **MCP server (`lmbrain-mcp`).** A stdio MCP server (prefer the official Rust MCP SDK; otherwise a minimal JSON-RPC) exposing specific per-verb tools with JSON-schema parameters, plus read tools (`validate`, `list_ready_handoffs`, `get_artifact`). Each tool is a thin wrapper over the core; the workspace root is resolved from the launch directory via `PathGuard`. Errors map to structured tool results.
7. **Distribution + kit wiring.** Build `lmbrain-mcp` per-OS in CI (extend `build-installers.yml`). The kit bootstrap registers the server in the agent host's MCP configuration. Update `AGENT.md`, the handoff prompt, and the bootstrap prompt to direct agents to the tools.

## Files and areas involved

- New: `lmbrain-core/` crate (models, parser, transitions, invariants, fs-safe).
- New: `lmbrain-mcp/` crate (server + tool definitions).
- Modified: `Cargo.toml` (workspace), `src-tauri/` (depend on core; `set_artifact_status` → wrapper), `src-tauri/src/commands/contract.rs` (`build_diagnostics` → shared invariants).
- Modified: `.github/workflows/build-installers.yml` (build/test the new crates; publish the MCP binary).
- Modified (kit + live): `AGENT.md`, `templates/project-lead-bootstrap-prompt.md`, the generated handoff prompt (`src/lib/handoffPrompt.ts`), and bootstrap registration logic.

## Acceptance criteria
- [ ] `lmbrain-core` exists as a tauri-free workspace crate; `src-tauri` builds against it and all pre-existing tests still pass.
- [ ] Every artifact type has transition functions covering its allowed status changes, with the file physically moved for status-directory artifacts and `updated` bumped.
- [ ] Each invariant in the invariant library is enforced as a hard block by the relevant transition, and the same predicate backs the corresponding diagnostic.
- [ ] `force: true` requires a `reason` and records it in the artifact's audit trail; without `reason` the override is refused.
- [ ] Frontmatter edits preserve key order, comments, and unrelated fields (verified by a round-trip/property test); writes are atomic.
- [ ] Creation operations allocate the next ID atomically and scaffold from the templates.
- [ ] `lmbrain-mcp` exposes the per-verb tools and read tools; a protocol-level integration test drives at least one full transition end to end and asserts the resulting file.
- [ ] `cargo test` passes for all crates on Linux and Windows in CI; the MCP binary is built per-OS.
- [ ] Kit prompts instruct agents to use the MCP tools; the bootstrap registers the server.
- [ ] Every acceptance item has automated test evidence.

## Implementation plan
1. Workspace refactor: extract `lmbrain-core`, move models/parser/filesystem/readers, repoint `src-tauri`. Green tests.
2. Transitions + invariants for **Task** first (start/submit/complete/plan/block/cancel/create), with unit tests per `(from,verb)` (valid + illegal) and per invariant.
3. Extend to spec/review/adr/agent/mcp/handoff; add creation + targeted setters; add override + audit.
4. Refactor `build_diagnostics` and `set_artifact_status` to consume the core.
5. Build `lmbrain-mcp`; wire tools; protocol-level integration tests; per-OS CI build.
6. Kit prompt migration + bootstrap MCP registration.

## Required verification
- `cargo test` (all crates) green on Linux + Windows in CI.
- Frontmatter property test (preservation + idempotency).
- MCP protocol integration test (spin server, drive a transition, assert file + structured result) on a temporary workspace.
- Manual smoke: an agent host connects the MCP server and performs a task `start`/`complete` cycle.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Update `CONTRACT.md` to point at the engine as the executable source of truth for transitions, and the task/spec/review READMEs where the tools change the workflow.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- **Frontmatter preservation** is the main technical risk; if surgical editing cannot robustly preserve arbitrary human formatting, escalate per ADR-004's review condition before falling back to a structured approach.
- **MCP host integration** (how the agent host launches/connects the server, root resolution) must be validated early in phase 5.
- **Cross-platform paths** (Windows short/long path, verbatim prefixes) — reuse `PathGuard`/`clean_path`; cover in tests.

## Instructions for the assigned specialist
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

- Added the tauri-free `lmbrain-core` workspace crate for path-safe, atomic controlled mutations, state machines, invariant checks, ID allocation, surgical frontmatter updates, audit activity, and forced-override reasons.
- Added the `lmbrain-mcp` stdio JSON-RPC server with constrained per-verb task/spec/review/ADR tools, creation/setter tools, and read/validation tools.
- Repointed the app's existing approval/rejection command to the core while preserving its deliberately narrow operator-facing proposed-only behavior.
- Added CI build/test coverage for the workspace and per-OS MCP binary artifact, kit MCP registration metadata, and agent workflow documentation.
- Addressed REVIEW-014: diagnostics now consume the core invariant predicates; the desktop status command is a clean compatibility wrapper over the core; and the stale implementation body has been removed.
- Added table-driven valid/illegal transition coverage, direct invariant coverage, creation/ID-allocation coverage, and forced-override audit coverage. MCP tool schemas are now tool-specific and the server version is package-derived.

### Files changed

- `Cargo.toml`, `Cargo.lock`, `lmbrain-core/`, `lmbrain-mcp/`
- `src-tauri/Cargo.toml`, `src-tauri/src/commands/contract.rs`
- `.github/workflows/build-installers.yml`, `kit/.lmbrain/`, and delegated LMBrain documentation/prompt files

### Verification performed

- `cargo test -p lmbrain-core -p lmbrain-mcp` (passed; includes core transition and MCP protocol integration coverage)
- Existing `src-tauri` tests passed in the earlier workspace run. A final all-workspace retry was blocked by a Windows linker/PDB file-lock error (`LNK1201` / `os error 5`), after compilation, rather than a test failure.
- `pnpm test -- --run` (43 tests passed)
- `pnpm lint` (passed)
- Core round-trip, force-override, transition, and MCP protocol integration tests are included in the workspace.
- Correction pass: `cargo test` passed for the full workspace; `pnpm lint` passed; `pnpm test` passed (43 tests).

### Deviations from the specification

- No linked task artifacts were present (`related_tasks: []`), so there was no task to move to `in-progress` or `review`.
- The repository does not have an agent-host configuration API. Bootstrap ships a repository-scoped MCP registration descriptor for the host to consume rather than mutating a global host configuration.
- None for the REVIEW-014 correction scope.

### Handoff status
- [x] Ready for Project Lead review
