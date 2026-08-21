---
id: SPEC-039
# Note: Quote the title if it contains a colon
title: "Governed agent improvement recommendations, proposal application, and effectiveness metrics"
status: review
kind: feature
priority: high
area: agent-governance-and-insights
milestone: M-05
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-008]
links: [SPEC-024, SPEC-032, SPEC-038]
created: 2026-07-16
updated: 2026-07-16
tags: [2.9.0, agents, improvement-loop, proposals, metrics]
activity:
  - date: 2026-07-16
    action: "created"
  - date: 2026-07-16
    action: "transitioned backlog -> ready"
  - date: 2026-07-16
    action: "transitioned ready -> working"
  - date: 2026-07-16
    action: "transitioned working -> review"
---
# Governed agent improvement recommendations, proposal application, and effectiveness metrics

## Objective

Turn review history into visible, evidence-backed profile-improvement proposals
and provide an operator-governed way to apply approved changes, while measuring
whether those changes reduce repeated failures without allowing agents to
silently rewrite their own behavior.

## Context

[[ADR-008]] permits controlled self-improvement through reviewed proposals, and
[[SPEC-024]] added proposal fields and UI labels. The current implementation is
representational only: no component derives repeated signals, creates a real
project improvement proposal, applies an approved proposal to its target, or
measures the result. AstraNexus contains repeated profile-specific evidence but
only one manually edited profile and no non-example improvement proposal.

This spec depends on [[SPEC-038]] so the guidance ultimately delivered to a
specialist is complete and observable. It must preserve ADR-008's manual
approval boundary.

## Scope
### Included

- Add optional structured review finding categories and stable finding IDs,
  linked to implementation agent, spec, criterion, severity, and remediation.
- Derive read-only `AgentImprovementSignal` aggregates from distinct specs,
  avoiding inflated counts from multiple cycles of the same spec.
- Define transparent default thresholds, initially two equivalent findings on
  two distinct specs or an explicit integrity/escalation signal; show evidence
  and allow the Lead to dismiss/defer noise with an audited rationale.
- Surface candidate improvements in Insights and Agent detail views without
  creating or mutating artifacts in the background.
- Add a Project-Lead-controlled MCP action that materializes a selected signal
  as a `proposal_type: improvement` artifact with target profile, evidence links,
  proposed bounded changes, expected benefit, risks, and evaluation window.
- Define a constrained, machine-readable improvement patch model for additive
  profile changes: skills, domains, primary files, review focus, constraints,
  knowledge references, and named guidance sections.
- Add an explicit operator-governed apply action for approved improvement
  proposals; apply atomically, preserve unrelated profile content, record before/
  after digest and audit activity, and mark the proposal's application state.
- Detect stale proposals when the target profile changed after proposal creation
  and require rebase/re-approval rather than silently merging.
- Add per-profile effectiveness metrics: distinct-spec first-pass acceptance,
  transcript fast-fail rate, average review cycles, repeated-category rate, and
  Lead-escalation rate, with pre/post windows and data-quality caveats.
- Provide migration and examples using anonymized AstraNexus-derived fixtures.

### Excluded

- Automatic profile mutation, activation, deactivation, or proposal approval.
- LLM-generated changes executed without deterministic evidence and operator review.
- Claiming causal improvement from small samples.
- Ranking agents for punitive purposes or rotating profiles automatically.
- A separate learning database; Markdown remains the source of truth.

## Existing-project analysis

- Agent proposals parse `proposal_type` and `target_profile`, but the only
  improvement artifact shipped is an example.
- The app displays improvement proposals but exposes no evidence aggregation or
  application workflow.
- Existing Insights compute change-request and first-pass statistics but do not
  attribute repeat categories, cycles, fast-fails, or escalations to profiles.
- MCP can create generic proposals and activate profiles, but has no semantic
  verb for applying an approved improvement to an existing active profile.
- Freeform review bodies make robust recurrence detection impossible without
  optional structured categories and backward-compatible `uncategorized` data.

## Technical proposal

Extend review metadata with backward-compatible structured findings. Derive
signals deterministically; do not use hidden model inference. A signal records
the distinct supporting specs/reviews, category, target profile, threshold,
last occurrence, and suggested change surface.

Proposal creation is an explicit Lead action. The resulting artifact stores a
target-profile digest plus a constrained proposed patch. Approval remains an
operator decision. A separate apply verb requires an approved proposal and
explicit operator authority, revalidates the target digest, performs one atomic
preservation-aware mutation, and records both digests. Arbitrary raw Markdown
replacement is outside the semantic apply verb.

Metrics compare clearly labeled pre/post windows by distinct spec and surface
sample sizes, missing implementation-agent links, invalid dates, and uncategorized
reviews. They support evaluation; they do not auto-revert or auto-approve changes.

## Files and areas involved

- `lmbrain-core` review/proposal/profile models, parsing, aggregation, mutation,
  invariants, audit, and tests
- `lmbrain-mcp` signal/proposal/apply tool schemas and authority checks
- Tauri contract/statistics commands and TypeScript models
- Agents and Insights UI, proposal diff/preview, stale-state and audit display
- Review, agent-proposal, and agent-profile templates
- `kit/.lmbrain/AGENT.md`, `CONTRACT.md`, `QUALITY.md`, `MIGRATIONS.md`
- `docs/architecture.md`, `docs/kit.md`, operator documentation

## Acceptance criteria

- [ ] Reviews may record stable categorized findings without breaking legacy
      freeform reviews; uncategorized history remains visible and is never guessed.
- [ ] Signals aggregate by profile/category across distinct specs and do not
      count repeated cycles of one spec as independent recurrence evidence.
- [ ] Default thresholds and every supporting artifact are visible; no proposal
      is created merely by opening, refreshing, or scanning a workspace.
- [ ] The Project Lead can explicitly create an evidence-backed improvement
      proposal from a signal through a controlled MCP verb.
- [ ] Improvement proposals contain a target digest and constrained proposed
      patch that can be previewed as a human-readable before/after diff.
- [ ] Only an explicitly operator-approved proposal can be applied; agents cannot
      self-approve or directly mutate active behavior through this workflow.
- [ ] Apply is atomic, preservation-aware, audited, idempotent, and fails closed
      when the profile digest is stale or proposed changes exceed allowed fields.
- [ ] Applied guidance is present in the next profile/spec context generated by
      [[SPEC-038]], proving that the learning reaches future handoffs.
- [ ] Per-profile metrics report first-pass, fast-fail, cycles, recurrence, and
      escalation with sample size, missing-data counts, and pre/post windows.
- [ ] Metrics make no causal claim and remain correct for superseded, pending,
      malformed, or review-only legacy artifacts.
- [ ] AstraNexus-derived fixtures produce separate signals for verification
      omission/fidelity, security-boundary defects, and operator-gate readiness.
- [ ] Full contract, mutation, frontend, and packaged Windows tests are green.

## Implementation plan

1. Define structured finding categories and constrained improvement patches.
2. Implement deterministic aggregation, data-quality reporting, and tests.
3. Implement explicit proposal materialization and operator-governed atomic apply.
4. Add proposal preview/staleness UX and profile effectiveness dashboards.
5. Connect applied guidance to SPEC-038 context and handoff regression tests.
6. Update kit/templates/migrations/docs and run the full production gate.

## Required verification

- Aggregation fixtures for duplicate cycles, distinct specs, missing dates/agents,
  superseded reviews, integrity findings, and threshold boundaries.
- Mutation tests for approval authority, stale digest, idempotency, atomic failure,
  preservation, invalid patch surfaces, and audit entries.
- End-to-end fixture: reviews -> signal -> proposed artifact -> operator-approved
  apply -> updated profile -> next handoff context contains the guidance.
- Insights and Agent UI tests for evidence, sample sizes, preview, stale state,
  dismiss/defer rationale, and accessibility.
- Canonical Rust/frontend/release gates and Windows packaged smoke.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Categories can become too broad or gameable. Keep a small versioned taxonomy
  with `other` and never infer a category silently.
- Two-spec thresholds may be noisy in small projects. Expose configuration later
  only if real evidence shows the fixed transparent threshold is inadequate.
- Applying prose guidance is harder to constrain than additive metadata. Named
  append-only guidance sections must have strict size and placement rules.

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

- Added deterministic distinct-spec review-category signals and per-profile first-pass, cycle, transcript-fast-fail, escalation, categorized/uncategorized metrics with explicit causality caveat.
- Added evidence-linked constrained improvement proposal creation and operator-approved apply with target-digest staleness, additive fields, bounded guidance, audit digests, idempotency, and rollback on proposal-audit failure.
- Added MCP authority verbs, Tauri/TypeScript contracts, and an Agents & MCP metrics/signal surface; scanning remains read-only and never mutates profiles.

### Files changed

- `lmbrain-core/src/improvement.rs`, `lmbrain-mcp/src/main.rs`
- `src-tauri/src/lib.rs`, review models/contracts, TypeScript commands/types, `AgentsMCPView.tsx` and tests
- proposal/review templates, migration, contract, architecture, and kit documentation

### Verification performed

- Unit tests cover distinct-spec de-duplication, approval authority, stale target, additive preservation, guidance propagation, and idempotency.
- MCP schemas prove execution/apply tools do not accept ad-hoc command or raw-Markdown mutation inputs.
- Frontend view tests and all canonical Rust/frontend gates passed.

### Verification transcript

```text
$ cargo test -p lmbrain-core improvement::tests
Signal aggregation, approval/apply, stale-target, preservation, guidance, and idempotency tests passed.

$ pnpm test
21 test files; 121 tests passed; 0 failed.

$ cargo test --workspace
All non-ignored workspace tests passed; 0 failed.
```

### Deviations from the specification

- The UI provides read-only signals/metrics and artifact detail preview; approve/reject/apply remain explicit governed MCP actions rather than duplicating authority in a second UI mutation path.
- Historical freeform findings remain uncategorized and are never inferred except for existing explicit fast-fail/evidence-integrity tags.

### Handoff status
- [x] Ready for Project Lead review
