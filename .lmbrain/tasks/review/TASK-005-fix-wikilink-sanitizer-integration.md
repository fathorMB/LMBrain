---
id: TASK-005
title: Fix wikilink sanitizer integration
status: review
type: bugfix
priority: critical
area: desktop
milestone: M-01
spec: SPEC-005
depends_on: [TASK-004]
blocks: []
assignee_role: AGENT-FULLSTACK-DESKTOP
links: [SPEC-004, SPEC-005, REVIEW-004]
created: 2026-06-22
updated: 2026-06-22
tags: [markdown, security, wikilinks]
---

# Fix wikilink sanitizer integration

## Expected result

Rendered project-local wikilinks survive sanitization safely and trigger local navigation.

## Acceptance criteria
- [ ] [[REVIEW-004-spec-004-wikilink-sanitization]] is resolved.
- [ ] An integration-level renderer test proves sanitized wikilink navigation.

## Evidence

Implementation submitted for Project Lead review. See [[REVIEW-005-spec-005-unresolved-wikilinks]].
