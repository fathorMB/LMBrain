---
id: SPEC-019
title: "Retire tasks and make the board track specs"
status: done
kind: feature
priority: high
area: core-tooling
milestone:
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-005, ADR-002, ADR-004]
links: [ADR-005]
created: 2026-06-23
updated: 2026-06-23
tags: [board, tasks, spec, refactor, breaking]
---

# Retire tasks and make the board track specs

> **Status: proposed.** Implement only after [[ADR-005-retire-tasks-spec-board]] is accepted.

## Objective

Remove `TASK` as a first-class LMBrain artifact and make the board track **specs** by status, with sub-spec granularity carried by each spec's acceptance-criteria checklist. Implements [[ADR-005-retire-tasks-spec-board]].

## Context

See ADR-005. The spec and task lifecycles are nearly identical, the spec is already the handoff/review unit, and agents repeatedly fail to create tasks — leaving the board empty. This change deletes the redundant layer.

## Scope

### Included
- **Spec status model:** redefine `SpecStatus` to `backlog | ready | working | review | done | discarded` (per ADR-005); rename the `specs/<status>/` directories to match and remove the old statuses.
- **Engine (`lmbrain-core`):** remove `TaskStatus`/`Task`/task transitions/`ArtifactKind::Task`. Implement the spec state machine: `backlog→ready` (`spec_ready`), `ready→working` (`spec_start`), `working→review` (`spec_submit`), `review→done` (`spec_done`), `*→discarded` (`spec_discard`). The `done` transition requires the spec's acceptance criteria checked, evidence present, and an accepted review.
- **MCP (`lmbrain-mcp`):** remove `task_*` and `lmbrain_set_task_evidence`. Expose the spec verbs above; `spec_ready` is operator-gated and `spec_discard` requires explicit operator approval (per [[ADR-004-controlled-mutation-engine-mcp]] / BUG-008); keep `review_accept`. Update the built-in tool catalog shown in the app.
- **App (`src-tauri` + UI):** remove `build_tasks`/`get_tasks`/task model and the task diagnostics; repurpose the Taskboard into a **Board** with columns `backlog → ready → working → review → done → discarded` that reads specs and shows each spec's acceptance-criteria progress on its card. Add a diagnostic that a spec cannot reach `done` without criteria + evidence + accepted review.
- **Kit + docs:** delete the `task.md` template and the `tasks/` scaffold; set the spec template default status to `backlog`; rewrite `AGENT.md`, the handoff and bootstrap prompts, and `CONTRACT.md` (IDs/locations, allowed statuses, invariants, authority) around the spec board; remove the task lifecycle docs.
- **Versioning:** major bump to `2.0.0` (breaking contract change).

### Excluded
- The roadmap/milestone layer (unchanged).
- **Migration of existing projects** — none; the app/kit are in early development and only being tested, so test projects and the kit are re-scaffolded to the new model rather than migrated.
- Reintroducing any optional sub-spec work-item layer (deferred per ADR-005's review condition).

## Existing-project analysis

- Task surface to remove: `src-tauri/src/models/task.rs`, `build_tasks` and task diagnostics in `src-tauri/src/commands/contract.rs`, the `get_tasks` command in `src-tauri/src/lib.rs`, `ArtifactKind::Task`/task arms in `lmbrain-core/src/transitions.rs`, task tools in `lmbrain-mcp/src/main.rs`, `src/components/Taskboard/*`, `Task`/`TaskStatus` in `src/types`, and the related frontend tests/mocks.
- Board reuse: the spec reader (`build_specs`) and `Spec` model already expose status; the Board reuses them. Spec acceptance-criteria parsing mirrors the old `parse_criteria`.
- Authority/engine: spec acceptance already requires an accepted review (ADR-002/CONTRACT) — extend it with the criteria+evidence check.

## Acceptance criteria
- [ ] No `TASK` artifact remains: the task model, `build_tasks`, `get_tasks`, task transitions, task MCP tools, and the `tasks/` scaffold are gone; build is green with no dead task code.
- [ ] `SpecStatus` is exactly `backlog | ready | working | review | done | discarded`, with matching `specs/<status>/` directories; old spec statuses are removed.
- [ ] The Board renders columns `backlog → ready → working → review → done → discarded` from specs and shows each spec's acceptance-criteria progress.
- [ ] The spec state machine matches ADR-005 (`spec_ready`/`spec_start`/`spec_submit`/`spec_done`/`spec_discard`); `spec_done` requires criteria checked + evidence + accepted review; `spec_ready`/`spec_discard` follow the operator-authority rules.
- [ ] `lmbrain-mcp` exposes the spec verbs and no `task_*` tools; the app's built-in tool list matches.
- [ ] Kit templates (spec default `backlog`), `CONTRACT.md`, `AGENT.md`, and the handoff/bootstrap prompts contain no task concept and describe the spec board.
- [ ] Versions bump to `2.0.0` and stay aligned (`check-version.mjs`).
- [ ] `pnpm lint`/`pnpm test` and `cargo test` green on Linux + Windows in CI; tests cover the `spec_done` invariant, the new state machine, and the Board.

## Implementation plan
1. Engine: drop task transitions/kind; move criteria+evidence onto spec acceptance; adjust tests.
2. MCP: remove `task_*`/`set_task_evidence`; update catalog; adjust protocol/unit tests.
3. App backend: remove task model/commands/diagnostics; add a spec-acceptance criteria diagnostic.
4. UI: convert Taskboard → spec Board; drop task components; show criteria progress; update frontend tests.
5. Kit + docs + prompts: remove task concept everywhere; rewrite lifecycle around specs.
6. Migration note + `2.0.0` bump; full green CI.

## Required verification
- `cargo test` (all crates) + `pnpm test` green on Linux + Windows in CI.
- A test that a spec cannot be accepted without criteria+evidence+accepted review.
- Manual: open a project, confirm the Board shows specs by status and criteria progress; confirm no task UI/commands remain.

## Production quality and documentation
- Follow [[QUALITY]]; production work, not a prototype.
- Update `CONTRACT.md`, `AGENT.md`, `tasks`→removed docs, and `MIGRATIONS.md` as delegated above.
- Report any quality-policy exception explicitly.

## Resolved decisions (operator)
- **Board columns:** exactly `backlog → ready → working → review → done → discarded` (all shown).
- **`blocked`:** removed; folded into `discarded`.
- **Migration:** none — re-scaffold test projects and the kit; no migration tooling.

## Risks
- It is a breaking contract change (`2.0.0`); since there is no migration, any in-flight test data using old statuses is simply re-scaffolded.
- `review` is sticky through the whole ping-pong (no `changes-requested`); make sure the review artifact lifecycle still records iterations while the spec stays in `review`.
- The `done` "commit created by the lead" part is a human/lead action and is not engine-enforceable; the engine enforces only criteria + evidence + accepted review.

## Instructions for the assigned specialist
- Implement only the stated scope, and only after ADR-005 is accepted.
- Produce production-grade, maintainable code; remove dead code rather than leaving it disabled.
- Report changed files, tests run, and any deviation; do not change product scope or other ADRs.
- Challenge fragile assumptions and propose the clean alternative; consult current official docs where behavior is uncertain.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Deviations from the specification

### Handoff status
- [ ] Ready for Project Lead review
