---
id: ADR-009
# Note: Quote the title if it contains a colon
title: "Pi MCP through a pinned project-local extension"
status: accepted
decision_date: 2026-07-10
decider: operator
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: []
superseded_by: []
links: [SPEC-029, REVIEW-031]
tags: [architecture, pi, mcp, security, dependencies]
created: 2026-07-10
updated: 2026-07-10
activity:
  - date: 2026-07-10
    action: "created"
  - date: 2026-07-10
    action: "transitioned proposed -> accepted"
---
# Pi MCP through a pinned project-local extension

## Context

[[SPEC-029-add-pi-agent-sessions-through-ollama]] adds Pi as an LMBrain agent host through the local Ollama gateway. Pi deliberately has no MCP client in its core distribution, while LMBrain host parity requires access to repository-scoped controlled mutation verbs from `lmbrain-mcp`.

Pi extensions execute with the user's permissions and may perform package lifecycle operations. An unpinned or automatically installed dependency would make agent startup non-deterministic and expand LMBrain's supply-chain authority.

## Decision

- Use `pi-mcp-extension` version `1.5.0` as the approved Pi MCP client.
- The exact package source is `npm:pi-mcp-extension@1.5.0`.
- Installation is manual and project-local: `pi install -l npm:pi-mcp-extension@1.5.0`.
- LMBrain never installs, upgrades, removes, or approves/trusts the package.
- LMBrain owns only the `mcpServers.lmbrain` entry in generated project `.pi/mcp.json`; unrelated Pi configuration is preserved.
- Pi launch is blocked before PTY allocation unless an offline `pi list` check reports the exact pin and Ollama/model readiness succeeds.
- App-launched Pi receives `PI_OFFLINE=1`, `PI_SKIP_VERSION_CHECK=1`, and `PI_TELEMETRY=0`.

## Alternatives considered

### First-party Pi MCP adapter

Deferred. It would remove the third-party runtime dependency but requires LMBrain to own MCP transport lifecycle, cancellation, schema conversion, and Pi extension compatibility. This is materially larger than the host integration.

### Unpinned or automatically installed extension

Rejected. It permits dependency drift or package mutation during session startup and conflicts with LMBrain's operator-controlled model.

### Pi without MCP

Rejected as host parity. Pi could edit application source but could not follow the controlled artifact workflow required by `.lmbrain/AGENT.md` and `.lmbrain/CONTRACT.md`.

## Consequences

- Operators must explicitly review, trust, and install the exact package once per project.
- Pi startup fails safely with setup guidance when the prerequisite is missing or different.
- `.pi/mcp.json` is machine-specific generated state and is ignored narrowly; other `.pi/` resources remain user-owned.
- Changing the approved extension version requires an explicit decision update or replacement ADR plus compatibility/security verification.
- Runtime parity still requires a later safe-window smoke test; compilation alone is not sufficient evidence.

## Review conditions

Revisit if Pi adds a native MCP client, the extension changes ownership/security posture or config schema, the exact version becomes unavailable, or LMBrain decides to own a first-party Pi adapter.
