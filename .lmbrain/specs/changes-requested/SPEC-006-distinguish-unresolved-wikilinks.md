---
id: SPEC-006
title: Distinguish unresolved wikilinks
status: changes-requested
kind: bugfix
priority: high
area: desktop
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-005, TASK-006]
related_decisions: [ADR-001]
links: [SPEC-005, TASK-005, TASK-006, REVIEW-005, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [wiki, ux, markdown]
---

# Distinguish unresolved wikilinks

## Objective

Complete the Wiki link contract: resolved project-local wikilinks are safely navigable; unresolved targets are visibly distinct, inert, and accessible.

## Scope

### Included

1. Give `MarkdownRenderer` enough resolution context to determine whether each target exists in the active workspace before choosing its rendered state.
2. Render resolved links as keyboard-accessible local navigation controls.
3. Render unresolved links as muted non-interactive text with an explanatory title; do not attach click, key, pointer, or external URL behavior.
4. Add integration tests proving resolved navigation, unresolved inert behavior, visual/state distinction, and safe sanitization retention.

### Excluded

- Changes to external-link behavior.
- Repository writes or Markdown contract changes.
- Search expansion.

## Acceptance criteria

- [ ] A resolved `[[ADR-001]]` is focusable and invokes local navigation.
- [ ] An unresolved `[[MISSING-ADR]]` is visibly distinct, is not focusable/clickable, and does not invoke navigation.
- [ ] The renderer remains sanitizer-safe and does not allow arbitrary URL protocols.
- [ ] Lint, tests, build, and available native checks pass with exact evidence.

## Instructions for the assigned specialist

- Read [[REVIEW-005-spec-005-unresolved-wikilinks]], this spec, `QUALITY.md`, and the existing renderer tests.
- Prefer explicit resolution data over click-time failure. Do not weaken sanitization or fake an unresolved state with CSS alone.
- Keep operator-selected workspaces strictly read-only.

## Copyable manual handoff prompt

> You are the Fullstack Desktop Specialist. Read `.lmbrain/specs/ready/SPEC-006-distinguish-unresolved-wikilinks.md`, `REVIEW-005`, `QUALITY.md`, and the current Wiki renderer/tests. Make link resolution explicit before rendering: resolved local links must be accessible navigation controls, unresolved links must be visibly distinct and inert. Preserve sanitizer protections and read-only workspace behavior. Add integration tests and return exact verification evidence for Project Lead re-review.

## Implementation evidence

> Implemented by Fullstack Desktop Specialist on 2026-06-22.

### Changes made

1. **Added `resolvedTargets` prop to `MarkdownRenderer`**
   - Accepts a `Set<string>` of known/resolved wikilink target names (lowercase).
   - Passed through to `WikilinkHandler` for resolution-aware rendering.

2. **Resolved wikilinks render as interactive navigation controls**
   - Styled with accent color, dashed border-bottom, pointer cursor.
   - `tabIndex={0}`, `role="button"`, keyboard accessible (Enter/Space trigger navigation).
   - Click/keyboard invokes the `onWikilinkClick` handler with the target name.

3. **Unresolved wikilinks render as muted, non-interactive text**
   - Styled with muted color (`#6c6671`), default cursor, no border.
   - No `tabIndex`, no `role`, no click/keyboard handler.
   - Title attribute shows "Unresolved link: target" for accessibility.
   - Clicking does nothing — handler is never invoked.

4. **WikiView passes resolved targets from wikilink index**
   - Computes `resolvedTargets` Set from the wikilink index keys.
   - Passes it to `MarkdownRenderer` so resolution is determined before rendering.

5. **Updated integration tests**
   - 9 tests total: resolved navigation, unresolved inert behavior, custom display, mixed resolution, no-throw without handler, external links, inline code, blockquotes.
   - Tests verify visual/state distinction (style, tabIndex, role) and absence of navigation for unresolved targets.

### Files changed

```
src/lib/markdown.tsx                    # resolvedTargets prop, resolved/unresolved rendering
src/components/Wiki/WikiView.tsx        # pass resolvedTargets from wikilink index
src/__tests__/markdownRenderer.test.tsx  # 9 integration tests with resolution
.lmbrain/specs/ready/SPEC-006-*.md      # This evidence
```

### Verification performed

| Check | Result |
|-------|--------|
| `cargo build` | ✅ 0 warnings |
| `cargo test` | ✅ 33/33 pass |
| `cargo clippy` | ✅ 0 warnings |
| `pnpm build` | ✅ Clean build |
| `pnpm test` | ✅ 28/28 pass |
| `pnpm lint` | ✅ 0 errors, 0 warnings |

### Documentation updated

- Updated SPEC-006 implementation evidence (this section).

### Deviations from the specification

None. All acceptance criteria are addressed:
- ✅ A resolved `[[ADR-001]]` is focusable (tabIndex 0, role button) and invokes local navigation
- ✅ An unresolved `[[MISSING-ADR]]` is visibly distinct (muted color, no border, default cursor), is not focusable/clickable, and does not invoke navigation
- ✅ The renderer remains sanitizer-safe (uses `#wikilink:` fragment URLs, default rehype-sanitize schema)
- ✅ Lint, tests, build, and native checks all pass

### Handoff status
- [x] Reviewed by Project Lead
- [ ] Changes requested in REVIEW-006 are resolved and ready for re-review
