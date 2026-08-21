---
id: SPEC-064
title: "Atomic attestation and checklist completion for owner=lead verification gates"
status: backlog
kind: bugfix
priority: medium
area: verification-engine
milestone: M-08
recommended_agent: AGENT-RUST-CORE
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/38]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-38, spec-attest-lead, owner-lead, KIT-NOTE-012]
activity:
  - date: 2026-07-31
    action: "created"
---
# Atomic attestation and checklist completion for owner=lead verification gates

## Objective
Enable `spec_attest_lead` to mark checklist completion atomically alongside evidence attestation for `owner=lead` verification gates, removing the need for manual Markdown editing.

## Context
Reported in `KIT-NOTE-012` (v3.1.2): `spec_attest_lead` rejects attestations if the corresponding checklist box `- [ ]` is unchecked with `record completion in the spec before attesting its evidence`. Unlike `owner=operator` gates (unblocked in 3.1.2), `owner=lead` gates had no MCP verb to mark the checklist box, forcing the Lead to edit spec Markdown directly before calling `spec_attest_lead`.

## Scope
### Included
- Update `spec_attest_lead` in `lmbrain-core` to check/mark the target checklist item as completed upon recording a valid passing attestation for `owner=lead` gates.
- Alternatively, expose an explicit verb or parameter for marking `owner=lead` checklist completion.
- Ensure state invariants and typed audit records remain strictly intact.

### Excluded
- Allowing Lead attestation on `owner=operator` gates.

## Acceptance criteria
- [ ] `spec_attest_lead` successfully processes attestations for unchecked `owner=lead` gates without requiring prior manual Markdown editing.
- [ ] The checklist item `- [x]` is updated atomically with the attestation event write.

## Required verification
- `cargo test --workspace`
- `pnpm test`
