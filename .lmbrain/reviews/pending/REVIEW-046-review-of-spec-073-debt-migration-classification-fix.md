---
id: REVIEW-046
# Note: Quote the title if it contains a colon
title: "Review of SPEC-073 debt migration classification fix"
status: pending
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-073
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
# Taxonomy v1 canonical values are listed in CONTRACT.md. New reviews use canonical values only.
finding_taxonomy_version: 1
finding_categories: []
# Managed append-only history. Use semantic review MCP verbs; do not edit events by hand.
review_events:
  - schema_version: "1"
    id: "REVIEW-046-EVENT-001"
    timestamp: "2026-08-23T20:47:23.597433500+02:00"
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

Pass recommended. The implementation removes both reported misclassification classes on the real 5.0.1 reference workspace without weakening any refusal, and without touching digest binding, explicit confirmation, or the atomic staged swap.

## Acceptance-criteria compliance

All eleven SPEC-073 criteria are satisfied with direct regression coverage. Qualified declarations map to review-scoped `RF-MMM`; bare declarations backed by a durable artifact map to `DEBT-NNN` and are no longer called collisions; bare prose references bind to the declaring review; durable wikilinks map to `DEBT-*`; the both-forms-plus-durable shape and the resolves-to-nothing shape still fail closed; overlapping number ranges bind local-first; the aggregated report is preserved; and the preview inventory now carries an `occurrences` count per token per file.

## Code observations

`ReviewLocalDeclarations` makes the two declaration forms distinct data rather than a single flattened set, which is what allowed the Class A rule to be corrected without losing the genuine-collision guard. The classification match arms read in the same order as the documented rule, and rule 3 sitting ahead of rule 4 is exercised by a dedicated overlapping-range fixture rather than left as an untested comment. `RF-*` deriving from the declared number removes a scan-order dependency that made preview rows hard to trace back to their declaration.

The reference inventory is aggregated rather than deduplicated, and review artifacts are inventoried exactly once, so occurrence counts on review files are neither doubled by the durable rewrite pass nor lost. Preview schema version `3` marks the shape change.

Class A originated in an incorrect SPEC-072 acceptance criterion, not in the SPEC-072 implementation, which followed that criterion faithfully. The correction is recorded as a specification repair.

## Tests and verification

Independent reruns passed: 257 `lmbrain-core` tests (13 of them debt-migration fixtures), 33 `lmbrain-mcp` tests, Clippy for both crates with `--all-targets -- -D warnings` at exit 0 on the first run, ESLint, 392 frontend tests across 55 files, `git diff --check`, and version alignment at 5.0.2.

The decisive evidence is the read-only preview against the real 5.0.1 workspace: 88 blocking issues in the two corrected classes fall to zero. On a scratch copy with the two out-of-scope shapes neutralised, the preview completes and reports 636 mapping rows covering 1553 token occurrences, with 48 distinct qualified declarations resolved review-locally, 33 distinct bare review-local tokens, and 555 distinct durable tokens. Every `RF-*` and `DEBT-*` preserves its source number.

## Production quality and documentation compliance

Production-scoped and dependency-free. The ordered rule is documented for operators in `docs/MIGRATIONS.md` and summarised in `docs/CHANGELOG.md`, and versions are aligned at 5.0.2 across application, crates, kit, and lockfile.

## Review findings

<!-- Stable form: RF-001 | category=... | severity=... | criterion=... | remediation=... -->

None blocking. Two shapes remain refused in the reference workspace and are correctly out of this spec's scope, but they still block the migration and need an operator decision before 5.0.3:

1. Cross-review qualified references. Four reviews cite `REVIEW-009-FINDING-004`. The object is decidable, but `RF-*` is review-scoped, so a foreign reference cannot be rewritten to a bare `RF-004` without silently rebinding it to the citing review's own local finding. A qualifier-preserving target form must be chosen first.
2. Free-form `origin_ref` on durable artifacts whose `origin_artifact` is a review, for example `"REVIEW-053-08"` and `"REVIEW-052 findings 01 and 02"`. These are prose, not identifiers. This path also raises outside the aggregated report, so it surfaces one artifact at a time.

## Required follow-up

Merge through the release pull request after CI passes. Then decide the two shapes above, and fold the review-origin resolution failure into the aggregated preflight report so the remaining work can be surfaced in one pass.

## Final decision

Recommend acceptance. The governed review remains `pending` until the operator explicitly records the verdict.
