---
id: SPEC-033
# Note: Quote the title if it contains a colon
title: "Add explicit current-view data refresh"
status: review
kind: feature
priority: high
area: workspace-ux
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-10
updated: 2026-07-10
tags: [refresh, stale-data, diagnostics, ux]
activity:
  - date: 2026-07-10
    action: "created"
activity:
  - date: 2026-07-10
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-10
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-10
    action: "transitioned working -> review"
---
# Add explicit current-view data refresh

## Objective

Give operators an explicit header action that discards stale view state and reloads the currently displayed workspace page from fresh backend data.

## Context

The file watcher refreshes shared workspace collections, but several views fetch their own data only when mounted. A watcher event can therefore update Pulse while leaving Insights, Roadmap, Design, Wiki, or list-local state stale. Operators currently cannot distinguish a real unresolved warning from cached presentation state.

## Scope
### Included

- Header refresh button with accessible loading, success, and failure feedback.
- Strict refresh of shared workspace data, with failures propagated to the UI.
- Remount only the current non-session view so view-local queries execute again.
- Refresh session metadata without remounting terminal instances or losing scrollback.
- Disable duplicate refresh requests while one is running.
- Focused tests and product/session documentation.

### Excluded

- Browser/webview reload or returning to the repository picker.
- Restarting the watcher, app, PTYs, or agent processes.
- Background polling or changes to file-watcher debounce behavior.

## Existing-project analysis

- `WorkspaceContext.loadAllData()` refreshes common artifacts and diagnostics but swallows command failures.
- Insights, Agents, Reviews, Decisions, Design, Roadmap, Wiki, and Board execute view-local loading effects on mount.
- Sessions must remain mounted to preserve xterm state; a generic application remount would regress SPEC-030.

## Technical proposal

Expose a strict `refreshWorkspaceData()` context operation backed by the same shared-data fetch as normal loading. `TopBar` owns the refresh action and feedback, invokes the strict shared refresh plus session metadata refresh when relevant, then asks `AppShell` to increment a current-view key. `AppShell` applies that key only to the non-session view container; the persistent Sessions layer remains untouched.

## Files and areas involved

- `src/context/WorkspaceContext.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/components/Layout/TopBar.tsx`
- focused frontend tests
- `docs/product.md`

## Acceptance criteria
- [x] Header exposes an accessible Refresh current view button.
- [x] Clicking refresh retrieves shared workspace artifacts and diagnostics from the backend before reloading view-local state.
- [x] The current non-session view remounts so its own loading effects rerun.
- [x] Sessions refresh metadata without remounting terminal components or losing scrollback.
- [x] Repeated clicks are disabled while refreshing.
- [x] Success and failure are visible and announced; failure does not present stale data as refreshed.
- [x] Refresh does not restart the app, watcher, PTYs, or repository preparation.
- [x] Focused tests cover success, failure, duplicate-click protection, and current-view remount signaling.
- [x] Full frontend tests, lint, build, Rust checks, version alignment, and diff checks pass for `2.6.1`.

## Implementation plan
1. Refactor shared workspace fetching into safe automatic and strict manual paths.
2. Add TopBar refresh state/action and AppShell current-view remount key.
3. Preserve Sessions mounting while refreshing its metadata.
4. Add tests/docs and rerun patch-release gates.

## Required verification

- Focused header/context tests.
- `pnpm test`, `pnpm lint`, `pnpm build`.
- `cargo check --workspace --tests`.
- `node scripts/check-version.mjs` and `git diff --check`.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Refreshing a Wiki tree must not reload the whole desktop application; view remount is scoped to the active content layer.
- A refresh can race with file-watcher refresh. Both use complete snapshots, so the last completed fetch wins; duplicate manual clicks are suppressed.
- No migration or dependency is required.

## Escalated implementation authority

The operator explicitly requested this bounded addition for the pending `2.6.1` patch on 2026-07-10. It remains within the active corrective implementation authority and does not change persistence, security, or external integrations.

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

- Added a header refresh control with accessible label, disabled/loading state, and announced success/error feedback.
- Refactored shared workspace fetching into safe automatic refresh and strict manual refresh paths.
- Manual refresh reloads artifacts, diagnostics, Git metadata, the selected spec, and the open Wiki page before remounting the current non-session view.
- Sessions refresh only their metadata and preserve the persistent xterm tree, scrollback, and PTYs.
- Added focused success, failure, duplicate-click, and Sessions preservation tests.
- Updated product, release, and no-rewrite migration documentation for `2.6.1`.

### Files changed

- `src/context/WorkspaceContext.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/components/Layout/TopBar.tsx`
- `src/__tests__/TopBar.test.tsx`
- `docs/product.md`
- `kit/.lmbrain/CHANGELOG.md`
- `kit/.lmbrain/MIGRATIONS.md`

### Verification performed

- `pnpm test` - passed, 20 files / 114 tests.
- `pnpm lint` - passed.
- `pnpm build` - passed; existing large-chunk warning remains (main bundle approximately 813 kB).
- `cargo check --workspace --tests` - passed.
- `node scripts/check-version.mjs` - passed at `2.6.1`.
- `git diff --check` - passed.

### Deviations from the specification

- None. No app, watcher, session, or PTY restart is part of refresh.

### Handoff status
- [x] Ready for Project Lead review
