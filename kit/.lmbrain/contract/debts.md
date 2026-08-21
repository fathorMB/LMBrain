# First-Class Debts Capability Module

This module defines the optional technical and design debt tracking subsystem for LMBrain workspaces.

## Scope & Application

When durable findings survive their originating spec or direct debt items are recorded, debts are tracked in `debts/<status>/` using `DEBT-*` IDs.

## Lifecycle & Statuses

- **open**: Initial state for newly created debts.
- **planned**: Linked to one or more active target specs (`target_specs`).
- **deferred**: Intentionally postponed with explicit rationale and revisit condition.
- **resolved**: Terminal state requiring resolution references and verification evidence.
- **accepted-risk**: Operator-only terminal state with rationale and revisit condition.
- **superseded**: Retired in favor of a successor debt or obsolescence rationale.

Reopening `resolved` or `accepted-risk` debts is operator-only. `superseded` history is never reopened.

## Debt Verbs

- `debt_create`: Allocate and create a new open debt.
- `debt_plan`: Link target specs to an open or deferred debt.
- `debt_defer`: Defer a debt with rationale and revisit condition.
- `debt_resolve`: Close a debt with resolution references and evidence.
- `debt_accept_risk`: Operator-governed risk acceptance.
- `debt_supersede`: Supersede with successor reference or rationale.
- `debt_reopen`: Operator-governed reopening of resolved/accepted debts.
- `debt_context`: Bounded query for relations and blockers.
- `debt_candidates`: Read-only candidate inventory from legacy findings.
- `debt_migration_preview`: Preview migration of legacy findings to first-class debts.
- `debt_migrate`: Execute digest-confirmed debt migration.
