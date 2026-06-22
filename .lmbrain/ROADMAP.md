---
title: Roadmap
updated: 2026-06-23
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
