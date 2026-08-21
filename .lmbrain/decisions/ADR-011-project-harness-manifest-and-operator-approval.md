---
id: ADR-011
title: "Project harness manifest with digest-bound operator approval"
status: accepted
decision_date: 2026-07-12
decider: operator
supersedes: []
superseded_by: []
links: [SPEC-035]
tags: [architecture, harnesses, governance, security, settings]
created: 2026-07-12
updated: 2026-07-12
activity:
  - date: 2026-07-12
    action: "accepted by operator"
  - date: 2026-07-12
    action: "created"
---

# Project harness manifest with digest-bound operator approval

## Context

LMBrain currently materializes narrow MCP integration into four different native harness formats. Project Leads need a way to govern project environment expectations, including host availability and LSP readiness, without learning or directly rewriting every native configuration. Repository-controlled configuration can also cause tools to execute binaries or alter session environments, so treating it as trusted merely because a workspace was opened would cross a security boundary.

## Proposed decision

- Define one schema-versioned `.lmbrain/HARNESSES.json` manifest as the versioned project source of intent.
- Let the Project Lead mutate it only through typed, controlled MCP commands with validation, atomic writes, and audit records.
- Keep host-specific materialization inside LMBrain adapters that own a narrow capability matrix and preserve unrelated native settings.
- Store operator approval outside the repository, keyed by canonical workspace identity and canonical manifest digest.
- Invalidate approval whenever material configuration changes; validation and preview remain read-only and do not require approval.
- Never permit secrets, credentials, absolute machine paths, arbitrary command strings, install scripts, hooks, or user-global configuration in the manifest.
- Keep machine concerns such as executable overrides and installed harness versions in local Settings, not the project manifest.
- Treat configured, prerequisite-ready, active, and healthy as distinct states; report `unknown` when a harness exposes no reliable evidence.

## Alternatives considered

### Let the Lead edit native harness files directly

Rejected because it duplicates host-specific formats, makes preservation and migration inconsistent, and grants a broad execution surface without a single validation boundary.

### Put all settings in application-local storage

Rejected because project environment intent would not be versioned, reviewable, portable, or available to the Lead and collaborators.

### Trust and apply repository configuration automatically

Rejected because opening an untrusted or modified repository could cause executable or environment-affecting changes without informed operator consent.

### Use free-form native configuration fragments

Rejected for the initial version because schema validation cannot reliably constrain arbitrary host configuration or maintain stable cross-version ownership.

## Consequences

- LMBrain gains a new governed artifact and must update its contract, bundled kit, diagnostics, migration guidance, and MCP surface.
- Each host needs an explicit supported-capability adapter and preservation tests.
- Operators see a preview and approve materialization once per manifest digest on each machine.
- Lead-authored intent stays portable while executable resolution, versions, and approval remain machine-local.
- Not every host will expose the same features or runtime observability; the UI must present those differences explicitly.

## Review conditions

Revisit when a supported harness introduces a stable project-environment schema, when secrets management is added to LMBrain, when approval identity must work across repository moves, or when the capability matrix needs a breaking schema revision.
