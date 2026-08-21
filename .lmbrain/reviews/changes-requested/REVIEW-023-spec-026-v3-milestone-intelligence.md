---
id: REVIEW-023
title: "Review SPEC-026 v3 milestone intelligence"
status: changes-requested
spec_id: SPEC-026
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, roadmap, milestones]
links: [SPEC-026]
---
# Review SPEC-026 v3 milestone intelligence

## Verdict

Changes requested.

The implementation adds a real milestone overview model, backend joins, a redesigned Roadmap view, documentation, and passing tests. However, it does not yet satisfy the explicit click-through requirement for linked specs/reviews/ADRs.

## Findings

### [P1] Spec click-through builds invalid artifact paths for real spec filenames

`RoadmapView` opens linked specs by constructing a synthetic path from status and ID:

- `src/components/Roadmap/RoadmapView.tsx:319`

That produces paths like `.lmbrain/specs/done/SPEC-023.md`, but LMBrain spec files use ID plus slug, such as `.lmbrain/specs/done/SPEC-023-v3-context-economy.md`. The backend `MilestoneSpecSummary` does not expose the actual parsed spec path:

- `src-tauri/src/models/roadmap.rs:25`
- `src-tauri/src/commands/contract.rs:1055`

As a result, clicking a milestone spec can open a missing artifact instead of the real spec. This breaks the acceptance criterion requiring linked specs to be clickable through existing detail/wiki navigation.

Required remediation:

- Include the real artifact path in `MilestoneSpecSummary` from `Spec.path`.
- Add the field to TypeScript types.
- Use the provided path in `RoadmapView` instead of reconstructing it.
- Add a frontend test asserting `openDetailArtifact` receives the real path with slug.

### [P1] Reviews and decisions are displayed but not clickable

SPEC-026 requires linked specs, reviews, and ADRs to be clickable. The implementation renders reviews and decisions as plain rows without click handlers or real paths:

- `src/components/Roadmap/RoadmapView.tsx:359`
- `src/components/Roadmap/RoadmapView.tsx:381`

The backend summary models also omit actual review/ADR paths:

- `src-tauri/src/models/roadmap.rs:36`
- `src-tauri/src/models/roadmap.rs:45`

This leaves two of the three required linked artifact families non-navigable.

Required remediation:

- Include real artifact paths in `MilestoneReviewSummary` and `MilestoneAdrSummary`.
- Wire review and ADR rows to `openDetailArtifact` or the existing navigation surface, consistent with current artifact detail behavior.
- Add tests proving linked reviews and decisions invoke navigation with real paths.

## Acceptance Criteria Assessment

- Roadmap/milestone data includes derived spec, review, ADR, dependency, risk, and missing-reference state: partial; derived state exists, but path metadata needed for click-through is missing.
- Roadmap view shows milestones with progress and next-action information: pass.
- Milestone detail exposes linked specs with title/status/priority/agent and linked reviews/decisions: partial; visible but not correctly navigable.
- Unresolved milestone references are visible as warnings: pass.
- Existing ROADMAP markdown continues to parse: pass by tests.
- UI readability: pass by code inspection for desktop-width layout.
- Tests cover parsing, derived joins, missing references, frontend rendering: partial; missing click-through/path tests.
- `pnpm lint`, `pnpm test`, and Rust tests pass: pass.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 77 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- `cargo test` - pass.
- Static review of `RoadmapView`, milestone summary models, backend overview builder, and roadmap tests.

## Required Remediation

1. Add real artifact paths to milestone spec/review/ADR summaries.
2. Use those paths for all milestone click-through navigation.
3. Add frontend tests for spec, review, and ADR click-through behavior using slugged filenames.
4. Add or adjust backend tests to verify summary paths are populated.
5. Re-run `pnpm lint`, `pnpm test`, `pnpm build`, and `cargo test`.
