---
id: TASK-009
title: Add real WikiView integration test
status: done
type: test
priority: high
area: desktop
milestone: M-01
spec: SPEC-009
depends_on: [TASK-008]
blocks: []
assignee_role: AGENT-FULLSTACK-DESKTOP
links: [SPEC-008, SPEC-009, REVIEW-008]
created: 2026-06-22
updated: 2026-06-22
tags: [wiki, testing, integration]
---

# Add real WikiView integration test

## Expected result

The actual WikiView command/tree/render path is covered by regression tests.

## Acceptance criteria
- [ ] [[REVIEW-008-spec-008-real-wiki-view-test]] is resolved.
- [ ] Test renders WikiView, mocks the command boundary, and verifies existing/missing behavior.

## Evidence

Project Lead escalation was authorized on 2026-06-22 after repeated missed component-boundary test requirements. The corrective change and separate verification are recorded in [[REVIEW-009-spec-009-project-lead-escalation]].
