---
id: SPEC-069
# Note: Quote the title if it contains a colon
title: "Group Reviews by status and foreground actionable review work"
status: ready
kind: feature
priority: medium
area: reviews
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
depends_on: []
dependency_events: []
parking_events: []
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/46]
created: 2026-07-31
updated: 2026-07-31
tags: [reviews, ui, 4.0.0]
activity:
  - date: 2026-07-31
    action: "created"
activity:
  - date: 2026-07-31
    action: "transitioned backlog -> ready"
---
# Group Reviews by status and foreground actionable review work

## Objective

Restructure the Reviews view from one chronological list into accessible, status-based accordion groups that foreground reviews requiring operator action. Preserve the existing review detail navigation, lifecycle evidence, malformed-artifact warnings, and promoted-finding navigation.

## Context

GitHub issue [#46](https://github.com/fathorMB/LMBrain/issues/46) requests a more actionable Reviews information architecture. The current implementation in `src/components/Reviews/ReviewsList.tsx` renders `state.reviews` as one flat list and does not provide a stable ordering or status grouping. The current tests cover lifecycle rendering and legacy uncertainty, but not grouping, ordering, accordion behavior, or keyboard interaction.

The current Review model already provides `status`, `created`, `updated`, `path`, lifecycle metadata, and promoted-finding relationships. No backend or Markdown contract change is required for the requested behavior.

## Scope
### Included

- Group every loaded review by its current status.
- Order groups deterministically with actionable groups first: `changes-requested`, `pending`, and `blocked`; then `accepted`, `superseded`; then any unknown/malformed statuses.
- Expand actionable groups by default and keep accepted/superseded groups collapsed by default.
- Render each group as an accessible accordion control with status label, visual status treatment, and item count.
- Sort reviews within each group newest first using `updated`, falling back to `created`, then a deterministic ID tie-breaker.
- Preserve review detail opening, lifecycle metadata, malformed warnings, and promoted-finding navigation.
- Make review cards operable by keyboard as well as pointer input, including a visible focus state.
- Add focused automated coverage for grouping, ordering, expansion, interaction, empty groups, legacy reviews, and unknown statuses.

### Excluded

- Changes to review lifecycle semantics, statuses, MCP verbs, Markdown contract, or mutation authority.
- Changes to the backend review payload unless implementation evidence proves an existing field is insufficient.
- Unread/read-state badges from issue #47.
- Changes to promoted-finding lifecycle or navigation destinations.
- Broad visual redesign outside the Reviews view.

## Existing-project analysis

- `src/components/Reviews/ReviewsList.tsx` currently maps the flat `state.reviews` array directly into clickable `div` cards.
- `src/types/index.ts` defines the known `ReviewStatus` values and review dates, but malformed/unknown values can still arrive through tolerant parsing and must not break rendering.
- `src/__tests__/ReviewsList.test.tsx` already supplies structured-event and status-only fixtures and should be extended rather than replaced.
- Review cards currently use pointer-only `onClick` behavior. This is an existing accessibility defect relevant to the required accessible accordion/card interaction and must be corrected as part of this implementation.
- The branch contains recently reviewed finding-detail focus work; promoted-finding navigation must remain intact while changing the surrounding list structure.

## Technical proposal

Introduce a small pure grouping/sorting helper, either local to the Reviews component or in a review-specific utility module, so ordering rules are deterministic and directly unit-testable. The helper must:

1. normalize the display bucket without rewriting the source status;
2. retain unknown statuses in a clearly labeled `Unknown status` group rather than silently treating them as `pending`;
3. compare valid ISO-like artifact dates by timestamp/date value;
4. fall back from `updated` to `created`, and finally to the stable review ID;
5. never mutate `state.reviews`.

Render groups with native `button` controls (or equivalent fully tested accessible disclosure semantics) using `aria-expanded` and a unique `aria-controls` target. Actionable groups are initially expanded; user toggles are local UI state and must not mutate artifacts or workspace data. Empty groups should not be rendered unless the chosen accordion pattern requires them; the no-reviews state remains understandable.

Review cards should retain their current detail action but expose an accessible name, keyboard activation (`Enter`/`Space` if not implemented as a native button), and visible focus styling. Nested promoted-finding actions must stop propagation and remain independently operable.

## Files and areas involved

- `src/components/Reviews/ReviewsList.tsx`
- `src/__tests__/ReviewsList.test.tsx`
- Optional review grouping utility and focused unit test, if extraction materially improves testability
- Relevant UI documentation only if the Reviews operator workflow is described there

## Acceptance criteria
- [ ] Reviews are visibly grouped by current status rather than rendered as one undifferentiated list.
- [ ] Each non-empty status group has an accessible expand/collapse control with its status label and item count.
- [ ] `changes-requested`, `pending`, and `blocked` groups appear before history-oriented groups; `changes-requested` is first whenever present.
- [ ] Actionable groups are expanded by default; accepted and superseded groups are collapsed by default.
- [ ] Unknown or malformed statuses render safely in an understandable fallback group and do not get silently relabeled as `pending`.
- [ ] Reviews within each group are sorted newest first by `updated`, falling back to `created`, with deterministic ordering for missing, invalid, or equal dates.
- [ ] Existing review detail navigation continues to open the correct artifact.
- [ ] Lifecycle information, malformed-artifact warnings, and promoted-finding navigation remain visible and functional.
- [ ] Accordion controls and review cards are keyboard operable, have meaningful accessible names, and expose visible focus styling.
- [ ] Empty, legacy status-only, malformed, unknown-status, and mixed-status collections render without exceptions or misleading content.
- [ ] Automated tests cover grouping, group order, item order, default expansion, toggle interaction, keyboard activation, preserved card actions, unknown statuses, and empty state.
- [ ] No lifecycle mutation, artifact write, backend contract change, or new notification/read-state behavior is introduced.

## Implementation plan
1. Extract and test deterministic status-group and date-order rules.
2. Replace the flat review mapping with ordered accordion groups and local expansion state.
3. Preserve existing card content/actions while repairing keyboard semantics and focus styling.
4. Add regression tests for normal, legacy, malformed, unknown-status, empty, and mixed collections.
5. Run the project’s frontend quality gates and perform a manual desktop keyboard smoke test.

## Required verification

- `pnpm test --run` (or the repository-equivalent Vitest command) passes, including `ReviewsList` tests.
- `pnpm lint` passes.
- `pnpm build` passes.
- Manual smoke check in the desktop app: actionable sections are visible/expanded, history sections are collapsed, toggles work with keyboard, cards open the correct detail artifact, and promoted-finding navigation still works.
- Manual check with malformed/unknown status fixture or workspace data confirms safe fallback rendering.
- `git diff --check` reports no whitespace errors.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Review dates may be empty or malformed in legacy artifacts; the fallback chain must be explicit and deterministic rather than relying on JavaScript string ordering accidentally.
- The current status fallback paints unknown statuses as pending. The feature must avoid changing lifecycle semantics while making the display honest; an `Unknown status` presentation is preferred.
- Native disclosure elements are preferred for accessibility, but any custom accordion must meet equivalent keyboard and ARIA behavior and be covered by tests.
- The exact visual treatment may follow existing `statusConfig`; introducing a new design system is not justified by this issue.

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


## Mutation override
Operator explicitly requested implementation of GitHub issue #46 after approving the feature candidate.