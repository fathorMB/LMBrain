---
id: REVIEW-020
title: "Re-review SPEC-025 v3 session tabs remediation"
status: changes-requested
spec_id: SPEC-025
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, sessions, ui, remediation]
links: [SPEC-025, REVIEW-019]
---
# Re-review SPEC-025 v3 session tabs remediation

## Verdict

Changes requested.

Most remediation is complete and the full quality gates pass, but one review finding remains unresolved: the new state-transition tests duplicate the reducer logic instead of testing the production reducer/provider. This leaves the critical tab-state behavior unprotected.

## Findings

### [P1] Tab-state tests duplicate production logic instead of exercising it

`src/__tests__/sessionReducer.test.ts` defines a local `sessionReducer` copy "for focused testing without importing the full WorkspaceContext module." The copied reducer includes the same intended logic for `SET_SESSIONS`, `ADD_SESSION`, and `REMOVE_SESSION`, but it is not the production reducer in `src/context/WorkspaceContext.tsx`.

This does not satisfy the intent of REVIEW-019's required remediation for reducer or provider-level coverage. If the real `WorkspaceContext` reducer regresses, these tests can still pass because they test the copy, not the implementation shipped in the app.

Required remediation:

- Exercise the real production logic. Acceptable approaches:
  - export the reducer and initial state in a test-friendly way from `WorkspaceContext.tsx`, while keeping runtime API unchanged; or
  - test through `WorkspaceProvider` with mocked commands and a small consumer component that dispatches real actions; or
  - extract the session-state reducer/helper into a small production module and import that module from both `WorkspaceContext` and tests.
- Remove the duplicated reducer from the test.
- Keep coverage for:
  - `ADD_SESSION` activates the created session;
  - `REMOVE_SESSION` selects the predictable neighbor;
  - `SET_SESSIONS` preserves a valid active session;
  - `SET_SESSIONS` repairs stale active session IDs;
  - empty refreshed lists set active session to `null`.

## Remediation Status

- REVIEW-019 P1 stale `activeSessionId`: implemented in production code by normalizing `activeSessionId` during `SET_SESSIONS`.
- REVIEW-019 P2 `react-rnd` dependency cleanup: resolved; removed from `package.json` and `pnpm-lock.yaml`.
- REVIEW-019 P2 close-button assertion reliability: resolved in `SessionsView.test.tsx`.
- REVIEW-019 P2 tab-state test coverage: not yet resolved because tests cover a copied reducer, not production code.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 74 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- `cargo test` - pass.
- Static search: no `react-rnd` references remain in package files or source code. The only remaining `SessionWindowGeometry` / `SessionWindowState` match is a historical comment in `src/types/index.ts`.

## Required Remediation

1. Replace the copied reducer test with tests that execute the actual production state logic.
2. Re-run `pnpm lint`, `pnpm test`, and `pnpm build`.
3. No further backend/Rust verification is required unless the remediation touches Rust or session command behavior.
