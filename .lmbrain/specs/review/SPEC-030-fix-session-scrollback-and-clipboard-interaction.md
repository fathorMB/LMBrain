---
id: SPEC-030
# Note: Quote the title if it contains a colon
title: "Fix session scrollback and clipboard interaction"
status: review
kind: bugfix
priority: high
area: sessions
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-006]
links: [SPEC-029, REVIEW-031]
created: 2026-07-10
updated: 2026-07-10
tags: [sessions, terminal, scrollback, clipboard, xterm]
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
# Fix session scrollback and clipboard interaction

## Objective

Preserve terminal scrollback and text selection when operators switch session tabs or temporarily leave the Sessions view, and make copy/paste behavior discoverable and reliable without breaking terminal interrupt semantics.

## Context

`SessionsView` currently renders only the active `SessionTerminal`. Switching tabs unmounts and disposes the xterm instance, so returning creates a fresh terminal without the previous xterm scrollback or selection. The backend's first-attach buffer cannot reconstruct interactive terminal state after every remount.

`SessionTerminal` also has no explicit clipboard integration or visible shortcut guidance. In a terminal, bare `Ctrl+C` must remain SIGINT when there is no selection, while selected text should be copyable and paste should use xterm's paste path so bracketed-paste behavior is preserved.

After release `2.6.0`, operator runtime testing exposed a distinct Codex failure: the custom DOM wheel listener always calls `Terminal.scrollLines()` and prevents native xterm wheel handling. Codex uses the alternate buffer/TUI mouse path, where normal local scrollback is unavailable, so swallowing the event makes conversation scrolling impossible.

## Scope
### Included

- Keep every open session's xterm instance mounted while its tab exists; hide inactive terminals and resize/focus them when activated.
- Preserve scrollback, selection, and accumulated output across tab/view switches.
- Support copy through toolbar, `Ctrl+Shift+C`, macOS `Cmd+C`, and bare `Ctrl+C` only when text is selected.
- Support paste through toolbar, `Ctrl+Shift+V`, and macOS `Cmd+V`, using `Terminal.paste()`.
- Keep bare `Ctrl+C` without a selection available to the child process as interrupt.
- Provide concise visible shortcut guidance and success/error/empty-selection feedback.
- Delegate alternate-buffer wheel events to xterm so Codex and other full-screen TUIs can handle scrolling; retain explicit local scrollback for the normal buffer.
- Add focused frontend tests and update session documentation.

### Excluded

- Persisting terminal history across LMBrain restarts.
- OS-global clipboard plugins or new native dependencies.
- Changing backend PTY buffering or session persistence.
- Starting/stopping agents during this protected production window.

## Existing-project analysis

- `src/components/Sessions/SessionsView.tsx` mounts only `activeSession`, causing xterm disposal on tab change.
- `src/components/Sessions/SessionTerminal.tsx` configures wheel scrolling but has no key interception, clipboard actions, or guidance.
- Keeping terminals mounted is lower risk than reconstructing terminal state from ANSI transcripts and avoids backend protocol changes.

## Technical proposal

Render one absolute terminal pane per open `SessionInfo`, keyed by session ID. Set inactive panes to `display: none`; pass `active` only to the selected pane so its existing activation effect performs focus/fit/resize.

Add clipboard actions to `SessionTerminal`. Use `attachCustomKeyEventHandler` to intercept only recognized copy/paste gestures. Return normal key handling for bare `Ctrl+C` when there is no selection. Use `navigator.clipboard.writeText` for selected text and `navigator.clipboard.readText` followed by `term.paste()` for paste. Surface failures without swallowing terminal input silently.

Use xterm's supported `attachCustomWheelEventHandler` rather than intercepting the outer DOM node. Delegate wheel events when the alternate buffer or a zoom modifier is active; manually scroll only normal-buffer history.

Codex is a special case: its installed CLI exposes the supported `--no-alt-screen` option specifically to run inline and preserve terminal scrollback. LMBrain launches native Codex sessions with that flag so xterm owns the conversation history instead of depending on Codex consuming wheel or translated arrow events.

## Files and areas involved

- `src/components/Sessions/SessionsView.tsx`
- `src/components/Sessions/SessionTerminal.tsx`
- `src/__tests__/SessionsView.test.tsx`
- optional focused `SessionTerminal` helper tests
- `docs/sessions.md`

## Acceptance criteria

- [x] Switching between two session tabs does not unmount either `SessionTerminal`; inactive terminals remain mounted but hidden.
- [ ] Returning to a session preserves its xterm scrollback, selection, and output accumulated while inactive.
- [ ] Mouse-wheel scrolling continues to work after repeated tab/view switches, including Codex alternate-buffer sessions.
- [x] Alternate-buffer wheel events are delegated to xterm; normal-buffer wheel events retain local scrollback.
- [ ] Copy toolbar action copies the current selection and reports success, missing selection, or clipboard failure.
- [ ] `Ctrl+Shift+C`/`Cmd+C` copy selection; bare `Ctrl+C` copies only with a selection and otherwise reaches the child as SIGINT.
- [ ] Paste toolbar action and `Ctrl+Shift+V`/`Cmd+V` use xterm paste semantics and report clipboard failure.
- [x] Visible UI text documents the supported shortcuts without obscuring terminal content.
- [x] Frontend tests cover persistent terminal mounting and clipboard key-decision behavior.
- [x] `pnpm build` passes; runtime interaction tests remain deferred until an operator-approved safe window.
- [x] `docs/sessions.md` documents scrollback persistence scope and clipboard controls.

## Implementation plan

1. Keep tab terminal components mounted and hide inactive panes.
2. Add clipboard helpers, key interception, toolbar, and feedback.
3. Add focused tests without running them in the protected window.
4. Update docs and run compilation-only verification.

## Required verification

- `pnpm build` and `git diff --check` in the current protected window.
- Later safe-window manual checks: tab switching with long scrollback, selection retention, copy, paste, SIGINT, active output, and Sessions-view navigation.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Keeping one xterm instance per open tab uses more renderer memory than mounting only the active tab; this is bounded by operator-opened sessions and preserves the state xterm cannot reconstruct safely from ANSI output.
- Clipboard read permission depends on the webview/platform security context. Failures are visible and do not inject partial data.
- Runtime scroll, selection, clipboard, bracketed paste, and SIGINT behavior remains unverified until the production instance constraint is lifted.

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

- Kept every open session's `SessionTerminal` mounted and hid inactive panes, preserving xterm-owned scrollback, selection, and inactive output.
- Added testable clipboard shortcut decision logic that leaves bare `Ctrl+C` as SIGINT unless text is selected.
- Added Copy/Paste toolbar actions, explicit cross-platform shortcut guidance, success/error feedback, and xterm bracketed-paste-compatible insertion.
- Added persistent-terminal and clipboard-decision tests.
- Replaced the outer DOM wheel interceptor with xterm's buffer-aware custom wheel hook: alternate-buffer events reach Codex/TUI mouse handling and normal-buffer events retain explicit local scrolling.
- Added focused wheel-policy tests for alternate/normal buffers, modifiers, and row conversion.
- After operator testing showed buffer delegation alone was insufficient, changed native Codex launch to include its supported `--no-alt-screen` flag and added a Rust launch-spec regression test.
- Updated session documentation and changelog.

### Files changed

- `src/components/Sessions/SessionsView.tsx`
- `src/components/Sessions/SessionTerminal.tsx`
- `src-tauri/src/commands/sessions.rs`
- `src/lib/terminalClipboard.ts`
- `src/lib/terminalWheel.ts`
- `src/__tests__/SessionsView.test.tsx`
- `src/__tests__/terminalClipboard.test.ts`
- `src/__tests__/terminalWheel.test.ts`
- `docs/sessions.md`
- `.lmbrain/CHANGELOG.md`

### Verification performed

- `pnpm build` - passed; existing Vite large-chunk warning remains (main JS approximately 805 kB).
- `git diff --check` - passed.
- No test suite, app, terminal, agent, daemon, clipboard, or session lifecycle command was executed under the operator-approved protected-production constraint.
- Patch-release remediation verification: `node scripts/check-version.mjs` passed at `2.6.1`; `cargo check --workspace --tests` passed; full `pnpm test` passed (19 files / 111 tests); `pnpm lint`, `pnpm build`, and `git diff --check` passed.
- Final Codex inline-launch verification: `cargo test -p lmbrain launches_codex_inline_to_preserve_xterm_scrollback` passed; subsequent workspace check, version alignment, and diff checks passed.

### Deviations from the specification

- Manual scrollback, selection retention, clipboard permission, bracketed paste, SIGINT, and repeated tab/view switching checks are deferred. Their criteria remain unchecked and block acceptance/done.
- Codex scrolling was confirmed broken in the released `2.6.0`; the buffer-aware wheel correction is prepared as a patch-release remediation.
- The first `2.6.1` runtime attempt still could not scroll because Codex did not consume delegated or arrow-translated wheel input. The final inline launch is now running locally; a newly created Codex session must be tested because existing PTYs cannot acquire new launch arguments.

### Handoff status
- [x] Ready for Project Lead review with runtime verification deferred.
