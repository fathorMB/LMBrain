---
title: Roadmap
updated: 2026-08-10
---

# Roadmap

## M-01 — Read-only desktop workspace

- `status`: active
- `outcome`: Operators can select an LMBrain repository and understand its project state through a production-grade, local-first, read-only desktop application.
- `specs`: [SPEC-001, SPEC-009, SPEC-011, SPEC-012, SPEC-013]
- `risks`: [filesystem permission boundaries, Markdown contract parsing, watcher reliability]

## M-02 — Operator approval workflow (read-write)

- `status`: active
- `outcome`: Operators can review proposed artifacts and approve/reject them in-app, with status writes that keep the Markdown source of truth consistent and git-friendly.
- `decisions`: [ADR-002]
- `specs`: [SPEC-014, SPEC-015, SPEC-016]
- `risks`: [contract-invariant violations on write, file-move/status consistency, watcher/git races, scope creep beyond operator-initiated writes]
- `depends_on`: M-01

## M-03 - LMBrain v3 workflow and workspace ergonomics

- `status`: active
- `outcome`: LMBrain reduces agent-token waste, recommends more precise specialist profiles, presents sessions as tabs, makes milestones useful for operational planning, and guides safe kit migration for older projects.
- `decisions`: [ADR-008]
- `specs`: [SPEC-023, SPEC-024, SPEC-025, SPEC-026, SPEC-027]
- `risks`: [overfitting agent profiles, context summaries hiding important evidence, terminal lifecycle regressions, milestone-derived state drifting from Markdown source of truth, kit migrations overwriting project-specific knowledge]
- `depends_on`: M-02

## M-04 - Settings and governed harness environments

- `status`: active
- `outcome`: Operators manage real local settings and harness installations in one place; Project Leads declare governed project environments; verification evidence has enforceable provenance; closing a spec also closes its resolved review lifecycle without erasing history.
- `decisions`: [ADR-011, ADR-012, ADR-013]
- `specs`: [SPEC-035, SPEC-036, SPEC-037]
- `risks`: [repository configuration executing untrusted tools, native harness configuration drift, false capability-health reporting, approval reuse across workspace identity changes, synthesized or stale verification evidence, verification process-tree leaks, partial cross-artifact closeout, historical metric drift]
- `depends_on`: M-03
- `release_note`: Settings and governed harness work shipped in 2.8.0; unfinished verification provenance and atomic closeout remain active prerequisites for the next coordinated release.

## M-05 - Trustworthy handoffs and governed learning (2.9.0)

- `status`: proposed
- `outcome`: Implementation agents receive a complete structured verification contract and approved profile guidance before work; LMBrain records trustworthy gate provenance, turns repeated review evidence into operator-governed improvement proposals, and measures whether applied changes reduce recurrence.
- `decisions`: [ADR-008, ADR-012]
- `specs`: [SPEC-036, SPEC-038, SPEC-039, SPEC-040]
- `risks`: [verification execution without sandboxing, context schemas omitting requirements, migration overwriting project-specific guidance, noisy improvement signals, unauthorized profile mutation, misleading small-sample effectiveness claims, Windows process-tree leaks]
- `depends_on`: M-03
- `prerequisites`: [SPEC-036]

## M-06 — Reliable agent sessions and observable operations (3.0.0)

- `status`: proposed
- `outcome`: Make long-running agent work reliable, observable, and easy to audit. This release establishes a first-class repository dashboard, splits the Agents & MCP screens, introduces the Leave Workspace safety guard, enforces strict handoff archival, unifies the invariant engine, and implements app-owned virtualized conversational transcripts independent of xterm.
- `specs`: [SPEC-042, SPEC-043, SPEC-044, SPEC-045]
- `risks`: [TUI buffer capturing/parsing complexity, Windows credential storage permission issues, watcher/Git event racing during debounce, performance degradation from transcript virtualization]
- `depends_on`: M-05

## M-07 - Trustworthy lifecycle and cross-spec project intelligence (3.1.0)

- `status`: proposed
- `outcome`: LMBrain records review and verification history truthfully, exposes reconciled actionable project state, enforces spec lifecycle prerequisites, and manages durable cross-spec findings through governed kit/MCP contracts and an accessible desktop experience.
- `issues`: [10, 11, 12, 13, 14, 15, 16, 17, 18]
- `specs`: [SPEC-049, SPEC-050, SPEC-051, SPEC-052, SPEC-053, SPEC-054, SPEC-055, SPEC-056, SPEC-057, SPEC-058, SPEC-059]
- `risks`: [public MCP payload compatibility, authority confusion between Lead and operator, cross-artifact transaction failure, legacy history ambiguity, diagnostics divergence, migration of customized brains, release scope concentration]
- `depends_on`: M-06
- `prerequisites`: [operator decision on ADR-013 successor and SPEC-037 reconciliation before SPEC-057, UX authority decisions in SPEC-053 and SPEC-055]
- `release_note`: Planning is in backlog only. No leaf spec or implementation handoff is approved by this roadmap entry. Issue #12 is mandatory for 3.1.0; there is no planned fixes-only intermediate release.

## M-08 — LMBrain 3.1.3 maintenance and kit feedback fixes

- `status`: proposed
- `outcome`: Resolve 3.1.2 kit feedback issues covering kit file realignment during migrations, Windows Node REPL kernel assets path creation, Browser skill file:// policy alignment and claimed-tab read access, spec_attest_lead checklist completion for owner=lead, waived acceptance criteria with linked FINDINGs at spec closeout, async 3.1.x background loading for UI responsiveness, Lead remediation verification review events, and close confirmation for active sessions.
- `issues`: [34, 35, 36, 37, 38, 39, 40, 41, 42]
- `specs`: [SPEC-060, SPEC-061, SPEC-062, SPEC-063, SPEC-064, SPEC-065, SPEC-066, SPEC-067, SPEC-068]
- `risks`: [browser policy security boundary regressions, contract invariant looseness for spec closeout, migration overwrite of customized project guidance, desktop UI responsiveness regressions]
- `depends_on`: M-07

## M-09 - LMBrain 4.1.0 reliable context and reflective planning

- `status`: proposed
- `outcome`: Correct governed artifact/context failures reported by BaseballBoss and let an operator deliberately invoke bounded, provenance-preserving Project Lead reflection without turning tentative ideas into committed work.
- `issues`: [102, 103, 104, 105]
- `specs`: [SPEC-070, SPEC-071]
- `risks`: [speculative ideas mistaken for evidence, unintended mutations from ordinary conversation, cross-layer artifact-contract drift, raw transcript or sensitive-context retention, inaccessible secondary navigation]
- `depends_on`: M-07
- `release_note`: Planning only. Dream Journal lifecycle actions remain governed MCP operations; the first desktop surface is read-only.
