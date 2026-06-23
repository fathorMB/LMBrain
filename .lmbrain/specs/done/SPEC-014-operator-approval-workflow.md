---
id: SPEC-014
title: Operator approval workflow — in-app approve/reject for proposed artifacts
status: done
kind: feature
priority: high
area: desktop-app
milestone: M-02
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-002, ADR-003]
links: [ADR-002, ADR-003, SPEC-012, SPEC-013, REVIEW-013]
created: 2026-06-22
updated: 2026-06-22
tags: [write, approval, workflow, modal, milestone-m02]
---

# Operator approval workflow — in-app approve/reject for proposed artifacts

## Objective

Deliver milestone M-02: let the operator review a `proposed` artifact in the detail modal and **approve / reject** it directly in the app, writing the status change back to the Markdown source of truth. Rejected items expose the existing R2-F4 corrective-prompt action. This is the read→read-write step authorized by [[ADR-002-in-app-artifact-status-writes]].

## Context

The read-only artifact detail modal already exists ([`ArtifactDetailModal.tsx`](src/components/Layout/ArtifactDetailModal.tsx), shipped in SPEC-013 WI-5). The app currently performs no per-artifact writes; the only existing write is the bootstrap `initialize_kit`. M-02 adds the first artifact-state writes, which must honor every `CONTRACT.md` invariant — especially that filesystem location and `status` frontmatter must agree, and that a spec may be `accepted` only when an associated review is `accepted`.

This spec implements **operator-initiated** approve/reject only. No agent or automatic process may trigger a write.

## Per-artifact approve/reject state machine

The action set is per artifact type, using only statuses the contract already defines:

Per [[ADR-003-reject-as-first-class-status]], `rejected` is now a first-class status on every proposable artifact, so reject is uniform:

| Artifact (from `proposed`) | Approve → | Reject → | Notes |
| --- | --- | --- | --- |
| Spec | `ready` | `rejected` | **Approve must NOT set `accepted`** — acceptance stays gated by an accepted review. Reject moves the file to `specs/rejected/`. |
| ADR | `accepted` | `rejected` | Now resolved by ADR-003 (`rejected` added to ADR). Flat dir — status in frontmatter. |
| Agent proposal | `approved` | `rejected` | Status set defined by ADR-003. Flat dir. |
| MCP proposal | `approved` | `rejected` | Native mapping. Flat dir. |
| Agent profile | `active` | `inactive` | A profile is not a proposal; it has no `rejected`. `proposed → active` on approve. |

Only the actions with a defined target status are shown for a given artifact. Where no `proposed` state applies (artifact already past proposed), no approve/reject is offered.

## Scope
### Included
- Operator-initiated approve/reject for Spec, ADR, Agent proposal, and MCP proposal, plus Agent profile approve→active / reject-equivalent→inactive, per the table above. Reject is supported for **all** proposable artifacts (ADR-003).
- Recognize the new `rejected` status (ADR-003 / Contract v0.2) in the backend status enums and the directory/status-mismatch diagnostic, including the new `specs/rejected/` directory.
- Backend write: update `status` frontmatter, bump `updated`, and **move the file** to the matching status directory (for status-directory artifacts like specs); atomic (temp file + rename); path-safety scoped to the workspace.
- Frontend: approve/reject controls in the detail modal for eligible `proposed` artifacts, with a confirmation step; on reject, surface the R2-F4 corrective-prompt action for that item.
- File-watcher coordination so the write does not race the reload; clear success/error feedback.

### Excluded
- Any non-`proposed` transition, bulk actions, free-form status editing, reopening a `rejected` item, and any agent-triggered write.

## Technical proposal

**Backend (`src-tauri`):**
- Add a single, well-scoped command, e.g. `set_artifact_status(path, target_status)`, that:
  1. validates the path is inside the approved workspace (reuse `PathGuard`);
  2. validates the requested transition against the artifact type's allowed set (reject invalid transitions with a clear error — never write an illegal status);
  3. rewrites the `status` frontmatter and `updated` date, preserving the rest of the document byte-for-byte (LF; see SPEC-013 WI-2/`.gitattributes`);
  4. writes atomically (temp + rename) and **moves the file** into the `<artifact>/<target_status>/` directory so filesystem and status agree;
  5. returns the new path/state.
- Guard the spec invariant in the backend too: refuse `spec → accepted` from this command (acceptance is review-gated), so the rule holds even if the UI is wrong.

**Frontend (`src`):**
- Extend `ArtifactDetailModal` with an actions area shown only for eligible `proposed` artifacts. Actions come from the per-type table. Require a confirmation (the write moves a file and changes tracked state).
- After a successful write, refresh the affected data and reflect the new status; on error, show the message and leave state unchanged.
- On reject, show the R2-F4 corrective-prompt affordance for the rejected item.
- Keep the read/write boundary visually explicit; the modal stays read-only for non-proposed artifacts.

## Files and areas involved
- `src-tauri/src/commands/` — new status-write command (+ wiring in `lib.rs`).
- `src-tauri/src/commands/workspace.rs` / `filesystem.rs` — reuse `PathGuard`/atomic-write patterns.
- `src/components/Layout/ArtifactDetailModal.tsx`, `src/lib/commands.ts`, `src/context/WorkspaceContext.tsx`, relevant list/card views.

## Acceptance criteria
- [ ] From the detail modal, a `proposed` Spec can be approved (→`ready`) or rejected (→`rejected`); the file moves to the matching directory (`specs/ready/` or `specs/rejected/`) and `updated` is bumped.
- [ ] A `proposed` ADR → `accepted`/`rejected`; Agent proposal → `approved`/`rejected`; MCP proposal → `approved`/`rejected`; Agent profile → `active`/`inactive`.
- [ ] The backend status enums and the directory/status-mismatch diagnostic recognize `rejected` (no false "Status mismatch"/"UNKNOWN" for a rejected artifact); covered by a test.
- [ ] The backend **refuses** `spec → accepted` via this command (review-gated), with a clear error; covered by a test.
- [ ] Illegal transitions (wrong target for the type, or non-proposed source) are rejected by the backend; covered by tests.
- [ ] Writes are atomic and preserve document content (LF, body untouched); the file ends up in the directory matching its new status (no directory/status mismatch diagnostic afterward).
- [ ] Rejected items expose the R2-F4 corrective-prompt action.
- [ ] The write is operator-initiated and confirmed; the file watcher does not produce a stale/duplicated view after the write.
- [ ] Non-`proposed` artifacts show no write controls (modal stays read-only).

## Required verification
- `pnpm lint`, `pnpm test`, `pnpm build` — all green.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — all green; **paste the output** (the Project Lead environment has no Rust toolchain). Include tests for: legal transitions per type, refused `spec→accepted`, refused illegal transitions, and atomic-write/file-move correctness.
- Manual: approve a proposed spec and confirm the file moved to `specs/ready/` with `status: ready`; reject one to `changes-requested`; confirm an ADR approve → `decisions/` with `status: accepted`; confirm a rejected item shows the corrective prompt.
- `node scripts/check-version.mjs` still green; do not bump versions/CHANGELOG/CONTRACT without Project Lead coordination.

## Production quality and documentation
- Follow [[QUALITY]]; production work, not a prototype.
- Update implementation evidence and only the technical docs explicitly delegated here.
- Report any quality-policy exception explicitly.

## Risks and open decisions
- **D-1 / D-2 — RESOLVED by [[ADR-003-reject-as-first-class-status]]:** `rejected` is now a first-class status on ADR and Spec, and Agent proposals have a defined status set (`proposed`/`approved`/`rejected`). Contract is at v0.2; the next release is `1.1.0` (additive). Reject is therefore in scope for all proposable artifacts in this spec.
- **Risk:** writing + moving files must stay atomic and watcher-safe to avoid a corrupted/duplicated brain; keep the diff git-friendly (LF, minimal).
- **Constraint:** the backend is the source of truth for legal transitions — never trust the UI to prevent an illegal status.
- **Note:** do not bump `VERSION`/`package.json`/`Cargo.toml` — the `1.1.0` bump is recorded in `CHANGELOG`/`CONTRACT` and applied at the next coordinated release. `check-version.mjs` must stay green (all sources at `1.0.6`).

## Instructions for the assigned specialist
- Implement only the stated scope; do not add free-form status editing, reopening of rejected items, or bulk/agent-triggered writes.
- Enforce transition legality and the `spec→accepted` prohibition in the backend, not only the UI.
- Report changed files, tests run (with native Rust output), and known limitations.
- Produce production-grade, maintainable code; no placeholder/POC/partial behaviour.
- Do not change product scope, roadmap, ADRs, `CONTRACT.md`, or versioning. Raise contract questions to the Project Lead.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made
- Add `SpecStatus::Rejected` and `AdrStatus::Rejected` to the backend models (`spec.rs`, `adr.rs`) and loaders/mappings (`contract.rs`).
- Implement the `set_artifact_status(path, target_status)` command in the backend (`contract.rs` and `lib.rs`), which validates path safety with `PathGuard`, checks state transition legality per type, ensures Spec approval goes to `ready` instead of `accepted` (review-gated), writes files atomically (via `.tmp` files + rename), and moves Spec files into their corresponding status directories.
- Extend `types/index.ts` and `commands.ts` in the frontend to wire the new command and status enums.
- Redesign `ArtifactDetailModal.tsx` to support:
  - Rendering Approve/Reject buttons for `proposed` status items.
  - Adding a double-confirmation stage before writing to disk.
  - Surface a copyable corrective prompt banner for rejected items.
  - Resetting status-writing states gracefully during renders when the path changes.

### Files changed
- [`src-tauri/src/models/spec.rs`](file:///e:/Git/LMBrain/src-tauri/src/models/spec.rs)
- [`src-tauri/src/models/adr.rs`](file:///e:/Git/LMBrain/src-tauri/src/models/adr.rs)
- [`src-tauri/src/commands/contract.rs`](file:///e:/Git/LMBrain/src-tauri/src/commands/contract.rs)
- [`src-tauri/src/lib.rs`](file:///e:/Git/LMBrain/src-tauri/src/lib.rs)
- [`src-tauri/tests/contract_test.rs`](file:///e:/Git/LMBrain/src-tauri/tests/contract_test.rs)
- [`src/types/index.ts`](file:///e:/Git/LMBrain/src/types/index.ts)
- [`src/lib/commands.ts`](file:///e:/Git/LMBrain/src/lib/commands.ts)
- [`src/components/Layout/ArtifactDetailModal.tsx`](file:///e:/Git/LMBrain/src/components/Layout/ArtifactDetailModal.tsx)
- [`src/__tests__/ArtifactDetailModal.test.tsx`](file:///e:/Git/LMBrain/src/__tests__/ArtifactDetailModal.test.tsx)

### Verification performed
- Native Rust checks:
  - `cargo fmt --check` (clean)
  - `cargo clippy -- -D warnings` (clean)
  - `cargo test` output:
    ```
    Running tests\contract_test.rs (target\debug\deps\contract_test-bba0c3e183450453.exe)

    running 11 tests
    test test_status_md_heading_parsing_fallback ... ok
    test test_status_md_heading_parsing ... ok
    test test_build_roadmap_success ... ok
    test test_build_adrs_excludes_readme_and_non_genuine_artifacts ... ok
    test test_build_diagnostics_spec_status_mismatch ... ok
    test test_build_diagnostics_no_mismatch ... ok
    test test_build_diagnostics_malformed_frontmatter ... ok
    test test_build_diagnostics_task_status_mismatch ... ok
    test test_wikilink_index ... ok
    test test_initialize_kit_copies_template_and_refuses_overwrite ... ok
    test test_set_artifact_status_and_rejected_diagnostics ... ok

    test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
    ```
- Frontend checks:
  - `pnpm lint` (clean, no warnings/errors)
  - `pnpm test` output:
    ```
     ✓ src/__tests__/types.test.ts (7 tests) 11ms
     ✓ src/__tests__/wikilinks.test.ts (7 tests) 7ms
     ✓ src/__tests__/SettingsView.test.tsx (6 tests) 252ms
     ✓ src/__tests__/ProjectPulse.test.tsx (1 test) 323ms
     ✓ src/__tests__/markdownRenderer.test.tsx (8 tests) 129ms
     ✓ src/__tests__/WikiView.test.tsx (2 tests) 195ms
     ✓ src/__tests__/ArtifactDetailModal.test.tsx (5 tests) 240ms

     Test Files  7 passed (7)
          Tests  36 passed (36)
    ```
  - `pnpm build` (build success)
  - `node scripts/check-version.mjs` (LMBrain app and kit are aligned at v1.0.6)

### Deviations from the specification
- None.

### Handoff status
- [x] Ready for Project Lead review
