---
id: REVIEW-015
title: "Review SPEC-023 v3 context economy"
status: changes-requested
spec_id: SPEC-023
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, context-economy]
links: [SPEC-023, ADR-004]
---

# Review SPEC-023 v3 context economy

## Verdict

Changes requested.

The implementation adds a sensible context-pack foundation and the local quality gates pass, but two acceptance criteria are not met yet: diagnostics are stubbed out in the new context layer, and version documentation now conflicts with the actual kit/app version.

## Findings

### R-1 - Context packs do not include real diagnostics

Severity: blocking

Evidence:

- `build_project_digest` derives `diagnostics_summary` and blockers from `scan_diagnostics(root)` in `lmbrain-core/src/context.rs`.
- `scan_diagnostics` currently returns `Vec::new()` unconditionally.
- `build_spec_context` derives its `diagnostics` field from `spec_diagnostics(&lmbrain, &id)`.
- `spec_diagnostics` currently returns `Vec::new()` unconditionally.

Code references:

- `lmbrain-core/src/context.rs:150`
- `lmbrain-core/src/context.rs:491`
- `lmbrain-core/src/context.rs:250`
- `lmbrain-core/src/context.rs:636`

Why this blocks acceptance:

SPEC-023 requires context-pack tools to resolve diagnostics deterministically and to include "diagnostics affecting the handoff." The implementation evidence lists this as a known limitation rather than a completed behavior. Missing-reference warnings are useful, but they do not replace the existing LMBrain diagnostics surface.

Required correction:

Implement real diagnostics in the context-pack layer or expose/reuse the existing diagnostic logic through a shared core-compatible API. At minimum, `lmbrain_project_digest` must report meaningful diagnostic totals/blockers, and `lmbrain_spec_context` must report diagnostics relevant to the requested spec. Add regression tests with a malformed or invariant-violating fixture that proves the diagnostics are non-empty.

### R-2 - Version documentation regresses to 2.1.2 while the kit/app are 2.2.7

Severity: blocking

Evidence:

- `kit/.lmbrain/VERSION` is `2.2.7`.
- `package.json` is `2.2.7`.
- `src-tauri/Cargo.toml` is `2.2.7`.
- The patch changes `kit/.lmbrain/MIGRATIONS.md` to say "The current released kit is `2.1.2`" and labels the new context-economy migration as `2.1.2`.
- `docs/architecture.md` says the v3 context-pack tools were "added in kit 2.1.2".

Code/document references:

- `kit/.lmbrain/MIGRATIONS.md:7`
- `kit/.lmbrain/MIGRATIONS.md:9`
- `kit/.lmbrain/MIGRATIONS.md:11`
- `docs/architecture.md:84`

Why this blocks acceptance:

SPEC-023 explicitly updates kit docs/templates/migrations for v3 context economy. Introducing stale version numbers makes the migration guidance unreliable and conflicts with the repo's existing version state.

Required correction:

Align the migration and docs language with the actual release/version plan. If this is part of app/kit `2.2.7`, use `2.2.7`; if it is intended for a later v3 release, describe it as unreleased/next without claiming `2.1.2`. Run the existing version check if available.

## Acceptance criteria assessment

- [x] Kit documents role-specific context tiers and discourages broad artifact reads.
- [x] `lmbrain-mcp` exposes read-only context-pack tools for project digest, spec handoff context, and review context.
- [ ] Context-pack tools resolve linked specs, ADRs, reviews, agent profiles, roadmap milestone, diagnostics, and missing-reference warnings deterministically. Diagnostics are stubbed.
- [x] Context-pack tools do not mutate files and are covered by protocol/core tests.
- [x] Generated specialist handoff prompts reference the assigned spec and recommend the new context-pack flow without weakening source-code inspection.
- [x] The app's Agents/MCP view lists the new tools with descriptions.
- [ ] Existing tests pass and new tests cover compact prompt generation and MCP context-pack behavior. MCP/core behavior is covered; compact prompt generation has no direct test coverage.
- [ ] Documentation explains the expected token-saving behavior without claiming unmeasured savings. Token-saving language is acceptable, but version documentation is inconsistent.

## Verification performed

- `cargo test -p lmbrain-core` - passed, 20 tests. Rust emitted warnings for unused `scan_md_files` and unread `DiagnosticEntry.path`.
- `cargo test -p lmbrain-mcp` - passed, 10 unit tests and 2 protocol tests.
- `pnpm test -- --runInBand` - passed, 47 tests.
- `pnpm lint` - passed.
- Inspected implementation evidence in `SPEC-023`.
- Inspected diffs for `lmbrain-core/src/context.rs`, `lmbrain-mcp/src/main.rs`, frontend prompt/UI changes, kit docs, and product docs.

## Required remediation

Keep remediation inside SPEC-023:

1. Replace diagnostic stubs with real, tested diagnostics or a shared diagnostic adapter that satisfies the spec.
2. Fix version/migration documentation so it matches the actual kit/app version or clearly marks the change as next/unreleased.
3. Add direct test coverage for generated handoff prompt/context-economy text, or document why existing frontend tests already cover it.
4. Remove or justify the new Rust warnings.

Resubmit SPEC-023 for review after those changes and updated implementation evidence.
