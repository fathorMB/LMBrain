---
id: ADR-010
# Note: Quote the title if it contains a colon
title: "Bootstrap pinned Pi MCP dependency during workspace preparation"
status: accepted
decision_date: 2026-07-10
decider: operator
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: [ADR-009]
superseded_by: []
links: [SPEC-029, SPEC-031]
tags: [architecture, pi, dependencies, workspace, security]
created: 2026-07-10
updated: 2026-07-10
activity:
  - date: 2026-07-10
    action: "created"
  - date: 2026-07-10
    action: "transitioned proposed -> accepted"
---
# Bootstrap pinned Pi MCP dependency during workspace preparation

## Context

[[ADR-009-pi-mcp-through-a-pinned-project-local-extension]] required operators to install `npm:pi-mcp-extension@1.5.0` manually. Real testing showed this produces a blocking session error after the operator has already selected Pi. Workspace opening also performs several synchronous preparation/data operations without visible progress, making the desktop app appear frozen.

## Decision

- Supersede [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]].
- During opening of a valid LMBrain workspace, check whether the exact project-local pin `npm:pi-mcp-extension@1.5.0` is ready.
- If missing, LMBrain runs `pi install -l npm:pi-mcp-extension@1.5.0 --approve` with the workspace as `cwd`.
- The command is idempotent and only runs when the exact pin is absent.
- Installation failure is non-blocking for workspace access: Pulse still opens and a persistent warning explains that Pi remains unavailable.
- Workspace preparation displays explicit progress stages while validation, Pi preparation, data loading, and watcher startup run.
- LMBrain still never upgrades to an unapproved version, installs Pi/Ollama/models, or mutates global Pi settings.

## Alternatives considered

### Keep manual installation

Rejected by the operator after usability testing: it delays failure until session launch and makes first use unnecessarily confusing.

### Install on first Pi session launch

Rejected because it makes session start slow and mixes package mutation with PTY creation. Workspace preparation is a clearer lifecycle boundary and can expose progress.

### Block workspace opening on install failure

Rejected because Pi is optional relative to core project-brain access.

## Consequences

- Opening a project may perform network/package work and write project-local `.pi/settings.json` plus `.pi/npm/` cache state.
- The exact package source and trust approval are application policy authorized by the operator; arbitrary packages remain forbidden.
- `.pi/npm/` is generated and ignored; `.pi/settings.json` remains visible/versionable because it may contain user-owned project configuration.
- A loading UI is required so the operation never appears as an unresponsive application.
- Supply-chain/version changes require another explicit ADR.

## Review conditions

Revisit when Pi adds native MCP, the approved extension/version changes, installation becomes non-idempotent, or workspace opening latency becomes unacceptable.
