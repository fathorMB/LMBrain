---
id: REVIEW-006
title: Re-review of SPEC-006 — wikilink resolution source
status: changes-requested
spec: SPEC-006
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-005, TASK-006]
links: [SPEC-005, SPEC-006, TASK-005, TASK-006, REVIEW-005, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, wiki, wikilinks, correctness]
---

# Re-review of SPEC-006 — wikilink resolution source

## Outcome

Changes requested. The Markdown renderer correctly supports resolved versus unresolved states when given valid resolution data. Wiki integration supplies invalid resolution data, so unknown targets remain incorrectly marked as resolved.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 4 files, 28 tests. |
| `pnpm build` | Passed. |
| Native Rust checks | Not reproducible in this environment because `cargo` is unavailable. |

## Finding

### F-1 — [P1] Derive resolved targets from the document tree, not outbound-link keys

`WikiView` builds `resolvedTargets` from `Object.keys(wikilinkIndex)`. That index contains every target mentioned by a source document, including nonexistent targets; it is an inbound/backlink index, not a document-existence index. For example, a page containing `[[MISSING-ADR]]` inserts `missing-adr` into the key set and causes the renderer to mark it resolved, after which click-time resolution silently fails.

Derive resolved targets from the actual `WikiTree` file nodes (including accepted aliases/path normalization), or expose a native canonical document index. Add an integration test at the `WikiView` level demonstrating that a mentioned-but-missing target remains inert and visually unresolved.

## Final decision

Do not mark SPEC-006 or preceding specs accepted until F-1 is resolved with an integration test using the real Wiki resolution source.
