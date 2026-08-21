---
id: SPEC-054
# Note: Quote the title if it contains a colon
title: "Bootstrap and diagnose the verification manifest safely"
status: backlog
kind: feature
priority: high
area: verification-onboarding
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/15]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, github-issue-15, verification, ux]
activity:
  - date: 2026-07-29
    action: "created"
---
# Bootstrap and diagnose the verification manifest safely

## Objective
Let an operator discover, preview, create/update, validate, approve, and maintain `.lmbrain/verification.toml` without hand-authoring internal schema or weakening execution trust.

## Context
GitHub issue #15 is confirmed. `load_verification_manifest` returns a bare `MissingManifest`, and MCP exposes get/approve/verify but no status, discovery, validation-only, or controlled create/update workflow. The bundled kit contains only a template and brief documentation. `origin/main` 3.0.2 adds safe declared build-output exclusions but does not add onboarding.

## Scope
### Included
- Add typed absent, invalid, unapproved, approved, and stale status with exact next safe actions.
- Discover likely native gates deterministically from supported Node scripts, Cargo workspace metadata, existing CI, and checked-in task configuration using bounded reads.
- Return a preview that distinguishes suggestions from approved commands and includes program/argv, cwd, timeout, expected result, output bound, environment policy, mutation/fingerprint exclusions, provenance, and exclusions.
- Add validation-only and controlled atomic create/update operations; approval remains a separate digest-bound operator act.
- Never execute discovered or newly written commands before explicit approval.
- Add a Settings → Verification experience for status, suggestions, editable/selected preview, diff, validation, approval identity/time, stale state, and rollback.
- Diagnose missing/unknown manifest gates from specs using canonical diagnostics.
- Redact secrets and reject unsafe paths, interpolation, credentials, and machine-local values.

### Excluded
- Treating discovery as proof a command is correct or trusted.
- Importing CI trust or executing arbitrary shell snippets by default.
- Automatic manifest creation, approval, or execution during workspace open.

## Existing-project analysis
Core already has strict parsing, canonical digests, approval storage, execution bounds, path confinement, and snapshot freshness. The clean solution extends this trust model with planning/writes rather than catching `MissingManifest` and suppressing it.

## Technical proposal
Model discovery as read-only adapters producing typed candidates with provenance and confidence, never raw executable policy. A controlled writer accepts a complete validated manifest and uses the existing lock/atomic-write/audit patterns. The app shows exact proposed content and keeps approval separate. Reuse 3.0.2 `fingerprint_exclude` semantics.

## Files and areas involved
- `lmbrain-core/src/verification.rs` plus discovery/status module
- MCP tool schemas and machine-local approval integration
- Tauri commands/approval store, Settings view, TypeScript contracts
- kit template, contract, migration, docs and release notes

## Acceptance criteria
- [ ] A repository without a manifest can reach an approved valid manifest through discover → preview → create → approve without manual editing.
- [ ] No discovered or newly written command runs before explicit approval.
- [ ] Preview exposes every execution/security field and source.
- [ ] Manifest changes invalidate prior approval and produce an actionable stale state.
- [ ] Missing, malformed, unsafe, unapproved, stale, and unknown-gate states are distinct typed errors.
- [ ] Absolute paths, shell interpolation, credentials, and machine-local secrets are never emitted.
- [ ] Existing valid manifests remain compatible or have an explicit migration.
- [ ] Concurrent/read-only/partial-write failures leave one authoritative manifest and recoverable diagnostics.

## Implementation plan
1. Define status/preview/write contracts and supported discovery sources.
2. Implement bounded adapters and deterministic merge/conflict rules.
3. Implement atomic validation/write and stale-approval behavior.
4. Add Settings UI and actionable diagnostics.
5. Add end-to-end discovery-to-verification tests.

## Required verification
- Empty, Node, Rust, mixed, CI-conflict, no-candidate, malformed, unsafe, stale, concurrent, read-only, and workspace-switch fixtures.
- End-to-end MCP/app flow through init, create, approve, verify, and digest.
- Full Rust/frontend/packaged Windows gates.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Decide whether the first release permits operator editing in-app or only candidate selection plus exact TOML preview. Free-form editing increases validation UX and secret-handling risk.
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
