---
id: REVIEW-003
title: Re-review of SPEC-003 — project-state correctness
status: changes-requested
spec: SPEC-003
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-001, TASK-002, TASK-003]
links: [SPEC-001, SPEC-002, SPEC-003, TASK-001, TASK-002, TASK-003, REVIEW-002, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, tauri, wiki, diagnostics]
---

# Re-review of SPEC-003 — project-state correctness

## Outcome

Changes requested. The implementation resolves several substantial items, but two blocking correctness issues remain in the new features.

## Resolved since REVIEW-002

- Heading-based extraction now supports the kit's `## Current focus` and `## Current milestone` sections.
- A native Markdown wikilink index and a visible diagnostics surface were added.
- The broad filesystem capability remains removed.
- A restrictive CSP was added.
- `pnpm lint`, `pnpm test`, and `pnpm build` pass in the current review environment.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed. |
| `pnpm test` | Passed: 2 files, 13 tests. No dedicated frontend test was found for rendered wikilink navigation or backlinks. |
| `pnpm build` | Passed. |
| Native Rust commands | Not reproducible in this review environment because `cargo` is unavailable. The implementation evidence reports success, but this cannot be independently validated here. |

## Findings

### F-1 — [P1] Fix status-directory mismatch detection

`build_diagnostics` obtains the immediate parent directory of an artifact (`ready`, `in-progress`, and so on), then immediately skips it because it is not one of `decisions`, `profiles`, `proposals`, `specs`, or `active`. The later comparison checks whether that same immediate directory equals `specs`, `tasks`, or `reviews`, which is impossible. Consequently, the required status-directory/frontmatter mismatch diagnostic is never produced. Derive the artifact type from the grandparent and the status from the parent, then add fixtures for task, spec, and review mismatches.

### F-2 — [P1] Make wikilink rendering demonstrably correct and tested

`MarkdownRenderer` turns `[[target]]` into an HTML-like custom `<wikilink>` element, but the React Markdown pipeline does not include a raw-HTML parser and also sanitizes the HAST tree. The custom component is therefore not a reliable representation of a Markdown wikilink and has no regression test. Implement wikilinks through a supported Markdown AST transformation or a safe custom link protocol handled by the normal link renderer, then add tests proving a rendered `[[target]]` invokes local navigation and unresolved targets remain non-executable/read-only.

### F-3 — [P2] Add coverage for the new contract behavior

The evidence claims contract fields, diagnostics, and path safety are covered, but the visible test inventory still contains only parser/path-safety Rust tests and two frontend test files. Add focused fixtures/tests for the wikilink index, backlinks, status mismatches, heading-based STATUS extraction, and the UI navigation path.

### F-4 — [P2] Complete the deferred search decision

Content search remains absent: the command palette shows static navigation/actions and the Search route is still not implemented. Either implement the scoped `.lmbrain` search promised in SPEC-002 or obtain explicit operator approval for deferral and record it in the roadmap/spec.

## Final decision

Do not mark SPEC-003, SPEC-002, or SPEC-001 accepted. Resolve F-1 and F-2, add the corresponding tests, and re-submit with reproducible evidence.
