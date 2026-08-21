---
id: REVIEW-007
title: Re-review of SPEC-007 — WikiView integration coverage
status: changes-requested
spec: SPEC-007
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-006, TASK-007]
links: [SPEC-006, SPEC-007, TASK-006, TASK-007, REVIEW-006, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, wiki, testing]
---

# Re-review of SPEC-007 — WikiView integration coverage

## Outcome

Changes requested. The production implementation now derives resolved targets from real WikiTree file nodes and appears functionally correct. However, the required WikiView-level integration test was not added; only isolated MarkdownRenderer tests exist.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 4 files, 28 tests. |
| `pnpm build` | Passed. |
| Native Rust checks | Not reproducible in this environment because `cargo` is unavailable. |

## Finding

### F-1 — [P1] Add the required WikiView integration fixture

SPEC-007 explicitly requires a component-boundary test containing an existing document and a mentioned-but-missing target. The current test suite passes a manually constructed `resolvedTargets` set directly into `MarkdownRenderer`; it does not exercise WikiTree traversal, the separation from `wikilinkIndex`, or the integration path that caused the original defect. Add a WikiView test with mocked native commands and a realistic tree/index fixture. Assert that `[[EXISTING]]` is interactive and that `[[MISSING]]`, even when present in the outbound-link index, remains inert.

## Final decision

Do not mark SPEC-007 or preceding specs accepted until F-1 is covered by an integration test and exact verification evidence is updated.
