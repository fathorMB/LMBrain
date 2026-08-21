---
id: REVIEW-030
title: "Re-review SPEC-027 kit migration detection remediation"
status: accepted
spec_id: SPEC-027
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, kit, migration, ui, remediation]
links: [SPEC-027, REVIEW-029]
---
# Re-review SPEC-027 kit migration detection remediation

## Verdict

Accepted.

The REVIEW-029 findings have been resolved. SPEC-027 is now in `review` with implementation evidence, the main migration prompt includes `.lmbrain/STATUS.md`, and the kit README version matches the canonical bundled kit version.

## Resolved Findings

- REVIEW-029 P1: `buildMigrationPrompt` now includes `.lmbrain/STATUS.md` in the normal `migration-available` workflow, and the prompt test asserts `STATUS.md`.
- REVIEW-029 P2: SPEC-027 was moved to `review`, acceptance criteria are checked, and implementation evidence is filled.
- REVIEW-029 P2: `kit/README.md` now reports kit version `2.2.7`, matching `kit/.lmbrain/VERSION`.

## Verified Behavior

- Backend workspace metadata exposes project kit version, bundled kit version, and semver-derived migration status.
- Project Pulse shows project version, bundled kit version, kit status, and copyable migration prompt for actionable/uncertain states.
- The migration prompt instructs the Project Lead to read `CONTRACT.md`, `QUALITY.md`, `STATUS.md`, `VERSION`, and `MIGRATIONS.md`.
- The prompt forbids blind overwrites, requires preserving project-specific content, and delays `VERSION` updates until migration steps and validation succeed.
- No automatic migration writes were introduced.

## Gates

- `pnpm lint` - pass.
- `pnpm test` - pass, 92 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- `cargo test -p lmbrain --test workspace_test` - pass, 6 tests.

## Result

SPEC-027 is accepted from review and can be moved to `done` after the implementation commit/evidence step required by the LMBrain workflow.
