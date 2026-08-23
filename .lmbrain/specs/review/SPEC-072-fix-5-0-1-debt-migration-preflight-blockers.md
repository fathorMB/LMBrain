---
id: SPEC-072
# Note: Quote the title if it contains a colon
title: "Fix 5.0.1 debt migration preflight blockers"
status: review
kind: bugfix
priority: critical
area: core-tooling
milestone: 5.0.1
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
# Implementation estimate. Required before this spec can become `ready`.
# capability_tier: luna | terra | sol   (expected change footprint)
# thinking_level: minimal | standard | extended | maximum (defaults from the tier)
capability_tier: sol
thinking_level: extended
effort_observations: []
depends_on: []
dependency_events: []
parking_events: []
skills: []
verification_gates: []
related_decisions: []
links: []
created: 2026-08-23
updated: 2026-08-23
tags: [migration, debt, release]
activity:
  - date: 2026-08-23
    action: "created"
  - date: 2026-08-23
    action: "transitioned backlog -> ready"
  - date: 2026-08-23
    action: "transitioned ready -> working"
  - date: 2026-08-23
    action: "transitioned working -> review"
---
# Title

Fix 5.0.1 debt migration preflight blockers

## Objective

Restore the fail-closed `FINDING-*` to `DEBT-*` migration for upgraded 4.x workspaces by excluding non-artifact scaffolding, classifying review-local and durable references by explicit precedence, and reporting the complete preflight error set.

## Context

LMBrain 5.0.0's `debt_migration_preview` currently rejects kit-shipped README scaffolding as malformed artifacts and treats resolvable durable wikilinks in review prose as ambiguous. Since `debt_migrate` is digest-bound to a successful preview, both defects block the migration entirely.

## Scope
### Included

- Shared artifact discovery used by validation and debt migration.
- Deterministic token classification for qualified review-local and bare durable/local references.
- Aggregated malformed-source and ambiguous/unresolved-input reporting.
- Regression coverage in `lmbrain-core` and MCP exposure where applicable.
- Operator-facing migration/changelog documentation and patch-version metadata for 5.0.1.

### Excluded

- Compatibility aliases for legacy mutation tools.
- Automatic edits intended to resolve genuinely ambiguous user prose.
- Changes to digest binding, explicit confirmation, or atomic migration semantics.

## Existing-project analysis

The migration implementation lives in `lmbrain-core/src/debt_migration.rs` and is exposed by `lmbrain-mcp/src/registry.rs`. Ordinary artifact discovery already filters scaffolding, while the migration walks every Markdown file. The current review preflight also rejects any review containing `[[FINDING-*]]` before resolving it against durable artifacts.

## Technical proposal

Reuse one canonical artifact-file eligibility predicate for discovery and migration. Parse qualified `REVIEW-NNN-FINDING-MMM` tokens atomically before bare tokens, then classify bare IDs using the durable artifact index and that review's local findings section. Durable-only and local-only references are deterministic; collisions or unresolved tokens remain errors. Accumulate all preflight issues, sort them deterministically, and return one failure without weakening the migrate transaction.

## Files and areas involved

- `lmbrain-core/src/debt_migration.rs`
- Shared discovery implementation under `lmbrain-core/src/`
- `lmbrain-mcp` regression tests if the public tool contract needs coverage
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- Version metadata consistently used by release checks

## Acceptance criteria
- [x] Scaffolding READMEs and templates are excluded through shared artifact discovery, while malformed artifact-shaped sources still fail closed.
- [x] Reviews with no local findings and resolvable bare durable wikilinks preview successfully and map those references to `DEBT-*`.
- [x] Qualified `REVIEW-NNN-FINDING-MMM` references map to review-scoped `RF-*` identifiers without separately matching their tails.
- [x] A bare ID present both as a durable artifact and a local finding in the same review is reported as ambiguous and migration remains refused.
- [x] One preflight run reports all malformed, ambiguous, and unresolved items deterministically.
- [x] Preview output exposes every classification used for rewriting; `debt_migrate` remains digest-bound, explicitly confirmed, and atomic.
- [x] Migration and changelog documentation explain the precedence rules and 5.0.1 fix.

## Implementation plan
1. Reproduce both failures and trace discovery/tokenization/classification paths.
2. Centralize source eligibility and implement precedence-based classification with complete issue aggregation.
3. Add focused regression fixtures plus invariants for digest binding and atomic refusal.
4. Update operator documentation and release version metadata.
5. Run targeted and workspace-wide checks before push and pull request creation.

## Required verification

<!-- Canonical form: ID | kind=executable|manual|operator | owner=agent|kit|lead|operator | phase=before-submit|before-done | evidence=transcript|observation|artifact | requirement -->
- [x] DEBT-MIGRATION-CORE | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-core` test suite including all new migration fixtures.
- [x] DEBT-MIGRATION-MCP | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-mcp` test suite and verify public preview/migrate registration and confirmation behavior.
- [x] WORKSPACE-QUALITY | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run repository formatting, linting, version consistency, and relevant workspace tests.
- [x] DIFF-AUDIT | kind=manual | owner=lead | phase=before-done | evidence=observation | Confirm the diff preserves fail-closed classification, digest binding, explicit confirmation, and atomic writes.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

The rewrite surface includes historical Markdown and source comments, so false classification is data-corrupting. Deterministic precedence, explicit preview inventory, and collision refusal are mandatory. No open product decision is expected; this is a correctness repair to the documented 5.0.0 migration contract.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.
- **V3 context-economy:** Read mandatory policy files (`QUALITY.md`, `CONTRACT.md`, `AGENT.md`) first. Use `lmbrain_spec_context` for a compact handoff context. Expand to full artifacts and source code only when the context pack points to them or verification requires it. Record evidence when you expand scope beyond the context pack.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

- Centralized artifact/scaffolding eligibility in workspace discovery and reused it in the migration preflight.
- Replaced the blanket review-wikilink rejection with qualified-first, durable/local-index classification and deterministic complete issue aggregation.
- Added digest-bound `reference_mappings` plus a separate `scaffolding_items` inventory to preview schema v2.
- Preserved confirmed staged migration behavior and allowed only byte-identical kit-scaffolding reconciliation; conflicting destinations still abort before the atomic swap.
- Added regression fixtures for all reported cases and aligned release metadata/documentation at 5.0.1.

### Files changed

- `lmbrain-core/src/debt_migration.rs`, `lmbrain-core/src/workspace_index.rs`, `lmbrain-core/src/lib.rs`
- `lmbrain-mcp/src/lib.rs`
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- Workspace crate/app/kit version manifests and `Cargo.lock`

### Verification performed

- Full `lmbrain-core` suite: 253 tests passed (178 unit, 52 transitions, 23 verification), plus doc tests.
- Full `lmbrain-mcp` suite: 33 tests passed (30 library, 3 protocol), plus doc tests.
- Targeted migration suite: 9 tests passed, covering scaffolding, durable wikilinks, qualified-local precedence, collisions, complete issue aggregation, confirmation/digest binding, and atomic destination handling.
- Rust formatting check, ESLint, version alignment, `git diff --check`, and `lmbrain_validate` passed.
- CI remediation reran Clippy for all core/MCP targets with `-D warnings`; the original `type_complexity` failure is resolved by a named internal analysis structure rather than a lint suppression.
- Manual diff audit confirmed that confirmation, digest comparison, staging validation, backup/swap, and rollback paths were not weakened.

### Verification transcript

<!-- Required before spec_submit. Paste actual command/result output in a fenced block, or use approved `spec_verify` gates. Predictions and summaries are not execution evidence. -->

```text
$ cargo test -p lmbrain-core
test result: ok. 178 passed; 0 failed
test result: ok. 52 passed; 0 failed
test result: ok. 23 passed; 0 failed

$ CARGO_TARGET_DIR=target/mcp-verify cargo test -p lmbrain-mcp
test result: ok. 30 passed; 0 failed
test result: ok. 3 passed; 0 failed

$ cargo test -p lmbrain-core debt_migration -- --nocapture
test result: ok. 9 passed; 0 failed

$ rustfmt --check --edition 2021 lmbrain-core/src/debt_migration.rs lmbrain-core/src/workspace_index.rs
passed

$ cargo clippy -p lmbrain-core -p lmbrain-mcp --all-targets -- -D warnings
passed

$ pnpm lint
passed

$ node scripts/check-version.mjs
LMBrain workspace crates, app, and kit are aligned at v5.0.1.

$ git diff --check
passed (no output)

$ lmbrain_validate
{"unique_ids":true}
```

### Deviations from the specification

None. The MCP suite used an isolated Cargo target directory because the running repository-scoped MCP server locks its default Windows executable; this changes only build-artifact placement, not test scope or behavior.

### Handoff status
- [x] Ready for Project Lead review
