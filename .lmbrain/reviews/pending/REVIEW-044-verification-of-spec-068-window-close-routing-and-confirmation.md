---
id: REVIEW-044
# Note: Quote the title if it contains a colon
title: "Verification of SPEC-068 window close routing and confirmation"
status: pending
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-068
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-LEAD
related_tasks: []
links: []
created: 2026-07-31
updated: 2026-07-31
tags: [desktop-app, window-close, 3.1.4]
finding_taxonomy_version: 1
activity:
  - date: 2026-07-31
    action: "created"
review_events:
  - schema_version: "1"
    id: "REVIEW-044-EVENT-001"
    timestamp: "2026-07-31T01:43:00.417856100+02:00"
    action: "submitted"
    from_status: "none"
    to_status: "pending"
    actor_role: "project-lead"
    reason: "review artifact created"
    implementation_agent: "AGENT-LEAD"
---
# Review

## Outcome

Source, automated, and zero-session runtime verification pass with no code finding. The active-session branch is implemented as an application modal and remains pending only for operator visual smoke in the currently running local build.

## Acceptance-criteria compliance

- Closing with zero running sessions explicitly destroys the Tauri window; a clean local process exited successfully after a native close request.
- Closing with any open session tab routes to the application confirmation without destroying the window.
- The guard includes every backend-registered session; destructive process cleanup remains limited to sessions whose status is `running`.

## Code observations

- The close listener synchronously calls `preventDefault()` before any asynchronous work because `@tauri-apps/api` otherwise destroys the window immediately after the handler resolves. It is stable across session updates and StrictMode lifecycle replay, queries the authoritative backend registry at close time, falls back to current React state if that query fails, serializes duplicate requests, and removes a listener that resolves after disposal.
- The routing decision is isolated from React and covered independently.
- The Tauri capability grants only `core:window:allow-destroy`, the exact permission required by the explicit terminal path; a regression test asserts that contract.
- The in-app modal follows the existing workspace-exit visual language, traps focus, defaults focus to the safe action, supports Escape/backdrop cancellation, and reports cleanup failures before offering force close.
- Active sessions and the file watcher are stopped before final destruction.
- The modal snapshots the sessions covered by the confirmation so exit events cannot rewrite its message mid-flow; explicit force close skips cleanup and has dedicated regression coverage.
- Backend session termination is idempotent and distinguishes already-terminated Windows ConPTY handles from genuine retryable failures.

## Tests and verification

- `pnpm test` — 37 files, 192 tests passed, including an integration simulation of the installed Tauri close-handler protocol and explicit force-close behavior.
- `pnpm lint` — passed.
- `pnpm build` — passed with only the pre-existing bundle-size advisory.
- `cargo test --workspace` — 317 passed, 0 failed, 3 ignored manual/long-running harness tests.
- `node scripts/check-version.mjs` — app and kit aligned at 3.1.4.
- `git diff --check` — passed.
- Clean rebuilt local Tauri zero-session close smoke — passed after capability validation.

## Production quality and documentation compliance

- No native-dialog dependency was removed because it remains required by the repository picker.
- No backend, schema, artifact, or release-version migration is introduced.
- The implementation replaces the faulty 3.1.3 behavior on the existing 3.1.4 feature branch.

## Findings

No actionable code finding.

## Required follow-up

1. In the running local build, start one agent session and click the window close button.
2. Confirm the LMBrain-styled dialog is visually correct.
3. Verify “Keep app open” preserves the session and “Close LMBrain” stops it and exits.

## Final decision

Pending operator visual smoke for the active-session dialog; automated and zero-session runtime verification recommend acceptance.
