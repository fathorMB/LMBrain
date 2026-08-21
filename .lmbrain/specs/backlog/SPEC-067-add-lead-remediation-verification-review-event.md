---
id: SPEC-067
title: "Add Lead remediation verification review event verb and actor support"
status: backlog
kind: feature
priority: low
area: review-lifecycle
milestone: M-08
recommended_agent: AGENT-RUST-CORE
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/41]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-41, review-lifecycle, remediation-verification, KIT-NOTE-015]
activity:
  - date: 2026-07-31
    action: "created"
---
# Add Lead remediation verification review event verb and actor support

## Objective
Provide an authoritative review event verb or explicit actor parameter so Project Lead remediation checks are accurately recorded without misattributing them to the implementation specialist.

## Context
Reported in `KIT-NOTE-015` (v3.1.2): When a Project Lead verifies an implementation remediation cycle without changing status or executing a full takeover, calling `review_remediation` automatically hardcodes `actor_role: implementation-specialist`. This distorts the typed lifecycle history of the review.

## Scope
### Included
- Add a new verb `review_remediation_verified` reserved for Project Lead verification notes, OR extend `review_remediation` with an optional `actor` parameter.
- Ensure the recorded review event accurately reflects `actor_role: AGENT-LEAD` when invoked by the Lead.
- Update review event schema and diagnostic checks.

### Excluded
- Changing existing closed review event records retroactively.

## Acceptance criteria
- [ ] Project Lead remediation verification checks can be recorded without altering review status or misattributing the actor role.
- [ ] Structured review events record `actor_role: AGENT-LEAD` for Lead verification actions.

## Required verification
- `cargo test --workspace`
- `pnpm test`
