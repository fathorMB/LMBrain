# Git Branching Strategy Capability Module

This module defines the declarative Git branching policy for LMBrain workspaces.

## Scope & Application

When `.lmbrain/BRANCHING.json` is configured, it documents project topology and branch naming intent for agents and operators.

## Branching Policy

- Topologies: `main-only`, `github-flow`, `git-flow`, `custom`.
- Declarative guidance only: LMBrain never executes automated `git` checkout, branch creation, merge, or push commands.
- Strategy mutations require operator authority and are audited to `.lmbrain/BRANCHING.audit.jsonl`.

## Branching Verbs

- `branching_strategy_get`: Read declared branching policy.
- `branching_strategy_set`: Set declared branching policy (operator-only).
