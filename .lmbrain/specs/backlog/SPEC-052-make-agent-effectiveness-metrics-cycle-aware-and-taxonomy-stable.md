---
id: SPEC-052
# Note: Quote the title if it contains a colon
title: "Make agent effectiveness metrics cycle-aware and taxonomy-stable"
status: backlog
kind: bugfix
priority: high
area: agent-governance-and-insights
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/13]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-13, metrics, taxonomy]
activity:
  - date: 2026-07-29
    action: "created"
---
# Make agent effectiveness metrics cycle-aware and taxonomy-stable

## Objective
Derive trustworthy agent-effectiveness signals from structured review history and a versioned category taxonomy, with explicit attribution, coverage, confidence, and uncertainty.

## Context
GitHub issue #13 is confirmed in `lmbrain-core/src/improvement.rs`: `review_cycles` increments once per review file, first-pass is inferred from the first current-status artifact, escalation recognizes only two narrow markers, and categories group by exact raw text. Edited-in-place reviews can therefore report perfect first-pass performance after multiple remediation cycles.

## Scope
### Included
- Consume the structured lifecycle events introduced by [[SPEC-051-add-governed-review-verdict-transitions-and-structured-lifecycle-events]].
- Define a small versioned canonical finding-category registry with documented legacy aliases.
- Preserve raw category values for audit, warn on unknown values, and normalize only known aliases.
- Calculate passes, remediation cycles, initial/final verdict, escalation, takeover, and attribution by original implementer, remediation agent, and takeover owner.
- Expose sample size, structured-history coverage, categorized-data coverage, confidence/uncertainty, and contradictory data diagnostics.
- Add deterministic legacy reading that prefers valid structured data, recognizes explicit legacy fields/activity, and never treats uncertain history as first-pass success.
- Align core, MCP, Tauri, TypeScript, Agents/Insights UI, templates, and migration preview.

### Excluded
- Automatic profile mutation or proposal approval.
- Treating promoted first-class findings as agent failures without independent review evidence.
- Guessing categories from arbitrary prose.

## Existing-project analysis
The existing aggregation de-duplicates recurrence by distinct spec but does not preserve cycles inside a single review artifact. Tauri also calculates review-quality statistics independently, so the corrected history/taxonomy engine must be shared rather than patched only in the Agents view.

## Technical proposal
Build a canonical per-spec review timeline from event identity and review linkage, validate summary fields against that timeline, then derive separately labeled metrics for each attribution mode. Normalize categories at the core boundary using a versioned registry. Return data-quality diagnostics through SPEC-050’s common diagnostic type.

## Files and areas involved
- `lmbrain-core/src/improvement.rs` and shared review/taxonomy modules
- MCP signal payloads and proposal evidence
- Tauri statistics, TypeScript models, Agents and Insights UI
- review/template/contract/migration documentation

## Acceptance criteria
- [ ] Multi-pass accepted reviews are not counted as first-pass successes.
- [ ] Escalations and takeovers are counted from structured events with explicit owner/attribution.
- [ ] Equivalent known aliases aggregate under one canonical category while raw values remain inspectable.
- [ ] Unknown, missing, or contradictory data lowers coverage/confidence and emits actionable diagnostics.
- [ ] Metrics never mix original implementer, remediation agent, and takeover-owner attribution without labels.
- [ ] Superseded/addressed reviews and repeated reads do not double count events.
- [ ] XenoMark-shaped fixtures reproduce the pass/escalation examples from issue #13.
- [ ] Existing proposal thresholds consume canonical categories and remain operator-governed.

## Implementation plan
1. Add taxonomy registry and review-timeline builder.
2. Replace file-count metrics with event-derived metrics.
3. Add compatibility/uncertainty reporting and migration preview.
4. Align MCP/app consumers and tests.

## Required verification
- Fixtures for one pass, repeated remediation, escalation, takeover, supersession, contradictory legacy data, aliases, unknown categories, and partial metadata.
- Determinism/idempotence and cross-layer parity tests.
- Full Rust/frontend gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Depends on SPEC-051 and the diagnostic contract in SPEC-050.
- Do not publish a numeric “confidence score” without a documented formula; categorical confidence plus coverage may be less misleading.

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
