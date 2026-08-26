# Git Branching Strategy Capability Module

This module defines the declarative Git branching policy for LMBrain workspaces and how agents follow it.

## Scope & Application

`.lmbrain/BRANCHING.json` documents the project's Git topology, branch naming, authority, and commit triggers. LMBrain never executes automated `git` checkout, branch creation, merge, or push commands: the declared strategy binds the agents and operators who do.

## Following the strategy

- Read the strategy with `branching_strategy_get` before preparing any spec assignment, and name the target branch in the assignment, derived from `topology` and `branch_naming`.
- Respect `authority`: only actors the strategy names may push the branches it protects, and `require_pr_for_merge` is binding when set.
- Respect `commit_triggers`: commit when a declared trigger fires and not otherwise. In particular, `commit_on_doc_change: false` means artifact edits (specs, reviews, debts, ADRs, status) do not get commits of their own.
- When the strategy reports `absent`, ask the operator to declare one with `branching_strategy_set` before the first spec assignment; do not improvise a topology.

## Branching Policy

- Topologies: `main-only`, `github-flow`, `git-flow`, `custom`.
- Strategy mutations require operator authority and are audited to `.lmbrain/BRANCHING.audit.jsonl`.

## Branching Verbs

- `branching_strategy_get`: Read declared branching policy.
- `branching_strategy_set`: Set declared branching policy (operator-only).
