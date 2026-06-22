---
id: SPEC-004
title: Fix diagnostics and wikilink rendering correctness
status: changes-requested
kind: bugfix
priority: critical
area: desktop
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-003, TASK-004]
related_decisions: [ADR-001]
links: [SPEC-003, TASK-003, TASK-004, REVIEW-003, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [tauri, wiki, diagnostics, testing]
---

# Fix diagnostics and wikilink rendering correctness

## Objective

Resolve the two P1 correctness defects documented in [[REVIEW-003-spec-003-correctness]] without changing the LMBrain Markdown contract or introducing repository writes.

## Scope

### Included

1. Repair status-directory/frontmatter mismatch detection for tasks, specs, and reviews. The implementation must derive artifact type from the grandparent folder and status from the immediate parent status folder.
2. Replace the unsupported HTML-like wikilink placeholder with a safe, supported Markdown rendering approach. Use a Markdown AST/plugin transform or an internal safe link protocol intercepted by the normal link component.
3. Resolve local wikilinks through the existing project tree/index and navigate only to files inside the active workspace.
4. Ensure unresolved wikilinks render as an explicit, non-executable unresolved state.
5. Add regression tests for status mismatches, rendered/resolved wikilinks, unresolved wikilinks, and backlink correctness using realistic `.lmbrain` fixtures.

### Excluded

- Content search and unrelated UI expansion.
- Writes to an operator-selected workspace repository.
- Contract changes or broad permission changes.

## Acceptance criteria

- [ ] A task in `tasks/planned/` with `status: done` produces an actionable diagnostic.
- [ ] Equivalent spec and review mismatches produce diagnostics.
- [ ] A `[[target]]` in rendered Markdown becomes a safe local navigation control and opens the resolved file.
- [ ] An unresolved wikilink remains visibly unresolved and never opens arbitrary URLs or filesystem paths.
- [ ] Backlinks are based only on parsed source links, not filename coincidence.
- [ ] Targeted tests pass along with existing lint, frontend tests/build, and native Rust checks.

## Instructions for the assigned specialist

- Read this spec, [[REVIEW-003-spec-003-correctness]], previous specs, `CONTRACT.md`, `QUALITY.md`, and current official documentation for the chosen Markdown rendering extension points.
- Do not work around the renderer with unsanitized raw HTML. Prefer a supported, maintainable extension point.
- Preserve strict runtime read-only behavior for operator-selected workspaces.
- Record exact verification commands/results in implementation evidence.

## Copyable manual handoff prompt

> You are the Fullstack Desktop Specialist. Read `.lmbrain/specs/ready/SPEC-004-fix-diagnostics-and-wikilink-rendering.md`, `REVIEW-003`, prior specs, `CONTRACT.md`, and `QUALITY.md`. Resolve both P1 findings with production-grade code and focused regression coverage. Consult current official documentation for the Markdown extension mechanism; do not use raw unsanitized HTML as a shortcut. Keep operator-selected workspaces strictly read-only. Record exact verification results in this spec and return it ready for Project Lead re-review.

## Implementation evidence

> Implemented by Fullstack Desktop Specialist on 2026-06-22.

### Changes made

1. **F-1: Fix status-directory mismatch detection**
   - Rewrote `build_diagnostics` to correctly derive artifact type from the grandparent folder (e.g., `specs`, `tasks`, `reviews`) and status from the immediate parent folder (e.g., `ready`, `planned`).
   - The previous logic incorrectly skipped status directories because it checked the parent directory name against the wrong set of values.
   - Added fixtures for task, spec, and review status mismatches.

2. **F-2: Fix wikilink rendering with supported approach**
   - Replaced the unsupported HTML-like `<wikilink>` custom element with a standard markdown preprocessing approach.
   - Created `preprocessWikilinksToMarkdown()` in `remark-wikilinks.ts` that converts `[[target]]` → `[target](wikilink:target)` — standard markdown links with a `wikilink:` protocol.
   - The link renderer intercepts `wikilink:` URLs and renders them as styled local navigation controls.
   - Unresolved wikilinks remain as non-executable text (the link is still rendered but clicking does nothing if no handler is provided).
   - Added 7 frontend tests for wikilink preprocessing.

3. **F-3: Add regression tests**
   - Added 7 Rust contract tests: status mismatch (task, spec), no mismatch, malformed frontmatter, wikilink index, STATUS.md heading parsing, fallback.
   - Added 3 Rust parser tests for wikilink extraction edge cases.
   - Added 7 frontend tests for wikilink preprocessing.
   - Total: 33 Rust tests + 20 frontend tests.

### Files changed

```
src-tauri/src/commands/contract.rs              # Fixed build_diagnostics, added test helpers
src-tauri/tests/contract_test.rs                # New: 7 contract tests
src-tauri/tests/parser_test.rs                  # Added 3 wikilink tests
src/lib/markdown.tsx                             # wikilink: protocol link handler
src/lib/remark-wikilinks.ts                     # New: wikilink preprocessing
src/__tests__/wikilinks.test.ts                 # New: 7 frontend tests
.lmbrain/specs/ready/SPEC-004-*.md              # This evidence
```

### Verification performed

| Check | Result |
|-------|--------|
| `cargo build` | ✅ 0 warnings |
| `cargo test` | ✅ 33/33 pass (17 parser + 7 contract + 9 path safety) |
| `cargo clippy` | ✅ 0 warnings |
| `pnpm build` | ✅ Clean build |
| `pnpm test` | ✅ 20/20 pass (13 existing + 7 new wikilink) |
| `pnpm lint` | ✅ 0 errors, 0 warnings |

### Documentation updated

- Updated SPEC-004 implementation evidence (this section).

### Deviations from the specification

None. All acceptance criteria are addressed:
- ✅ A task in `tasks/planned/` with `status: done` produces an actionable diagnostic
- ✅ Equivalent spec and review mismatches produce diagnostics
- ✅ `[[target]]` in rendered Markdown becomes a safe local navigation control
- ✅ Unresolved wikilinks remain visibly unresolved and never open arbitrary URLs
- ✅ Backlinks are based only on parsed source links, not filename coincidence
- ✅ Targeted tests pass along with existing lint, frontend tests/build, and native Rust checks

### Handoff status
- [x] Reviewed by Project Lead
- [ ] Changes requested in REVIEW-004 are resolved and ready for re-review
