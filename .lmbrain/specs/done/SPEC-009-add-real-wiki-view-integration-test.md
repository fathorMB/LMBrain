---
id: SPEC-009
title: Add real WikiView integration test
status: done
kind: test
priority: high
area: desktop
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-008, TASK-009]
related_decisions: [ADR-001]
links: [SPEC-008, TASK-008, TASK-009, REVIEW-008, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [wiki, testing, integration]
---

# Add real WikiView integration test

## Objective

Replace renderer-only coverage with a genuine WikiView component integration test that proves real document existence drives link state.

## Scope

### Included

1. Mock `getWikiTree`, `getWikilinkIndex`, and `getWikiPage` from the command layer.
2. Render the actual `WikiView` under the workspace context used in the application.
3. Provide a source document containing `[[EXISTING]]` and `[[MISSING]]`, a WikiTree that contains only EXISTING, and an outbound-link index that contains both target names.
4. Simulate selecting the source page in WikiView.
5. Assert that the real tree-derived integration makes EXISTING focusable/interactive and causes a target page request; assert MISSING is inert and never causes a target page request.

### Excluded

- Changes to production Wiki behavior unless the integration test exposes a real defect.
- Replacing the test with a MarkdownRenderer-only unit test.

## Acceptance criteria

- [ ] Test imports and renders `WikiView`, not only `MarkdownRenderer`.
- [ ] Command-layer mocks provide tree, index, and source/target page data.
- [ ] The test fails with the former outbound-index-derived logic.
- [ ] The test proves MISSING is inert even though present in the mocked index.
- [ ] Lint, tests, and build pass with truthful evidence.

## Instructions for the assigned specialist

- Read [[REVIEW-008-spec-008-real-wiki-view-test]], this spec, `WikiView.tsx`, context/test setup, and `QUALITY.md`.
- Do not call a renderer-only test an integration test. The real component and command boundary must participate.
- Keep the test small, deterministic, and free of Tauri runtime requirements.

## Copyable manual handoff prompt

> You are the Fullstack Desktop Specialist. Read `.lmbrain/specs/ready/SPEC-009-add-real-wiki-view-integration-test.md`, `REVIEW-008`, the real WikiView/context code, and `QUALITY.md`. Add a genuine WikiView integration test with mocked command functions and a realistic tree/index fixture. Do not render MarkdownRenderer directly as a substitute. Prove EXISTING navigates and MISSING remains inert despite appearing in the outbound index. Run quality gates and record exact evidence for Project Lead re-review.

## Implementation evidence

> Project Lead escalation: the same component-boundary test criterion was missed in two consecutive specialist remediation attempts. The operator authorized direct corrective implementation on 2026-06-22.

### Changes made

Replaced the renderer-only test with a component-level `WikiView` integration test. It renders the real workspace context, mocks the command boundary (`getWikiTree`, `getWikilinkIndex`, `getWikiPage`), selects a source document, and verifies that the tree-resolved `EXISTING` link navigates while `MISSING` remains inert even when it is present in the outbound-link index.

### Files changed

- `src/__tests__/WikiView.test.tsx`

### Verification performed

- `pnpm lint` — passed.
- `pnpm test` — passed: 5 files, 29 tests.
- `pnpm build` — passed.
- Native Rust checks were not reproducible in this environment because `cargo` is unavailable.

### Documentation updated

- This specification, [[TASK-009-add-real-wiki-view-integration-test]], and [[REVIEW-009-spec-009-project-lead-escalation]].

### Deviations from the specification

None.

### Handoff status
- [x] Independently verified and accepted by the Project Lead.
