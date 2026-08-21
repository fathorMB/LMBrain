---
id: SPEC-050
# Note: Quote the title if it contains a colon
title: "Unify actionable diagnostics and reconcile the project digest"
status: backlog
kind: bugfix
priority: critical
area: core-context-and-diagnostics
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/16]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-16, diagnostics, context-pack]
activity:
  - date: 2026-07-29
    action: "created"
---
# Unify actionable diagnostics and reconcile the project digest

## Objective
Make `lmbrain_project_digest`, core validation, and the desktop app expose the same bounded, actionable project truth, including backlog work and conflicts between declared and derived milestone state.

## Context
GitHub issue #16 is reproduced in this repository: the live digest reports `diagnostics_summary.warnings = 20` while returning `warnings: []`. `lmbrain-core/src/context.rs` counts `scan_diagnostics` results but populates `warnings` only from unresolved recommended agents. It reads current state from `STATUS.md`, omits backlog and working specs, and does not reconcile `ROADMAP.md` membership with spec frontmatter. The Tauri app separately implements `build_diagnostics`, creating rule drift.

## Scope
### Included
- Define one versioned core diagnostic record with stable ID/rule, severity, artifact/path, message, safe next action, fixability, and source.
- Reuse the same rule engine from validation, context packs, Tauri commands, statistics, and UI.
- Parse roadmap milestone declarations and reconcile them with spec milestone metadata and lifecycle.
- Preserve STATUS as declared narrative state while returning an explicit derived state and source/conflict metadata.
- Return deterministic counts for every spec lifecycle status, bounded prioritized item groups, and explicit `total`, `returned`, and `omitted` values.
- Include backlog and working work in JSON and Markdown digest output.
- Version or preserve compatibility fields so MCP consumers do not silently misread the new payload.
- Add drill-down in the app for all digest-reported diagnostics.

### Excluded
- Automatic edits to STATUS, ROADMAP, specs, or diagnostics during reads.
- Automatic conflict resolution or inference from prose.
- Unbounded context payloads.

## Existing-project analysis
Core diagnostics are private to `context.rs`, while `src-tauri/src/commands/contract.rs` owns a second, richer diagnostic builder and frontend `KitDiagnostic` contract. `ProjectDigest` exposes only ready/review lists and plain string warnings/blockers. This duplication must be removed at the rule boundary, not papered over in Markdown formatting.

## Technical proposal
Move the canonical diagnostic and project-state derivation model into `lmbrain-core`. Build the complete collection once, then derive severity counts and bounded groups from it. Add explicit declared/derived milestone records and a lifecycle summary map. Use stable ordering and stable diagnostic IDs. Keep legacy fields for one compatibility window only when their semantics can be defined exactly.

## Files and areas involved
- `lmbrain-core/src/context.rs` and a dedicated diagnostic/reconciliation module
- `lmbrain-core/src/lib.rs`, MCP serialization and contract tests
- `src-tauri/src/commands/contract.rs`, workspace/statistics models
- `src/types/index.ts`, Pulse/Insights diagnostic surfaces
- kit contract/migration/changelog and public architecture/kit documentation

## Acceptance criteria
- [ ] Every nonzero diagnostic total has inspectable records or an explicit bounded subset with correct omitted count.
- [ ] `warnings: []` cannot coexist ambiguously with a nonzero warning total.
- [ ] Validation, MCP digest, app diagnostics, and statistics agree on IDs, severities, totals, and messages.
- [ ] Backlog-only and working-only projects remain visible in the digest.
- [ ] STATUS, ROADMAP, and spec milestone conflicts are shown without silently choosing or mutating a source.
- [ ] Ready, working, review, backlog, done, discarded, and malformed counts are deterministic.
- [ ] Large repositories are bounded, stably ordered, and report truncation.
- [ ] Existing MCP consumers receive a documented compatibility/version signal.

## Implementation plan
1. Freeze the typed diagnostic and derived-state contract with compatibility tests.
2. Extract and consolidate diagnostic rules in core.
3. Add roadmap/spec/status reconciliation and complete lifecycle summaries.
4. Adapt MCP Markdown/JSON and Tauri/frontend consumers.
5. Add fixtures for conflicts, malformed data, warning-only projects, and truncation.

## Required verification
- Core snapshot/contract tests for JSON and Markdown.
- Cross-layer parity tests using the same fixtures.
- `cargo test --workspace`
- `pnpm test`, `pnpm lint`, `pnpm build`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- This is a public MCP payload evolution; compatibility must be decided before the spec becomes `ready`.
- Implement before the later 3.1.0 diagnostics-heavy features so they extend one rule engine instead of recreating parallel rules.

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
