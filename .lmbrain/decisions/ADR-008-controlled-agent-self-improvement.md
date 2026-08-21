---
id: ADR-008
title: Controlled agent self-improvement through reviewed proposals
status: accepted
decision_date: 2026-07-02
decider: operator
supersedes: []
superseded_by: []
links: [ADR-004, SPEC-024]
tags: [architecture, agents, governance, v3]
---

# Controlled agent self-improvement through reviewed proposals

## Context

The operator wants LMBrain agents to become self-improving on a project over time. This is valuable: repeated reviews and remediation loops produce evidence about what each specialist should read, avoid, verify, and document.

Uncontrolled self-improvement would conflict with the LMBrain contract. Agent profiles shape future behavior, so allowing agents to silently rewrite their own profiles or prompts would bypass operator approval, weaken auditability, and make regressions difficult to trace.

Existing governance already provides useful constraints:

- Agent profiles are managed artifacts.
- Every profile uses `activation: manual`.
- The Project Lead recommends existing profiles before proposing new ones.
- Profile approval/activation is the operator's prerogative.
- `lmbrain-mcp` exists to encode controlled mutations instead of ad hoc frontmatter edits.

## Decision

LMBrain v3 will support self-improvement only as a controlled proposal loop.

Agents may propose improvements based on project evidence, such as accepted reviews, repeated remediation findings, implementation evidence, diagnostics, or operator feedback. Those proposals can recommend changes to agent profiles, templates, context packs, review focus, or handoff guidance.

No agent may directly apply behavior-affecting changes to an active profile, template, contract, or prompt without the normal Project Lead/operator workflow. The Project Lead may curate and write proposals, but approval and activation remain operator-controlled.

The preferred implementation is to extend the existing agent proposal mechanism so a proposal can target either a new specialist profile or an update to an existing specialist profile/template. A new artifact family should be introduced only if the existing proposal model cannot represent improvement proposals clearly.

## Alternatives considered

### Agents directly edit their own profiles

Rejected. It would make agent behavior mutable without review, weaken the manual activation model, and make profile regressions hard to audit.

### Store lessons only in freeform knowledge pages

Rejected as the primary mechanism. Knowledge pages are useful supporting context, but they do not create an approval gate for behavior-changing profile updates.

### Add a fully separate learning database

Rejected for v3. Markdown artifacts are the product's source of truth, and a separate store would add complexity before the workflow proves it needs one.

### Do nothing

Rejected. Review findings and repeated handoff misses should improve future recommendations instead of becoming one-off memory in chat transcripts.

## Consequences

### Positive

- Agent behavior can improve over time while preserving operator control.
- Improvements remain auditable as Markdown artifacts.
- Review evidence can be turned into reusable profile guidance.
- The model fits the existing LMBrain artifact lifecycle and MCP philosophy.

### Constraints

- The workflow is not fully automatic; it requires proposal and approval steps.
- Agents must distinguish evidence-backed improvements from personal preference or one-off noise.
- UI and diagnostics should make pending improvement proposals visible enough to use.

## Review conditions

Revisit this ADR if LMBrain later introduces trusted autonomous agent orchestration, a different artifact store, or first-class measured evaluation data that justifies a stricter improvement artifact type.
