---
id: REVIEW-032
# Note: Quote the title if it contains a colon
title: "Review SPEC-030 session scrollback and clipboard"
status: changes-requested
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-030
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-LEAD
related_tasks: []
links: [SPEC-030, SPEC-029, REVIEW-031]
created: 2026-07-10
updated: 2026-07-10
tags: [review, sessions, terminal, scrollback, clipboard]
activity:
  - date: 2026-07-10
    action: "created"
---
# Review SPEC-030 session scrollback and clipboard

## Outcome

Changes requested. Operator runtime testing on released `2.6.0` identified an alternate-buffer wheel interception bug affecting Codex sessions.

## Acceptance-criteria compliance

- Persistent mounting is implemented by rendering a keyed terminal pane for every open session and hiding inactive panes rather than unmounting them.
- Clipboard actions use xterm selection and paste APIs, preserve bare `Ctrl+C` without selection, and provide explicit shortcuts/buttons/feedback.
- Focused source tests cover persistent mounting and key-decision behavior.
- Runtime-dependent scroll, selection, clipboard permission, paste, SIGINT, and repeated-navigation criteria remain unchecked.

## Code observations

Keeping xterm mounted is preferable to replaying ANSI output: xterm owns scrollback, alternate-buffer state, selection, and paste modes that cannot be reconstructed reliably from the existing backend attach snapshot. The trade-off is bounded renderer memory per operator-opened session.

The clipboard key decision is isolated in `src/lib/terminalClipboard.ts`, allowing the critical SIGINT rule to be tested without importing xterm/Tauri runtime modules.

## Tests and verification

- `pnpm build` - pass; existing Vite large-chunk warning remains, main JS approximately 805 kB.
- `git diff --check` - pass.
- Tests were added but not executed under the active operator constraint.
- No app, terminal, agent, daemon, clipboard, or session lifecycle command was executed.

## Production quality and documentation compliance

The source and documentation changes are scoped, dependency-free, and consistent with existing xterm lifecycle behavior. `docs/sessions.md` documents persistence boundaries, shortcuts, SIGINT, bracketed paste, feedback, and clipboard privacy.

## Findings

### [P1 verification gate] Runtime interaction remains unverified

The active production instance prevents the manual checks needed to prove the reported bug is resolved in the real webview/xterm environment. Clipboard API permission and actual TUI behavior cannot be established through compilation.

### [P1] Codex alternate-buffer wheel events are swallowed

`SessionTerminal` installs a DOM `wheel` listener that always calls `term.scrollLines()` and prevents default handling. Codex runs as a full-screen TUI in xterm's alternate buffer, which has no normal scrollback; xterm must receive the event so it can forward TUI mouse input. The current handler therefore prevents Codex from scrolling.

Initial remediation for `2.6.1` removed the DOM listener and delegated alternate-buffer events, but operator testing proved Codex still did not consume wheel or arrow translation. Final remediation now launches Codex with its supported `--no-alt-screen` option, preserving the transcript in normal xterm scrollback. The launch-spec regression test and compile gates pass; a new Codex PTY requires operator runtime confirmation.

## Required follow-up

In an operator-approved safe window:

1. Open two sessions with enough output to create scrollback.
2. Scroll/select text, switch tabs and leave/return to Sessions, then confirm scrollback and selection remain usable.
3. Verify Copy button, selected-text `Ctrl+C`, `Ctrl+Shift+C`, and macOS `Cmd+C` where available.
4. Verify bare `Ctrl+C` without selection still interrupts the child process.
5. Verify Paste button and `Ctrl+Shift+V`/`Cmd+V`, including a bracketed-paste-aware TUI.
6. Run the frontend test suite and lint when allowed.
7. Replace DOM wheel interception with buffer-aware xterm wheel delegation and add a focused policy test.

## Final decision

Changes requested pending runtime/test evidence. Do not mark SPEC-030 done yet.
