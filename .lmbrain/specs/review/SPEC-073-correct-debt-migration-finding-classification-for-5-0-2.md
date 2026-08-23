---
id: SPEC-073
# Note: Quote the title if it contains a colon
title: "Correct debt migration finding classification for 5.0.2"
status: review
kind: bugfix
priority: critical
area: core-tooling
milestone: 5.0.2
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
    action: "set recommended_agent"
  - date: 2026-08-23
    action: "set tags"
  - date: 2026-08-23
    action: "set effort"
  - date: 2026-08-23
    action: "transitioned backlog -> ready"
  - date: 2026-08-23
    action: "transitioned ready -> working"
  - date: 2026-08-23
    action: "transitioned working -> review"
---
# Title

Correct debt migration finding classification for 5.0.2

## Objective

Make `debt_migration_preview` decide every mechanically decidable `FINDING-*` token on a real 5.0.1 workspace, so the digest-bound `debt_migrate` transaction can run, while keeping fail-closed refusal for the shapes that are genuinely two objects.

## Context

On the reference 5.0.1 workspace (67 durable artifacts, 63 reviews) `debt_migration_preview` reports 88 blocking issues in two classes and produces no preview. Both classes are misclassifications, not ambiguity.

Class A (31 issues, 20 reviews): a review declares a bare `FINDING-NNN` in its `## Findings` section while a durable `FINDING-NNN` artifact exists. SPEC-072 treated that as a collision because its acceptance criteria said so. The criterion was wrong: this is the workspace's normal promotion convention, where the review declares the finding it promoted to a durable artifact. One object, not two.

Class B (57 issues): reviews `REVIEW-001` through `REVIEW-009` declare qualified `REVIEW-NNN-FINDING-MMM` findings but write bare `FINDING-MMM` in their own prose. Durable resolution fails for those numbers, so the preflight refuses them, even though the declaring review defines them.

Measured on the reference workspace, the two declaration conventions partition cleanly and temporally with zero exceptions: `REVIEW-001` through `REVIEW-009` use the qualified form (48 declarations, numbers `001` to `008`, never durable-backed); `REVIEW-021` onward uses the bare form (31 declarations, numbers `009` and above, always durable-backed). No review mixes conventions.

## Scope
### Included

- Ordered per-review classification of `FINDING-*` tokens in `lmbrain-core/src/debt_migration.rs`.
- A per-review local symbol table built from qualified declarations, resolved before durable resolution.
- Review-scoped `RF-MMM` identifiers that preserve the declared finding number.
- Auditable preview output: every decided token, its replacement, its classification, and its occurrence count.
- Regression coverage for both corrected classes and for the adversarial shapes that must still fail closed.
- Operator documentation and 5.0.2 release metadata.

### Excluded

- Any relaxation of digest binding, explicit operator confirmation, or atomic staged migration.
- Automatic edits to review prose to resolve ambiguity.
- Changes to the durable artifact schema or to the `DEBT-*` target contract.
- Re-litigating the SPEC-072 scaffolding exclusion or batch reporting behaviour; both are correct and must be preserved.

## Existing-project analysis

`collect_review_id_mappings` in `lmbrain-core/src/debt_migration.rs` classifies each token found by `migration_token_regex`. Qualified tokens are consumed atomically and assigned `RF-*` identifiers from a per-review encounter-order counter. Bare tokens are matched against the durable index and against `review_local_bare_tokens`, a set collected from the review's `## Findings` section; the combination `(durable = Some, local = true)` is reported as ambiguous and `(durable = None, local = false)` is reported as unresolved. Those two arms produce Class A and Class B respectively. `DebtMigrationReference` records one row per distinct token per file but no occurrence count, so an operator cannot reconcile the preview against the rewrite volume.

## Technical proposal

Per review file, in this order:

1. Build the review's local symbol table from its qualified declarations `REVIEW-NNN-FINDING-MMM`, each mapping to `RF-MMM` scoped to that review.
2. A token matching the qualified form is review-local and maps to that review's `RF-MMM`. It is consumed as one token; its numeric tail is never re-matched.
3. A bare `FINDING-MMM` whose number is in this review's local symbol table is review-local and maps to `RF-MMM`.
4. A bare `FINDING-NNN` (declaration, prose reference, or `[[wikilink]]`) that resolves to an existing durable artifact is durable and maps to `DEBT-NNN`.
5. Anything else is ambiguous and fails closed: a token that resolves to nothing, or a number declared both qualified and bare in the same review while a durable artifact of that number exists.

Rule 3 precedes rule 4 so that a workspace whose local and durable number ranges overlap binds local-first instead of silently binding a review-local reference to an unrelated durable artifact.

`RF-*` identifiers derive from the declared number rather than from encounter order, so an operator auditing the preview can match `FINDING-013` to `RF-013` without reconstructing the scan order.

`DebtMigrationReference` gains an `occurrences` count and the preview schema version moves to `3`. Review files are inventoried once, by the review analysis, so occurrence counts are neither doubled nor lost.

## Files and areas involved

- `lmbrain-core/src/debt_migration.rs`
- `lmbrain-mcp/src/lib.rs` (public preview contract coverage)
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- Workspace crate, application, and kit version metadata

## Acceptance criteria
- [x] All qualified `REVIEW-NNN-FINDING-MMM` declarations map to review-scoped `RF-MMM`, never to a durable artifact.
- [x] A bare declaration backed by a durable artifact maps to `DEBT-NNN` and is not reported as a collision.
- [x] Bare prose references to a review's own qualified findings map to that review's `RF-MMM`, not to an unrelated durable artifact.
- [x] `[[FINDING-NNN]]` wikilinks resolving to durable artifacts map to `DEBT-NNN`.
- [x] A review declaring the same number both qualified and bare while a durable artifact of that number exists is still ambiguous and still fails closed.
- [x] A bare reference resolving to nothing at all is still reported and still fails closed.
- [x] Local-first ordering holds when local and durable number ranges overlap.
- [x] Batch reporting is preserved: every remaining issue is named in one pass.
- [x] The preview exposes the decided mapping and occurrence count for every rewritten token.
- [x] `debt_migrate` remains digest-bound, operator-confirmed, and atomic.
- [x] Migration and changelog documentation state the ordered rule, and release metadata is aligned at 5.0.2.

## Implementation plan
1. Replace the bare-token set with a per-review declaration model separating qualified declarations from bare declarations.
2. Implement the five-step ordered classification, deriving `RF-*` from the declared number.
3. Add occurrence counting to the preview reference inventory and bump the preview schema version.
4. Add regression coverage for both corrected classes and every fail-closed shape.
5. Update documentation and release metadata, then run the full verification gates.

## Required verification

<!-- Canonical form: ID | kind=executable|manual|operator | owner=agent|kit|lead|operator | phase=before-submit|before-done | evidence=transcript|observation|artifact | requirement -->
- [x] DEBT-MIGRATION-CORE | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-core` test suite including every new classification fixture.
- [x] DEBT-MIGRATION-MCP | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-mcp` test suite and confirm the public preview contract.
- [x] WORKSPACE-QUALITY | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run formatting, Clippy with `-D warnings`, ESLint, the frontend test suite, and version alignment.
- [x] DIFF-AUDIT | kind=manual | owner=lead | phase=before-done | evidence=observation | Confirm the diff preserves digest binding, explicit confirmation, atomic writes, and every fail-closed refusal.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

The migration rewrites roughly 1736 references across roughly 261 files, including shipped source comments and review history, so a wrong mapping corrupts source and history at once. The mitigation is the ordered rule, local-first precedence, the retained fail-closed arms, and a preview inventory complete enough for an operator to audit before confirming.

This spec corrects a specification error in SPEC-072, not an implementation defect. SPEC-072 implemented its stated acceptance criterion faithfully; the criterion described the workspace's promotion convention as a collision.

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

- Replaced the per-review bare-token set with `ReviewLocalDeclarations`, which separates the numbers a review declares in the qualified `REVIEW-NNN-FINDING-MMM` form from the numbers it declares bare in its findings section.
- Implemented the ordered classification. A review's qualified declarations form its local symbol table and are consulted before durable resolution, so bare prose references to a review's own findings resolve locally and an overlapping durable number range cannot capture them.
- Corrected the Class A rule: a bare declaration backed by a durable artifact is that durable artifact. The only shape still refused as a genuine collision is a number declared in both forms in the same review while a durable artifact of that number exists.
- Derived `RF-*` identifiers from the declared finding number instead of an encounter-order counter, so `FINDING-013` and `REVIEW-021-FINDING-013` both map to `RF-013`.
- Added an `occurrences` count to `DebtMigrationReference`, aggregated the inventory instead of deduplicating it, and inventoried each review artifact exactly once. Preview schema version moved to `3`.
- Left every fail-closed arm intact: unresolved references, cross-review qualified references, and genuine collisions still abort in one aggregated report, and `debt_migrate` is unchanged.

### Files changed

- `lmbrain-core/src/debt_migration.rs`
- `lmbrain-mcp/src/lib.rs`
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- `.lmbrain/VERSION`, `kit/.lmbrain/VERSION`, `package.json`, `lmbrain-core/Cargo.toml`, `lmbrain-mcp/Cargo.toml`, `src-tauri/Cargo.toml`, `Cargo.lock`

### Verification performed

- Full `lmbrain-core` suite: 257 tests passed (182 unit including 13 debt-migration fixtures, 52 transitions, 23 verification).
- Full `lmbrain-mcp` suite: 33 tests passed (30 library, 3 protocol).
- Clippy for both crates with `--all-targets -- -D warnings`: clean, exit 0, on the first run.
- ESLint clean; frontend suite 392 tests across 55 files passed; `git diff --check` clean; version alignment reports 5.0.2.
- `rustfmt --check` clean on both touched Rust files. Repository-wide `cargo fmt --all --check` is not clean on `main` and was not made a gate here; no unrelated file was reformatted.
- End-to-end read-only preview against the real 5.0.1 reference workspace (67 durable artifacts, 68 review artifacts): the 88 previously reported issues in the two corrected classes are gone. 48 distinct qualified declarations map to review-scoped `RF-*`, 33 distinct bare review-local tokens map to `RF-*`, 555 distinct durable tokens map to `DEBT-*`, and every `RF-*`/`DEBT-*` preserves the source number. The inventory reports 636 rows covering 1553 token occurrences.

### Verification transcript

<!-- Required before spec_submit. Paste actual command/result output in a fenced block, or use approved `spec_verify` gates. Predictions and summaries are not execution evidence. -->

```text
$ CARGO_TARGET_DIR=target/mcp-verify cargo test -p lmbrain-core
test result: ok. 182 passed; 0 failed
test result: ok. 52 passed; 0 failed
test result: ok. 23 passed; 0 failed

$ CARGO_TARGET_DIR=target/mcp-verify cargo test -p lmbrain-mcp
test result: ok. 30 passed; 0 failed
test result: ok. 3 passed; 0 failed

$ CARGO_TARGET_DIR=target/mcp-verify cargo test -p lmbrain-core debt_migration
test result: ok. 13 passed; 0 failed; 169 filtered out

$ CARGO_TARGET_DIR=target/mcp-verify cargo clippy -p lmbrain-core -p lmbrain-mcp --all-targets -- -D warnings
clippy exit: 0

$ rustfmt --check --edition 2021 lmbrain-core/src/debt_migration.rs
core fmt exit: 0

$ pnpm lint
$ eslint .
lint exit: 0

$ pnpm test
 Test Files  55 passed (55)
      Tests  392 passed (392)

$ git diff --check
diff --check exit: 0

$ node scripts/check-version.mjs
LMBrain workspace crates, app, and kit are aligned at v5.0.2.

$ debt_migration_preview --root <5.0.1 reference workspace>   # before this change: 88 issues
debt migration preflight failed: 4 issue(s):
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-010-...md belongs to REVIEW-009, not REVIEW-010
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-012-...md belongs to REVIEW-009, not REVIEW-012
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-024-...md belongs to REVIEW-009, not REVIEW-024
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/superseded/REVIEW-023-...md belongs to REVIEW-009, not REVIEW-023

$ debt_migration_preview --root <same workspace, the 4 out-of-scope shapes neutralised in a scratch copy>
schema 3 source 5.0.1 -> 4.2.0
items 215 scaffolding 12
distinct reference rows 636 total occurrences 1553
rows by class {'durable': 555, 'review-local': 81}
occurrences by class {'durable': 1386, 'review-local': 167}
reviews with local RF mappings 12
distinct qualified-local tokens 48 | bare-local 33 | durable 555 (in reviews: 115)
RF preserves number: True
DEBT preserves number: True
```

### Deviations from the specification

None within scope. Two blocking shapes remain in the reference workspace and are deliberately still refused, because resolving either requires a product decision rather than a mechanical rule:

1. **Cross-review qualified references** (4 occurrences of `REVIEW-009-FINDING-004` cited by four other reviews). The referenced object is decidable, but `RF-*` is review-scoped, so rewriting a foreign reference to a bare `RF-004` inside another review would silently rebind it to that review's own `RF-004`. A qualifier-preserving target form has to be chosen before this can be classified.
2. **Free-form `origin_ref` on durable artifacts whose `origin_artifact` is a review** (for example `origin_ref: "REVIEW-053-08"` and `origin_ref: "REVIEW-052 findings 01 and 02"`). These are prose, not identifiers, so no mechanical rule maps them to an `RF-*`. This path also raises its failure outside the aggregated preflight report, so it surfaces one artifact at a time rather than in one pass.

Both are reported to the operator rather than guessed. The MCP suite used an isolated Cargo target directory because a running MCP server holds a lock on the default Windows executable path; this changes only build-artifact placement.

### Handoff status
- [x] Ready for Project Lead review
