---
id: SPEC-032
# Note: Quote the title if it contains a colon
title: "Make review history and insight reliability actionable"
status: review
kind: feature
priority: medium
area: insights
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-10
updated: 2026-07-10
tags: [insights, reviews, diagnostics, ux]
activity:
  - date: 2026-07-10
    action: "created"
activity:
  - date: 2026-07-10
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-10
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-10
    action: "transitioned working -> review"
---
# Make review history and insight reliability actionable

## Objective

Replace two ambiguous Insights panels with a full-width reliability view that discloses whether incomplete artifact metadata can reduce confidence in the displayed metrics and exposes the underlying workspace diagnostics on demand.

## Context

The current `Review Trend` panel has no legend or numerical outcome breakdown and renders a nearly empty chart when only one dated period exists. `Diagnostics By Area` groups diagnostics by the first path segment, so root files such as `ROADMAP.md` are presented as if they were areas; it also duplicates Project Pulse without message, severity, or remediation affordance.

## Scope
### Included

- Remove the temporal review panel; review outcome metrics already exist in the higher-value Review Quality section.
- Replace the path-family diagnostic summary with a full-width insight-reliability panel based on already-derived missing-link, missing-date, error, and warning counts.
- Add an accessible expandable detail section for the workspace diagnostics already loaded by `WorkspaceContext`.
- Show each diagnostic's severity, complete message, and path when present.
- Provide a per-diagnostic button that copies the same corrective prompt generated in Project Pulse.
- Direct users to Project Pulse for diagnostic remediation.
- Add focused frontend regression coverage and update relevant product documentation.

### Excluded

- New backend statistics or schema changes.
- Diagnostic detail, file navigation, or remediation controls inside Insights.
- Changes to Project Pulse diagnostics.

## Existing-project analysis

- `ReviewQualityStats` already exposes reviews without spec references and reviews without valid dates.
- `DiagnosticStats` already exposes aggregate error and warning counts.
- `WorkspaceContext.state.diagnostics` already exposes each diagnostic's severity, message, and optional path.
- Project Pulse remains the actionable diagnostic surface; duplicating its raw path grouping in Insights is misleading.

## Technical proposal

Use one full-width reliability component combining aggregate metric-integrity checks with a native accessible disclosure for diagnostic detail. Treat missing spec references, missing dates, and diagnostic errors as issues; warnings remain visible as cautions. When no reliability issues exist, show a positive state. Preserve the existing statistics command, backend models, and diagnostic remediation ownership in Pulse.

## Files and areas involved

- `src/components/Insights/InsightsView.tsx`
- `src/components/Pulse/ProjectPulse.tsx`
- `src/__tests__/InsightsView.test.tsx`
- `src/lib/diagnosticPrompt.ts`
- `docs/product.md`
- `.lmbrain/CHANGELOG.md`
- `.lmbrain/STATUS.md`

## Acceptance criteria
- [x] The temporal review panel is removed without removing the existing Review Quality metrics.
- [x] The reliability panel reports reviews without spec links, reviews without valid dates, diagnostic errors, and diagnostic warnings.
- [x] Reliability copy explains the effect on metrics and directs diagnostic remediation to Project Pulse.
- [x] Insight Reliability spans the available content width.
- [x] Workspace diagnostics can be expanded and collapsed using an accessible disclosure control.
- [x] Expanded diagnostics show severity, complete message, and path when available.
- [x] Each diagnostic can copy the same fix prompt used by Project Pulse and reports clipboard success or failure.
- [x] A zero-diagnostic state remains clear and does not render an empty disclosure.
- [x] The obsolete raw `Diagnostics By Area` path grouping is no longer displayed.
- [x] Focused frontend tests cover expansion, diagnostic content, and the zero-diagnostic state.
- [x] Frontend compilation and relevant automated tests pass after the operator-requested revision.
- [x] Documentation describes the revised Insights behavior.

## Implementation plan
1. Remove the temporal review-history presentation.
2. Expand the reliability presentation and add diagnostic disclosure using workspace context.
3. Replace temporal tests with diagnostic detail and empty-state coverage; update documentation.
4. Run focused tests, frontend build, and diff checks without starting or stopping the application.

## Required verification

- `pnpm test -- --run src/__tests__/InsightsView.test.tsx`
- `pnpm build`
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Workspace diagnostics and aggregate statistics are loaded through separate established commands; brief refresh skew is possible but resolves on the next workspace data reload.
- No quality-policy exception is approved or required.

## Escalated implementation authority

The operator explicitly directed the Project Lead to proceed with this bounded pre-release UI correction on 2026-07-10, continuing the active escalation. The change does not alter architecture, security boundaries, external integrations, or backend contracts. Verification is limited to focused tests and compilation; the running application will not be started or stopped.

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

- Removed the low-value temporal review panel while preserving the existing Review Quality KPIs and breakdowns.
- Expanded `Insight Reliability` to the full content width with four clear aggregate checks for missing spec links, missing dates, diagnostic errors, and warnings.
- Added a native accessible disclosure containing workspace diagnostic severity, full message, and optional file path, ordered by severity.
- Extracted the Pulse diagnostic prompt builder into a shared helper and added per-diagnostic copy buttons with success/error feedback.
- Added a clear no-diagnostics state and retained Project Pulse as the remediation surface.
- Replaced temporal tests with focused coverage for collapsed/expanded diagnostic detail and the no-diagnostics state.

### Files changed

- `src/components/Insights/InsightsView.tsx`
- `src/components/Pulse/ProjectPulse.tsx`
- `src/__tests__/InsightsView.test.tsx`
- `src/lib/diagnosticPrompt.ts`
- `docs/product.md`
- `.lmbrain/CHANGELOG.md`
- `.lmbrain/STATUS.md`
- `.lmbrain/specs/review/SPEC-032-make-review-history-and-insight-reliability-actionable.md`

### Verification performed

- `pnpm test -- --run src/__tests__/InsightsView.test.tsx src/__tests__/ProjectPulse.test.tsx` - passed, 6 tests across both shared-prompt consumers.
- `pnpm build` - passed; the existing large-chunk warning remains (main bundle approximately 812 kB).
- `pnpm lint` - passed.
- `git diff --check` - passed.
- Full release gate: `pnpm test` - passed, 18 files / 107 tests after adding a stable accessible name to the Sessions `New session` button exposed by the suite.
- `cargo check --workspace --tests` - passed at application version `2.6.0`.
- `node scripts/check-version.mjs` - passed; package, Tauri crate, and bundled kit aligned at `2.6.0`.
- No application start or stop command was run.
- Operator visually verified the first revision, then explicitly requested removal of the temporal review view and expansion of Reliability with diagnostic detail. The running dev instance received the final frontend change through hot reload; final visual confirmation remains with the operator.

### Deviations from the specification

- Operator feedback replaced the initially approved temporal review summary with a full-width reliability/detail surface. This remains frontend-only and does not change backend contracts, dependencies, or diagnostic behavior.
- Release preparation added a shared minor version bump and explicit no-rewrite kit migration guidance; it does not alter SPEC-032 behavior.

### Handoff status
- [x] Ready for Project Lead review after operator-requested remediation
