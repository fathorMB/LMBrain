---
id: SPEC-068
title: "Confirmation dialog on app window close when active sessions are present"
status: ready
kind: feature
priority: low
area: desktop-app
milestone: M-08
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/42]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-42, desktop-app, window-close, KIT-NOTE-016]
activity:
  - date: 2026-07-31
    action: "created"
  - date: 2026-07-31
    action: "transitioned backlog -> ready"
---
# Confirmation dialog on app window close when active sessions are present

## Objective
Prevent accidental application exit by intercepting the window close event and showing a confirmation dialog only when active agent sessions exist.

## Context
Reported in `KIT-NOTE-016` (v3.1.2): Closing the desktop app window immediately exits the process without confirmation, even if open agent sessions exist, potentially causing lost session context or untracked state.

## Corrective diagnosis (2026-07-31)

The 3.1.3 implementation registered a JavaScript `onCloseRequested` listener and assumed that doing nothing would preserve normal window closing. In Tauri 2, the native window manager automatically prevents a close whenever a JavaScript close-request listener exists, then delegates the decision to that listener. The handler showed a native OS confirmation only when sessions existed but never explicitly destroyed the window when no sessions existed. As a result, the window close button is inert for the normal zero-session case.

The native plugin dialog also bypasses the application's visual system, focus conventions, and reusable modal behavior.

## Project Lead corrective takeover

- **Authorized by:** human operator on 2026-07-31 after runtime confirmation against the local 3.1.4 branch.
- **Scope:** restore immediate close with zero sessions; replace the native confirmation with an accessible LMBrain-styled in-app modal for active sessions; clean up listener lifecycle under React StrictMode; add regression tests.
- **Technical direction:** keep one stable close listener, explicitly destroy the window for zero sessions, synchronously surface application state for active sessions, and use `destroy()` only after explicit modal confirmation.
- **Verification:** focused close-routing/modal tests, full frontend suite, production build, lint, Rust regression suite, and local runtime smoke.

## Scope
### Included
- Intercept Tauri window close event (`tauri::window::CloseRequested` or frontend window beforeunload/close handler).
- Check if any agent session is currently in an open state (open tabs/active session handles).
- If open sessions exist, display a native or app confirmation modal ("Active sessions are open. Are you sure you want to close LMBrain?").
- If no active sessions exist, close immediately without prompting.

### Excluded
- Blocking un-dismissable application termination.

## Acceptance criteria
- [x] Closing the desktop app window when active sessions are open prompts for confirmation.
- [x] Closing the desktop app window when zero sessions are open closes immediately without a prompt.

## Required verification
- `pnpm test`
- `pnpm lint`
- `pnpm build`

## Implementation evidence

- `src/App.tsx` synchronously prevents Tauri's automatic post-handler destruction, installs one stable close-request listener, resolves open sessions from the authoritative backend registry at close time, falls back safely to the latest React state, and serializes repeated close requests.
- `src/lib/windowClose.ts` owns the deterministic close decision: zero open session tabs destroy the window immediately; one or more open session tabs show the application confirmation.
- `src-tauri/capabilities/default.json` grants the narrow `core:window:allow-destroy` permission required by that explicit Tauri close path.
- `src/components/Layout/WindowCloseConfirmModal.tsx` replaces the native OS prompt with an accessible LMBrain-styled modal, stops the watcher and running sessions before closing, and exposes an explicit force-close recovery only after cleanup failure.
- `src-tauri/src/commands/sessions.rs` makes session termination idempotent, retains genuinely failed sessions for retry, and treats Windows ConPTY `ERROR_SUCCESS`/closed-handle results as already stopped instead of surfacing false cleanup failures.
- `src/context/WorkspaceContext.tsx` and `src/components/Layout/AppShell.tsx` keep confirmation visibility in application state instead of coupling it to an asynchronous native dialog.
- Regression tests cover the real `onCloseRequested` callback contract, synchronous native-close prevention before an unresolved backend query, both routing branches, stale-state/backend resolution, backend failure fallback, running-process cleanup filtering, safe default focus, cancellation, cleanup ordering, and final window destruction.

## Verification evidence (2026-07-31)

- `pnpm test` — 37 files and 192 tests passed.
- `pnpm lint` — passed.
- `pnpm build` — passed; only the pre-existing bundle-size advisory remains.
- `cargo test --workspace` — 317 passed, 0 failed, 3 ignored manual/long-running harness tests.
- `node scripts/check-version.mjs` — application and bundled kit aligned at 3.1.4.
- `git diff --check` — passed.
- Clean rebuilt local Tauri smoke with zero active sessions — native close request terminated `lmbrain.exe` successfully after capability validation.
- Active-session confirmation — automated component and routing coverage passed; operator visual smoke remains appropriate in the running local build.
