---
id: ADR-004
title: Controlled-mutation engine for .lmbrain artifacts, delivered to agents via MCP
status: accepted
decision_date: 2026-06-23
decider: user
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: []
superseded_by: []
links: [ADR-001, ADR-002, SPEC-017]
tags: [architecture, mcp, transitions, core, workflow]
---

# Controlled-mutation engine for `.lmbrain` artifacts, delivered to agents via MCP

## Context

LMBrain's state rules — the allowed status transitions per artifact and the cross-artifact invariants — currently live **only as prose** in `CONTRACT.md`, plus one partial, fragile implementation in the app ([[ADR-002-in-app-artifact-status-writes]] introduced operator-driven writes via `set_artifact_status`). That function covers only `proposed → {ready|rejected|approved|active|inactive}` for spec/adr/agent/mcp, performs the folder move for specs only, does **not** handle tasks, and rewrites frontmatter with naive line-based string manipulation.

Agents (Project Lead, specialists, reviewer) run **outside** the desktop app and today mutate artifacts by **editing markdown by hand**. With no executable encoding of the rules, agents drift: tasks never moved to `in-progress`, tasks `planned` without a ready spec, `recommended_agent` left as the `AGENT-XXX` placeholder, and the Project Lead implementing scaffolding after ADR approval. Each was patched individually (template defaults, prompt wording, diagnostics), but the **root cause is structural**: there is no single, tested place that performs controlled mutations.

`ADR-002` already established operator-driven, per-artifact, contract-honoring writes inside the app and explicitly rejected a "generic set-any-status editor" in favor of constrained actions. This decision **extends that principle to agents** and unifies the implementation.

## Decision

Introduce a single **controlled-mutation engine** as the one source of truth for changing `.lmbrain/` state, and expose it to agents as **MCP tools**.

Principles:

- **Shared, tauri-free core.** Extract a `lmbrain-core` crate holding the domain models, the markdown/frontmatter parser, a `transitions` module (per-artifact state machines + cross-artifact invariants + a surgical frontmatter mutator), and the path-safety/atomic-write primitives. Both the Tauri app and the agent-facing surface depend on it; `set_artifact_status` becomes a thin wrapper over the core.
- **Agent surface is MCP.** Agents operate the system through an `lmbrain-mcp` server exposing **specific per-verb tools** (e.g. `task_start`, `task_complete`, `spec_approve`, `review_accept`, `adr_accept`) plus a few **read tools** (e.g. `validate`, `list_ready_handoffs`, `get_artifact`). Specific, tightly-schemaed tools are preferred over one generic `transition(artifact, verb)` because they constrain LLM misuse. No CLI is shipped.
- **Scope.** State transitions **+** artifact creation (with atomic progressive ID allocation) **+** a small set of targeted field setters (e.g. `recommended_agent`, task evidence).
- **Surgical frontmatter edits.** Mutations change only the targeted fields (status, `updated`, appended activity), preserving key order, comments, and human formatting. No full `serde_yaml` round-trip (which reorders keys and drops comments). Writes are atomic (temp-file + rename); status-directory artifacts are physically moved to match their new status.
- **Invariants are hard-blocking, with a recorded override.** Contract violations fail the operation. A deliberate override is possible via `force: true` + a required `reason`, which is recorded in the artifact's audit trail. The same invariant functions back the app diagnostics, so UI, MCP, and tests state the same thing.
- **Audit trail.** Every mutation appends a structured activity entry (`{verb, actor, timestamp}`) to the frontmatter and bumps `updated`; overrides additionally record their justification in the artifact body.
- **Distribution.** `lmbrain-mcp` is built as a per-OS binary (like the installers) and the kit bootstrap registers it in the agent host's MCP configuration, scoped to the repository root via the existing `PathGuard`.

## Alternatives considered

- **CLI instead of MCP.** Simpler to test and ship, invoked via shell. Rejected as the agent surface in favor of native MCP tool-calls (tighter schemas, less prompt plumbing); the shared core keeps a CLI possible later at low cost.
- **Generic "set any field/status" tool.** Maximum flexibility, but invites contract violations and LLM misuse — same reason `ADR-002` rejected a generic editor. Rejected in favor of specific verbs.
- **`serde_yaml` round-trip for frontmatter.** Less code, but reorders keys and discards comments, degrading human-authored files and producing noisy diffs. Rejected in favor of surgical edits.
- **Warn-only invariants.** Lower friction, but reintroduces the drift this decision exists to prevent. Rejected in favor of hard-block + recorded override.

## Consequences

### Positive
- One tested implementation of the state rules, reused by operator (app), agents (MCP), and the diagnostics.
- Structurally prevents the drift classes seen in BUG-001/003/004 once the kit prompts point agents at the tools.
- Auditable history of who changed what.

### Constraints / costs
- A workspace refactor (`lmbrain-core` extraction) before behavior changes.
- A new MCP server, its per-OS build in CI, and kit bootstrap registration.
- Kit prompts (`AGENT.md`, handoff, bootstrap) must be migrated to instruct agents to use the tools instead of editing markdown.

## Review conditions

Revisit if MCP delivery proves impractical in the agent host (fall back to a CLI over the same core), or if surgical frontmatter editing cannot robustly preserve arbitrary human formatting at acceptable complexity (reconsider a structured round-trip with formatting preservation).
