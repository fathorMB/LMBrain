---
id: ADR-002
title: In-app operator-driven artifact status writes (approve/reject)
status: accepted
decision_date: 2026-06-22
decider: user
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: []
superseded_by: []
links: [ADR-001, SPEC-012]
tags: [architecture, write, workflow, milestone]
---

# In-app operator-driven artifact status writes (approve/reject)

## Context

Through M-01 the desktop app is a **read-only** visualizer: the Taskboard states "status changes happen in Markdown", and the only write the app performs is the one-time kit bootstrap (`initialize_kit`). The operator has requested the ability to review a `proposed` artifact in a detail modal and **approve / reject** it directly in the app, with a corrective-prompt action on rejected items (see [[SPEC-012-test-round-2-findings]] R2-F5).

[[ADR-001-desktop-first-tauri]] already states the app "reads **and writes** `.lmbrain/` in place", so in-app writes are within the established architecture in principle. What is new is making the app mutate artifact **state**, which moves it from a viewer to a workflow tool. This is a deliberate product step and must be a recorded decision, not an incidental change inside a bug-fix round.

The read-only detail modal (viewing the full document) is **not** part of this decision — it is pure read and ships in M-01.

## Decision

Introduce operator-driven artifact status writes as a new milestone **M-02**, separate from M-01.

Principles:
- **Operator-initiated only.** No agent and no automatic process triggers a status write. This matches the authority model in `OPERATOR.md`/`CONTRACT.md` (the operator approves/edits decisions, specs, roadmap, etc.).
- **Status change implies a file move.** Changing an artifact's `status` must also move the file into the matching status directory, because the contract requires filesystem and `status` frontmatter to agree. `updated` is bumped on every write.
- **Per-artifact state machines.** "Approve/reject" is not a universal status flip; each artifact type has its own allowed statuses, so the available actions differ:
  - *MCP proposal* — has explicit `approved` / `rejected`; maps directly.
  - *Agent profile* — `proposed → active` (approve) / `inactive` or `retired` (reject-equivalent).
  - *ADR* — allowed statuses are `proposed`/`accepted`/`superseded`/`deprecated`; there is **no `rejected`**. Approve → `accepted`; "reject" has no native target and must either be unsupported or defined explicitly (e.g. leave `proposed` with recorded rationale, or `deprecated`).
  - *Spec* — has `changes-requested`/`archived` but no `rejected`. The contract also says a spec may be `accepted` only when an associated review is `accepted`, so the app must **not** offer a blind "approve → accepted" on specs; spec acceptance stays gated by the review workflow.
- **Safety.** Writes are atomic (reuse the bootstrap's temp-file+rename pattern), validated before commit, coordinated with the file watcher to avoid reload races, and confirmable/undoable. Filesystem access stays scoped to the selected workspace.

The detail modal and approve/reject UI must keep the read/write boundary visually explicit.

## Alternatives considered

- **Keep all status changes in Markdown (status quo).** Lowest risk; rejected as the chosen direction because the operator wants in-app approval and ADR-001 already permits writes. Remains the fallback if the write workflow proves fragile.
- **Generic "set any status" editor.** More flexible but invites contract violations (e.g. bypassing the spec-needs-review invariant). Rejected in favour of constrained, per-type approve/reject actions.

## Consequences

### Positive
- The operator can approve/reject proposed items without leaving the app.
- Rejected items can carry the [[SPEC-012-test-round-2-findings]] R2-F4 corrective-prompt action.

### Constraints / costs
- New backend write commands (status transition + file move) with validation and atomic writes.
- File-watcher coordination and git-friendliness (clean, reviewable diffs).
- Per-artifact-type action mapping must be designed before implementation, honoring all `CONTRACT.md` invariants.
- UI must not let the operator reach an invalid state (e.g. spec `accepted` without an accepted review).

## Review conditions

Revisit if the write workflow cannot honor the contract invariants at acceptable complexity, or if in-app writes produce unsafe/ambiguous git state — in which case fall back to Markdown-only status changes.
