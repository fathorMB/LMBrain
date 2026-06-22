---
id: REVIEW-008
title: Re-review of SPEC-008 — real WikiView test
status: changes-requested
spec: SPEC-008
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-007, TASK-008]
links: [SPEC-007, SPEC-008, TASK-007, TASK-008, REVIEW-007, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, wiki, testing, integration]
---

# Re-review of SPEC-008 — real WikiView test

## Outcome

Changes requested. The new test file is named `WikiView.test.tsx`, but it renders `MarkdownRenderer` directly. It therefore does not exercise WikiView, mocked native commands, WikiTree traversal, or the actual integration boundary specified in SPEC-008.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 5 files, 31 tests. |
| `pnpm build` | Passed. |
| Native Rust checks | Not reproducible in this environment because `cargo` is unavailable. |

## Finding

### F-1 — [P1] Replace the renderer-only test with a real WikiView integration test

The current test manually supplies `resolvedTargets` to `MarkdownRenderer`, recreating exactly the isolated coverage that already existed. Mock `getWikiTree`, `getWikilinkIndex`, and `getWikiPage`; render `WikiView` inside the real workspace provider or a faithful test provider; wait for the tree/index load; select a source page; and assert that the actual tree-derived data makes EXISTING interactive while MISSING stays inert despite appearing in the mocked outbound index. Verify that activating EXISTING calls `getWikiPage` for the target, while MISSING never does.

## Final decision

Do not mark SPEC-008 or preceding specs accepted until F-1 is covered by an actual WikiView component integration test.
