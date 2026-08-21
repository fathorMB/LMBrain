---
id: REVIEW-022
title: "Accepted SPEC-025 v3 session tabs final remediation"
status: accepted
spec_id: SPEC-025
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, sessions, ui, remediation, accepted]
links: [SPEC-025, REVIEW-019, REVIEW-020, REVIEW-021]
---
# Accepted SPEC-025 v3 session tabs final remediation

## Verdict

Accepted.

The Project Lead performed the operator-authorized narrow mitigation from REVIEW-021. `UPDATE_SESSION` handling is restored in the production session reducer path, and targeted production reducer coverage now verifies that session status/exit-code updates preserve tab order and active selection.

## Changes Verified

- `src/context/WorkspaceContext.tsx`
  - `SessionAction` includes `UPDATE_SESSION`.
  - `sessionReducer` patches the matching session while preserving the rest of session tab state.
  - The main workspace reducer routes `UPDATE_SESSION` through the production session reducer, so `session-exit` events update UI state again.

- `src/__tests__/sessionReducer.test.ts`
  - Adds production reducer coverage for `UPDATE_SESSION`.
  - Verifies session order is unchanged.
  - Verifies `activeSessionId` is preserved.
  - Verifies the matching session receives `status: "exited"` and `exit_code`.

## Acceptance Criteria Assessment

- Sessions render as tabs, not floating windows: pass.
- Creating a session adds a new tab and makes it active: pass.
- Switching tabs changes the active terminal without killing/restarting other sessions: pass by implementation and tests.
- Closing the active tab kills/removes that session and selects a predictable neighboring tab: pass.
- Exited sessions remain visible until closed and show exit status: pass after restored `UPDATE_SESSION` handling.
- Terminal resize is correct after tab switch, session creation, and app view activation: pass by implementation evidence; no backend change introduced in final remediation.
- The new-session modal remains usable and visually above the tab workspace: pass.
- Obsolete geometry/z-index state and tests are removed or migrated: pass.
- Automated tests cover tab creation, switching, close behavior, exited status, and modal visibility: pass, including production reducer state coverage.
- Existing `pnpm lint`, `pnpm test`, and relevant Rust tests pass: pass for frontend gates; Rust was previously verified during review and not affected by this final TypeScript-only mitigation.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 75 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.

## Notes

`cargo test` was not re-run for the final mitigation because it touched only TypeScript frontend state/tests. The prior SPEC-025 review pass verified `cargo test` successfully before this frontend-only correction.
