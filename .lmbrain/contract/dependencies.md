# Spec Hard Dependencies & Parking Capability Module

This module defines hard prerequisites and parking protocols for specs in LMBrain workspaces.

## Scope & Application

When a spec has strict ordering requirements (`depends_on`), this module enforces DAG constraints and parking semantics.

## Dependencies

- `depends_on: [SPEC-*]` defines hard prerequisites.
- Entries must be unique, non-self, and acyclic.
- Normal `spec_ready` and `spec_start` require all direct prerequisites to be `done`.
- `spec_dependencies_set` replaces dependencies and is allowed only in `backlog`.

## Spec Parking

- `spec_park`: Moves a spec from `ready` back to `backlog` with `readiness_invalidated: true`.
- Used when requirements, prerequisites, or priorities change after readiness approval.
- Parking preserves full history; returning to `ready` requires normal re-approval.

## Dependency Verbs

- `spec_dependencies_set`: Atomically update declared dependencies for a backlog spec.
- `spec_dependency_context`: Query direct and transitive prerequisites, dependents, and blockers.
- `spec_dependency_candidates`: Read-only candidate suggestion from prose or wikilinks.
- `spec_park`: Park a ready spec back to backlog with reason and optional revisit condition.
