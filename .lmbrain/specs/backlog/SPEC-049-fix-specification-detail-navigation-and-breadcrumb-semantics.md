---
id: SPEC-049
# Note: Quote the title if it contains a colon
title: "Fix specification detail navigation and breadcrumb semantics"
status: backlog
kind: bugfix
priority: high
area: desktop-navigation
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/11]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-11, accessibility]
activity:
  - date: 2026-07-29
    action: "created"
---
# Fix specification detail navigation and breadcrumb semantics

## Objective
Return from specification detail to the Board through an accessible control without retaining stale selection state or mutating workspace data.

## Context
GitHub issue #11 is confirmed in `src/components/Spec/SpecDetail.tsx`: the breadcrumb is a clickable `div` hard-coded to `navigateTo("reviews")`. `WorkspaceContext.openSpec` selects the spec and enters the `spec` view, but generic navigation does not clear `selectedSpec`.

## Scope
### Included
- Replace the breadcrumb with a semantic button or link preserving the current visual hierarchy.
- Introduce one explicit “return to Board” action that clears `selectedSpec`, closes command-palette state if needed, and navigates to `taskboard`.
- Add an accessible name and visible keyboard focus.
- Add focused regression coverage for routing, selection clearing, keyboard activation, and absence of workspace writes.

### Excluded
- Backend, kit, artifact-format, or Board-card redesign work.
- General navigation-history infrastructure unless a failing regression demonstrates it is required.

## Existing-project analysis
The defect is frontend-only and unchanged between local `v3.0.1` and `origin/main` (`v3.0.2`). The fallback `selectedSpec || readySpecs[0] || specs[0]` means merely navigating away leaves stale detail state available for a later visit; the exit action must clear it deliberately.

## Technical proposal
Add a narrowly named context action such as `closeSpecDetail()` rather than duplicating dispatch order in the component. It should update `selectedSpec`, view, and command-palette state as one reducer action so intermediate state cannot render a fallback spec. Use native button semantics and CSS reset styles for visual compatibility.

## Files and areas involved
- `src/components/Spec/SpecDetail.tsx`
- `src/context/WorkspaceContext.tsx`
- focused component/context tests under `src/__tests__/`

## Acceptance criteria
- [x] Opening a spec from Board and activating the breadcrumb returns to Board, never Reviews.
- [x] The selected specification is cleared as part of the same navigation action.
- [x] Mouse, Enter, and Space activation work through native semantics and the control has an accessible name/focus indicator.
- [x] Returning to Board performs no repository mutation and preserves all workspace collections.
- [x] Direct or fallback entry to the spec view remains deterministic when no selection exists.
- [x] Existing Board and specification rendering tests remain green.

## Implementation plan
1. Add the atomic context/reducer exit action and tests.
2. Replace the clickable `div` with the semantic control.
3. Add the focused regression test and run frontend gates.

## Required verification
- `pnpm test -- SpecDetail`
- `pnpm test`
- `pnpm lint`
- `pnpm build`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
Low risk. Do not add a global browser-like history model for this isolated defect.

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

- Replaced the clickable breadcrumb `div` with a native `button` carrying an
  accessible name and preserved visual treatment.
- Added a dedicated `CLOSE_SPEC_DETAIL` reducer action that atomically selects
  `taskboard`, clears `selectedSpec`, and closes the command palette.
- Added focused component regression coverage for the governed exit action,
  accessible role/name, native button type, click behavior, and keyboard focus.

### Files changed

- `src/components/Spec/SpecDetail.tsx`
- `src/context/WorkspaceContext.tsx`
- `src/__tests__/SpecDetail.test.tsx`

### Verification performed

```text
pnpm test -- SpecDetail.test.tsx
1 file passed; 2 tests passed

pnpm test
29 files passed; 166 tests passed

pnpm lint
eslint completed with exit 0

pnpm build
TypeScript and Vite production build completed with exit 0

git diff --check
exit 0
```

### Deviations from the specification

None.

### Handoff status
- [x] Ready for Project Lead review
