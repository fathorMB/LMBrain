---
id: SPEC-037
title: "Close related reviews when a specification is done"
status: ready
kind: feature
priority: medium
area: artifact-lifecycle
milestone: M-04
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-013]
links: []
created: 2026-07-12
updated: 2026-07-12
tags: [reviews, lifecycle, spec-done, insights, migration, 2.8.0]
activity:
  - date: 2026-07-12
    action: "approved by operator"
  - date: 2026-07-12
    action: "created"
---

# Close related reviews when a specification is done

## Objective

When a Project Lead completes a specification, atomically mark all resolved corrective reviews as addressed while preserving their original verdict and historical contribution to quality metrics.

## Context

Review artifacts currently combine verdict and lifecycle in one status. A `changes-requested` review remains permanently in `reviews/changes-requested/` even after a later review accepts the remediation and the related spec reaches `done`. The Reviews page consequently presents already-resolved findings as if they still require action.

### 3.1.0 reconciliation with ADR-014

ADR-014 revises the meaning of “addressed” before this closeout work proceeds. Addressing a corrective review means that no further action remains inside the originating spec’s corrective cycle. It does **not** mean that every underlying cross-spec obligation is resolved. A review may therefore become addressed while a canonical promoted `FINDING-*` remains `open`, `planned`, or `deferred`.

Closeout must preserve the original review verdict, body, event history, and metric contribution. It derives promoted-finding backlinks from canonical finding metadata and never rewrites or closes those findings. A target spec reaching `done` creates attention for an explicitly planned finding but is not resolution evidence. Any implementation of this spec must follow ADR-014’s independent finding lifecycle and must not restore ADR-013’s earlier assumption that one review status represents every finding disposition.

Reusing `superseded` would make the UI quieter but would damage history: current Insights derives the change-request rate from review status, so converting corrective reviews to an undifferentiated superseded state would erase evidence that a spec required changes.

The closeout belongs in the controlled `spec_done` operation rather than Lead memory. Cross-artifact writes must be all-or-nothing so a spec cannot become done while its related review lifecycle remains half-updated.

## Scope

### Included

- Add terminal review status `addressed` for a prior `changes-requested` or `blocked` verdict whose findings were resolved by a later accepted review.
- Preserve the original verdict in explicit `outcome` metadata when moving a review to `addressed`.
- Record `resolved_by` with the accepted review ID, `resolved_at`, updated date, and an audit activity.
- Extend `spec_done` to identify reviews by the canonical `spec` frontmatter field.
- Require a related accepted review and select the resolving review deterministically; allow the caller to identify it when multiple accepted reviews exist.
- Atomically update the spec plus every related corrective review.
- Leave accepted reviews accepted; they are already terminal and retain their verdict.
- Reject `spec_done` while any related review is still `pending`, because a pending review has no disposition and must not be silently hidden.
- Define explicit behavior for related `blocked` reviews: address them only when the resolving accepted review's canonical `links` includes that blocked review; otherwise stop closeout with an actionable error.
- Preserve truly superseded/legacy review artifacts without rewriting them.
- Make review loaders, lists, filters, detail views, roadmap joins, context packs, diagnostics, and status-directory checks understand `addressed`.
- Show unresolved reviews by default and make addressed review history accessible through a clear filter/history section.
- Keep historical Insights accurate by counting addressed reviews according to their preserved `outcome`.
- Add a controlled, preview-first reconciliation command for existing done specs that still have legacy `changes-requested` or `blocked` reviews.
- Make reconciliation idempotent, operator-authorized, auditable, and atomic per spec; never run it merely because a workspace is opened.
- Update the kit contract, templates/guidance, migration notes, changelog, and bundled directory structure.

### Excluded

- Rewriting a corrective review to `accepted`.
- Deleting reviews, changing their findings, or pretending the original request for changes never occurred.
- Addressing reviews linked only by title, filename, body wikilink, or naming convention when canonical `spec` metadata is missing.
- Silently resolving pending reviews or choosing among ambiguous accepted reviews.
- Recomputing historical quality metrics from Git history.
- Automatically mutating existing projects during ordinary workspace open.

## Existing-project analysis

- `lmbrain-core::transition` currently writes one artifact at a time; `spec_done` is a target passed through the generic transition path.
- `spec_has_accepted_review` scans `reviews/accepted`, but closeout does not update other reviews for that spec.
- Review transitions currently accept `pending -> accepted|changes-requested|blocked` and any status to `superseded`.
- Rust/TypeScript review status enums, loaders, status directories, UI color maps, statistics, context packs, milestone joins, and diagnostics enumerate the existing statuses explicitly.
- Insights currently counts `ReviewStatus::ChangesRequested`; lifecycle closure must not cause historical rates to drop.
- Existing LMBrain dogfooding data already contains many closed specs with related reviews still under `changes-requested`, demonstrating the need for migration/reconciliation as well as prospective behavior.

## Technical proposal

### Review lifecycle

Extend Review status with `addressed`. An addressed review must contain:

```yaml
status: addressed
outcome: changes-requested
resolved_by: REVIEW-024
resolved_at: 2026-07-12
```

`outcome` is restricted to `changes-requested` or `blocked` for this transition. Legacy review interpretation remains backward compatible: when `outcome` is absent, use the current status as the outcome. Accepted reviews do not need duplicated outcome metadata.

### Atomic spec closeout

Implement a dedicated core `complete_spec` transaction instead of adding side effects to the generic single-artifact transition. It must:

1. Parse and validate the review-state spec, checked criteria/evidence, and accepted-review prerequisite.
2. Load all reviews whose canonical `spec` equals the spec ID.
3. Resolve the final accepted review uniquely or from an optional `accepted_review` argument.
4. Fail on pending, malformed, ambiguous, mismatched, or unresolved-blocked reviews before any write. A blocked review is resolved only by an explicit ID in the selected accepted review's `links`.
5. Render the done spec and every addressed review in memory.
6. Commit all destination writes/moves under a workspace mutation lock using staged temporary files and rollback/recovery semantics.
7. Record matching audit entries on every affected artifact.

`spec_done` in MCP delegates to this dedicated operation. The result lists the completed spec, resolving accepted review, and every addressed review so the caller/UI can refresh deterministically.

### Existing-project reconciliation

Add a read-only preview that groups stale corrective reviews under done specs and reports blockers such as missing `spec`, no accepted review, multiple accepted reviews, or pending reviews. An explicit apply operation uses the same closeout subtransaction for review-only reconciliation, records a migration audit action, and is safe to rerun. The 2.8 migration UI/prompt offers this operation but does not execute it automatically.

## Files and areas involved

- `lmbrain-core` transition/completion transaction, review parsing, invariants, recovery, statistics, and context packs
- `lmbrain-mcp` `spec_done` schema/result and reconciliation tools
- Tauri review models/loaders/statistics/milestone logic and commands
- TypeScript review types, Reviews/Insights/Roadmap/Pulse/detail UI and tests
- kit `CONTRACT.md`, review directories/readmes, `AGENT.md`, `OPERATOR.md`, templates, changelog and migration guidance
- architecture/product documentation

## Acceptance criteria

- [ ] Review status `addressed` is defined consistently in contract, kit directories, core/Tauri/TypeScript models, UI, diagnostics, and documentation.
- [ ] Only `changes-requested` and eligible `blocked` reviews can become addressed through normal lifecycle operations.
- [ ] Addressed reviews retain `outcome`, `resolved_by`, `resolved_at`, and an audit entry identifying spec closeout.
- [ ] `spec_done` requires a canonical related accepted review and never infers association from filenames, titles, or prose.
- [ ] If multiple accepted reviews exist, closeout requires an unambiguous explicit selection unless exactly one qualifies.
- [ ] Accepted reviews remain accepted and corrective findings remain unchanged in their bodies.
- [ ] Related pending reviews block closeout with paths/IDs and actionable guidance.
- [ ] A blocked review is addressed only when its ID appears in the selected accepted review's canonical `links`; otherwise it blocks closeout.
- [ ] The spec and all affected reviews move/update atomically; injected failure leaves every artifact in its original state or is recoverable deterministically.
- [ ] Concurrent artifact mutations are serialized and cannot produce split lifecycle state.
- [ ] `spec_done` returns the resolver and addressed-review IDs for immediate application refresh.
- [ ] Reviews UI defaults to actionable unresolved items while exposing addressed history and its original verdict/resolver.
- [ ] Insights continues counting specs that received changes requests after their corrective reviews become addressed.
- [ ] First-pass acceptance and other historical metrics remain stable across reconciliation.
- [ ] Context packs and roadmap joins distinguish active findings from addressed historical findings.
- [ ] A preview-first reconciliation identifies stale reviews for existing done specs and reports every ambiguous/unrepairable case without writing.
- [ ] Reconciliation apply is explicit, idempotent, audited, and atomic per spec; workspace open never triggers it automatically.
- [ ] Migration tests cover 2.7.x projects and the bundled kit supplies the new addressed directory/contract.
- [ ] Full Rust/frontend/contract gates and a representative existing-project reconciliation fixture pass before 2.8.0.

## Implementation plan

1. Approve [[ADR-013-address-corrective-reviews-during-atomic-spec-closeout]] and the explicit blocked-review linking rule.
2. Add the addressed schema/status, backward-compatible outcome interpretation, directories, loaders, and diagnostics.
3. Introduce the dedicated atomic `complete_spec` operation and route MCP `spec_done` through it.
4. Update statistics/context joins so verdict history is independent from current review lifecycle status.
5. Update Reviews and related UI to separate actionable findings from addressed history.
6. Add preview/apply reconciliation for already-done specs and 2.8 migration guidance.
7. Run failure-injection, concurrency, migration, metric-stability, full quality, and packaged Windows checks.

## Required verification

- Core tests for zero/one/multiple accepted reviews, pending review block, corrective review closure, blocked ambiguity, malformed/missing links, force behavior, and idempotence.
- Atomic batch failure injection before staging, during staging, during replace/move, and during cleanup/recovery.
- Concurrency tests for simultaneous review mutation and spec completion.
- Metric fixtures proving change-request and first-pass figures remain stable before/after addressing.
- Frontend tests for default actionable filter, addressed-history detail, resolver navigation, and immediate refresh.
- Reconciliation fixtures for clean, ambiguous, partially migrated, repeated, and corrupted projects.
- Full workspace tests/checks and operator-coordinated packaged Windows migration smoke test without starting or stopping an existing production instance.

## Production quality and documentation

- Follow [[QUALITY]]; artifact history must be preserved rather than cosmetically rewritten.
- Treat multi-artifact completion as a transaction with tested recovery, not a sequence of best-effort writes.
- Document the distinction between verdict (`outcome`) and lifecycle (`status`).
- Report migration ambiguities rather than guessing.

## Risks and open decisions

- The current frontmatter model conflates verdict and lifecycle; backward compatibility must avoid requiring a bulk rewrite before reads work.
- Multiple accepted reviews may be legitimate history. Resolver selection must be explicit rather than date-based guessing.
- A blocked review may describe an external condition not resolved by code acceptance; requiring an explicit link from the accepted review prevents accidental closure but adds one deliberate bookkeeping step.
- Filesystem transactions across multiple renames need crash recovery, especially on Windows where open handles can prevent replacement.

## Instructions for the assigned specialist

- Start only after this spec and ADR are explicitly approved; call `spec_start` first.
- Preserve all existing review bodies and verdict history.
- Do not implement closeout as non-atomic sequential calls.
- Report changed files, transaction/failure tests, migration fixtures, metric comparisons, and limitations.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Verification transcript

### Deviations from the specification

### Handoff status

- [ ] Ready for Project Lead review
