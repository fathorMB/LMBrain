---
id: SPEC-074
# Note: Quote the title if it contains a colon
title: "Resolve qualified review-local references across documents for 5.0.3"
status: review
kind: bugfix
priority: critical
area: core-tooling
milestone: 5.0.3
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

Resolve qualified review-local references across documents for 5.0.3

## Objective

Make `debt_migration_preview` resolve a qualified `REVIEW-NNN-FINDING-MMM` reference against the review that declares it, wherever that reference appears, and rewrite it to a qualifier-preserving `REVIEW-NNN-RF-MMM`, so the last mechanically decidable class of blocker on the reference workspace stops refusing migration without ever risking a silent rebind.

## Context

After 5.0.2, `debt_migration_preview` on the reference workspace reported 4 remaining issues, all of one class:

```
qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-010-….md belongs to REVIEW-009, not REVIEW-010
```

`REVIEW-009` declares `REVIEW-009-FINDING-004` in its own findings section. Four other reviews cite it. Each citation was refused solely because the citing file was a different review.

That inverts the purpose of the qualifier. `REVIEW-NNN-FINDING-MMM` is the least ambiguous reference form in the corpus: it names its own scope explicitly, precisely so the reference survives leaving its home document. The resolver already knew `REVIEW-009` declares finding `004`; it simply refused to look outside the citing file.

Failing closed was not the wrong instinct. Rewriting these to a bare `RF-004` would rebind each citation to the *citing* review's own finding `004` and corrupt the record. The guard was right and the rule was wrong. The fix is to widen the symbol table the resolver consults, not to loosen the guard.

Three of the reference workspace's occurrences sit inside managed review-event frontmatter — the `reason:` field of a `review_events` entry, at `REVIEW-012` line 32, `REVIEW-023` line 41, `REVIEW-024` line 32. An operator cannot lawfully hand-edit immutable managed lifecycle fields, so the governed migration is the only lawful path for those three, and it must rewrite frontmatter string values without corrupting YAML. At least one of those values carries escaped quotes and embedded newlines around the reference.

## Scope
### Included

- Corpus-wide resolution of qualified `REVIEW-NNN-FINDING-MMM` references in `lmbrain-core/src/debt_migration.rs`.
- The qualifier-preserving `REVIEW-NNN-RF-MMM` target form, decided by the operator.
- Rewriting qualified references inside managed frontmatter string values as well as body prose.
- Carrying decidable qualified references in non-review Markdown into the same target form.
- Two narrowed fail-closed arms: an unknown review, and a number the named review never declared.
- Reading the qualifier-preserving declaration form in the desktop review reader.
- Regression coverage, operator documentation, and 5.0.3 release metadata.

### Excluded

- Any relaxation of digest binding, explicit operator confirmation, or atomic staged migration.
- Automatic edits to review prose beyond the identifier rewrite itself.
- Changes to the durable artifact schema or the `DEBT-*` target contract.
- Re-litigating the 5.0.1 scaffolding exclusion, batch reporting, the five ordered classification rules, local-before-durable resolution, or the same-number-both-forms collision guard.
- The free-form `origin_ref` class recorded as deviation 2 on [SPEC-073]. Those values are prose, not identifiers, and mapping them needs a product decision.

## Existing-project analysis

`collect_review_id_mappings` in `lmbrain-core/src/debt_migration.rs` walks review artifacts one at a time. For each it builds `ReviewLocalDeclarations` from that file alone and classifies every token the migration regex finds. Rule 2 handled the qualified form, but only after asserting `qualifier == review_id`; any other qualifier became an issue. Nothing in the pipeline held a corpus-wide view, so a reference that named a scope outside the current file could not be decided even in principle.

`replace_durable_references` skips every token beginning with `REVIEW-`, so qualified references in specs, handoffs, and reports were left as stale `FINDING-*` tokens after migration.

Rewriting happens as a regex pass over the whole file text, frontmatter included, so managed frontmatter was already in the rewrite path; what was missing was a decision that produced a replacement for these tokens at all.

## Technical proposal

`collect_review_id_mappings` runs in two passes. The first reads every review artifact and builds a corpus-wide symbol table mapping each review id to the finding numbers that review declares in qualified form. The second classifies tokens per file against that table.

Rule 2 becomes: a qualified `REVIEW-NNN-FINDING-MMM` token resolves against `REVIEW-NNN`'s entry in the corpus symbol table, regardless of which file it appears in, and maps to `REVIEW-NNN-RF-MMM`. The qualifier is preserved so the reference keeps meaning the same finding after it leaves its home document. A cross-review citation is never emitted as a bare `RF-MMM`; that is the corrupting outcome the guard exists to prevent and it stays impossible by construction, because the replacement is derived from the qualifier rather than from the citing file.

Rule 5 keeps two arms for the qualified form: the named review declares no findings in this workspace, or the named review never declared that number. Both fail closed inside the single aggregated preflight report.

Mapping rows carry a `cross-review-local` classification when the citing file is not the declaring review, so an operator auditing the preview can see every reference that crossed a document boundary. `review-local` and `durable` are unchanged.

Markdown outside `.lmbrain/reviews/` gets the same qualified rewrite through `replace_qualified_review_references`, which resolves against the corpus table and leaves undecidable references untouched. Those documents are not subject to the review preflight, so an undecidable reference there must not become a new blocker; no previously migratable workspace may be blocked by this change.

`origin_ref` on a durable debt is normalised back to the bare `RF-MMM`. Its sibling `origin_artifact` already carries the review, and the debt contract requires the bare form there.

`parse_review_findings` in the desktop contract reader strips an optional `REVIEW-NNN-` qualifier, so a review declaring its own finding in the qualifier-preserving form is still surfaced under its bare local id.

## Files and areas involved

- `lmbrain-core/src/debt_migration.rs`
- `src-tauri/src/commands/contract.rs`, `src-tauri/tests/contract_test.rs`
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- Workspace crate, application, and kit version metadata

## Acceptance criteria
- [x] A qualified `REVIEW-A-FINDING-003` cited by review B, where A declares it, resolves to `REVIEW-A-RF-003` without error.
- [x] The same citation inside a `review_events` `reason:` frontmatter string is rewritten, the YAML still parses, and the surrounding event fields are byte-identical.
- [x] A `reason:` string containing escaped quotes and embedded newlines around the reference is rewritten without corrupting the string.
- [x] A qualified reference to a nonexistent `REVIEW-999-FINDING-001` still fails closed.
- [x] A qualified reference to `REVIEW-A-FINDING-099` where A declares only 001–008 still fails closed.
- [x] No cross-review citation is ever emitted as a bare `RF-*`, asserted on the emitted identifier.
- [x] Decidable qualified references outside `.lmbrain/reviews/` are carried into the same target form; undecidable ones there are left untouched rather than blocking.
- [x] The 5.0.1 and 5.0.2 behaviours are unchanged: scaffolding-README exclusion, batch reporting in one pass, the five ordered classification rules, local-before-durable resolution order, and the same-number-both-forms collision guard.
- [x] `debt_migrate` remains digest-bound, operator-confirmed, and atomic.
- [x] Preview mapping rows show source token and target form for every token, including these qualified rewrites.
- [x] The reference workspace's 4 remaining issues drop to 0 and every occurrence of `REVIEW-009-FINDING-004` maps to `REVIEW-009-RF-004`.
- [x] Migration and changelog documentation state the corrected rule, and release metadata is aligned at 5.0.3.

## Implementation plan
1. Split `collect_review_id_mappings` into a corpus pass and a classification pass, exposing the qualified symbol table.
2. Rewrite rule 2 to resolve against the declaring review and emit the qualifier-preserving form, with the two narrowed fail-closed arms.
3. Add the corpus-driven qualified rewrite for non-review Markdown, resolve-or-leave.
4. Normalise `origin_ref` to the bare contract form and teach the desktop review reader the qualified declaration form.
5. Add regression coverage for every acceptance criterion, including the frontmatter byte-identity assertion.
6. Update documentation and release metadata, then run the full verification gates.

## Required verification

<!-- Canonical form: ID | kind=executable|manual|operator | owner=agent|kit|lead|operator | phase=before-submit|before-done | evidence=transcript|observation|artifact | requirement -->
- [x] DEBT-MIGRATION-CORE | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-core` test suite including every new qualified-reference fixture.
- [x] DEBT-MIGRATION-MCP | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run the complete `lmbrain-mcp` test suite and confirm the public preview contract.
- [x] WORKSPACE-QUALITY | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run Clippy with `-D warnings`, formatting on touched files, ESLint, the frontend test suite, and version alignment.
- [x] REFERENCE-WORKSPACE | kind=executable | owner=agent | phase=before-submit | evidence=transcript | Run a read-only preview against the real reference workspace and confirm the 4 issues are gone and no bare `RF-*` is emitted for a cross-review citation.
- [x] DIFF-AUDIT | kind=manual | owner=lead | phase=before-done | evidence=observation | Confirm the diff preserves digest binding, explicit confirmation, atomic writes, and every fail-closed refusal.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

The qualifier-preserving target form was decided by the operator. It is the only form that keeps a cross-document reference correct: any bare form would rebind the reference to the citing review's own finding of that number.

The declaring review's own qualified declaration now reads `REVIEW-NNN-RF-MMM` rather than the bare `RF-MMM` 5.0.2 produced. That is deliberate and consistent — one token, one target form, everywhere — but it means the desktop review reader has to accept both declaration forms, which it now does.

Widening the rewrite to non-review Markdown could in principle block a workspace that was previously migratable. It cannot here, because an undecidable qualified reference outside a review is left as written and never raised as an issue.

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

- Split `collect_review_id_mappings` into a corpus pass that builds a workspace-wide symbol table of every review's qualified finding declarations, and a classification pass that resolves against it.
- Rewrote rule 2: a qualified `REVIEW-NNN-FINDING-MMM` token resolves against `REVIEW-NNN`'s declarations wherever it appears, and maps to the qualifier-preserving `REVIEW-NNN-RF-MMM`. The replacement is derived from the qualifier, not from the citing file, so a cross-review citation cannot become a bare `RF-MMM` by construction.
- Narrowed the fail-closed arm for the qualified form to the two undecidable cases: the named review declares nothing in this workspace, or it never declared that number. Both still land in the single aggregated preflight report.
- Added the `cross-review-local` classification to preview mapping rows, so an operator can see every reference that crossed a document boundary before confirming.
- Added `replace_qualified_review_references` so decidable qualified references in non-review Markdown are carried into the same target form. Undecidable ones there are left untouched rather than raised as new blockers.
- Normalised `origin_ref` on durable debts back to the bare `RF-MMM` contract form, scoped by its `origin_artifact`.
- Taught `parse_review_findings` in the desktop contract reader to strip an optional `REVIEW-NNN-` qualifier from a declaration id.
- Left every other fail-closed arm, the five ordered rules, local-before-durable ordering, batch reporting, scaffolding exclusion, and the digest-bound atomic `debt_migrate` transaction unchanged. Preview schema stays at `3`.

### Files changed

- `lmbrain-core/src/debt_migration.rs`
- `src-tauri/src/commands/contract.rs`, `src-tauri/tests/contract_test.rs`
- `docs/MIGRATIONS.md`, `docs/CHANGELOG.md`
- `.lmbrain/VERSION`, `kit/.lmbrain/VERSION`, `package.json`, `lmbrain-core/Cargo.toml`, `lmbrain-mcp/Cargo.toml`, `src-tauri/Cargo.toml`, `Cargo.lock`

### Verification performed

- Full `lmbrain-core` suite: 264 tests passed (189 unit including 20 debt-migration fixtures, 52 transitions, 23 verification). Seven fixtures are new.
- Full `lmbrain-mcp` suite: 33 tests passed (30 library, 3 protocol). The public preview contract still reports schema `3` with `occurrences`, `replacement`, and `classification` on every mapping row.
- `cargo clippy --all-targets -- -D warnings` across the whole workspace, including the Tauri crate: clean, exit 0.
- `rustfmt --check` clean on all three touched Rust files. Repository-wide `cargo fmt --all --check` is already dirty on `main` (`attestation.rs`, `registry.rs`) and was not made a gate here; no unrelated file was reformatted.
- ESLint clean; frontend suite 392 tests across 55 files passed; `git diff --check` clean; version alignment reports 5.0.3.
- End-to-end read-only preview against the real reference workspace. The 4 issues 5.0.2 left are gone: the preflight report is empty and the migration now reaches the planning phase. On a scratch copy with the out-of-scope free-form `origin_ref` values neutralised, the preview builds 223 items and 14 scaffolding operations from 664 mapping rows covering 1597 token occurrences (555 durable rows, 81 review-local, 28 cross-review-local). All 26 occurrences of `REVIEW-009-FINDING-004` across 12 files map to `REVIEW-009-RF-004`, including the three inside managed `review_events` `reason:` values. No cross-review citation is emitted as a bare `RF-*`.

### Verification transcript

<!-- Required before spec_submit. Paste actual command/result output in a fenced block, or use approved `spec_verify` gates. Predictions and summaries are not execution evidence. -->

```text
$ CARGO_TARGET_DIR=target/verify cargo test -p lmbrain-core
test result: ok. 189 passed; 0 failed
test result: ok. 52 passed; 0 failed
test result: ok. 23 passed; 0 failed
core exit: 0

$ CARGO_TARGET_DIR=target/verify cargo test -p lmbrain-mcp
test result: ok. 30 passed; 0 failed
test result: ok. 3 passed; 0 failed
mcp exit: 0

$ CARGO_TARGET_DIR=target/verify cargo test -p lmbrain-core debt_migration
test result: ok. 20 passed; 0 failed; 169 filtered out

$ CARGO_TARGET_DIR=target/verify cargo clippy --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s
clippy exit: 0

$ rustfmt --check --edition 2021 lmbrain-core/src/debt_migration.rs
lmbrain-core/src/debt_migration.rs fmt exit: 0
$ rustfmt --check --edition 2021 src-tauri/src/commands/contract.rs
src-tauri/src/commands/contract.rs fmt exit: 0
$ rustfmt --check --edition 2021 src-tauri/tests/contract_test.rs
src-tauri/tests/contract_test.rs fmt exit: 0

$ pnpm lint
$ eslint .
lint exit: 0

$ pnpm test
 Test Files  55 passed (55)
      Tests  392 passed (392)
test exit: 0

$ git diff --check
diff --check exit: 0

$ node scripts/check-version.mjs
LMBrain workspace crates, app, and kit are aligned at v5.0.3.

$ debt_migration_preview --root <reference workspace>   # before this change, on 5.0.2:
debt migration preflight failed: 4 issue(s):
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-010-...md belongs to REVIEW-009, not REVIEW-010
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-012-...md belongs to REVIEW-009, not REVIEW-012
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/accepted/REVIEW-024-...md belongs to REVIEW-009, not REVIEW-024
- qualified review-local reference REVIEW-009-FINDING-004 in .lmbrain/reviews/superseded/REVIEW-023-...md belongs to REVIEW-009, not REVIEW-023

$ debt_migration_preview --root <same workspace, after this change>
debt migration preflight failed: unresolved review-local origin REVIEW-053/REVIEW-053-08 in .lmbrain/findings/deferred/FINDING-065-...md
# The 4-issue preflight report is gone. What remains is the free-form `origin_ref` class
# already recorded as deviation 2 on SPEC-073, raised from the planning phase rather than
# from the preflight report.

$ debt_migration_preview --root <scratch copy, free-form origin_ref values neutralised>
schema 3 source 5.0.2 -> 4.2.0
items 223 scaffolding 14
rows 664 occurrences 1597
rows by class {'durable': 555, 'cross-review-local': 28, 'review-local': 81}
occ by class {'durable': 1385, 'review-local': 167, 'cross-review-local': 45}
REVIEW-009-FINDING-004 rows 12 occurrences 26 targets {'REVIEW-009-RF-004'}
bare RF emitted for cross-review: []
every row has token+replacement: True
```

### Deviations from the specification

None within scope.

Two observations for the operator.

1. **The reference workspace still cannot complete a migration**, for a different and previously unreachable reason. With the 4 qualified-reference issues resolved, the preflight report is empty and the plan builder now reaches the free-form `origin_ref` class already recorded as deviation 2 on [SPEC-073] — `origin_ref: "REVIEW-053-08"`, `origin_ref: "REVIEW-052 findings 01 and 02"`, `origin_ref: "REVIEW-029-NO-TYPE-BARRIER"`, and 27 more. Thirty durable artifacts carry a `REVIEW-*`-shaped `origin_ref` in a form no mechanical rule maps to an `RF-*`, and many reviews declare their local findings with mnemonic slugs (`REVIEW-024-DENSITY`) or a bare `REVIEW-NNN-NN` rather than `REVIEW-NNN-FINDING-MMM`. Mapping those needs a target-form decision the operator has not made, and the numeric `RF-NNN` contract form does not accommodate a mnemonic. This class also raises its failure one artifact at a time, outside the aggregated preflight report, which is worth folding into batch reporting when it is addressed.

2. **The occurrence count in the kit-feedback report is low.** It records "seven occurrences cite it from four other reviews"; the corpus actually holds 25 citing occurrences of `REVIEW-009-FINDING-004` across 11 files, of which 10 across 4 files are in review artifacts. The three managed-frontmatter occurrences it names are correct and are all rewritten. Nothing is blocked by the discrepancy; it is recorded so the figure is not carried forward.

### Handoff status
- [x] Ready for Project Lead review
