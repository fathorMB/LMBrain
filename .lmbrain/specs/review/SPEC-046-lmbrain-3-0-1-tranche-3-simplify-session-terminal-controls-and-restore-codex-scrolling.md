---
id: SPEC-046
# Note: Quote the title if it contains a colon
title: "LMBrain 3.0.1 tranche 3: simplify session terminal controls and restore Codex scrolling"
status: review
kind: feature
priority: medium
area: 
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-XXX
related_tasks: []
related_decisions: []
links: []
created: 2026-07-18
updated: 2026-07-18
tags: []
activity:
  - date: 2026-07-18
    action: "created"
activity:
  - date: 2026-07-18
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-18
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-18
    action: "transitioned working -> review"
---
# LMBrain 3.0.1 tranche 3: simplify session terminal controls and restore Codex scrolling

## Objective
Make embedded agent sessions behave like a normal terminal: native wheel scrolling for Codex, standard keyboard clipboard interaction, and no redundant copy/paste/page navigation button cluster.

## Context
Operator testing of the 3.0.1 release branch found that the Sessions toolbar was visually dense and duplicated standard terminal interactions. Codex sessions still did not respond reliably to wheel scrolling despite being launched with `--no-alt-screen`.

## Scope
### Included
- Remove the Select text, Copy, Copy visible, Paste, Page up, Page down, and Bottom controls.
- Preserve copy/paste through standard terminal shortcuts and selection through native xterm behavior.
- Preserve Search logs as the durable transcript action.
- Delegate normal-buffer wheel events to xterm so Codex scrollback remains native.
- Keep explicit alternate-buffer mappings required by Pi and OpenCode.

### Excluded
- Changes to session persistence, transcript retention, or harness launch routes.
- Redesign of session tabs or the session creation dialog.

## Existing-project analysis
Codex is launched with `--no-alt-screen`, which stores its output in xterm's normal buffer. The custom wheel handler nevertheless prevented xterm's native event path and manually approximated scrolling. This unnecessary interception was the most fragile part of the flow.

## Technical proposal
Delegate all normal-buffer wheel gestures directly to xterm. Retain custom PTY input only for alternate-screen tools with known bindings. Reduce the terminal chrome to a short interaction hint and the Search logs action.

## Files and areas involved
- `src/components/Sessions/SessionTerminal.tsx`
- `src/lib/terminalWheel.ts`
- `src/lib/terminalSelection.ts` (removed)
- `src/__tests__/terminalWheel.test.ts`
- `src/__tests__/terminalSelection.test.ts` (removed)
- `docs/sessions.md`

## Acceptance criteria
- [x] Codex normal-buffer sessions use xterm's native wheel scrolling path.
- [x] Redundant clipboard and page navigation buttons are absent.
- [x] Clipboard shortcuts remain functional and Search logs remains available.
- [x] Pi and OpenCode alternate-buffer wheel mappings remain covered by tests.
- [x] Sessions documentation describes the simplified interaction model.

## Implementation plan
1. Simplify the terminal toolbar without removing keyboard behavior.
2. Correct the wheel policy at the buffer boundary.
3. Update focused tests and session documentation.
4. Run focused and repository-wide frontend gates.

## Required verification
- `pnpm exec vitest run src/__tests__/terminalWheel.test.ts src/__tests__/terminalClipboard.test.ts src/__tests__/SessionsView.test.tsx`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
Alternate-screen TUIs do not expose ordinary scrollback. Their existing host-specific behavior is intentionally preserved; this tranche targets Codex's normal-buffer path.

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
- Removed the seven redundant terminal action buttons and selection-mode state.
- Restored xterm's native wheel processing for all normal buffers, including Codex.
- Retained keyboard clipboard handling, concise feedback, and Search logs.

### Files changed
- `src/components/Sessions/SessionTerminal.tsx`
- `src/lib/terminalWheel.ts`
- `src/lib/terminalSelection.ts` (removed)
- `src/__tests__/terminalWheel.test.ts`
- `src/__tests__/terminalSelection.test.ts` (removed)
- `docs/sessions.md`

### Verification performed
- Focused ESLint passed.
- Focused terminal and Sessions tests passed.
- `pnpm lint`: passed.
- `pnpm test`: 26 files / 139 tests passed.
- `pnpm build`: passed; only the existing Vite chunk-size advisory remains.
- `git diff --check`: passed.

### Verification transcript
```text
$ pnpm lint
$ eslint .
exit code: 0

$ pnpm test
$ vitest run
Test Files  26 passed (26)
Tests       139 passed (139)
exit code: 0

$ pnpm build
$ tsc -b && vite build
320 modules transformed.
dist/assets/index-CMzqEUmD.js  919.86 kB | gzip: 245.60 kB
built in 323ms
exit code: 0

$ git diff --check
exit code: 0
```

### Deviations from the specification
- None.

### Handoff status
- [x] Ready for Project Lead review
