---
id: SPEC-027
title: "Kit migration detection and Project Lead prompt"
status: review
kind: feature
priority: high
area: kit
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-008]
links: [ADR-008]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, kit, migration, ui, project-lead]
---

# Kit migration detection and Project Lead prompt

## Objective

Make LMBrain detect when an opened project uses an older or incompatible `.lmbrain` kit version than the application would bootstrap today, and provide a copyable Project Lead migration prompt that points to versioned migration guidance instead of attempting an automatic migration.

## Context

Existing projects can carry older `.lmbrain` kits. The app already shows project metadata such as repository, branch, path, and `.lmbrain version`, but it does not clearly tell the operator whether the current project kit is up to date with the kit bundled in the app.

The operator wants the app to surface version mismatch in the project metadata panel and, when migration is available, generate a prompt that can be copied into an agent session running the Project Lead role. The Project Lead should then read the project state and `MIGRATIONS.md`, prepare a migration plan, preserve project-specific content, and only apply controlled `.lmbrain/` updates with explicit operator approval.

This deliberately keeps migration execution human-mediated. The app detects and assists; it does not rewrite old projects automatically.

## Scope

### Included

- Detect the current project kit version from `.lmbrain/VERSION`.
- Detect the application bundled/bootstrap kit version from the same source used by workspace initialization.
- Compare project kit version and bundled kit version.
- Show migration state in the project metadata / project pulse UI:
  - up to date;
  - migration available;
  - project newer than app/bundled kit;
  - unknown/unreadable version;
  - migration guidance unavailable or incomplete.
- Add a copyable "Project Lead migration prompt" when migration is available or when version state is uncertain enough to require Lead review.
- Ensure the prompt instructs the Project Lead to read `CONTRACT.md`, `QUALITY.md`, `STATUS.md`, `VERSION`, and `MIGRATIONS.md` before proposing or applying migration steps.
- Ensure the prompt explicitly forbids blind overwrite of project-customized kit files.
- Update kit migration documentation so every future kit release must include migration guidance.
- Add tests for version comparison, UI state rendering, and prompt content.

### Excluded

- Automatic migration execution from the app.
- Direct app writes to `.lmbrain/` for migration.
- Full three-way merge of kit templates.
- Remote update/download of kit assets.
- Activation of proposed agents or external MCPs.
- Changing the Project Lead's ordinary authority boundaries outside the migration prompt/workflow.

## Existing-project analysis

- `kit/.lmbrain/VERSION` is the canonical bundled kit version for bootstrap.
- `kit/.lmbrain/MIGRATIONS.md` already describes the current migration policy and released-version guidance.
- `kit/.lmbrain/CONTRACT.md` states `VERSION` is canonical and that migrations should read `MIGRATIONS.md`.
- `src/components/Pulse/ProjectPulse.tsx` renders the visible project metadata panel with `.lmbrain version`.
- `src-tauri/src/commands/contract.rs` and related models expose workspace metadata, including `kit_version`.
- Workspace initialization already copies or references bundled kit files, so the implementation should reuse that version source rather than duplicating a hard-coded value.
- Existing repair/inconsistency prompts can be used as UX precedent for a copyable prompt flow.

## Technical proposal

Introduce a small derived kit-version status model:

```ts
type KitMigrationStatus =
  | "up-to-date"
  | "migration-available"
  | "project-newer-than-app"
  | "unknown-project-version"
  | "unknown-bundled-version"
  | "migration-guidance-missing";
```

The backend should expose both:

- `project_kit_version`: read from the opened workspace `.lmbrain/VERSION`;
- `bundled_kit_version`: read from the app's bundled/bootstrap kit `VERSION`.

Version comparison should use semantic-version parsing where possible. If either side is missing or unparsable, the app should avoid claiming a migration path and instead show an "unknown" state with a diagnostic prompt.

The project metadata panel should stay compact. Suggested UI:

- `.lmbrain version`: current project version;
- `Bundled kit`: app/bootstrap kit version;
- `Kit status`: colored status label;
- button: `Copy migration prompt` when actionable.

The migration prompt should be generated deterministically from the project path, current version, bundled version, and detected status. It should instruct the Project Lead to:

1. work in the current repository;
2. read mandatory policy files;
3. read `.lmbrain/MIGRATIONS.md`;
4. inspect git state before writing;
5. compare migration notes across the version interval;
6. produce a migration plan;
7. preserve project-specific edits;
8. ask operator confirmation before additive migration writes;
9. create a migration spec or report conflicts when changes are breaking or ambiguous;
10. update `.lmbrain/VERSION` only after required migration steps and validation succeed.

## Files and areas involved

- `kit/.lmbrain/MIGRATIONS.md`
- `kit/.lmbrain/CONTRACT.md`
- `kit/.lmbrain/AGENT.md`
- `src-tauri/src/models/workspace.rs` or equivalent workspace metadata model
- `src-tauri/src/commands/workspace.rs` or equivalent open-workspace command path
- `src-tauri/src/commands/contract.rs` if kit diagnostics are extended there
- `src/types/index.ts`
- `src/components/Pulse/ProjectPulse.tsx`
- `src/lib/commands.ts`
- `src/lib/handoffPrompt.ts` or a new prompt helper if that is the existing local pattern
- `src/__tests__/ProjectPulse.test.tsx` or the relevant pulse test file
- `src/__tests__/handoffPrompt.test.ts`
- Rust tests covering bundled/project version status if backend logic is added
- `docs/kit.md`
- `docs/architecture.md`

## Acceptance criteria

- [x] The app exposes the bundled/bootstrap kit version without hard-coding it in the frontend.
- [x] The app compares project kit version and bundled kit version using semver-aware logic, with safe fallback for missing/unparsable values.
- [x] The project metadata panel shows project version, bundled kit version, and a clear kit status.
- [x] Older project version + newer bundled version shows `migration available` and a copyable Project Lead migration prompt.
- [x] Equal versions show `up to date` and do not pressure the operator to migrate.
- [x] Project version newer than bundled version shows a caution state explaining the app may be older than the project kit.
- [x] Missing/unparsable project or bundled version shows an unknown state and produces a diagnostic Project Lead prompt rather than a confident migration prompt.
- [x] The generated prompt references `MIGRATIONS.md`, `CONTRACT.md`, `QUALITY.md`, `STATUS.md`, and `VERSION`.
- [x] The generated prompt forbids blind overwrites and requires preserving project-specific content.
- [x] The generated prompt tells the Project Lead to update `VERSION` only after migration steps and validation succeed.
- [x] `MIGRATIONS.md` documents the release requirement: every kit-changing release must include migration guidance with supported source versions, required edits, validation, and rollback notes.
- [x] Automated tests cover version comparison states, metadata UI rendering, and prompt content.
- [x] `pnpm lint`, `pnpm test`, `pnpm build`, and relevant Rust tests pass.

## Implementation plan

1. Identify the canonical backend path that reads bundled bootstrap kit metadata.
2. Add a reusable kit-version comparison helper with semver parsing and explicit unknown states.
3. Extend workspace metadata returned to the frontend with bundled kit version and kit migration status.
4. Add a deterministic migration prompt helper.
5. Update the project metadata panel with compact version/status rows and a copy prompt action.
6. Extend `MIGRATIONS.md` with release-authoring requirements for future kit changes.
7. Add focused frontend and backend tests.
8. Run quality gates.

## Required verification

- `pnpm lint`
- `pnpm test`
- `pnpm build`
- Relevant Rust tests for workspace metadata/version comparison
- Manual check with at least:
  - project version equal to bundled version;
  - project version older than bundled version;
  - project version newer than bundled version;
  - missing/unparsable project `VERSION`.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Do not implement automatic migration writes in this spec.
- Do not hard-code the current bundled version in React code.
- Keep the Project metadata panel compact and operational, matching the existing dense UI style.
- Update `docs/kit.md` and `docs/architecture.md` if backend/frontend metadata contracts change.
- Update `kit/.lmbrain/MIGRATIONS.md` with the migration-note authoring policy.

## Risks and open decisions

- The app must not overstate migration safety. A version mismatch means "needs Project Lead migration review", not necessarily "safe one-click upgrade".
- Some old projects may not have a `VERSION` file or may have customized kit files. The prompt must guide review and preservation rather than replacement.
- If bundled kit version cannot be read consistently in dev/test/production builds, the implementation should expose `unknown-bundled-version` rather than falling back to a misleading constant.
- Open decision for implementation: whether migration guidance availability can be machine-checked from `MIGRATIONS.md` in this spec, or whether `migration-guidance-missing` is reserved for a later structured migration-index feature.

## Instructions for the assigned specialist

- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

### Changes made
- Implemented Rust-side `KitMigrationStatus` enum and version properties inside `WorkspaceInfo`.
- Added semver comparison logic to `WorkspaceService::validate_workspace` and verified presence of target guidance headers (`### <bundled_version>`) in MIGRATIONS.md.
- Updated `open_workspace` Tauri command to pass the resolved bundled kit path down.
- Implemented `buildMigrationPrompt` helper creating deterministic manual upgrade/diagnostic prompts.
- Added project version, bundled kit version, colored status labels, and a Copy Migration Prompt button to the project metadata panel.
- Added comprehensive backend integration tests (`workspace_test.rs`) and frontend unit tests (`handoffPrompt.test.ts`, `ProjectPulse.test.tsx`).
- Updated documentation in `MIGRATIONS.md`, `kit.md`, `architecture.md`, and `kit/README.md`.

### Files changed
- [src-tauri/Cargo.toml](file:///E:/Git/LMBrain/src-tauri/Cargo.toml)
- [src-tauri/src/models/workspace.rs](file:///E:/Git/LMBrain/src-tauri/src/models/workspace.rs)
- [src-tauri/src/commands/workspace.rs](file:///E:/Git/LMBrain/src-tauri/src/commands/workspace.rs)
- [src-tauri/src/lib.rs](file:///E:/Git/LMBrain/src-tauri/src/lib.rs)
- [src-tauri/tests/workspace_test.rs](file:///E:/Git/LMBrain/src-tauri/tests/workspace_test.rs)
- [src/types/index.ts](file:///E:/Git/LMBrain/src/types/index.ts)
- [src/lib/handoffPrompt.ts](file:///E:/Git/LMBrain/src/lib/handoffPrompt.ts)
- [src/components/Pulse/ProjectPulse.tsx](file:///E:/Git/LMBrain/src/components/Pulse/ProjectPulse.tsx)
- [src/__tests__/handoffPrompt.test.ts](file:///E:/Git/LMBrain/src/__tests__/handoffPrompt.test.ts)
- [src/__tests__/ProjectPulse.test.tsx](file:///E:/Git/LMBrain/src/__tests__/ProjectPulse.test.tsx)
- [src/__tests__/WikiView.test.tsx](file:///E:/Git/LMBrain/src/__tests__/WikiView.test.tsx)
- [kit/.lmbrain/MIGRATIONS.md](file:///E:/Git/LMBrain/kit/.lmbrain/MIGRATIONS.md)
- [.lmbrain/MIGRATIONS.md](file:///E:/Git/LMBrain/.lmbrain/MIGRATIONS.md)
- [kit/README.md](file:///E:/Git/LMBrain/kit/README.md)
- [docs/kit.md](file:///E:/Git/LMBrain/docs/kit.md)
- [docs/architecture.md](file:///E:/Git/LMBrain/docs/architecture.md)

### Verification performed
- Ran Rust tests: `cargo test` (all 86 tests passed, including `workspace_test.rs`).
- Ran frontend tests: `pnpm test` (all 92 tests passed).
- Built frontend production code: `pnpm build` (successful compilation and bundling).
- Verified linter rules: `pnpm lint` (passed without warnings).

### Deviations from the specification
- None.

### Handoff status

- [x] Ready for Project Lead review
