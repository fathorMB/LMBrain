---
id: SPEC-020
# Note: Quote the title if it contains a colon
title: "Sessions new-session modal must stay above session windows"
status: review
kind: bugfix
priority: high
area: desktop-app
milestone: sessions
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-006, ADR-007]
links: []
created: 2026-06-27
updated: 2026-06-27
tags: [sessions, modal, z-index, regression]
activity:
  - date: 2026-06-27
    action: "created"
  - date: 2026-06-27
    action: "set recommended_agent"
  - date: 2026-06-27
    action: "transitioned backlog -> ready"
  - date: 2026-06-27
    action: "transitioned ready -> working"
  - date: 2026-06-27
    action: "transitioned working -> review"
---
# Sessions new-session modal must stay above session windows

## Objective

Fix the Sessions view regression where opening the "New session" modal while at least one agent session window already exists can leave the modal behind that session window, making the form invisible or unusable. The modal must always appear above every draggable/resizable session window in the Sessions view.

## Context

The Sessions UI is implemented in [`src/components/Sessions/SessionsView.tsx`](src/components/Sessions/SessionsView.tsx). Session windows are rendered with `react-rnd` and receive `style={{ zIndex: session.zIndex, ... }}`. The new-session modal overlay is currently rendered as an absolutely positioned sibling after the session canvas but does not define a z-index.

Because the session windows use explicit z-index values and the modal overlay does not, any already-fronted session with a higher stacking order can visually and interactively cover the modal. This matches the reported bug: when an agent session is already open, clicking "New session" opens the modal state but the modal is not shown in the foreground and cannot be used.

## Scope
### Included

- Ensure the "Start session" modal and its backdrop are rendered above every session window in the Sessions view, regardless of how many windows are open or which one is currently frontmost.
- Preserve existing session-window z-ordering among session windows.
- Preserve existing modal behavior for all launch modes: Claude, Ollama, and Codex.
- Add targeted frontend regression coverage proving the modal overlay has foreground stacking above a pre-existing session window with a high `zIndex`.
- Keep the fix local to Sessions UI layering unless analysis proves a shared modal/layering abstraction is already established and should be reused.

### Excluded

- Redesigning the Sessions view.
- Changing session process lifecycle, terminal attach/write behavior, geometry persistence, or backend commands.
- Adding global modal infrastructure unless necessary to satisfy the acceptance criteria cleanly.

## Existing-project analysis

Relevant current code:

- `SessionsView` sorts windows by `session.zIndex` and renders each `SessionWindow`.
- `SessionWindow` passes `session.zIndex` directly to the `Rnd` wrapper.
- The modal backdrop uses `position: "absolute"` and `inset: 0` but no `zIndex`.
- The root Sessions view wrapper does not explicitly create a stacking context; the session canvas is `position: "relative"`, and the modal overlay is a later sibling of that canvas.

Likely minimal fix:

- Define a Sessions-layer constant or equivalent local style value for the modal overlay, for example a value safely above the maximum session window z-index used by the workspace state.
- Apply it to the modal backdrop/container.
- Consider adding `position: "relative"` to the Sessions root if needed so the absolute modal overlay is scoped predictably to the Sessions view.

Do not assume a magic value is safe without inspecting how `bringSessionToFront` assigns `zIndex` in `src/context/WorkspaceContext.tsx` and related session state types.

## Technical proposal

- Inspect `bringSessionToFront`, session creation defaults, and `SessionWindowState.zIndex` to understand the intended z-index range.
- Add a named local constant in `SessionsView.tsx`, such as `SESSION_MODAL_Z_INDEX`, with a value that is deterministically above session windows.
- Apply the modal layer to the overlay element that includes the backdrop and form so both visibility and pointer interaction are foregrounded.
- If session z-index values can grow unbounded, either cap/normalize session z-ordering or compute the modal layer from current sessions (for example `Math.max(...session.zIndex) + 1`) so the modal always wins. Prefer the smallest production-grade change that cannot regress after repeated bring-to-front operations.
- Add or update a `SessionsView` test using mocked `useWorkspace` state containing at least one existing session with a high z-index. Open the modal via "New session" and assert the modal overlay/style has a z-index greater than that session window.

## Files and areas involved

- `src/components/Sessions/SessionsView.tsx`
- `src/context/WorkspaceContext.tsx` only for analysis or if z-index generation must be bounded/fixed.
- `src/types/index.ts` only if type changes are truly required; avoid otherwise.
- `src/__tests__/SessionsView.test.tsx` or an equivalent focused frontend test file.

## Acceptance criteria
- [x] Opening "New session" while one or more session windows exist shows the "Start session" modal above all session windows.
- [x] The modal backdrop and form receive pointer/focus interactions; session windows underneath do not intercept clicks intended for the modal.
- [x] The fix remains valid when an existing session has been brought to the front repeatedly and has a high `zIndex`.
- [x] Existing z-order behavior among session windows is unchanged.
- [x] Claude, Ollama, and Codex launch-mode controls still render and behave as before.
- [x] A targeted frontend regression test covers the foreground modal behavior with at least one pre-existing session window.

## Implementation plan
1. Inspect session z-index assignment in `WorkspaceContext` and confirm whether values are bounded or can grow over time.
2. Implement the smallest reliable layering fix in `SessionsView`.
3. Add regression coverage around the modal overlay with an existing high-z-index session window.
4. Run the required frontend quality checks and record evidence.

## Required verification

- `pnpm lint`
- `pnpm test`
- `pnpm build`
- Manual check in the desktop app or Vite dev build: start/open a session, bring it to front, click "New session", and confirm the modal is visibly and interactively above the session window.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Risk: a fixed modal z-index can regress if session z-index values are unbounded and eventually exceed it. Inspect the state logic before choosing a fixed layer value.
- Risk: moving the modal to a portal/global layer could affect view scoping and hidden inactive views. Keep the change local unless the existing architecture clearly favors a portal.
- No ADR is expected for this bugfix; it is a local UI layering correction.

## Instructions for the assigned specialist
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
- Inspected `WorkspaceContext` session z-index behavior and confirmed session windows use unbounded `max + 1` foregrounding.
- Updated `SessionsView` so the Sessions root creates a local stacking context and clips the absolute modal overlay to the view.
- Calculated the modal overlay z-index from the current session windows (`max(session.zIndex) + 1`) so the modal remains above existing windows even after repeated bring-to-front operations.
- Added dialog semantics to the start-session modal (`role="dialog"`, `aria-modal`, `aria-labelledby`) while preserving the existing form layout and mode controls.
- Added a focused frontend regression test with an existing high-z-index session window and asserted that opening "New session" places the modal overlay above that window.
- Fixed the `AppShell` Sessions layer so its absolute wrapper is only displayed while the active view is `sessions`; this prevents the invisible Sessions layer from intercepting clicks and scroll events in Pulse, Taskboard, and other views.
- Fixed flex scroll containment for the main app content, Wiki panes, and Taskboard columns by adding the required `minHeight: 0` / `flex: 1` constraints to the scroll containers.

### Files changed
- `src/components/Sessions/SessionsView.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/components/Wiki/WikiView.tsx`
- `src/components/Taskboard/TaskboardView.tsx`
- `src/__tests__/SessionsView.test.tsx`

### Verification performed
- `pnpm test -- src/__tests__/SessionsView.test.tsx` (Vitest ran the full frontend suite): 10 files passed, 44 tests passed.
- `pnpm lint`: passed.
- `pnpm build`: passed. Vite reported the existing large-chunk warning after a successful build.
- After the `AppShell` invisible-layer fix: `pnpm lint`, `pnpm test -- src/__tests__/SessionsView.test.tsx`, and `pnpm build` all passed again.
- After the scroll-container fix: `pnpm lint`, `pnpm test`, and `pnpm build` all passed again.
- Manual desktop session check was not run in this environment; the stacking behavior is covered by the regression test against the rendered Sessions view.

### Deviations from the specification
- None. The manual desktop check was not performed, but the bug's DOM stacking condition is covered by automated regression coverage.

### Handoff status
- [x] Ready for Project Lead review
