---
id: SPEC-053
# Note: Quote the title if it contains a colon
title: "Enforce before-done verification gates with distinct Lead and operator attestation"
status: backlog
kind: bugfix
priority: critical
area: verification-governance
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/14]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-14, verification, authority]
activity:
  - date: 2026-07-29
    action: "created"
---
# Enforce before-done verification gates with distinct Lead and operator attestation

## Objective
Prevent normal `spec_done` when required before-done verification is incomplete, stale, unsupported, or attested by the wrong authority.

## Context
GitHub issue #14 is confirmed in core. `context.rs` parses requirement ID, kind, owner, phase, checked state, evidence, and source, but `transitions.rs::invariant_failure` checks only acceptance evidence and an accepted review for `review -> done`. The current owner vocabulary conflates independent Lead review with human-only operator gates.

## Scope
### Included
- Add distinct `agent`, `lead`, and `operator` owners and validate supported owner/phase/kind combinations.
- Define typed attestations containing requirement identity, actor/role, timestamp, result, evidence reference/digest, and freshness inputs.
- Add governed Lead and human-operator attestation operations without allowing one authority to impersonate another.
- Enforce agent before-submit requirements at submission and Lead/operator before-done requirements at closeout.
- Return all blocking requirement IDs and precise causes in one response.
- Preserve the audited `force + reason` exception and record every unmet requirement and authority.
- Surface unresolved forced gates in canonical diagnostics/digest until reconciled.
- Diagnose completed legacy specs without reopening or rewriting them.
- Add conservative migration preview for legacy `owner=operator` entries that explicitly name the Lead.

### Excluded
- Automatically checking a Markdown box as proof of human action.
- Automatically executing manual/operator gates.
- Reopening existing done specs.

## Existing-project analysis
Verification requirements currently exist only as context records parsed from checklist lines. They are not a shared transition policy and carry no attestor or freshness record. The app handoff prompt already distinguishes before-submit and before-done conceptually, but enforcement is absent.

## Technical proposal
Create one typed verification-requirement and attestation rule engine in `lmbrain-core`, reused by context, transitions, diagnostics, MCP, and app. Store attestations as append-only governed records bound to requirement content/digest so editing the requirement makes old attestation stale. Keep human operator action on an explicit operator-owned surface.

## Files and areas involved
- core verification requirement parser/policy, transitions, diagnostics and tests
- MCP semantic attestation/closeout schemas
- Tauri/TypeScript contracts and spec/review verification UI
- contract, templates, AGENT/OPERATOR/QUALITY, migration and release notes

## Acceptance criteria
- [ ] Normal `spec_submit` and `spec_done` reject every unmet requirement relevant to their phase.
- [ ] Failure reports all requirement IDs, cause, current evidence/freshness, and responsible authority.
- [ ] A Lead cannot attest an operator-owned item and an operator attestation cannot substitute for agent evidence.
- [ ] Editing a requirement or referenced evidence invalidates stale attestations deterministically.
- [ ] Forced completion records reason, actor, and exact unresolved IDs and remains diagnostic-visible.
- [ ] Unsupported owner/phase/kind combinations fail validation with safe remediation.
- [ ] Existing done specs are diagnosed without automatic mutation.
- [ ] Core, MCP, validation, digest, and app use the same rule engine.

## Implementation plan
1. Freeze owner/phase policy and attestation schema.
2. Implement shared parsing, freshness, and transition invariants.
3. Add semantic Lead/operator actions and authority tests.
4. Add UI/context/diagnostic surfaces and migration preview.

## Required verification
- Exhaustive owner/phase matrix, multiple blockers, stale/forged/missing attestations, force/no-force, legacy done specs, and concurrency.
- XenoMark-shaped Lead and human gate fixtures.
- Full Rust/frontend/sidecar gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- UX decision required before approval: human operator attestations should likely be performed in the desktop app, while Lead attestations use MCP. Confirm whether the operator also needs a CLI/MCP path.
- Depends on SPEC-050 for canonical diagnostics.

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
