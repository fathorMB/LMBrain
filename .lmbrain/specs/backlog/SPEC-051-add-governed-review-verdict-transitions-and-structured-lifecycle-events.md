---
id: SPEC-051
# Note: Quote the title if it contains a colon
title: "Add governed review verdict transitions and structured lifecycle events"
status: backlog
kind: bugfix
priority: critical
area: review-lifecycle
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/10]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-10, reviews, mcp]
activity:
  - date: 2026-07-29
    action: "created"
---
# Add governed review verdict transitions and structured lifecycle events

## Objective
Allow the Project Lead to record negative and non-final review verdicts through controlled MCP operations while preserving an auditable event history suitable for later cycle metrics and finding promotion.

## Context
GitHub issue #10 is confirmed by the current tool surface. Core already permits `pending -> accepted|changes-requested|blocked` and any review to `superseded`, but `lmbrain-mcp` exposes only `review_accept`; creation correctly permits only `pending`. The missing semantic verbs force metadata/body disagreement. GitHub issue #13 then cannot measure edited-in-place remediation history reliably.

## Scope
### Included
- Add semantic `review_changes_requested`, `review_block`, and `review_supersede` MCP verbs with explicit authority descriptions.
- Keep review creation `pending` only.
- Append typed, uniquely identified lifecycle events for initial submission, each verdict/pass, re-review/remediation, escalation, takeover, block, acceptance, and supersession where applicable.
- Define actor/role, timestamp, prior and resulting state/outcome, reason, linked evidence, implementation/remediation agent, and event identity.
- Make status-directory move, frontmatter/body metadata, lifecycle event, and audit write atomic.
- Preserve compatibility with legacy reviews and with the planned addressed-review closeout in ADR-013/SPEC-037.
- Return all invariant failures precisely and expose the history in review context and app detail.

### Excluded
- Computing effectiveness metrics; that belongs to [[SPEC-052-make-agent-effectiveness-metrics-cycle-aware-and-taxonomy-stable]].
- Implementing first-class findings or SPEC-037 closeout semantics.
- Allowing arbitrary target status or raw frontmatter mutation.

## Existing-project analysis
`lmbrain-core/src/transitions.rs` contains the legal transitions and generic mutation implementation. `lmbrain-mcp/src/main.rs` maps only `review_accept` to `accepted`. Generic activity strings are insufficient for repeated passes, attribution, deduplication, and historical verdict preservation.

## Technical proposal
Introduce a typed review event schema and semantic core functions rather than adding three string aliases to the generic transition dispatcher. Events must be append-only, deterministic, schema-versioned, and checked for duplicate IDs. Read legacy status/activity without inventing missing events; expose uncertainty diagnostics. Design the event model so an eventual `addressed` lifecycle status preserves original outcome.

## Files and areas involved
- `lmbrain-core` review lifecycle, frontmatter preservation, atomic mutation, context and tests
- `lmbrain-mcp/src/main.rs` schemas, dispatch and authority tests
- Tauri/TypeScript review contracts and review detail
- review template, CONTRACT, AGENT/OPERATOR guidance, migration and changelog

## Acceptance criteria
- [ ] A pending review can be recorded as changes-requested or blocked through a semantic MCP verb.
- [ ] Supersession is controlled and records its successor or reason.
- [ ] Every review pass/verdict appends exactly one attributable event and repeated reads do not duplicate it.
- [ ] Status, directory, event history, updated date, and audit evidence change atomically.
- [ ] Invalid source states, missing reasons/evidence, stale concurrent writes, and destination collisions fail without partial mutation.
- [ ] Legacy reviews remain readable and uncertain history is reported rather than fabricated.
- [ ] The model is compatible with addressed reviews and promoted findings without rewriting historical verdicts.
- [ ] MCP discovery, review context, app types, and documentation agree.

## Implementation plan
1. Define and test the versioned event/authority contract.
2. Implement semantic core mutations with locking and failure injection.
3. Expose dedicated MCP verbs and context output.
4. Align Tauri/frontend models and review detail.
5. Add conservative legacy diagnostics and migration guidance.

## Required verification
- Transition matrix, event deduplication, atomic failure, concurrency, authority, and legacy fixtures.
- MCP schema/dispatch tests proving no arbitrary status parameter is accepted.
- Full Rust/frontend gates and packaged sidecar smoke.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Decide before approval whether acceptance remains operator-authorized only or whether the Project Lead’s accepted-review authority is represented separately from operator approval. Do not encode both as the same actor.
- This spec must land before SPEC-052.

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
