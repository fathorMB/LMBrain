---
id: ADR-003
title: Reject as a first-class status across proposable artifacts
status: accepted
decision_date: 2026-06-22
decider: user
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: []
superseded_by: []
links: [ADR-002, SPEC-014]
tags: [architecture, contract, status, reject, workflow]
---

# Reject as a first-class status across proposable artifacts

## Context

[[ADR-002-in-app-artifact-status-writes]] introduced operator-driven approve/reject. While scoping the workflow ([[SPEC-014-operator-approval-workflow]]) two contract gaps surfaced:

- **ADR** statuses were `proposed`/`accepted`/`superseded`/`deprecated` — with **no way to reject** a proposed decision.
- **Agent proposals** (`AGENT-PROP-*`) had **no defined status set** in `CONTRACT.md` at all.

The operator decided that "reject" should be a well-documented, first-class concept present consistently across the system, not an app-only special case or an undefined gap.

## Decision

Make `rejected` a **first-class terminal status** on every proposable artifact, and define agent-proposal statuses explicitly. `CONTRACT.md` (v0.2) is amended:

- **Spec:** add `rejected`.
- **ADR:** add `rejected`.
- **Agent proposal:** new status set `proposed` / `approved` / `rejected`.
- **MCP proposal:** already has `rejected` — unchanged, now consistent with the rest.

Semantics (recorded as a contract invariant):
- `rejected` means "declined at proposal/decision time" and is terminal.
- It is **distinct from** `changes-requested` (a review asking for revision and resubmission) and from `archived`/`superseded`/`deprecated` (retiring something that was once active).
- A rejected artifact records its rejection rationale in the body and is not silently reopened.

Filesystem: spec is a status-directory artifact, so a `specs/rejected/` directory is added to the kit and the live brain. ADRs and proposals are flat directories, so their `rejected` state lives in frontmatter only.

This is a backward-compatible addition (no existing artifact changes meaning), so it is a **minor** kit bump: the next release is `1.1.0`. No release is cut now; the bump is recorded for the next coordinated release.

## Alternatives considered

- **Per-artifact ad-hoc mapping** (e.g. ADR reject → `deprecated`, spec reject → `changes-requested`). Rejected: it overloads statuses with the wrong meaning (`deprecated`/`changes-requested` mean different things) and leaves "reject" inconsistent across the system.
- **App-only reject** without a contract status. Rejected: it would make the UI write a status the contract does not define, violating the contract/source-of-truth model.
- **Leave ADR/agent-proposal reject unsupported.** Rejected by the operator: reject must be uniformly available.

## Consequences

### Positive
- Reject is uniform, documented, and visible across Spec, ADR, Agent proposal, and MCP proposal.
- [[SPEC-014-operator-approval-workflow]] can offer reject for all these types (its open decisions D-1/D-2 are now resolved).

### Constraints / costs
- Backend status enums and the directory/status-mismatch diagnostic must recognize `rejected` (handled in SPEC-014 implementation).
- The kit ships a new `specs/rejected/` directory; scaffolds include it.
- Kit version moves to `1.1.0` at the next release; `MIGRATIONS.md` documents the additive change.

## Review conditions

Revisit if a clearer lifecycle model emerges (e.g. a need to reopen rejected items), in which case define an explicit reopen transition rather than silently mutating a rejected artifact.
