---
id: REVIEW-019
title: "Review SPEC-025 v3 session tabs"
status: changes-requested
spec_id: SPEC-025
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, sessions, ui]
links: [SPEC-025]
---
# Review SPEC-025 v3 session tabs

## Verdict

Changes requested.

The implementation moves the Sessions UI away from floating windows and the main quality gates pass, but two acceptance criteria are not fully satisfied yet: obsolete dependency cleanup and robust tab-state behavior/coverage.

## Findings

### [P1] Active tab can become stale after session refresh

`WorkspaceContext` keeps the previous `activeSessionId` during `SET_SESSIONS` whenever it is non-null:

- `src/context/WorkspaceContext.tsx:169`

If the backend session list no longer contains that ID, the UI still renders tabs but `SessionsView` cannot find an `activeSession`, so no terminal pane is shown. This can happen after a backend-driven session list refresh, workspace reopen reconciliation, or any stale frontend state. The tab state should normalize `activeSessionId` to an existing session ID, or `null` when no sessions remain.

Required remediation:

- In `SET_SESSIONS`, preserve the active ID only if it exists in `action.sessions`.
- Otherwise select a predictable fallback, normally the first session in the returned order.
- Add reducer or provider-level coverage for this refresh case.

### [P2] `react-rnd` was removed from usage but not from dependencies

The code no longer imports `react-rnd`, but it remains in:

- `package.json:23`
- `pnpm-lock.yaml`

SPEC-025 explicitly requires obsolete geometry/z-index state and dependency usage to be removed or migrated. Keeping an unused windowing dependency is unnecessary production debt and contradicts the cleanup part of the handoff.

Required remediation:

- Remove `react-rnd` from `package.json`.
- Regenerate/update `pnpm-lock.yaml`.
- Re-run frontend gates.

### [P2] Automated tests do not cover creation and active-close state semantics

`src/__tests__/SessionsView.test.tsx` verifies tab rendering and callback invocation, but it does not validate the state transitions required by the spec:

- creating a session adds a tab and makes it active;
- closing the active tab selects the predictable neighboring tab;
- refresh preserves or repairs active-tab selection.

The current close test also silently passes if the close button is not found because the assertion is guarded by `if (closeButton)`.

Required remediation:

- Add focused reducer/provider tests or an integration-style test around `WorkspaceContext` actions for `ADD_SESSION`, `REMOVE_SESSION`, and `SET_SESSIONS`.
- Make the close-button test fail when the button cannot be found.

## Acceptance Criteria Assessment

- Sessions render as tabs, not floating windows: pass.
- Creating a session adds a new tab and makes it active: partially evidenced in reducer code, not covered by tests.
- Switching tabs changes active terminal without killing/restarting other sessions: partially pass; callback and active-only terminal behavior are present, but refresh stale-state handling needs correction.
- Closing active tab kills/removes session and selects predictable neighboring tab: partially implemented, insufficiently covered.
- Exited sessions remain visible until closed and show exit status: pass.
- Terminal resize after tab switch/session creation/view activation: partially pass by code inspection; no targeted test, but acceptable if smoke evidence is real.
- New-session modal remains usable and above tab workspace: pass.
- Obsolete geometry/z-index state and tests removed or migrated: partial; dependency remains.
- Automated tests cover tab creation, switching, close behavior, exited status, and modal visibility: partial; creation and active-close semantics missing.
- `pnpm lint`, `pnpm test`, and relevant Rust tests pass: pass in reviewer verification.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test -- src/__tests__/SessionsView.test.tsx --runInBand` - command completed with 63 tests passing.
- `pnpm build` - pass; Vite reports the existing large chunk warning.
- `cargo test` - pass.
- Static search for obsolete window APIs: no source imports/usages of `react-rnd`, `SessionWindowState`, `SessionWindowGeometry`, `updateSessionGeometry`, or `bringSessionToFront`; package files still retain `react-rnd`.

## Required Remediation

1. Normalize `activeSessionId` on `SET_SESSIONS` when the current active session is absent from the refreshed list.
2. Remove the unused `react-rnd` dependency and lockfile entries.
3. Strengthen tab-state tests for creation, active close neighbor selection, stale active refresh, and close-button assertion reliability.
4. Re-run `pnpm lint`, `pnpm test`, `pnpm build`, and `cargo test` if lockfile or backend-adjacent behavior changes.
