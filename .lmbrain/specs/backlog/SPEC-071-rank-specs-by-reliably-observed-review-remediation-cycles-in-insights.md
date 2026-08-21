---
id: SPEC-071
# Note: Quote the title if it contains a colon
title: "Rank specs by reliably observed review remediation cycles in Insights"
status: backlog
kind: feature
priority: medium
area: insights-review-reliability
milestone: M-09
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
depends_on: []
dependency_events: []
parking_events: []
related_tasks: []
related_decisions: []
links: []
created: 2026-08-10
updated: 2026-08-10
tags: [review-cycles, insights, reliability, desktop]
activity:
  - date: 2026-08-10
    action: "created"
---
# Rank specs by reliably observed review remediation cycles in Insights

## Objective

Replace the Insights panels **Changes Requested By Area** and **Changes
Requested By Agent** with a useful, honest ranking of the specifications that
required the most review remediation cycles.

## Context

This is the planned implementation handoff for GitHub issue #105. The desired
metric is available, but only for review histories that contain authoritative
structured lifecycle events or internally consistent explicit legacy counters.
Status-only review files cannot establish whether a spec really passed on its
first review, so treating them as zero would produce a misleading ranking.

BaseballBoss validates the distinction: 18 of its 49 reviewed specs have
structured lifecycle history. `SPEC-002` and `SPEC-003` each have three
observed changes-requested verdict cycles; `SPEC-001`, `SPEC-004`, `SPEC-008`,
`SPEC-011`, and `SPEC-047` each have two. The remaining status-only histories
must be reported as outside coverage, not ranked as low-friction work.

## Scope
### Included

- Extend the shared review-quality statistics contract with a deterministic
  spec-level remediation-cycle ranking.
- Aggregate all reviews for each linked spec while preserving the existing
  `analyze_review_lifecycle` source and confidence rules.
- Return spec identity, title/path/status, review count, review passes,
  observed remediation-cycle count, history source, confidence, and lifecycle
  warnings needed to explain the row.
- Replace exactly the two requested dimension panels in `InsightsView` with one
  accessible ranking panel.
- Display coverage as ranked specs over reviewed specs, an explicit confidence
  legend, source-aware indicators, deterministic tie-breaking, and an empty
  state when no reliable lifecycle data exists.
- Make rows keyboard-accessible and navigate to the existing spec detail view.

### Excluded

- Inferring cycles from review prose, timestamps, terminal transcripts, current
  status, review-file count, or any unstructured historical field.
- Displaying status-only reviews as zero remediation cycles.
- Changing review lifecycle storage, mutation authority, the existing Review
  Quality summary, or the Insight Reliability panel.
- Reintroducing agent/area ranking elsewhere in this feature.

## Existing-project analysis

- `lmbrain-core/src/review.rs` already provides `ReviewLifecycleAnalysis`.
  Structured `review_events` are authoritative; it counts only `verdict` events
  whose target status is `changes-requested`, and detects contradictions against
  legacy counters. Explicit legacy `review_cycles`/`remediation_cycles` are a
  lower-confidence fallback. A status-only review has low confidence and an
  assumed one pass/zero remediations only for aggregate compatibility, not as
  evidence of a first-pass outcome.
- `src-tauri/src/commands/contract.rs` already groups reviews by `spec` to
  calculate area/agent change-request rates. The same grouping is the correct
  place to derive a spec ranking; React must not recompute lifecycle semantics.
- `ReviewQualityStats` currently exposes aggregate counts plus `by_area` and
  `by_agent`. `InsightsView.tsx` renders those two panels independently. This
  feature replaces those visual consumers with a new ranked contract while
  preserving existing aggregate fields for compatibility.
- BaseballBoss has multi-review specs (including `SPEC-047`), so the aggregation
  must sum trustworthy cycles across linked review artifacts with a stable
  review-ID/spec-ID tie break. It must never count remediation or verification
  events as a fresh changes-requested verdict.

## Technical proposal

Add a `ReviewCycleRankingEntry` to the shared statistics model and a bounded,
deterministically sorted `review_cycle_ranking` collection plus coverage counts
to `ReviewQualityStats`. Include an entry only when every contributing review
has a usable structured or internally consistent explicit lifecycle analysis;
if a spec mixes sources, expose the least confidence and all warnings rather
than upgrading the row silently. Status-only-only specs remain outside the
ranking denominator.

Sort by remediation cycles descending, then review passes descending, then spec
ID ascending. The UI presents a table/list with rank, clickable spec identity,
title, observed cycles, review passes, review count, confidence/source, and
warnings where applicable. It must use labels and text, not colour alone.

## Files and areas involved

- `lmbrain-core/src/review.rs` lifecycle tests and public source/confidence
  semantics where a narrow helper is needed.
- `src-tauri/src/models/statistics.rs`, contract statistics builder, snapshot
  model, Rust contract tests, and fixtures.
- `src/types/index.ts`, workspace state/snapshot contracts, and frontend tests.
- `src/components/Insights/InsightsView.tsx`, replacing the two dimension
  panels with the ranking, empty/coverage/legend/click-through states.
- `docs/architecture.md`, product documentation, changelog, and any affected
  insight reliability explanation.

## Acceptance criteria
- [ ] The ranking counts only observed `verdict -> changes-requested` cycles;
  remediation, remediation-verification, escalation, takeover, and review-file
  counts do not increase it.
- [ ] Multiple review artifacts for a spec aggregate deterministically without
  double counting an event, and ties have a stable documented order.
- [ ] Structured, consistent legacy, mixed, contradictory, and status-only
  lifecycle fixtures yield the documented source/confidence/inclusion result.
- [ ] Status-only specs are excluded from ranking values and visibly included in
  the coverage limitation; no row communicates a guessed zero-cycle outcome.
- [ ] The two requested area/agent panels are absent and replaced by one
  accessible ranking with coverage, legend, empty/error/loading states,
  non-colour indicators, and spec-detail navigation.
- [ ] Existing aggregate review-quality fields and Insight Reliability remain
  backward compatible and cross-layer contracts stay aligned.

## Implementation plan
1. Add and test source-aware aggregate ranking semantics at the core/Tauri
   boundary with BaseballBoss-shaped fixtures.
2. Extend Rust and TypeScript statistics contracts and snapshot refresh tests.
3. Replace the two Insights panels with the ranking and test interaction,
   accessibility, coverage, and edge states.
4. Run full quality gates, inspect the packaged desktop against a mixed-history
   workspace, and update documentation.

## Required verification

- [ ] CYCLES-CORE | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Core/Tauri tests prove structured event counting, legacy fallback, contradictory data, status-only exclusion, multi-review aggregation, and stable sort.
- [ ] CYCLES-CONTRACT | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Rust snapshot and TypeScript contracts preserve existing statistics while exposing the ranking and coverage.
- [ ] CYCLES-UI | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Frontend tests prove both retired panels are absent and ranking navigation, empty/loading/error, confidence and accessibility states work.
- [ ] CYCLES-SMOKE | kind=operator | owner=operator | phase=before-done | evidence=observation | Packaged desktop smoke on BaseballBoss or an equivalent mixed-history workspace confirms the displayed ranking and coverage explain themselves.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

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
