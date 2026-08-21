---
id: REVIEW-029
title: "Review SPEC-027 kit migration detection"
status: changes-requested
spec_id: SPEC-027
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, kit, migration, ui]
links: [SPEC-027]
---
# Review SPEC-027 kit migration detection

## Verdict

Changes requested.

The implementation covers most of the intended product behavior: backend version comparison is semver-aware, the Project Pulse metadata panel shows project/bundled kit versions and migration status, migration prompts are copyable, and automated gates pass. Two acceptance and handoff issues remain.

## Findings

### [P1] The main migration prompt does not reference `STATUS.md`

SPEC-027 requires the generated migration prompt to reference `MIGRATIONS.md`, `CONTRACT.md`, `QUALITY.md`, `STATUS.md`, and `VERSION` before migration work:

- `.lmbrain/specs/backlog/SPEC-027-kit-migration-detection-and-project-lead-prompt.md:46`
- `.lmbrain/specs/backlog/SPEC-027-kit-migration-detection-and-project-lead-prompt.md:138`

The `migration-available` prompt reads policy files and `MIGRATIONS.md`, and later mentions `VERSION`, but it does not instruct the Project Lead to read `.lmbrain/STATUS.md`:

- `src/lib/handoffPrompt.ts:69`

The tests also do not assert this required prompt content:

- `src/__tests__/handoffPrompt.test.ts:48`

Required remediation:

1. Add `.lmbrain/STATUS.md` to the `migration-available` prompt's mandatory context.
2. Add or tighten the prompt test so the normal migration path asserts `STATUS.md` as well as `MIGRATIONS.md`, `CONTRACT.md`, `QUALITY.md`, and `VERSION`.

### [P2] SPEC-027 was not submitted for review and contains no implementation evidence

The spec file is still in backlog and its frontmatter still says `status: backlog`, despite the operator receiving it as ready for review:

- `.lmbrain/specs/backlog/SPEC-027-kit-migration-detection-and-project-lead-prompt.md:4`

Its implementation evidence section is still the template placeholder:

- `.lmbrain/specs/backlog/SPEC-027-kit-migration-detection-and-project-lead-prompt.md:195`

This breaks the LMBrain lifecycle and makes the review less auditable. Required remediation:

1. Move/transition SPEC-027 through the normal lifecycle to `review`.
2. Fill implementation evidence with changed files, verification performed, deviations, and handoff status.
3. Check off completed acceptance criteria.

### [P2] Kit README still advertises the old bundled kit version

The canonical bundled kit version is now `2.2.7`:

- `kit/.lmbrain/VERSION:1`

but the kit README still says `2.1.2`:

- `kit/README.md:3`

This is adjacent documentation, but it is operator-facing kit guidance and now conflicts with the version the app surfaces. Required remediation: update the README version text or remove the duplicated literal if the project wants `VERSION` to be the only source of truth.

## Verified Behavior

- Backend `WorkspaceInfo` exposes `project_kit_version`, `bundled_kit_version`, and `kit_migration_status`.
- Version comparison uses Rust `semver`.
- Project Pulse renders `.lmbrain version`, bundled kit, and kit status.
- Copy migration prompt is shown for `migration-available`, unknown version states, and missing migration guidance, while not shown for `up-to-date` or `project-newer-than-app`.
- `MIGRATIONS.md` includes a release-authoring policy for future kit-changing releases.

## Gates

- `pnpm lint` - pass.
- `pnpm test` - pass, 92 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- `cargo test -p lmbrain --test workspace_test` - pass, 6 tests.
- `cargo test` from `src-tauri` - pass.

## Required Remediation

Fix the missing `STATUS.md` prompt reference and test coverage, update the stale kit README version, and submit the spec through the proper LMBrain review lifecycle with implementation evidence.
