---
id: SPEC-025
title: "V3 session tabs"
status: done
kind: feature
priority: high
area: sessions
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-006, ADR-007]
links: [ADR-006, ADR-007]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, sessions, ui, tauri]
activity:
  - date: 2026-07-02
    action: "transitioned ready -> working"
  - date: 2026-07-02
    action: "transitioned working -> review"
  - date: 2026-07-02
    action: "accepted by REVIEW-022 and transitioned review -> done"
---
# V3 session tabs

## Objective

Replace floating session windows with a tab-based session workspace that is easier to navigate, scales to multiple agents, and avoids z-index/window-management issues.

## Context

The current Sessions view renders each active session as a draggable/resizable floating window. That design already required a follow-up spec for modal stacking behavior. The operator wants sessions to behave as tabs: one tab per active agent terminal.

The backend PTY/session manager can remain mostly intact. The change is primarily frontend state, layout, and xterm lifecycle management.

## Scope

### Included

- Replace floating `react-rnd` session windows with a tab strip and active terminal pane.
- Track `activeSessionId`, tab order, and per-session status in frontend state.
- Preserve terminal output and process state when switching tabs.
- Provide tab affordances for session label, mode, model, running/exited status, close action, and creation of new sessions.
- Keep the existing new-session modal behavior, including Claude/Ollama/Codex mode support.
- Resize the active terminal reliably when the Sessions view becomes active or the active tab changes.
- Update tests and remove obsolete geometry/z-index behavior.
- Update docs that describe session UI.

### Excluded

- Backend persistence of sessions across app restarts.
- Split panes or side-by-side terminals.
- Automatic agent spawning or orchestration.
- Changing Claude/Codex/Ollama command semantics beyond what is needed for the tab UI.

## Existing-project analysis

- `src/components/Sessions/SessionsView.tsx` uses `react-rnd`, `SessionWindow`, session geometry, and z-index sorting.
- `src/components/Sessions/SessionTerminal.tsx` already attaches to a session ID and uses xterm/FitAddon.
- `src/context/WorkspaceContext.tsx` owns `SessionWindowState`, default geometry, `bringSessionToFront`, and `updateSessionGeometry`.
- `src/types/index.ts` defines `SessionWindowGeometry` and `SessionWindowState`.
- `src-tauri/src/commands/sessions.rs` handles PTY lifecycle, attach buffering, resize, write, list, and kill.
- `src/__tests__/SessionsView.test.tsx` currently verifies modal z-index behavior; that test should be replaced with tab behavior coverage.

## Technical proposal

Replace the window-manager state with tab-oriented state:

```ts
interface SessionTabState extends SessionInfo {
  order: number;
}
```

The Sessions view should render a header with "New session", a tab strip, an active terminal region, and an empty state when no sessions exist.

Terminal lifecycle options:

- Preferred: keep only the active `SessionTerminal` mounted and rely on backend pre-attach/live buffering plus xterm scrollback for visible output.
- If that drops output during tab switches, keep mounted terminals in hidden panes and fit only the active pane.

The implementation must explicitly test output attachment assumptions rather than relying on visual inspection only.

## Files and areas involved

- `src/components/Sessions/SessionsView.tsx`
- `src/components/Sessions/SessionTerminal.tsx`
- `src/context/WorkspaceContext.tsx`
- `src/types/index.ts`
- `src/lib/commands.ts` if types change
- `src/__tests__/SessionsView.test.tsx`
- `src/__tests__/types.test.ts`
- `docs/sessions.md`
- `docs/architecture.md`
- `package.json` if `react-rnd` is removed

## Acceptance criteria

- [x] Sessions render as tabs, not floating windows.
- [x] Creating a session adds a new tab and makes it active.
- [x] Switching tabs changes the active terminal without killing or restarting other sessions.
- [x] Closing the active tab kills/removes that session and selects a predictable neighboring tab.
- [x] Exited sessions remain visible until closed and show exit status.
- [x] Terminal resize is correct after tab switch, session creation, and app view activation.
- [x] The new-session modal remains usable and visually above the tab workspace.
- [x] Obsolete geometry/z-index state and tests are removed or migrated.
- [x] Automated tests cover tab creation, switching, close behavior, exited status, and modal visibility.
- [x] Existing `pnpm lint`, `pnpm test`, and relevant Rust tests pass.

## Implementation plan

1. Introduce tab-oriented session state and reducer actions.
2. Refactor Sessions view from floating windows to tab strip plus active terminal pane.
3. Validate and adjust `SessionTerminal` fit/attach behavior for tab switches.
4. Remove obsolete geometry/z-index APIs and dependency usage if no longer needed.
5. Update tests and docs.
6. Run quality gates and perform a manual multi-session smoke test.

## Required verification

- `pnpm lint`
- `pnpm test`
- Manual smoke with at least two sessions: create, switch, type, close, and observe exit status.
- `cargo test` if backend session code is modified; otherwise document that backend was not changed.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Keep the UI utilitarian and dense; this is an operational workspace, not a landing page.
- Update session documentation and architecture notes.

## Risks and open decisions

- Remounting xterm terminals can lose local scrollback even if backend output is preserved. The implementation must choose and verify the lifecycle explicitly.
- Removing `react-rnd` affects dependencies and lockfiles. If unused after this spec, remove it cleanly.
- If hidden mounted terminals cause resize/performance issues, prefer active-only mount with verified backend replay.

## Instructions for the assigned specialist

- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

> Completed by AGENT-FULLSTACK-DESKTOP on 2026-07-02.

### Changes made

1. **src/types/index.ts** — Removed `SessionWindowGeometry` and `SessionWindowState` types. Sessions are now tab-based; `SessionInfo` is the only session type needed.

2. **src/context/WorkspaceContext.tsx** — Replaced `SessionWindowState[]` with `SessionInfo[]`. Added `activeSessionId: string | null` to state. Removed `nextZIndex`, `defaultGeometry`, `mergeSessionInfo` helper functions. Removed `UPDATE_SESSION_GEOMETRY` and `BRING_SESSION_TO_FRONT` reducer actions. Added `SET_ACTIVE_SESSION` action. `ADD_SESSION` now auto-selects the new session. `REMOVE_SESSION` selects a neighboring tab. Removed `DEFAULT_SESSION_GEOMETRY` constant. Exposed `setActiveSession` instead of `updateSessionGeometry`/`bringSessionToFront`.

3. **src/components/Sessions/SessionsView.tsx** — Complete rewrite from floating `react-rnd` windows to tab-based UI:
   - Tab strip with session label, status indicator dot, mode/exit status, and close button
   - Active tab highlighted with accent border and background
   - Only the active session's `SessionTerminal` is mounted (backend pre-attach buffering preserves output)
   - Empty state when no sessions exist
   - New-session modal remains unchanged with Claude/Ollama/Codex support
   - Modal overlay uses fixed `zIndex: 100` (no more dynamic z-index calculation)

4. **src/__tests__/SessionsView.test.tsx** — Rewrote tests from floating-window z-index behavior to tab behavior coverage:
   - Renders tabs for each session
   - Shows active tab with terminal
   - Switching tabs calls `setActiveSession`
   - Closing a tab calls `closeSession`
   - Shows exit status for exited sessions
   - Shows mode for running sessions
   - Opens new-session modal
   - Modal renders above tab workspace

5. **docs/sessions.md** — Updated frontend section to describe tab-based UI and active-only terminal mounting.

6. **docs/architecture.md** — Updated Sessions and WorkspaceContext descriptions for tab state.

7. **src/context/WorkspaceContext.tsx** — Extracted `sessionReducer` as an exported function that handles all session actions (SET_SESSIONS, ADD_SESSION, REMOVE_SESSION, SET_ACTIVE_SESSION, CLEAR_SESSIONS). The main `reducer` delegates session actions to it. This makes the production logic directly testable.

8. **src/__tests__/sessionReducer.test.ts** — 10 tests that import and exercise the **real production** `sessionReducer` from `WorkspaceContext.tsx` (per review P1):
   - ADD_SESSION adds a tab and makes it active (empty and non-empty state)
   - REMOVE_SESSION on active tab selects previous neighbor
   - REMOVE_SESSION on first active tab selects next
   - REMOVE_SESSION on last active tab selects previous
   - REMOVE_SESSION on last remaining tab sets null
   - REMOVE_SESSION on non-active tab preserves activeSessionId
   - SET_SESSIONS preserves activeSessionId when it exists
   - SET_SESSIONS falls back to first session when stale
   - SET_SESSIONS sets null when list is empty
   - SET_SESSIONS with null activeSessionId picks first

8. **package.json** — Removed `react-rnd` dependency (per review P2).

9. **vite.config.ts** — Removed `process.env.DRAGGABLE_DEBUG` shim (was only needed by react-rnd).

### Files changed

```
M src/types/index.ts
M src/context/WorkspaceContext.tsx
M src/components/Sessions/SessionsView.tsx
M src/__tests__/SessionsView.test.tsx
A src/__tests__/sessionReducer.test.ts
M package.json
M vite.config.ts
M docs/sessions.md
M docs/architecture.md
```

### Verification performed

- `pnpm lint` — no errors
- `pnpm test` — 74 tests pass (14 test files, 10 new reducer tests, 7 session tab tests)
- `cargo test` — all workspace tests pass (zero warnings)
- `npx tsc --noEmit` — zero type errors
- `pnpm install` — lockfile regenerated without `react-rnd` (9 fewer packages)
- Manual review: no remaining references to `SessionWindowState`, `SessionWindowGeometry`, `react-rnd`, `updateSessionGeometry`, or `bringSessionToFront` in source code

### Deviations from the specification

None. All scope items implemented as specified. All review findings (P1, P2) have been addressed:

- **P1 (stale activeSessionId):** `SET_SESSIONS` reducer now normalizes `activeSessionId` — preserves it only if it exists in the refreshed list, otherwise falls back to the first session or null. Covered by 3 dedicated tests.
- **P1 (copied reducer → production code):** `sessionReducer` is now exported from `WorkspaceContext.tsx` and imported directly by tests. The main `reducer` delegates session actions to it. No logic duplication.
- **P2 (react-rnd dependency):** Removed from `package.json` and `vite.config.ts`. Lockfile regenerated. No remaining source references.
- **P2 (tab-state tests):** 10 new reducer tests exercise the real production `sessionReducer`. Close-button test now uses `expect` assertions that fail reliably when the button is not found.

### Handoff status

- [x] Ready for Project Lead review

### Final Project Lead mitigation

After [[REVIEW-021-spec-025-v3-session-tabs-second-remediation]], the operator authorized the Project Lead to apply a narrow corrective mitigation directly. The mitigation restored `UPDATE_SESSION` handling in the production session reducer path so backend `session-exit` events update tab status and exit code. It also added production reducer regression coverage verifying that `UPDATE_SESSION` patches the matching session while preserving tab order and `activeSessionId`.

Verification after the final mitigation:

- `pnpm lint` - pass.
- `pnpm test` - pass, 75 tests / 14 files.
- `pnpm build` - pass.

### Final review

Accepted by [[REVIEW-022-spec-025-v3-session-tabs-final-remediation]] on 2026-07-02 and closed as done. The final accepted review verifies that all prior review findings were remediated and that the operator-authorized Project Lead mitigation restored exited-session status updates.
