---
id: SPEC-065
title: "Support waived acceptance criteria linked to active FINDINGs during spec closeout"
status: backlog
kind: feature
priority: medium
area: spec-lifecycle
milestone: M-08
recommended_agent: AGENT-RUST-CORE
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/39]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-39, spec-done, waived-criteria, findings, KIT-NOTE-013]
activity:
  - date: 2026-07-31
    action: "created"
---
# Support waived acceptance criteria linked to active FINDINGs during spec closeout

## Objective
Provide a structured syntax and invariant rule for waiving individual acceptance criteria backed by an open `FINDING-*` during `spec_done`, avoiding unmotivated `force: true` closeouts.

## Context
Reported in `KIT-NOTE-013` (v3.1.2): `spec_done` requires all acceptance criteria checked or forces an all-or-nothing `force: true` override. When 12 out of 13 criteria pass and the 13th is partially unfulfilled but tracked by an open `FINDING-003`, there is no way to close the spec honestly with a granular waiver.

## Scope
### Included
- Support a waived criterion format in spec Markdown (e.g. `- [~] text | waived=FINDING-xxx`).
- Validate during `spec_done` that referenced `FINDING` IDs exist and are currently open.
- Distinguish granularly waived spec closeout from unmotivated global `force: true` in diagnostics and metrics.
- Document the waiver lifecycle rule in `CONTRACT.md`.

### Excluded
- Allowing arbitrary waivers without a valid open `FINDING` reference.

## Acceptance criteria
- [ ] `spec_done` accepts specs containing waived criteria (`- [~] ... | waived=FINDING-xxx`) when the referenced finding is active.
- [ ] Invalid or missing finding references in waived criteria are rejected by `spec_done` and `lmbrain_validate`.

## Required verification
- `cargo test --workspace`
- `pnpm test`
