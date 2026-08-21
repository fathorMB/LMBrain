---
id: SPEC-044
# Note: Quote the title if it contains a colon
title: "LMBrain 3.0.1 tranche 1: restore installer publication gates"
status: review
kind: bugfix
priority: high
area: release-and-ci
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-18
updated: 2026-07-18
tags: [3.0.1, github-actions, installer, lint]
activity:
  - date: 2026-07-18
    action: "created"
activity:
  - date: 2026-07-18
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-18
    action: "set recommended_agent"
activity:
  - date: 2026-07-18
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-18
    action: "transitioned working -> review"
---
# LMBrain 3.0.1 tranche 1: restore installer publication gates

## Objective

Restore deterministic installer publication after the failed LMBrain 3.0.0
release run, and ship the correction as the first bounded tranche of patch
release 3.0.1.

## Context

GitHub Actions run [29582472695](https://github.com/fathorMB/LMBrain/actions/runs/29582472695)
failed in the `Linux installer` job during `Lint and test`. Six ESLint errors in
the newly added repository dashboard and transcript-search UI stopped Bash
before tests and packaging. The Windows matrix leg reported the same lint
errors but continued to the passing test command because a multiline PowerShell
step returned only the final native command's status; it therefore produced a
false-green installer result.

The operator explicitly authorized the Project Lead on 2026-07-18 to take over
this narrow corrective implementation, treat it as the first tranche of 3.0.1,
and work on a dedicated branch. This satisfies the operator-directed takeover
condition in `AGENT.md`.

## Scope
### Included

- Resolve all six observed frontend lint errors without weakening shared lint rules.
- Preserve repository-dashboard loading, refresh, PAT, and error behavior.
- Preserve transcript ANSI stripping and Fast Refresh compatibility.
- Make lint and test independent workflow gates on Windows and Linux.
- Align the app package, Tauri package, lockfile, and bundled-kit versions at 3.0.1.
- Add the corresponding bundled-kit changelog and no-rewrite migration entry.
- Run focused and complete frontend gates plus version and diff checks.

### Excluded

- Product or architecture changes beyond the failing components.
- New dependencies or broad lint-rule suppression.
- Publishing, pushing, or rerunning GitHub Actions without a separate handoff decision.
- Resolving unrelated board/status drift or modifying the operator's untracked files.

## Existing-project analysis

- `RepositoryView.tsx` catches three values as `any`; the repository already uses
  `unknown`-based error-message helpers elsewhere.
- Its mount effect calls a loader that synchronously sets loading/error state,
  triggering `react-hooks/set-state-in-effect`.
- `HistorySearchPanel.tsx` exports `stripAnsi` beside a component, violating the
  Vite Fast Refresh contract, and its intentional control-code regex triggers
  `no-control-regex`.
- `build-installers.yml` combines `pnpm lint` and `pnpm test` in one multiline
  platform-default shell step. PowerShell continues after the failed native lint
  command and returns the later test result; Bash exits immediately.
- Version alignment is enforced by `scripts/check-version.mjs`, and the release
  gate triggers only when `package.json` changes version.

## Technical proposal

Extract the ANSI helper into a non-component library module and keep a narrowly
documented lint exception on the deliberate control-code matcher. Refactor the
repository loader so the mount effect begins with an asynchronous boundary and
event-driven refresh paths own their synchronous busy-state changes. Convert
all caught values through an `unknown`-safe message helper. Split workflow lint
and test into separate steps so each native exit code is independently enforced.
Apply the coordinated patch-version metadata and document that the kit requires
no artifact rewrite.

## Files and areas involved

- `src/components/Repository/RepositoryView.tsx`
- `src/components/Sessions/HistorySearchPanel.tsx`
- `src/lib/ansi.ts`
- relevant frontend tests
- `.github/workflows/build-installers.yml`
- version manifests/lockfile and `.lmbrain`/`kit/.lmbrain` release documentation

## Acceptance criteria

- [x] `pnpm lint` reports no errors on the 3.0.1 branch.
- [x] Repository-dashboard initial load, refresh, token save/delete, and error paths remain correctly typed and tested.
- [x] Transcript output still strips ANSI control sequences and the component file exports components only.
- [x] Lint and test are independent GitHub Actions gates on both matrix platforms.
- [x] All release-canonical version sources are aligned at 3.0.1 and the version gate passes; independently versioned core/MCP crates and the development workspace kit are unchanged.
- [x] Bundled-kit changelog/migration guidance truthfully describes tranche 1 and requires no artifact rewrite.
- [x] Frontend tests and production build pass with no unrelated source changes.

## Implementation plan

1. Refactor the two failing frontend components and add targeted regression coverage.
2. Split the workflow quality commands into deterministic independent steps.
3. Align 3.0.1 version and release/migration documentation.
4. Run lint, focused/full tests, build, version alignment, and diff checks.
5. Perform a separate acceptance-criteria and diff review, then record evidence.

## Required verification

- `pnpm lint`
- focused Vitest files for repository dashboard and transcript search
- `pnpm test`
- `pnpm build`
- `node scripts/check-version.mjs`
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- A broad ESLint disable would hide future defects and is not permitted; only
  the intentional ANSI control-code matcher may carry a targeted explanation.
- The remote Linux packaging/build and asset upload remain unverified until the
  branch is pushed/merged and a release-triggering run executes.
- Existing untracked `.pi/` and `ISSUE-4-PLAN.md` are operator-owned and excluded.

## Project Lead corrective takeover

- **Authorized by:** human operator on 2026-07-18.
- **Rationale:** a bounded set of known frontend lint defects blocks the installer release.
- **Branch:** `codex/v3.0.1-ci-hotfix` from the failed 3.0.0 release commit.
- **Scope boundary:** the six lint findings, cross-platform gate correctness,
  coordinated patch version, and required release documentation only.
- **Verification plan:** targeted tests, full frontend lint/test/build, version
  alignment, diff checks, and a separate acceptance-criteria review before handoff.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
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

- Refactored repository data loading around an asynchronous fetch boundary,
  cancellation guard, and `unknown`-safe error messages.
- Added regression coverage for initial failure, manual refresh, PAT save/delete,
  and ANSI cleanup while retaining the existing dashboard/search coverage.
- Moved ANSI cleanup to `src/lib/ansi.ts`, leaving the component module
  Fast-Refresh-safe and documenting the single intentional control-regex exception.
- Split frontend lint and tests into independent installer workflow steps.
- Aligned release-canonical app, Tauri, lockfile, and bundled-kit versions at
  3.0.1; documented the patch and no-rewrite migration.
- Corrected session documentation that still described the 3.0 transcript
  search as deferred.

### Files changed

- `.github/workflows/build-installers.yml`
- `Cargo.lock`, `package.json`, `src-tauri/Cargo.toml`
- `src/components/Repository/RepositoryView.tsx`
- `src/components/Sessions/HistorySearchPanel.tsx`
- `src/lib/ansi.ts`
- `src/__tests__/RepositoryView.test.tsx`, `src/__tests__/ansi.test.ts`
- `kit/.lmbrain/VERSION`, `kit/.lmbrain/CHANGELOG.md`, `kit/.lmbrain/MIGRATIONS.md`
- `docs/sessions.md`

### Verification performed

- `node scripts/check-version.mjs` — passed at 3.0.1.
- `pnpm lint` — passed with zero errors.
- `pnpm vitest run src/__tests__/RepositoryView.test.tsx src/__tests__/HistorySearchPanel.test.tsx src/__tests__/ansi.test.ts` — passed, 3 files / 13 tests.
- `pnpm test` — passed, 25 files / 141 tests.
- `pnpm build` — passed; TypeScript and Vite production build completed.
- `cargo test --workspace` — passed across the complete Rust workspace.
- `git diff --check` — passed.
- `cargo fmt --all -- --check` — blocked by pre-existing formatting drift in
  unrelated Rust files already present in the 3.0.0 release commit; no Rust
  source file is changed by this tranche.
- `cargo clippy --workspace --all-targets -- -D warnings` — blocked by three
  pre-existing `too_many_arguments` findings in `lmbrain-core/src/context.rs`;
  no affected Rust source is changed by this tranche.
- `actionlint` — unavailable locally; workflow syntax remains a minimal split
  of an existing valid step and requires the remote Actions run for definitive validation.

### Verification transcript

```text
LMBrain app and kit are aligned at v3.0.1.
Test Files  3 passed (3)
Tests       13 passed (13)
Test Files  25 passed (25)
Tests       141 passed (141)
vite v8.0.16 building client environment for production...
318 modules transformed.
built in 506ms
cargo test --workspace: passed
git diff --check: passed
```

### Deviations from the specification

- No implementation-scope deviation.
- Remote Linux installer packaging, workflow parsing by GitHub Actions, release
  creation, and asset upload remain unverified until this dedicated branch is
  intentionally pushed/merged through the release workflow.
- The operator-owned untracked `.pi/` and `ISSUE-4-PLAN.md` remain untouched and excluded.

### Handoff status
- [x] Ready for Project Lead review
