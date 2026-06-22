---
id: REVIEW-004
title: Re-review of SPEC-004 — wikilink sanitization
status: changes-requested
spec: SPEC-004
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-003, TASK-004]
links: [SPEC-003, SPEC-004, TASK-003, TASK-004, REVIEW-003, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, markdown, security, wikilinks]
---

# Re-review of SPEC-004 — wikilink sanitization

## Outcome

Changes requested. Status mismatch detection is repaired, but rendered wikilinks are still non-functional after the configured sanitizer runs.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 3 files, 20 tests. |
| `pnpm build` | Passed. |
| Native Rust checks | Not reproducible in this environment because `cargo` is unavailable. |
| Sanitizer configuration | The installed `hast-util-sanitize` default schema allows `http`, `https`, `irc`, `ircs`, `mailto`, and `xmpp` for `href`; it does not allow `wikilink`. |

## Findings

### F-1 — [P1] Allow or replace the internal wikilink protocol before sanitization

The preprocessor converts `[[target]]` into `[target](wikilink:target)`, but `rehype-sanitize` strips this unapproved `href` protocol before `WikilinkHandler` receives it. The apparent handler therefore cannot navigate a rendered wikilink. Fix this by either:

1. extending a cloned sanitizer schema to allow only the inert `wikilink` protocol for `href`, while retaining every existing safety restriction; or
2. using a supported Markdown AST transform that creates an internal navigation node without relying on a stripped protocol.

Add an integration-level rendering test—not merely a preprocessor-string test—that proves the sanitized rendered output invokes the local navigation handler for a resolved wikilink and retains no executable external behavior for unresolved input.

## Final decision

Do not mark SPEC-004 or preceding specs accepted. Resolve F-1 with a sanitizer-aware implementation and re-submit with an end-to-end renderer test.
