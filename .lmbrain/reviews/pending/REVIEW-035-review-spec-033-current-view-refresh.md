---
id: REVIEW-035
# Note: Quote the title if it contains a colon
title: "Review SPEC-033 current-view refresh"
status: pending
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-033
reviewer: Project Lead escalation verification
review_requested_by: user
implementation_agent: Project Lead escalation
related_tasks: []
links: []
created: 2026-07-10
updated: 2026-07-10
tags: [refresh, stale-data, verification]
activity:
  - date: 2026-07-10
    action: "created"
---
# Review SPEC-033 current-view refresh

## Outcome

Pass. The header action provides a strict, visible refresh path for shared and view-local data while preserving persistent terminal state.

## Acceptance-criteria compliance

- Refresh control is accessible, disables duplicate clicks, and announces success/failure.
- Shared artifacts, diagnostics, Git state, selected spec, and open Wiki page are refreshed before view remount.
- Non-session views remount through a scoped key, rerunning their own query effects.
- Sessions refresh metadata without remounting `SessionsView` or any `SessionTerminal`.
- No watcher, app, PTY, workspace-preparation, or agent lifecycle command is invoked.

## Code observations

- Automatic watcher refresh retains its tolerant error handling; manual refresh uses the same fetch snapshot but propagates failures so stale data is never labelled updated.
- The refresh key applies only to the ordinary content container. The separately mounted Sessions layer remains stable.
- Selected spec and Wiki page references are reconciled to fresh objects rather than preserving stale context selections.

## Tests and verification

- `pnpm test` - passed, 20 files / 114 tests.
- `pnpm lint` - passed.
- `pnpm build` - passed; existing bundle-size warning only.
- `cargo check --workspace --tests` - passed.
- `node scripts/check-version.mjs` - passed at `2.6.1`.
- `git diff --check` - passed.

## Production quality and documentation compliance

Compliant with [[QUALITY]]. The change is dependency-free, bounded, failure-aware, tested, and documented. No migration or policy exception is required.

## Findings

None.

## Required follow-up

- Optional operator runtime confirmation that resolving a diagnostic and pressing Refresh removes it while open sessions retain their scrollback.

## Final decision

Recommend acceptance of [[SPEC-033-add-explicit-current-view-data-refresh]].
