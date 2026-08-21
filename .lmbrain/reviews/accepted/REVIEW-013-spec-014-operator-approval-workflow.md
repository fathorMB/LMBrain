---
id: REVIEW-013
title: Review of SPEC-014 — operator approval workflow (approve/reject writes)
status: accepted
spec: SPEC-014
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: external-coding-agent
related_tasks: []
links: [SPEC-014, ADR-002, ADR-003]
created: 2026-06-23
updated: 2026-06-23
tags: [review, write, approval, workflow, security]
---

# Review of SPEC-014 — operator approval workflow

## Outcome

**accepted.** The first write-capable feature is implemented correctly and defensively. Transition legality and the key invariants are enforced in the backend, the UI gates writes to `proposed` artifacts with confirmation, scope held (no version bump, no agent-triggered or free-form writes), and the verifiable gates pass. Native Rust is delegated to CI but the agent pasted real `cargo test` output this time.

## Independent verification (reviewer-run)

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed — 7 files, 36 tests (+2 in `ArtifactDetailModal` for approve/reject). |
| `pnpm build` | Passed. |
| `node scripts/check-version.mjs` | Passed — all sources at v1.0.6 (no premature bump). |
| Native `cargo fmt/clippy/test` | Not run locally (no toolchain); evidence pastes real output incl. `test_set_artifact_status_and_rejected_diagnostics`; gated by CI. |

## Backend review — `set_artifact_status` (security-sensitive)

Read in full ([`contract.rs`](src-tauri/src/commands/contract.rs), wired in [`lib.rs`](src-tauri/src/lib.rs)). Correct and defensive:
- **Path safety:** resolves via `PathGuard` (workspace-scoped) before any read/write.
- **Source guard:** only `proposed` artifacts can transition; otherwise a clear error.
- **Per-type legality** matches the spec/ADR-003 table: Spec→`ready`/`rejected` (and **`spec → accepted` is explicitly refused** — review-gated), ADR→`accepted`/`rejected`, agent/MCP proposal→`approved`/`rejected`, agent profile→`active`/`inactive`. Illegal targets/types error out.
- **No path injection:** `target_status` is validated against the allowed set *before* it is used to build the destination directory, so it cannot escape the workspace.
- **Atomic write + move:** rewrites `status`/`updated`, preserves the body, normalizes to LF, writes a `.tmp` then renames; for specs it moves the file into `specs/<status>/` (creating it) and removes the original, so filesystem and status stay in agreement. Flat artifacts (ADR/proposals) update frontmatter in place.

## Frontend review — `ArtifactDetailModal`

- Write controls appear **only** when `status === "proposed"`; non-proposed artifacts stay read-only.
- Per-type approve/reject mapping mirrors the backend (agent profile reject is labelled "Deactivate", correctly reflecting that a profile is not "rejected").
- A confirmation stage precedes the write; on success the view reflects the new status, on error it surfaces the message.
- Rejected artifacts show the R2-F4-style corrective-prompt banner with copy.

## Scope discipline

- **No version/CONTRACT/CHANGELOG edits** in this handoff (the 1.1.0 bump remains recorded for the next release); `check-version` green. ✔
- No free-form status editing, no reopening path beyond an explicit operator/agent revise flow, no bulk or agent-triggered writes. ✔
- Contract v0.2 `rejected` is recognized by the status enums/diagnostic (covered by `test_set_artifact_status_and_rejected_diagnostics`). ✔

## Minor observations (non-blocking)

- The corrective prompt for a rejected item suggests setting it back to `proposed` for re-review. This is operator/agent-driven (not a silent reopen), so it is consistent with ADR-003's intent; if a stricter "no reopen" stance is later wanted, define an explicit reopen transition.
- `.tmp` temp file uses `with_extension("tmp")`; fine for the kit's single-extension filenames.

## Conditional on CI

As established, native Rust checks are gated by `build-installers.yml`; acceptance assumes that run is green. The pasted `cargo` output and the green frontend gates support acceptance.

## Release note

M-02's core is accepted. The operator intends to cut the coordinated **1.1.0** release now (round-2 fixes + Contract v0.2/reject + this approval workflow): align `package.json`/`Cargo.toml`/`Cargo.lock`/kit+live `VERSION` to `1.1.0`, finalize the CHANGELOG date, and push to trigger the release CI.
