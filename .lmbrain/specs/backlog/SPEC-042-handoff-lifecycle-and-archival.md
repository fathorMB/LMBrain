---
id: SPEC-042
title: "Handoff lifecycle and archival"
status: backlog
kind: feature
priority: high
area: core
milestone: M-06
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-17
updated: 2026-07-17
tags: [core, rust, mcp]
---

# Handoff lifecycle and archival

## Objective
Implement physical archival and controlled MCP transitions for session handoffs, ensuring only a single `ready` handoff exists at any time in `.lmbrain/handoffs/active/` and moving consumed, superseded, or archived handoffs automatically to `.lmbrain/handoffs/archive/`.

## Context
In LMBrain 2.9.1, handoffs are created in `.lmbrain/handoffs/active/` but they never transition or move physically on the filesystem. This leaves old handoffs in `active/` and allows multiple legacy handoffs to remain in `ready` state, violating the single-ready-handoff invariant during validation.

## Scope
### Included
- Update `lmbrain-core` `ArtifactKind::Handoff` logic to support physical moves for status updates (`moves_for_status -> true`).
- Implement the transition path mapping in `transitions.rs` for Handoff: `ready -> consumed`, `ready -> superseded`, `ready -> archived`.
- Customize `destination_for` in `transitions.rs` to route `ready` handoffs to `.lmbrain/handoffs/active/` and others to `.lmbrain/handoffs/archive/`.
- Expose three new transition tools in `lmbrain-mcp`'s `tools()` list: `handoff_consume`, `handoff_supersede`, and `handoff_archive`.
- Enforce the `single_ready_handoff` invariant during creation inside `transitions::create`.
- Add test coverage for handoff transitions, physical moving, and creation invariants.

### Excluded
- Automated execution of migrations during Tauri application startup (this remains manual or guided by operator commands).

## Existing-project analysis
Handoffs currently use `ArtifactKind::Handoff`. Its base directory is `handoffs/active` and `moves_for_status` returns `false`. The core transitions library does not map `Handoff` in `allowed` transitions. `lmbrain_create` calls `single_ready_handoff` during creation, but it is not fully guarded if multiple calls occur or if legacy files exist.

## Technical proposal
1. **Modify `ArtifactKind` behavior:**
   In `lmbrain-core/src/transitions.rs`:
   - Change `moves_for_status(self)` to return `true` for `Self::Handoff`.
   - Update `destination_for` to determine the directory for `Handoff`:
     - If `target == "ready"`, folder is `active`.
     - If `target` is `"consumed"`, `"superseded"`, or `"archived"`, folder is `archive`.
     - Return path `.lmbrain/handoffs/<folder>/<filename>`.
2. **Define Transitions:**
   In `transitions.rs::allowed(kind, from, to)`:
   - For `ArtifactKind::Handoff`, allow transitions from `"ready"` to `"consumed"`, `"superseded"`, and `"archived"`.
3. **Register MCP Tools:**
   In `lmbrain-mcp/src/main.rs`:
   - Register `handoff_consume`, `handoff_supersede`, and `handoff_archive` as tools mapping to their respective target statuses.
4. **Handoff Creation Invariants:**
   Verify `single_ready_handoff` is checked early in `lmbrain-core/src/transitions.rs::create` before files are written.

## Files and areas involved
- `lmbrain-core/src/transitions.rs`
- `lmbrain-mcp/src/main.rs`
- `lmbrain-core/tests/transitions.rs`

## Acceptance criteria
- [ ] Transitioning a handoff using `handoff_consume` moves the file from `.lmbrain/handoffs/active/HANDOFF-XXX.md` to `.lmbrain/handoffs/archive/HANDOFF-XXX.md` and updates its frontmatter status to `consumed`.
- [ ] Transitioning a handoff using `handoff_supersede` moves it to `archive/` with status `superseded`.
- [ ] Transitioning a handoff using `handoff_archive` moves it to `archive/` with status `archived`.
- [ ] The `lmbrain_create` tool fails if a handoff is created with status `ready` when another ready handoff already exists in `handoffs/active/`.
- [ ] `cargo test` in `lmbrain-core` and `lmbrain-mcp` executes successfully and covers the transition matrix.

## Implementation plan
1. Update `lmbrain-core/src/transitions.rs` to implement `moves_for_status, transitions mapping, and `destination_for` custom routing for `Handoff`.
2. Update `lmbrain-mcp/src/main.rs` to register the new tools.
3. Write unit and integration tests in `lmbrain-core/tests/transitions.rs` checking handoff transitions.

## Required verification
### Automated Tests
- `cargo test --package lmbrain-core`
- `cargo test --package lmbrain-mcp`

### Manual Verification
- Execute `handoff_consume` via MCP tool call on a ready handoff and check that it is physically moved to the archive directory with its status updated.

## Production quality and documentation
- Follow [[QUALITY]].
- Document version 3.0.0 migration steps in `.lmbrain/MIGRATIONS.md`.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete.

## Implementation evidence
> Filled in by the specialist after completion.
