---
id: REVIEW-014
title: "Review of SPEC-017 — controlled-mutation engine (lmbrain-core + MCP)"
status: accepted
spec: SPEC-017
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
links: [SPEC-017, ADR-004]
created: 2026-06-23
updated: 2026-06-23
tags: [review, mcp, transitions, core]
---

# Review of SPEC-017 — controlled-mutation engine (lmbrain-core + MCP)

## Outcome

**Changes requested.** A credible first implementation with sound building blocks, but it deviates from the spec's core intent (ADR-004 single-source-of-truth) and misses the mandated test coverage. Rust correctness was not compiled locally (no toolchain); findings are from code inspection and must be validated by `cargo test` in CI after the corrections.

## Acceptance-criteria compliance

| Criterion | Status |
| --- | --- |
| `lmbrain-core` tauri-free crate; `src-tauri` builds against it; prior tests pass | ⚠️ Partial — core exists but **reimplements** parser/PathGuard/invariants instead of extracting them; not compiled here |
| Transition functions per artifact, file move + `updated` bump | ✅ Present (`transitions::allowed` + `transition`) |
| Each invariant enforced as a hard block, same predicate backs the diagnostic | ❌ Not met — diagnostics still use their own copies, not `core::invariants` |
| `force:true` requires `reason` and records it | ✅ Present (gating + body note) |
| Surgical frontmatter edit preserves order/comments; atomic writes | ✅ Present and partially tested |
| Creation allocates next ID atomically and scaffolds from templates | ✅ Present (lock + template fill) |
| MCP exposes per-verb + read tools; protocol integration test | ⚠️ Present but tool schemas are generic/inaccurate |
| `cargo test` green on Linux + Windows; MCP built per-OS | ⏳ Unverified (not run; CI step added) |
| Kit prompts direct agents to the tools; bootstrap registers server | ✅ Done |
| Every acceptance item has automated test evidence | ❌ Not met — see Tests |

## Code observations

Positives: clean per-artifact state machine in `allowed()`; surgical frontmatter mutator that preserves comments and key order; atomic write with Windows handling; ID allocation guarded by an exclusive lock; `force`+`reason` gating; correct review→spec link field (`spec:`); MCP read tools present; CONTRACT/AGENT/handoff docs and missing templates added.

## Tests and verification

`lmbrain-core/tests/transitions.rs` has 3 tests (one happy-path move, one force-needs-reason, one parser round-trip) and `lmbrain-mcp/tests/protocol.rs` has 1. The spec required a valid + illegal test for each `(from, verb)` and a test per invariant; creation/ID-allocation and override/audit are also untested. Coverage is far below the spec.

## Production quality and documentation compliance

`set_artifact_status` leaves the entire previous implementation as unreachable dead code behind `#[allow(unreachable_code)]` — not production-grade. Docs and templates were updated correctly.

## Findings

### Blocking
- **R-1 — Duplication instead of extraction (violates ADR-004 intent).** `lmbrain-core` reimplements the frontmatter parser (`frontmatter.rs`), `PathGuard` (`path.rs`), and the invariants (`invariants.rs`) in parallel with the existing `src-tauri` code, and `build_diagnostics` still uses its own invariant copies rather than `core::invariants`. This creates two sources of truth for every rule — the exact drift ADR-004 set out to remove. Make the diagnostics (and ideally the app parser/PathGuard) consume the core, so a rule cannot diverge.
- **R-2 — Dead code.** `set_artifact_status` returns early into the core and leaves ~120 unreachable lines behind `#[allow(unreachable_code)]`. Remove the old body.
- **R-3 — Test coverage below spec.** Add valid + illegal transition tests for each `(from, verb)`, a test per invariant (`spec_has_accepted_review`, `single_ready_handoff`, `task_planned_requires_ready_spec`, `task_criteria_complete_with_evidence`, `recommended_agent_resolves`), plus creation/ID-allocation and override/audit tests.
- **R-4 — Repository hygiene.** The workspace builds to a root `target/` that is **not** git-ignored (only `src-tauri/target/` is), risking committed build artifacts; and a stale `src-tauri/Cargo.lock` remains alongside the new workspace `Cargo.lock`. Add `/target` to `.gitignore` and remove the per-crate lockfile.

### Should-fix
- **R-5 — MSRV violation.** `Option::is_none_or` requires Rust ≥ 1.82, but `rust-version = "1.77.2"`. Raise the MSRV or avoid `is_none_or`.
- **R-6 — Inaccurate MCP tool schemas.** All tools advertise the same `{path, target_status, force, reason}` schema with `additionalProperties: true`; `lmbrain_create` and the `set_*` tools don't declare their real parameters. Provide tight per-verb schemas.
- **R-7 — Hardcoded server version.** `serverInfo.version` is `"1.2.6"`; derive it from `env!("CARGO_PKG_VERSION")`.
- **R-8 — Minor logic.** `next_id` always returns 1 for non-numeric agent IDs; `task_criteria_complete_with_evidence` treats a task with zero criteria as complete; `create` uses `join("")` for flat artifacts.

## Required follow-up

Address R-1 through R-4 (blocking) and R-5 through R-8 (should-fix), re-run the full suite (`pnpm test` + `cargo test` on Linux and Windows in CI), and resubmit. BUG-005 (handoff prompt spec-path slug) was fixed separately by the operator-side review and is out of scope for this round.

## Second review pass (re-review)

The implementer addressed the findings. Status after the second pass:

- **R-2 resolved** — `set_artifact_status` is now a clean wrapper over `transition_from_proposed`; the dead code is gone.
- **R-3 resolved** — strong coverage: a valid + illegal case for every declared `(from, verb)`, a test per invariant, plus creation/ID-allocation and force/reason-audit tests.
- **R-4 resolved** — `/target/` is git-ignored and the stale `src-tauri/Cargo.lock` was removed.
- **R-5 resolved** — `is_none_or` replaced with `map_or` (MSRV-safe).
- **R-6 resolved** — per-verb MCP tool schemas with `required`/`enum`/`additionalProperties: false`.
- **R-7 resolved** — server version from `env!("CARGO_PKG_VERSION")`.
- **R-8 resolved** — `next_id`, zero-criteria, and flat-artifact creation fixed and covered by tests.
- **R-1 partially resolved** — diagnostics now use `core::invariants` for `folder_matches_status` and `task_planned_requires_ready_spec`, and `set_artifact_status` routes through the core. The **`recommended_agent` diagnostic** in `build_diagnostics` still uses a local copy instead of `core::invariants::recommended_agent_resolves`; route it through the core to remove the last duplicated predicate.

**Outstanding before acceptance:**
1. R-1 residual: route the `recommended_agent` diagnostic through `core::invariants::recommended_agent_resolves` (~3 lines).
2. Acceptance evidence: `cargo test` (Linux + Windows) and `pnpm test` green in CI — the work is uncommitted and unverified by the build.

## Final decision

**Accepted (2026-06-23).** All findings R-1 through R-8 are resolved: the implementer addressed R-2..R-8, and R-1 was closed by routing the `recommended_agent` diagnostic through `core::invariants::recommended_agent_resolves`. Verifiable evidence: release **1.3.1** is green in CI on both Linux and Windows — `pnpm lint`/`pnpm test`, `cargo test` (lmbrain-core + lmbrain-mcp), the per-OS installer builds, and the MCP-binary build all pass, and the release published. SPEC-017 moves to `accepted`.

Two workspace-refactor side effects surfaced only once `cargo test`/build ran on this code and were fixed in 1.3.1 (CI artifact paths pointing at the relocated workspace `target/`, and a Windows short-path assertion in the `create` test).
