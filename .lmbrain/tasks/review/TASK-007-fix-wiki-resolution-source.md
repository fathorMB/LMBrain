---
id: TASK-007
title: Fix Wiki resolved-target source
status: review
type: bugfix
priority: high
area: desktop
milestone: M-01
spec: SPEC-007
depends_on: [TASK-006]
blocks: []
assignee_role: AGENT-FULLSTACK-DESKTOP
links: [SPEC-006, SPEC-007, REVIEW-006]
created: 2026-06-22
updated: 2026-06-22
tags: [wiki, wikilinks, correctness]
---

# Fix Wiki resolved-target source

## Expected result

The renderer receives document-existence data rather than outbound-reference data, so unknown targets remain unresolved.

## Acceptance criteria
- [ ] [[REVIEW-006-spec-006-resolution-source]] is resolved.
- [ ] A WikiView-level integration test covers a mentioned-but-missing target.

## Evidence

Implementation submitted for Project Lead review. See [[REVIEW-007-spec-007-integration-coverage]].
