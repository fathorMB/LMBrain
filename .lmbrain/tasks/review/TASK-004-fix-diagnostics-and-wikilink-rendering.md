---
id: TASK-004
title: Fix diagnostics and wikilink rendering correctness
status: review
type: bugfix
priority: critical
area: desktop
milestone: M-01
spec: SPEC-004
depends_on: [TASK-003]
blocks: []
assignee_role: AGENT-FULLSTACK-DESKTOP
links: [SPEC-003, SPEC-004, REVIEW-003]
created: 2026-06-22
updated: 2026-06-22
tags: [tauri, wiki, diagnostics, testing]
---

# Fix diagnostics and wikilink rendering correctness

## Expected result

Status mismatches and in-document wikilinks are correct, safe, and demonstrably covered by tests.

## Acceptance criteria
- [ ] All P1 findings in [[REVIEW-003-spec-003-correctness]] are resolved.
- [ ] Focused regression tests prove the repaired behavior.

## Evidence

Implementation submitted for Project Lead review. See [[REVIEW-004-spec-004-wikilink-sanitization]].
