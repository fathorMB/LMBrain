---
id: ADR-005
title: Retire tasks as first-class artifacts; the board tracks specs
status: accepted
decision_date: 2026-06-23
decider: user
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: []
superseded_by: []
links: [ADR-002, ADR-004, SPEC-019]
tags: [architecture, workflow, board, breaking]
---

# Retire tasks as first-class artifacts; the board tracks specs

> **Status: accepted** (operator, 2026-06-23).

## Context

Across repeated manual-testing runs the same failure recurs: the Project Lead produces a `SPEC` but does not decompose it into `TASK` artifacts, so the Taskboard stays empty. We mitigated it with a stronger `AGENT.md` instruction and a "ready spec without tasks" diagnostic, but the root cause is structural: **tasks are a second artifact layer the agents must create and keep in sync, and they handle it poorly.**

Crucially, the two lifecycles are nearly identical:

- Spec: `proposed → ready → in-progress → review → accepted` (+ `changes-requested`, `rejected`, `archived`)
- Task: `backlog → planned → in-progress → review → done` (+ `blocked`, `cancelled`)

The spec's own status already encodes implementation progress, and the spec is already the unit of handoff and of review. The task layer is largely redundant bookkeeping. Sub-spec granularity that does matter (the implementation checklist) already exists inside the spec as its **acceptance criteria**.

## Decision

Retire `TASK` as a first-class LMBrain artifact. The board (renamed from "Taskboard" to "Board") tracks **specs** by status. Granularity below a spec lives in that spec's **acceptance-criteria checklist** in its body, not in separate files.

The spec status set is **redefined** to the board's columns:

| Status | Meaning | Who moves it |
| --- | --- | --- |
| `backlog` | Spec created, not yet approved by the operator | Lead creates |
| `ready` | Approved, ready to be picked up by the implementer | Operator approval (lead executes on explicit operator request) |
| `working` | The implementer has started working the spec | Implementer (first action) |
| `review` | The implementer considers development complete; the spec **stays here for the whole lead↔implementer review ping-pong** | Implementer |
| `done` | Review passed and the lead has created the commit on the repo | Lead |
| `discarded` | Spec written then abandoned for any reason | Lead, only on explicit operator approval |

There is **no `blocked`** (folded into `discarded`) and **no `changes-requested`** (revisions happen while the spec stays in `review`). This replaces the previous spec statuses (`proposed/in-progress/accepted/changes-requested/rejected/archived`) **for specs only**; ADR / agent / MCP / handoff / review statuses are unchanged.

Principles:
- **Granularity = acceptance criteria** in the spec body. A spec reaches `done` only with its criteria checked, evidence recorded, and an accepted review; the commit itself is the lead's act, not engine-enforced.
- **Hierarchy is Milestone → Spec → criteria.** The roadmap groups specs under milestones; that layer is unchanged.
- **The engine/MCP drop the task surface.** The spec verbs become `spec_ready` (backlog→ready), `spec_start` (ready→working), `spec_submit` (working→review), `spec_done` (review→done), and `spec_discard` (→discarded), governed by the authority rules in [[ADR-004-controlled-mutation-engine-mcp]] (operator-gated approval; discard only on explicit operator approval).

## Alternatives considered

- **Keep tasks (status quo).** Lowest churn; rejected because the empty-board failure keeps recurring and the layer adds maintenance the agents fumble.
- **Demote tasks to an optional in-spec checklist but leave the infrastructure dormant.** Less destructive and reversible, but leaves dead code/paths and a half-present concept. Rejected in favor of a clean removal, with the dormant-checklist idea folded into "acceptance criteria."
- **Full removal (chosen).** Simplest end state; highest one-time refactor cost.

## Consequences

### Positive
- Eliminates the recurring empty-board failure mode and a whole class of cross-artifact invariants (planned-without-ready-spec, ready-spec-without-tasks).
- Simpler mental model and less for agents to maintain.
- The board is always meaningful once a spec exists.

### Costs / risks
- **Breaking contract change** (new spec status set + removed task layer) → major version bump (2.0.0).
- Loss of sub-spec parallelism, per-item assignment, and inter-task dependencies. Mitigated by acceptance-criteria checklists and milestone→spec grouping; revisit if a project genuinely needs finer tracking.
- Sizeable refactor touching `lmbrain-core`, `lmbrain-mcp`, the app (board + diagnostics), and the kit (templates/dirs/docs/prompts).
- **No migration.** The app and kit are in early (production-grade) development and only being tested; existing test projects (e.g. Brewlog) and the kit are re-scaffolded to the new model rather than migrated.

## Review conditions

Revisit if spec-level granularity proves insufficient for larger, multi-contributor projects — in which case reintroduce a lightweight, optional sub-spec work-item layer rather than the previous always-required tasks.
