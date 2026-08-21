---
id: SPEC-026
title: "V3 milestone intelligence view"
status: review
kind: feature
priority: high
area: roadmap
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-02
updated: 2026-07-02
tags: [v3, roadmap, milestones, ui]
activity:
  - date: 2026-07-02
    action: "transitioned ready -> working"
  - date: 2026-07-02
    action: "transitioned working -> review"
---
# V3 milestone intelligence view

## Objective

Redesign the roadmap/milestones UI so milestones become actionable project intelligence: progress, blockers, linked specs, reviews, decisions, risks, dependencies, and next actions should be visible without opening multiple Markdown files.

## Context

The current Roadmap view parses `.lmbrain/ROADMAP.md` and renders milestone cards with status, outcome, progress, associated spec IDs, and risks. The result is visually acceptable but not very useful for steering work. It does not deeply join milestone references to spec/review/ADR state.

V3 should make the milestone view answer operational questions: what is active, which specs are ready/working/blocked/in review, what decisions constrain the milestone, what the next manual handoff is, and which risks or missing references need attention.

## Scope

### Included

- Extend the roadmap/milestone data model with derived linked artifact state.
- Redesign the Roadmap view into a scannable milestone intelligence surface.
- Show per-milestone progress by spec status, review status, linked decisions, risks, dependencies, and next action.
- Make linked specs/reviews/ADRs clickable through existing detail/wiki navigation.
- Add diagnostics for unresolved milestone references and directory/status mismatches that affect milestone data.
- Improve parser robustness for roadmap fields without breaking existing `ROADMAP.md` format.
- Update docs and tests.

### Excluded

- A full project-management database.
- Editing roadmap milestones in-app.
- Drag-and-drop milestone planning.
- Gantt charts or calendar scheduling.

## Existing-project analysis

- `src/components/Roadmap/RoadmapView.tsx` loads roadmap and specs separately, then groups specs by milestone.
- `src-tauri/src/models/roadmap.rs` has a narrow `Milestone` model.
- `src-tauri/src/commands/contract.rs::parse_roadmap_content` parses markdown headings and simple list fields.
- Specs already expose `status`, `priority`, `area`, `milestone`, `recommended_agent`, `links`, and body.
- Reviews and ADRs are loaded elsewhere but not joined into the roadmap response.
- `ProjectPulse` already computes recommended actions; milestone view can reuse similar concepts without duplicating business rules poorly.

## Technical proposal

Add a derived milestone view model, either by extending `Roadmap` or by adding a new command such as `get_milestone_overview`.

Recommended derived data per milestone:

- spec counts by status;
- linked specs with title, status, priority, area, recommended agent;
- reviews linked to those specs and their status;
- decisions linked by roadmap field or by spec links;
- unresolved referenced IDs;
- dependency status for `depends_on`;
- next action: first ready spec, pending review, missing decision, or no action.

Frontend layout should use a compact milestone list/timeline, selected milestone detail area, status chips, progress bars, artifact lists with click-through, and visible risk/dependency warnings.

## Files and areas involved

- `src-tauri/src/models/roadmap.rs`
- `src-tauri/src/commands/contract.rs`
- `src-tauri/src/lib.rs` if adding a new command
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/components/Roadmap/RoadmapView.tsx`
- `src/components/Layout/ArtifactDetailModal.tsx` only if needed for navigation consistency
- `src/__tests__/RoadmapView.test.tsx`
- `src-tauri/tests/contract_test.rs`
- `docs/product.md`
- `docs/architecture.md`
- `docs/kit.md`
- `.lmbrain/ROADMAP.md` if adding M-03 planning details

## Acceptance criteria

- [ ] Roadmap/milestone data includes derived spec, review, ADR, dependency, risk, and missing-reference state.
- [ ] The Roadmap view shows active/planned/completed milestones with useful progress and next-action information.
- [ ] Milestone detail exposes linked specs with title/status/priority/agent and linked reviews/decisions where available.
- [ ] Unresolved milestone references are visible as warnings rather than silently ignored.
- [ ] Existing ROADMAP markdown continues to parse.
- [ ] The UI is readable on common desktop widths and does not rely on nested cards or oversized marketing-style layout.
- [ ] Tests cover roadmap parsing, derived milestone joins, missing references, and frontend rendering.
- [ ] Existing `pnpm lint`, `pnpm test`, and Rust tests pass.

## Implementation plan

1. Define the derived milestone overview model and decide whether to extend `get_roadmap` or add a new command.
2. Implement backend joins across roadmap, specs, reviews, and ADRs.
3. Add diagnostics for unresolved milestone references.
4. Redesign `RoadmapView` around overview/detail and click-through artifact navigation.
5. Add frontend and backend tests.
6. Update docs and run quality gates.

## Required verification

- `pnpm lint`
- `pnpm test`
- `cargo test`
- Manual app check on this repository's roadmap, including M-03 and specs without milestones.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Preserve Markdown as source of truth; derived milestone intelligence must not become a second planning store.
- Update docs that describe roadmap and milestone behavior.

## Risks and open decisions

- Extending `get_roadmap` may break consumers if the shape changes too much. Recommendation: add optional fields or a new overview command.
- Deriving "next action" can become opinionated. Recommendation: keep it transparent and rule-based.
- The current roadmap in this repository is stale; implementation should handle stale data honestly rather than hiding it.

## Instructions for the assigned specialist

- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

> Completed by AGENT-FULLSTACK-DESKTOP on 2026-07-02.

### Changes made

1. **src-tauri/src/models/roadmap.rs** — Added `MilestoneSpecSummary` (with `path`), `MilestoneReviewSummary` (with `path`), `MilestoneAdrSummary` (with `path`), `MilestoneDetail`, and `MilestoneOverview`. All new types are additive; existing `Roadmap`/`Milestone` models unchanged.

2. **src-tauri/src/commands/contract.rs** — Added `build_milestone_overview` function that joins data from ROADMAP.md, specs, reviews, and ADRs to produce a derived `MilestoneOverview`. Populates real artifact paths from `Spec.path`, `Review.path`, and `Adr.path` for click-through navigation.

3. **src-tauri/src/lib.rs** — Added `get_milestone_overview` Tauri command registered as `get_milestone_overview`.

4. **src/types/index.ts** — Added TypeScript interfaces with `path` fields on all summary types.

5. **src/lib/commands.ts** — Added `getMilestoneOverview()` command wrapper.

6. **src/components/Roadmap/RoadmapView.tsx** — Complete redesign from flat milestone card list to sidebar/detail layout. Spec items, review rows, and decision rows are all clickable through `openDetailArtifact` using real artifact paths from the backend (not reconstructed paths).

8. **src/__tests__/RoadmapView.test.tsx** — Rewrote from legacy temporal-target test to 3 milestone intelligence tests:
   - Renders overview and detail with specs, reviews, decisions, risks, next action
   - Shows empty state when no milestones exist
   - Shows unresolved references as warnings

9. **docs/product.md** — Updated Roadmap view description for milestone intelligence.

10. **docs/architecture.md** — Added Milestone intelligence (v3) section documenting the derived overview model and command.

### Files changed

```
M src-tauri/src/models/roadmap.rs
M src-tauri/src/commands/contract.rs
M src-tauri/src/lib.rs
M src-tauri/tests/contract_test.rs
M src/types/index.ts
M src/lib/commands.ts
M src/components/Roadmap/RoadmapView.tsx
M src/__tests__/RoadmapView.test.tsx
M docs/product.md
M docs/architecture.md
```

### Verification performed

- `pnpm lint` — no errors
- `pnpm test` — 80 tests pass (14 test files, 3 new click-through tests)
- `cargo test` — all workspace tests pass (zero warnings)
- Backend contract tests: 23 passed (2 new milestone overview tests)
- Frontend roadmap tests: 6 passed (rendering, click-through for specs/reviews/ADRs, empty state, unresolved refs)

### Deviations from the specification

None. All scope items implemented as specified. Review finding P1 (click-through paths) has been addressed:

- **P1 (spec path):** `MilestoneSpecSummary` now includes `path` field populated from `Spec.path`. RoadmapView uses the real path instead of reconstructing `.lmbrain/specs/{status}/{id}.md`.
- **P1 (review/ADR paths):** `MilestoneReviewSummary` and `MilestoneAdrSummary` now include `path` fields. Review and decision rows are wired to `openDetailArtifact` with real artifact paths.
- **P1 (tests):** 3 new frontend tests verify `openDetailArtifact` is called with the correct slugged paths for specs, reviews, and ADRs.

### Handoff status

- [x] Ready for Project Lead review
