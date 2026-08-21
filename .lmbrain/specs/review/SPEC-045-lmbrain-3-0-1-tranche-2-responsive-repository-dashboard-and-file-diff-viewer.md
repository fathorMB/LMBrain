---
id: SPEC-045
# Note: Quote the title if it contains a colon
title: "LMBrain 3.0.1 tranche 2: responsive repository dashboard and file diff viewer"
status: review
kind: bugfix
priority: high
area: repository-dashboard
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-18
updated: 2026-07-18
tags: [3.0.1, repository, git, diff, ui]
activity:
  - date: 2026-07-18
    action: "created"
activity:
  - date: 2026-07-18
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-18
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-18
    action: "transitioned working -> review"
---
# LMBrain 3.0.1 tranche 2: responsive repository dashboard and file diff viewer

## Objective

Make the repository dashboard usable with dense real-world worktrees: long file
paths must never widen the application or require page-level horizontal scroll,
the dashboard should use the available viewport effectively, and selecting a
changed file should open an accessible source-control-style diff modal.

## Context

Operator runtime evidence on 2026-07-18 shows long `.lmbrain/handoffs/...` paths
expanding the changed-file grid beyond the viewport. The dashboard's 1080px
content cap also wastes substantial horizontal space on wide displays. The
operator explicitly directed this mandatory second 3.0.1 correction and asked
for an in-app diff experience comparable to agentic coding/source-control apps.

This is an operator-directed corrective takeover under `AGENT.md`, continuing
on `codex/v3.0.1-ci-hotfix` without altering the 3.0.1 version established by
[[SPEC-044]].

## Scope
### Included

- Responsive dashboard sizing that uses wide screens and collapses cleanly on narrower windows.
- Strict overflow containment and ellipsis for long branch, remote, file, rename, PR, and workflow text.
- Click/keyboard activation for each changed-file row.
- A Tauri command returning a bounded, no-color, no-external-diff unified diff for staged, unstaged, conflicted, renamed, deleted, and untracked files.
- Path validation and repository confinement, including untracked-file canonicalization.
- An accessible modal with loading, error, empty/binary/truncated states, Escape/backdrop/close behavior, colored unified-diff lines, and line numbers.
- Focused Rust and frontend regression tests plus the complete 3.0.1 quality gates.
- Changelog/session or repository documentation updates required by the user-visible addition.

### Excluded

- Editing, staging, reverting, committing, or otherwise mutating repository files.
- Syntax-aware or side-by-side diff rendering, intra-line word diffs, or Git history browsing.
- Installing a diff dependency, invoking external diff tools, or opening an editor.
- Committing, pushing, merging, publishing, or touching operator-owned untracked files.

## Existing-project analysis

- `RepositoryView.tsx` uses a two-column CSS grid whose tracks default to an
  intrinsic minimum; nested long monospace paths can therefore widen the page.
- The page caps content at 1080px despite displaying four information-dense cards.
- Changed files are non-interactive `div` rows and no diff IPC contract exists.
- `git_details.rs` already runs Git without a shell and disables Windows console
  flashes; the new read-only diff command can follow the same boundary.
- Tauri state already exposes a canonical workspace root. A `--` separator
  prevents option injection for tracked paths; untracked paths additionally
  require canonical confinement before `git diff --no-index`.

## Technical proposal

Introduce a repository-specific responsive stylesheet using `minmax(0, ...)`,
explicit `min-width: 0`, overflow containment, and a larger bounded content
width. Convert file rows to accessible buttons. Add a typed `GitFileDiff` IPC
contract and a backend helper that validates status/path, disables external
diffs and color, handles expected `--no-index` exit code 1, truncates oversized
IPC payloads at a UTF-8 boundary, and returns actionable errors. Render the
result in an isolated modal component with parsed unified-diff line numbers and
semantic visual states.

## Files and areas involved

- `src/components/Repository/RepositoryView.tsx` and repository styles
- `src/components/Repository/GitDiffModal.tsx`
- `src/lib/commands.ts`, `src/types/index.ts`
- `src-tauri/src/commands/git_details.rs`, `src-tauri/src/lib.rs`
- focused frontend/Rust tests and repository documentation/changelog

## Acceptance criteria

- [x] Long paths are structurally prevented from creating page-level horizontal scroll and remain discoverable via title/accessible name; operator runtime confirmation remains pending.
- [x] The dashboard uses materially more of a wide viewport while retaining a readable maximum and a single-column narrow layout.
- [x] Every changed-file row is mouse- and keyboard-activatable and exposes status/path clearly.
- [x] Staged, unstaged, renamed/conflicted, deleted, and untracked selections use the correct safe read-only Git diff strategy.
- [x] Absolute paths, parent traversal, control characters, unknown statuses, and untracked paths escaping the repository fail closed.
- [x] The modal represents loading, error, empty, binary, truncated, hunk, addition, deletion, and context states without injecting diff content as HTML.
- [x] The modal closes by close button, Escape, and backdrop without changing repository state.
- [x] Focused and complete frontend/Rust/release gates pass or any unrelated pre-existing gate failure is evidenced precisely.

## Implementation plan

1. Implement and test the confined backend diff contract and Tauri wiring.
2. Implement the typed frontend command, responsive dashboard, and accessible file rows.
3. Add the isolated diff modal and parser/rendering tests.
4. Update 3.0.1 release/repository documentation without another version bump.
5. Run focused and full gates, inspect the diff independently, and record evidence/review.

## Required verification

- targeted Rust tests for path/status validation and staged/unstaged/untracked diff output
- focused RepositoryView/GitDiffModal frontend tests
- `pnpm lint`, `pnpm test`, `pnpm build`
- `cargo test --workspace`
- `node scripts/check-version.mjs`
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Diff payloads may contain arbitrary repository text; render as React text only,
  never HTML, and cap the IPC payload with an explicit truncation notice.
- Binary or empty diffs are valid outcomes and must not appear as crashes.
- Untracked-file diffing can read worktree content; canonical confinement must
  reject symlink/junction escape before invoking Git.
- This tranche remains read-only and must not add stage/revert/edit actions.

## Project Lead corrective takeover

- **Authorized by:** human operator on 2026-07-18 through the mandatory 3.0.1 request.
- **Rationale:** runtime evidence shows an unacceptable layout break and a missing read-only inspection path in the newly shipped repository page.
- **Branch:** `codex/v3.0.1-ci-hotfix`, continuing the coordinated patch release.
- **Scope boundary:** repository dashboard layout, bounded read-only diff retrieval/viewing, tests, and required documentation only.
- **Verification plan:** isolated Rust/frontend coverage, full frontend/Rust gates, version/diff checks, and a separate review before handoff.

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

- Reworked the Repository page around responsive `minmax(0, ...)` grid tracks,
  explicit min-width/overflow containment, a 1560px readable maximum, and a
  single-column breakpoint. Long values use ellipsis plus full-value titles.
- Converted changed-file rows to full-width accessible buttons and added a
  source-control-style unified-diff modal with old/new line numbers, semantic
  addition/deletion/hunk colors, loading/error/empty/binary/truncated states,
  Escape/backdrop/button close behavior, and focus restoration.
- Added typed frontend diff parsing and IPC contracts without HTML injection or
  a third-party rendering dependency.
- Added a read-only Tauri Git diff command. It distinguishes index, worktree,
  conflict, and untracked targets; disables external diff and text-conversion
  drivers plus color; validates
  repository-relative paths; canonically confines untracked files; accepts the
  expected no-index difference exit; and truncates payloads at 512 KiB on a
  valid UTF-8 boundary. Rendering is independently capped at 5,000 lines so
  pathological short-line patches cannot create an unbounded React tree.
- Corrected Git status parsing so leading porcelain status spaces are preserved;
  single unstaged changes are no longer misclassified. Deleted/renamed entries
  now retain an explicit diff target.
- Updated the 3.0.1 changelog and added repository-dashboard documentation.

### Files changed

- `src-tauri/src/commands/git_details.rs`, `src-tauri/src/lib.rs`
- `src/types/index.ts`, `src/lib/commands.ts`, `src/lib/gitDiff.ts`
- `src/components/Repository/RepositoryView.tsx`
- `src/components/Repository/RepositoryView.css`
- `src/components/Repository/GitDiffModal.tsx`
- `src/__tests__/RepositoryView.test.tsx`
- `src/__tests__/GitDiffModal.test.tsx`, `src/__tests__/gitDiff.test.ts`
- `README.md`, `docs/README.md`, `docs/repository.md`
- `kit/.lmbrain/CHANGELOG.md`

### Verification performed

- `cargo test -p lmbrain commands::git_details::tests -- --nocapture` — passed, 6 tests on Windows (plus Unix-only symlink-escape coverage on Unix).
- Focused RepositoryView/GitDiffModal/parser Vitest run — passed, 3 files / 14 tests.
- `rustfmt --edition 2021 --check src-tauri/src/commands/git_details.rs` — passed.
- `cargo clippy -p lmbrain --all-targets --no-deps -- -D warnings -A clippy::question_mark` — passed; the allowance is limited to a pre-existing finding in unrelated `harness_planner.rs`.
- Unmodified full Clippy command remains blocked only by that pre-existing `harness_planner.rs:233` `question_mark` finding; all findings in the touched Git module were resolved.
- `cargo test --workspace` — passed; application crate 70 passed / 3 intentionally ignored, with all core/MCP/integration/doc tests green.
- `pnpm lint` — passed.
- `pnpm test` — passed, 27 files / 147 tests.
- `pnpm build` — passed; TypeScript and Vite production build completed (existing bundle-size advisory only).
- `node scripts/check-version.mjs` — passed at 3.0.1.
- `git diff --check` — passed.

### Verification transcript

```text
LMBrain app and kit are aligned at v3.0.1.
Git diff targeted tests: 6 passed; 0 failed.
Focused frontend: 3 files passed; 14 tests passed.
Rust application: 70 passed; 0 failed; 3 intentionally ignored.
Complete Rust workspace: passed.
Frontend: 27 files passed; 147 tests passed.
ESLint: passed.
TypeScript/Vite production build: 321 modules transformed; built successfully.
git diff --check: passed.
```

### Deviations from the specification

- No source-scope deviation.
- A live packaged-window visual smoke against the operator's dense AstraNexus
  worktree was not run from this development session. The overflow correction
  is structurally covered by the responsive CSS contract and component tests,
  but operator runtime confirmation remains required before release acceptance.
- Full warning-denied application Clippy is blocked by one pre-existing finding
  in untouched `src-tauri/src/commands/harness_planner.rs`; a run allowing only
  that lint passes.
- The existing Vite bundle-size advisory remains unchanged and non-blocking.
- Operator-owned `.pi/` and `ISSUE-4-PLAN.md` remain untouched and excluded.

### Handoff status
- [x] Ready for Project Lead review
