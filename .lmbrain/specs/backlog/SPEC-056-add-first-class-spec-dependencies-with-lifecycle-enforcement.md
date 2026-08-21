---
id: SPEC-056
# Note: Quote the title if it contains a colon
title: "Add first-class spec dependencies with lifecycle enforcement"
status: backlog
kind: feature
priority: high
area: spec-dependency-graph
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/18]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-18, dependencies, lifecycle]
activity:
  - date: 2026-07-29
    action: "created"
---
# Add first-class spec dependencies with lifecycle enforcement

## Objective
Represent hard prerequisites as a validated acyclic spec graph and prevent downstream readiness/start while prerequisites remain incomplete.

## Context
GitHub issue #18 is confirmed: milestones have `depends_on`, but spec models/templates/transitions do not. Hard chains written in prose are invisible to validation, MCP, Board, and agents.

## Scope
### Included
- Add typed `depends_on: [SPEC-*]` metadata with no duplicates, self-reference, missing reference, or cycles.
- Build one deterministic dependency graph with direct prerequisite/dependent joins and bounded transitive paths.
- Enforce all direct hard prerequisites as `done` for normal `spec_ready` and `spec_start`.
- Treat missing, rejected/discarded, malformed, and cyclic prerequisites as blocking planning states with precise diagnostics.
- Add a controlled dependency-set mutation with optimistic concurrency, audit, and lifecycle restrictions.
- Expose dependency context in spec/project packs and blocked/ready-after states in Board/spec detail.
- Keep legacy prose advisory; a read-only candidate detector may suggest but never enforce inferred edges.
- Make any forced readiness/start record exact unsatisfied dependencies and remain diagnostic-visible.

### Excluded
- Soft/advisory dependency kinds in v1.
- Inferring hard edges from prose or wikilinks.
- Cross-artifact dependency semantics for findings, ADRs, or milestones.

## Existing-project analysis
Spec frontmatter is parsed independently across core and Tauri; readiness only validates recommended agent, and start uses only the state matrix. The graph must live in core and be consumed across layers.

## Technical proposal
Introduce a reusable core DAG index keyed by canonical spec ID with stable cycle reporting and bounded traversal. Validate on reads and governed writes. Dependency edits after approval must fail normally or require an explicit planning operation that invalidates stale authorization/context; do not allow casual generic field updates.

## Files and areas involved
- core spec parser, dependency graph, transition invariants, diagnostics/context
- MCP create/set/read schemas
- Tauri/TypeScript spec model, Board/spec detail
- template/contract/migration/docs

## Acceptance criteria
- [ ] `depends_on` round-trips through governed create/read/update paths.
- [ ] Missing, duplicate, self, discarded/rejected, and cyclic dependencies produce stable actionable diagnostics.
- [ ] A non-done prerequisite blocks normal readiness and start with every direct blocker listed.
- [ ] Completing prerequisites makes the dependent eligible without editing it.
- [ ] Forced transitions record blockers and remain visible.
- [ ] Large graphs are deterministic and bounded.
- [ ] Legacy prose remains non-binding and is never silently promoted.
- [ ] Board/context/app and MCP share the same graph state.

## Implementation plan
1. Freeze dependency field and lifecycle mutation rules.
2. Implement core DAG/index/diagnostics and transition enforcement.
3. Add governed MCP mutation and context payloads.
4. Add app visualization/filtering and migration suggestions.

## Required verification
- Parser/schema, DAG/cycle, missing/discarded, lifecycle/force, concurrency/staleness, bounded graph, MCP and keyboard UI tests.
- Four-node XenoMark-shaped chain fixture.
- Full Rust/frontend gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Decide whether dependency changes are allowed only in backlog or whether parking a ready spec via SPEC-055 is the required path before mutation.
- Depends on SPEC-050 diagnostics; coordinate lifecycle semantics with SPEC-055.

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
