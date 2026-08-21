---
id: REVIEW-021
title: "Second re-review SPEC-025 v3 session tabs remediation"
status: changes-requested
spec_id: SPEC-025
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, sessions, ui, remediation]
links: [SPEC-025, REVIEW-020]
---
# Second re-review SPEC-025 v3 session tabs remediation

## Verdict

Changes requested.

The REVIEW-020 finding was addressed correctly: `src/__tests__/sessionReducer.test.ts` now imports and exercises production session reducer logic from `WorkspaceContext`. However, the extraction introduced a regression in the main workspace reducer.

## Findings

### [P1] `UPDATE_SESSION` is no longer handled by the workspace reducer

`Action` still defines `UPDATE_SESSION`:

- `src/context/WorkspaceContext.tsx:88`

The `session-exit` listener still dispatches `UPDATE_SESSION` to mark sessions as exited and set `exit_code`:

- `src/context/WorkspaceContext.tsx:320`

But the main `reducer` no longer has a `case "UPDATE_SESSION"` after extracting `sessionReducer`; unhandled actions fall through to `default` and return unchanged state:

- `src/context/WorkspaceContext.tsx:214`

As a result, exited sessions will not update their UI status/exit code when the backend emits `session-exit`, breaking the acceptance criterion that exited sessions remain visible and show exit status.

Required remediation:

- Restore production handling for `UPDATE_SESSION`, either in the main reducer or by including it in the extracted session reducer/action type.
- Add a production reducer test proving `UPDATE_SESSION` patches the matching session without replacing session tab order or active selection.
- Re-run frontend gates.

## Resolved Items

- REVIEW-020 duplicated reducer test: resolved. Tests now import `sessionReducer` from production code.
- REVIEW-019 stale `activeSessionId`: still resolved.
- REVIEW-019 `react-rnd` cleanup: still resolved.
- REVIEW-019 close-button assertion reliability: still resolved.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 74 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- `cargo test` not re-run in this pass because the remediation inspected here only touched frontend TypeScript/tests.

## Required Remediation

1. Reintroduce real `UPDATE_SESSION` handling.
2. Add/extend production reducer coverage for `UPDATE_SESSION`.
3. Re-run `pnpm lint`, `pnpm test`, and `pnpm build`.

## Project Lead Escalation

The operator explicitly authorized the Project Lead to implement this narrow mitigation directly on 2026-07-02. Scope is limited to restoring `UPDATE_SESSION` session-state handling and adding targeted production reducer coverage. Verification plan: run `pnpm lint`, `pnpm test`, and `pnpm build` after the code change.
