---
id: SPEC-057
# Note: Quote the title if it contains a colon
title: "Define first-class cross-spec finding architecture and core lifecycle"
status: backlog
kind: feature
priority: high
area: finding-contract-and-core
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/12, ADR-013, SPEC-037]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-12, findings, core, mcp]
activity:
  - date: 2026-07-29
    action: "created"
---
# Define first-class cross-spec finding architecture and core lifecycle

## Objective
Introduce a repository-owned `FINDING-*` domain for durable cross-spec observations, risks, defects, limitations, and design questions without turning every review bullet into a separate artifact or authorizing work.

## Context
GitHub issue #12 documents repeated dogfooding evidence and explicitly triggers ADR-013’s review condition for independent finding lifecycles and cross-spec resolution. LMBrain currently has no finding artifact kind, parser, identity, lifecycle, relationship index, diagnostics, context, or MCP tools; the Tauri review model’s `findings` collection is not populated.

## Scope
### Included
- First create and obtain operator approval for an ADR that revises/supersedes ADR-013 and reconciles SPEC-037 semantics.
- Add globally unique `FINDING-*` IDs and directories for open, planned, deferred, resolved, accepted-risk, and superseded.
- Define flat canonical provenance/relationship fields and template body sections.
- Add `ArtifactKind::Finding`, allocation, parsing, validation, path/status agreement, semantic transitions, authority, locking, atomic writes, and audit.
- Enforce planned target specs, resolution evidence, accepted-risk rationale/revisit, supersession successor/reason, duplicate origin promotion, reference validity, and no silent resolution.
- Add semantic create/plan/defer/resolve/accept-risk/supersede/reopen/context/candidate-inventory MCP tools.
- Add bounded reverse indexes to project/spec/review/finding context.
- Add canonical diagnostics and read-only legacy candidate inventory.
- Preserve review history and agent metrics; promotion does not double count or rewrite outcomes.

### Excluded
- Desktop Findings experience and migration UI, delegated to [[SPEC-058-add-findings-desktop-experience-and-explicit-legacy-migration]].
- Auto-promoting review prose, auto-generating target specs, or auto-closing on target completion.
- Full graph visualization.

## Existing-project analysis
Every artifact family is enumerated across `transitions.rs`, ID allocation/templates, MCP schemas, context, Tauri loaders, TypeScript state, routes, and statistics. Review-local finding text has no independent identity. SPEC-037’s current addressed-review model assumes findings do not have independent lifecycles.

Read-only XenoMark analysis adds the following mandatory regression shapes:

- `FINDING-01` occurs in ten reviews; sixteen token values occur in more than one
  review. Global promotion identity must therefore be the allocated
  `FINDING-*` plus the source pair, never the local token alone.
- `REVIEW-054/FINDING-07` is historically blocking in its review but is routed
  as medium-priority planned debt to `SPEC-059`. The schema must preserve
  `origin_severity` separately from current `severity`.
- `REVIEW-049/FINDING-050-001` is a real limitation closed by documentation and
  measurement, while still constraining `SPEC-057` verification. Migration
  must not reopen it as debt; a selected promotion is `resolved` unless the
  operator separately accepts remaining behavior as risk.
- two retained design observations have no originating review and no target
  spec. Native findings must support evidence-backed direct creation with no
  fabricated `origin_artifact`.
- three unresolved `before-done` gates remain after XenoMark reconciliation.
  They stay verification diagnostics unless explicitly promoted.
- only six of fifty-four reviews declare `review_cycles`; candidate inventory
  must not infer final disposition from historical headings.

## Technical proposal
Use a first-class core domain with semantic operations rather than generic arbitrary-field mutation. Creation is open-only. `planned` is not resolved; target completion produces attention, not mutation. Any optional origin-review promotion marker must be written in a tested atomic multi-artifact transaction. Candidate detection remains bounded and read-only.

Adopt [[ADR-014-promoted-findings-have-independent-lifecycle-while-reviews-preserve-historical-outcome]]
as the normative contract. Keep the canonical origin link on the finding and
derive review/spec reverse joins; do not rewrite the origin review during
normal promotion. Add optional `origin_severity` and keep product finding
taxonomy separate from review/agent-effectiveness taxonomy.

## Files and areas involved
- new/updated ADR and reconciliation of SPEC-037 before approval
- core artifact, finding lifecycle, relationship index, diagnostics/context
- MCP schemas/dispatch/authority
- kit directories/template/contract/AGENT/OPERATOR/migrations/changelog
- public architecture/kit documentation and fixtures

## Acceptance criteria
- [ ] ADR-013/SPEC-037 semantics are revised and operator-approved before implementation starts.
- [ ] FINDING is a documented managed artifact with global identity, statuses, authority, template, and migration contract.
- [ ] Creation is open-only and every transition is semantic, atomic, audited, and fail-closed.
- [ ] Planned/resolved/accepted-risk/superseded invariants are enforced consistently.
- [ ] Origin, related, target, blocker, decision, and resolution references are validated and reverse-indexable.
- [ ] Current `severity` and optional `origin_severity` round-trip independently.
- [ ] Evidence-backed direct findings may omit origin metadata; promoted review findings require a unique valid `(origin_artifact, origin_ref)` pair.
- [ ] Target planning/completion never silently resolves a finding.
- [ ] Origin review verdict/body remains historically accurate and metrics do not double count promotion.
- [ ] Context packs expose bounded canonical relationships.
- [ ] Legacy review prose is never silently classified or promoted.
- [ ] XenoMark-shaped fixtures distinguish planned debt, a resolved documented limitation, two targetless design observations, repeated local IDs, and unresolved verification gates that are not automatically promoted.

## Implementation plan
1. Draft the successor ADR and reconcile SPEC-037; stop for operator decision.
2. Freeze schema, authority, relationships, rollback, and atomicity.
3. Implement core domain/invariants/diagnostics and semantic MCP tools.
4. Add context joins, candidate inventory, migration fixtures, and docs.

## Required verification
- Full transition/authority/invariant matrix, path/status/ID allocation, invalid graphs, concurrency, failure injection, and multi-artifact atomicity.
- Reverse-join/context bounds and metric-history regression.
- Legacy candidate/idempotence/duplicate-local-ID fixtures.
- Full Rust/MCP/contract gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- This spec is blocked until the operator accepts proposed ADR-014 and approves the corresponding revision of SPEC-037.
- Reopen authority and exact accepted-risk revisit semantics require explicit operator decision in that ADR.
- Depends on SPEC-050 diagnostics and should consume SPEC-051 review events.

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
