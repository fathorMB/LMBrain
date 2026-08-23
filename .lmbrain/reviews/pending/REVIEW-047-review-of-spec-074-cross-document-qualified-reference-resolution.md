---
id: REVIEW-047
# Note: Quote the title if it contains a colon
title: "Review of SPEC-074 cross-document qualified reference resolution"
status: pending
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-074
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
# Taxonomy v1 canonical values are listed in CONTRACT.md. New reviews use canonical values only.
finding_taxonomy_version: 1
finding_categories: []
# Managed append-only history. Use semantic review MCP verbs; do not edit events by hand.
review_events:
  - schema_version: "1"
    id: "REVIEW-047-EVENT-001"
    timestamp: "2026-08-23T21:37:14.795722100+02:00"
    action: "submitted"
    from_status: "none"
    to_status: "pending"
    actor_role: "project-lead"
    reason: "review artifact created"
    implementation_agent: "AGENT-FULLSTACK-DESKTOP"
links: []
created: 2026-08-23
updated: 2026-08-23
tags: [review]
activity:
  - date: 2026-08-23
    action: "created"
---
# Review

## Outcome

Pass recommended. The implementation stops refusing a decidable, self-scoping reference form without loosening a single guard, and it does so by widening the symbol table the resolver consults rather than by weakening the rule. Digest binding, explicit confirmation, and the atomic staged swap are untouched.

## Acceptance-criteria compliance

All twelve SPEC-074 criteria are satisfied with direct regression coverage. A cross-review citation resolves against the declaring review and keeps its qualifier; the same citation inside a `review_events` `reason:` value is rewritten with the surrounding YAML byte-identical; a `reason:` string carrying escaped quotes and embedded newlines survives; an unknown review and an undeclared number both still fail closed; decidable qualified references outside `.lmbrain/reviews/` are carried across while undecidable ones there are left alone; and the 5.0.1 and 5.0.2 behaviours are unchanged.

The criterion that matters most is the negative one, and it is asserted on the emitted identifier rather than on the absence of an error. `cross_review_citations_never_collapse_to_a_bare_local_identifier` builds the exact adversarial shape — REVIEW-010 declaring a finding 004 of its own while citing REVIEW-009's 004 — and asserts both that the citation keeps its qualifier and that REVIEW-010's own declaration is a separate identifier afterwards. That is the corruption the old guard existed to prevent, and it is now impossible by construction rather than by refusal: the replacement is derived from the qualifier in the token, so no code path can produce a bare `RF-*` from a foreign reference.

## Code observations

Splitting `collect_review_id_mappings` into a corpus pass and a classification pass is the right shape. The previous code could not decide a cross-document reference even in principle, because nothing in the pipeline ever held more than one review at a time; the defect was structural, not a missing branch. `qualified_declarations` is built once, keyed by review id, and the classification arms read against it in the same order the documentation states.

The two new failure arms are narrower than what they replace and are worded so an operator can act on them: one names the review that declares nothing in the workspace, the other names the number that review never declared. Neither is a catch-all.

`replace_qualified_review_references` deliberately resolves-or-leaves rather than resolves-or-fails. That asymmetry is correct and is argued in the spec: documents outside `.lmbrain/reviews/` are not subject to the review preflight, so raising there would newly block workspaces that migrate today. It is covered by a fixture that puts both a decidable and an undecidable qualified reference in the same spec file.

Managed frontmatter needed no new machinery, because the rewrite has always been a regex pass over the whole file text. What was missing was a decision that produced a replacement at all. The frontmatter test asserts this properly: it normalises the token on one side and the canonical heading on the other and requires the two files to be byte-identical, so any incidental reflow, requote, or re-serialisation of the managed events would fail it.

Two consequential edges are handled rather than left implicit. `origin_ref` is normalised back to the bare `RF-MMM`, which the debt contract requires and which `origin_artifact` already scopes; and `parse_review_findings` in the desktop reader strips an optional `REVIEW-NNN-` qualifier, so a review declaring its own finding in the new form is still surfaced. Without the second, the declaring review's own finding would have silently disappeared from the application.

## Tests and verification

Independent reruns passed: 264 `lmbrain-core` tests, 20 of them debt-migration fixtures and 7 of those new; 33 `lmbrain-mcp` tests; `cargo clippy --all-targets -- -D warnings` across the whole workspace including the Tauri crate at exit 0; `rustfmt --check` clean on all three touched Rust files; ESLint; 392 frontend tests across 55 files; `git diff --check`; and version alignment at 5.0.3.

The decisive evidence is the read-only preview against the real reference workspace. The 4-issue preflight report is gone. On a scratch copy with the out-of-scope free-form `origin_ref` values neutralised the preview completes with 664 mapping rows over 1597 token occurrences, and all 26 occurrences of `REVIEW-009-FINDING-004` across 12 files map to `REVIEW-009-RF-004` — including the three inside managed `review_events` `reason:` values that no operator could lawfully have edited by hand. No cross-review citation is emitted as a bare `RF-*`.

Repository-wide `cargo fmt --all --check` remains dirty on `main` in `attestation.rs` and `registry.rs`. That is pre-existing, was correctly not folded into this change, and no unrelated file was reformatted.

## Production quality and documentation compliance

Production-scoped and dependency-free. The corrected rule is documented for operators in `docs/MIGRATIONS.md`, including the reason a qualified reference is never reduced to a bare form, and summarised in `docs/CHANGELOG.md`. Versions are aligned at 5.0.3 across application, crates, kit, and lockfile.

## Review findings

<!-- Stable form: RF-001 | category=... | severity=... | criterion=... | remediation=... -->

None blocking.

One behavioural change deserves the operator's explicit attention because it was not a defect being fixed. A review's own qualified declaration now migrates to `REVIEW-NNN-RF-MMM` rather than the bare `RF-MMM` that 5.0.2 produced. This follows the operator's decision that the target form is qualifier-preserving everywhere, and it is the consistent reading — one source token, one target form, regardless of file. Three existing fixtures were updated to match, and the desktop review reader now accepts both declaration forms so nothing is lost from the application. It is recorded here rather than buried in a test diff.

## Required follow-up

Merge through the release pull request after CI passes.

The reference workspace still cannot complete a migration, for a different reason that this spec correctly excluded. With the qualified-reference class resolved, the plan builder now reaches the free-form `origin_ref` class recorded as deviation 2 on [SPEC-073]: thirty durable artifacts carry a `REVIEW-*`-shaped `origin_ref` in forms no mechanical rule maps to an `RF-*` — `"REVIEW-053-08"`, `"REVIEW-052 findings 01 and 02"`, `"REVIEW-029-NO-TYPE-BARRIER"` — and many reviews declare local findings with mnemonic slugs rather than numbers. The numeric `RF-NNN` contract form does not accommodate a mnemonic, so this needs an operator decision on the target form before it can be specified. That work should also fold the review-origin resolution failure into the aggregated preflight report, since it currently surfaces one artifact at a time from the planning phase rather than in one pass.

Separately, the kit-feedback report's count of "seven occurrences" of `REVIEW-009-FINDING-004` is low: the corpus holds 25 citing occurrences across 11 files, 10 of them across the 4 review artifacts. The three managed-frontmatter occurrences it names are correct. Nothing was blocked by the discrepancy; the figure should not be carried forward.

## Final decision

Recommend acceptance. The governed review remains `pending` until the operator explicitly records the verdict.
