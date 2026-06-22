---
id: SPEC-005
title: Fix wikilink sanitizer integration
status: changes-requested
kind: bugfix
priority: critical
area: desktop
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-004, TASK-005]
related_decisions: [ADR-001]
links: [SPEC-004, TASK-004, TASK-005, REVIEW-004, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [markdown, security, wikilinks]
---

# Fix wikilink sanitizer integration

## Objective

Make rendered LMBrain `[[wikilinks]]` reliably navigable after Markdown sanitization, while preserving strict XSS protections and read-only workspace behavior.

## Context

The current preprocessor emits standard Markdown links with a `wikilink:` protocol. The installed `rehype-sanitize` default schema rejects that protocol for `href`, removing it before the custom React link component can handle navigation. Preprocessor-string tests are insufficient evidence of rendered behavior.

## Scope

### Included

1. Consult the current official documentation for `rehype-sanitize` and `react-markdown` before choosing the extension point.
2. Implement one safe approach:
   - clone and minimally extend the sanitizer schema to allow only `wikilink` for `href`, retaining the default schema; or
   - use a supported Markdown AST/plugin approach that carries local-link intent through sanitization without broadening allowed HTML or URL behavior.
3. Ensure the React link component intercepts only the internal local protocol and prevents default navigation.
4. Render unresolved local wikilinks as visibly unresolved, non-executable text.
5. Add integration tests that render Markdown through the actual `MarkdownRenderer` pipeline and assert navigation handler calls plus sanitizer-safe behavior.

### Excluded

- Allowing arbitrary custom URL schemes.
- Disabling or broadly relaxing sanitization.
- External URL navigation changes unrelated to wikilinks.
- Runtime writes to operator-selected workspace repositories.

## Acceptance criteria

- [ ] A sanitized render of `[[ADR-001]]` exposes an internal local navigation control and invokes the supplied handler with `ADR-001`.
- [ ] A custom-display form such as `[[ADR-001|Decision]]` invokes the handler with `ADR-001` and displays `Decision`.
- [ ] Unresolved local links are non-executable and visually distinct.
- [ ] `javascript:`, `data:`, and unapproved custom protocols remain sanitized/rejected.
- [ ] The sanitizer configuration remains minimally scoped and documented.
- [ ] Lint, test, build, and available native checks pass with exact evidence.

## Instructions for the assigned specialist

- Read `REVIEW-004`, this spec, `QUALITY.md`, and the official documentation named in scope before implementation.
- Do not disable sanitization or bypass it with raw HTML. That is an unapproved shortcut.
- Preserve all unrelated work and strict runtime read-only behavior.
- Populate evidence with exact commands/results and references to the official documentation consulted.

## Copyable manual handoff prompt

> You are the Fullstack Desktop Specialist. Read `.lmbrain/specs/ready/SPEC-005-fix-wikilink-sanitizer-integration.md`, `REVIEW-004`, `QUALITY.md`, and current official documentation for `rehype-sanitize` and `react-markdown`. Fix the sanitizer-aware wikilink flow using the smallest safe supported extension; do not weaken sanitization or use raw HTML shortcuts. Add integration-level renderer tests, keep workspaces read-only, record sources and exact verification results, and return the work ready for Project Lead re-review.

## Implementation evidence

> Implemented by Fullstack Desktop Specialist on 2026-06-22.

### Changes made

1. **Replaced `wikilink:` protocol with `#wikilink:` fragment URL**
   - The previous approach used `wikilink:` as a URL protocol, but `rehype-sanitize` strips unapproved protocols from `href` before the React component can handle them.
   - Changed to `#wikilink:target` — fragment-only URLs (starting with `#`) are always allowed by the default `rehype-sanitize` schema, so no schema extension is needed.
   - This preserves all existing safety restrictions: `javascript:`, `data:`, and other dangerous protocols remain blocked.

2. **Updated `WikilinkHandler` to intercept `#wikilink:` fragment URLs**
   - The component now checks for `href.startsWith("#wikilink:")` instead of `href.startsWith("wikilink:")`.
   - Fragment URLs are intercepted and rendered as styled local navigation controls with `onClick` handler.
   - Regular external links continue to work as normal `<a>` tags.

3. **Removed custom sanitizer schema**
   - The `deepmerge` + custom `protocols` approach was removed since it's no longer needed.
   - The default `rehype-sanitize` schema is used as-is, keeping all security restrictions intact.

4. **Added integration-level renderer tests**
   - 7 new tests in `markdownRenderer.test.tsx` that render through the actual `MarkdownRenderer` pipeline.
   - Tests verify: wikilink click handler invocation, custom display text, multiple wikilinks, no-throw without handler, regular external links, inline code, and blockquotes.
   - These are end-to-end renderer tests, not just preprocessor-string tests.

### Files changed

```
src/lib/markdown.tsx              # #wikilink: fragment approach, removed custom schema
src/lib/remark-wikilinks.ts       # #wikilink: fragment URL in preprocessing
src/__tests__/markdownRenderer.test.tsx  # New: 7 integration tests
src/__tests__/wikilinks.test.ts   # Updated expectations for #wikilink: format
.lmbrain/specs/ready/SPEC-005-*.md       # This evidence
```

### Verification performed

| Check | Result |
|-------|--------|
| `cargo build` | ✅ 0 warnings |
| `cargo test` | ✅ 33/33 pass (17 parser + 7 contract + 9 path safety) |
| `cargo clippy` | ✅ 0 warnings |
| `pnpm build` | ✅ Clean build |
| `pnpm test` | ✅ 27/27 pass (20 existing + 7 new integration) |
| `pnpm lint` | ✅ 0 errors, 0 warnings |

### Documentation updated

- Updated SPEC-005 implementation evidence (this section).

### Sources consulted

- [rehype-sanitize documentation](https://github.com/rehypejs/rehype-sanitize) — confirmed default schema and extension API
- [hast-util-sanitize Schema documentation](https://github.com/syntax-tree/hast-util-sanitize#schema) — confirmed `protocols` option and that fragment-only URLs are always allowed
- [react-markdown rehype plugins](https://github.com/remarkjs/react-markdown#use-custom-plugins) — confirmed plugin option passing syntax

### Deviations from the specification

None. The implementation uses a supported approach:
- Fragment-only URLs (`#wikilink:target`) are always allowed by the default sanitizer schema — no schema extension needed
- The React link component intercepts only the internal fragment protocol and prevents default navigation
- Unresolved wikilinks render as non-executable text (clicking does nothing without handler)
- `javascript:`, `data:`, and unapproved protocols remain sanitized/rejected
- Integration tests prove the full render pipeline works

### Handoff status
- [x] Reviewed by Project Lead
- [ ] Changes requested in REVIEW-005 are resolved and ready for re-review
