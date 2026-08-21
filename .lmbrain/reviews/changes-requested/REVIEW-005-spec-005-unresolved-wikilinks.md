---
id: REVIEW-005
title: Re-review of SPEC-005 — unresolved wikilink behavior
status: changes-requested
spec: SPEC-005
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-004, TASK-005]
links: [SPEC-004, SPEC-005, TASK-004, TASK-005, REVIEW-004, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, markdown, wikilinks, ux]
---

# Re-review of SPEC-005 — unresolved wikilink behavior

## Outcome

Changes requested. The sanitizer-aware fragment implementation and renderer integration tests are sound. One acceptance criterion remains unmet: unresolved local wikilinks are not visibly distinct and are still rendered as interactive controls.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 4 files, 27 tests. |
| `pnpm build` | Passed. |
| Native Rust checks | Not reproducible in this environment because `cargo` is unavailable. |

## Finding

### F-1 — [P1] Resolve before rendering and make unresolved wikilinks inert

`MarkdownRenderer` renders every internal fragment as the same clickable dashed `span`. In Wiki, a resolver is invoked only after click; an unknown target therefore looks valid and silently does nothing. Pass a resolution predicate/map into the Markdown renderer, or preprocess only resolved targets into the internal fragment form. Render unknown targets as clearly muted text with an explanatory title and no click handler, keyboard action, or pointer cursor. Add an integration test that verifies both visual/state distinction and absence of navigation for an unresolved target.

## Final decision

Do not mark SPEC-005 or preceding specs accepted until F-1 is resolved and tested.
