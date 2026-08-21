---
id: SPEC-055
# Note: Quote the title if it contains a colon
title: "Add governed ready-spec parking with preserved approval history"
status: backlog
kind: feature
priority: medium
area: spec-lifecycle
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/17]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-17, lifecycle]
activity:
  - date: 2026-07-29
    action: "created"
---
# Add governed ready-spec parking with preserved approval history

## Objective
Allow a no-longer-current ready spec to return to backlog without discarding it or misrepresenting it as startable.

## Context
GitHub issue #17 is confirmed in the transition matrix: specs support the forward lifecycle and discard, but no legal `ready -> backlog` operation. Manual status/path edits would violate the controlled mutation contract.

## Scope
### Included
- Add a semantic `spec_park` operation for `ready -> backlog` only.
- Require a non-empty reason and optionally a revisit trigger/date.
- Preserve content, ID, links, decisions, recommended agent, and full approval/deferral history.
- Atomically update status/path/date and append a typed parking event with actor and prior approval invalidation.
- Require normal `spec_ready` re-approval before start and preserve parking history after re-approval.
- Distinguish parked backlog items in digest/context/Board without treating them as ready, rejected, discarded, or agent failure.
- Add operator confirmation UX and accessible success/error feedback.

### Excluded
- Parking working/review/done/discarded work.
- A new permanent `parked` status.
- Active-work pause/cancel semantics.

## Existing-project analysis
The generic transition function could technically be expanded, but a bare reverse edge would lose the reason and approval semantics. A dedicated operation is required.

## Technical proposal
Add a typed deferral/parking lifecycle event while retaining `status: backlog`. Use the existing mutation lock, destination collision checks, atomic write/move, and audit mechanism. Clear only the current readiness authorization, never historical events.

## Files and areas involved
- core spec lifecycle/transitions/context/diagnostics
- MCP semantic verb and authority schema
- Board/spec detail and TypeScript models
- contract/migration/changelog/docs

## Acceptance criteria
- [ ] A ready spec parks to backlog only with a reason.
- [ ] Status, directory, update time, deferral record, and audit change atomically.
- [ ] Parked work disappears from ready/startable lists and appears in backlog with reason/revisit metadata.
- [ ] `spec_start` rejects it until normal re-approval.
- [ ] Re-approval preserves deferral history.
- [ ] Every other source state fails with a precise explanation and no partial mutation.
- [ ] Collision, interruption, and concurrent mutation tests preserve one authoritative artifact.
- [ ] Parking does not affect agent-failure metrics.

## Implementation plan
1. Define parking event/authority and re-approval semantics.
2. Implement core/MCP operation with atomic failure tests.
3. Add context/digest/Board history and operator UX.
4. Update contract and migration guidance.

## Required verification
- Lifecycle matrix, reason validation, re-approval, collision/interruption/concurrency, digest visibility, and accessible confirmation tests.
- Full Rust/frontend gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- UX decision required before approval: direct operator mutation in the app versus a copyable governed MCP prompt, given the existing removal of direct spec approval actions.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Deviations from the specification

### Handoff status
- [ ] Ready for Project Lead review
