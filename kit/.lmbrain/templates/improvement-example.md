---
id: AGENT-PROP-IMPROVEMENT-EXAMPLE
title: "Example: Improve frontend specialist review focus"
status: example
requested_by: AGENT-LEAD
reason: repeated-review-finding
proposal_type: improvement
target_profile: AGENT-FRONTEND-UI
recommended_for: [SPEC-XXX]
links: [REVIEW-XXX]
created: 2026-07-02
updated: 2026-07-02
tags: [proposal, improvement, example]
---

# Example improvement proposal

## Observed problem

The Frontend UI Specialist's review_focus does not include performance profiling. Two consecutive reviews (REVIEW-XXX, REVIEW-YYY) flagged unnecessary re-renders that were not caught during implementation.

## Proposed change

Add `performance` to the `review_focus` list in AGENT-FRONTEND-UI.

## Evidence

- REVIEW-XXX: "Component re-renders on every keystroke — useMemo missing"
- REVIEW-YYY: "Large list has no virtualization — 200ms paint time"
- SPEC-XXX: "Optimize search results rendering"

## Decision requested
- [ ] Approve
- [ ] Defer
- [ ] Reject
