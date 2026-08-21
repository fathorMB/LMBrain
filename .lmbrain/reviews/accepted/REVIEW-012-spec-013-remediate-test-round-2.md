---
id: REVIEW-012
title: Review of SPEC-013 — remediate test round 2 findings
status: accepted
spec: SPEC-013
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: external-coding-agent
related_tasks: []
links: [SPEC-012, SPEC-013, ADR-002]
created: 2026-06-22
updated: 2026-06-22
tags: [review, parser, paths, roadmap, diagnostics, modal]
---

# Review of SPEC-013 — remediate test round 2 findings

## Outcome

**accepted.** All five work items (WI-1…WI-5) are implemented at the decided scopes, scope discipline held (no write/approval capability, no version/CONTRACT changes), and the verifiable gates pass. Native Rust checks are delegated to CI (no local toolchain). One minor evidence-accuracy nit noted below; it does not affect acceptance.

## Independent verification (reviewer-run)

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed — 7 files, 34 tests (was 5/30; +2 files: ProjectPulse, ArtifactDetailModal). |
| `pnpm build` | Passed. |
| `node scripts/check-version.mjs` | Passed — aligned at v1.0.6 (no version drift this round). |
| Native `cargo fmt/clippy/test` | Not run locally (no Rust toolchain); evidence reports specific counts; delegated to release CI. |

## Scope discipline (explicitly checked)

- **No approve/reject / status-write capability** was added (the only `approve`/`reject` hits are the pre-existing MCP-proposal enum and the `approved_root` path guard). R2-F5 parts 2–3 correctly deferred to M-02 / [[ADR-002-in-app-artifact-status-writes]]. ✔
- **No `VERSION`/`CHANGELOG`/`CONTRACT.md` changes** — the regression class from round 1 did not recur. ✔
- The many `models/*.rs` edits are confirmed to be **only** the `malformed` flag (WI-1), not unrelated changes. ✔

## Per-work-item assessment

### WI-1 — Malformed-frontmatter handling — ACCEPTED
- Kit + live templates quote `title` (`title: "…"`) and carry a colon-quoting note in all reference templates; the 1.0.6 "IDs only" note is preserved.
- A proper **explicit `malformed` flag** is threaded from [`parse_frontmatter`](src-tauri/src/commands/parser.rs:62) through `ParsedDocument` and the artifact models to the TS types — not a fragile `UNKNOWN` string match, as required.
- Cards in Taskboard/Decisions/Reviews flag the malformed state; covered by tests.

### WI-2 — Windows `\\?\` path normalization — ACCEPTED
- [`clean_path`](src-tauri/src/commands/filesystem.rs:8) correctly maps `\\?\UNC\…` → `\\…` and strips `\\?\`, applied in `set_root`, `resolve`, `validate_workspace`, and `initialize_kit`.
- Path-safety tests updated to expect clean paths (run in CI).

### WI-3 — Roadmap driven by `ROADMAP.md` — ACCEPTED
- New `models/roadmap.rs` + `build_roadmap`/`parse_roadmap_content` in `contract.rs` parse milestone headings (em-/en-dash/hyphen tolerant), outcome/target/status/risks/specs/decisions; `get_roadmap` registered in `lib.rs`. Missing `ROADMAP.md` returns a clean error.
- `RoadmapView` now pulls from the backend and renders an explicit "Unmapped Artifacts" group instead of a phantom milestone. Rust tests added (run in CI).

### WI-4 — "Copy fix prompt" on diagnostics — ACCEPTED
- Per-diagnostic toggle in `ProjectPulse` reveals a read-only textarea with a type-aware prompt + one-click copy; no backend change. Covered by `ProjectPulse.test.tsx`.

### WI-5 — Read-only artifact detail modal — ACCEPTED
- `ArtifactDetailModal` renders the full document with `role="dialog"`, `aria-modal`, Escape-to-close, and a focus trap. **No status-changing controls** (only a close affordance) — correctly read-only per scope. Covered by `ArtifactDetailModal.test.tsx`.

## Minor finding (non-blocking)

The evidence "Files changed" list omits the seven artifact `models/*.rs` files that received the `malformed` flag (only `models/mod.rs` and `models/roadmap.rs` are listed). The change itself is correct and in scope; the file list is just incomplete. Tighten future evidence to list all touched files.

## Conditional on CI

As in round 1, the native Rust checks (`cargo fmt/clippy/test`, including the new parser/path-safety/roadmap tests) are not locally reproducible here and are gated by `build-installers.yml`. Acceptance assumes that CI run is green; if the native build/tests fail there, revisit. The frontend gates and version alignment are confirmed locally.
