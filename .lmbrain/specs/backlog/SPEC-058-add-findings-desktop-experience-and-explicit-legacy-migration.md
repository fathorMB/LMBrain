---
id: SPEC-058
# Note: Quote the title if it contains a colon
title: "Add Findings desktop experience and explicit legacy migration"
status: backlog
kind: feature
priority: high
area: findings-desktop-and-migration
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/12]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-12, findings, desktop, migration]
activity:
  - date: 2026-07-29
    action: "created"
---
# Add Findings desktop experience and explicit legacy migration

## Objective
Expose first-class findings as a dedicated, accessible, read-only-by-default workspace experience and provide explicit, reviewable migration/promotion workflows.

## Context
This is the desktop and migration tranche of GitHub issue #12. It depends on the approved finding contract and core lifecycle in [[SPEC-057-define-first-class-cross-spec-finding-architecture-and-core-lifecycle]].

## Scope
### Included
- Add Rust/TypeScript models, workspace loading/refresh, commands, statistics, and malformed/error states.
- Add a Findings sidebar and command-palette destination with active/history views.
- Add status/severity/category/area/milestone/owner/target/source filters and stable sorting.
- Show identity, severity, status, age, origin, owner, targets, blockers, next action, and non-color-only indicators.
- Add detail relationship groups and audit-derived resolution timeline through the safe artifact-detail path.
- Add contextual active-finding counts/links in Board, Spec, Review, Pulse, and Roadmap without displacing primary workflows.
- Add read-only candidate inventory and explicit operator-reviewed promotion/migration preview with Git diff and idempotence.
- Preserve old-version rollback by retaining Markdown evidence.

### Excluded
- Ungoverned lifecycle buttons or mutation during load/refresh.
- Full graph visualization.
- Auto-conversion of legacy review findings.

## Existing-project analysis
WorkspaceContext currently loads a fixed set of artifact collections and AppView/sidebar/command palette enumerate routes explicitly. Review findings are not parsed into application data. Every layer must fail visibly on malformed canonical fields rather than silently defaulting.

The XenoMark migration fixture must show four visibly different dispositions:

- planned debt from `REVIEW-054/FINDING-07` targeting `SPEC-059`, with original
  blocking severity and current medium severity both visible;
- a resolved documented limitation from
  `REVIEW-049/FINDING-050-001`, still linked to a later verification constraint;
- two separate targetless design observations retained from BACKLOG, presented
  as “needs triage” rather than scheduled work;
- unresolved `before-done` gates shown by verification diagnostics, not
  duplicated into the Findings list.

Candidate inventory must also demonstrate repeated local identifiers:
`FINDING-01` occurs in ten XenoMark reviews.

## Technical proposal
Reuse the core finding index and context results; do not rebuild joins in React. Keep initial lifecycle actions as copyable governed prompts unless the operator explicitly approves direct app mutations. Migration is preview-first, selected by stable origin pair, additive, atomic, and idempotent.

## Files and areas involved
- Tauri finding loader/models/commands and workspace state
- TypeScript contracts, WorkspaceContext, sidebar, command palette
- new Findings components plus Board/Spec/Review/Pulse/Roadmap integration
- migration preview/apply and public documentation

## Acceptance criteria
- [ ] Findings is a dedicated route with loading, empty, malformed, and error states.
- [ ] Active/history, filtering, sorting, keyboard navigation, accessible labels, and non-color indicators work.
- [ ] Detail exposes only canonical clickable relationships and timeline evidence.
- [ ] Contextual surfaces show relevant active counts with correct click-through.
- [ ] Viewing/refreshing never mutates repository content.
- [ ] Migration preview handles duplicate local IDs, already-promoted items, malformed links, and repeat runs.
- [ ] Migration preview never treats an unchecked verification gate or historical “open” heading as an active finding without explicit selection.
- [ ] Detail distinguishes origin severity from current project severity and explains why an accepted review may have an active promoted finding.
- [ ] Rollback preserves all finding Markdown.
- [ ] Core/Tauri/TypeScript models remain aligned through contract tests.

## Implementation plan
1. Align models/loaders and add route/state/error handling.
2. Build list/filter/detail accessibility surface.
3. Add contextual counts and relations.
4. Add preview-first promotion/migration flow and fixtures.
5. Run packaged complex-project smoke and update docs.

## Required verification
- Frontend filters, keyboard/accessibility, empty/error/malformed, relations, counts, and refresh tests.
- Cross-layer contract and migration idempotence/duplicate-ID fixtures.
- Full Rust/frontend gates and packaged Windows smoke on a migrated complex project.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Decide before approval whether lifecycle actions remain prompt-only or selected operator mutations are implemented in-app.
- Depends on SPEC-057, SPEC-050, and SPEC-051.
- Issue #12 is release-blocking for 3.1.0; this spec is not an optional follow-up
  to the core domain.

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
