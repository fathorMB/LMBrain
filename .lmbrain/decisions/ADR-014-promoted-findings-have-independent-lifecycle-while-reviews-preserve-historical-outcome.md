---
id: ADR-014
# Note: Quote the title if it contains a colon
title: "Promoted findings have independent lifecycle while reviews preserve historical outcome"
status: proposed
decision_date: 2026-07-29
decider: AGENT-LEAD
# References use IDs only (e.g. [ADR-001]); use [[wikilinks]] in prose
supersedes: [ADR-013]
superseded_by: []
links: [SPEC-037, SPEC-051, SPEC-052, SPEC-057, SPEC-058]
tags: [3.1.0, findings, reviews, lifecycle, migration, atomicity]
created: 2026-07-29
updated: 2026-07-29
activity:
  - date: 2026-07-29
    action: "created"
---
# Promoted findings have independent lifecycle while reviews preserve historical outcome

## Context

LMBrain reviews contain local findings whose normal lifecycle belongs to one
specification and one corrective cycle. Most are resolved before the review is
accepted and should never become separate project objects. Some observations
survive the originating spec, span later work, record a durable limitation, or
remain design input before an implementation-ready spec exists. Today those
items are duplicated manually across a review, STATUS, BACKLOG, and later specs.

Accepted [[ADR-013-address-corrective-reviews-during-atomic-spec-closeout]]
assumes reviews do not have multiple independent finding lifecycles. GitHub
issue #12 intentionally crosses that review condition.

Read-only analysis of `E:\Git\XenoMark` on 2026-07-29 provides concrete
constraints:

- `FINDING-01` occurs in ten different reviews and sixteen local token values
  occur in multiple reviews. A local finding token is not a global identity.
- `REVIEW-054` contains a local blocking `FINDING-07` that was later routed as
  medium-priority debt to `SPEC-059`; the originating spec and review correctly
  closed while the underlying obligation remained planned.
- `REVIEW-049` closed `FINDING-050-001` by documenting and measuring a real
  limitation. The limitation still constrains later verification, but it is
  not open debt.
- two convergent design observations live only in BACKLOG because they have no
  target spec and are not yet operator-confirmed;
- thirty-four `before-done` gates exist in completed specs, three still
  unchecked after reconciliation. These remain verification obligations and
  diagnostics unless explicitly promoted; Findings must not duplicate them.
- only six of fifty-four reviews declare `review_cycles`; historical
  changes-requested passes mostly survive in prose. Migration cannot infer
  authoritative lifecycle from headings alone.

This ADR is proposed by the Project Lead. Its `decider` template value does not
grant acceptance authority; only the operator may accept it.

## Decision

### Local and promoted findings are different domains

A review-local finding is identified only inside its review by
`origin_artifact + origin_ref`. It remains review evidence and follows the
review/spec corrective cycle.

A first-class `FINDING-*` is created only when the observation:

- survives or is deliberately routed beyond the originating spec;
- spans multiple artifacts or later delivery;
- records an accepted limitation/risk requiring durable traceability; or
- is retained project evidence that is not ready to become a spec.

Promotion never authorizes implementation. Only a ready `SPEC-*` does.

### Identity and provenance

`FINDING-*` IDs are globally allocated. When a finding is promoted from a
review, `(origin_artifact, origin_ref)` is the canonical source identity and
cannot map to two active findings. Directly created observations may omit an
origin artifact, but must contain explicit evidence/provenance in the body.

The current project severity and the original review severity are separate:

- `severity` is the current triaged project impact;
- optional `origin_severity` preserves how the local finding affected its
  originating corrective cycle.

This permits XenoMark `FINDING-07` to remain historically blocking in
`REVIEW-054` while the promoted debt is currently medium priority.

Product finding categories and agent-effectiveness categories are separate
versioned namespaces. Promotion never creates an agent-performance signal
unless an independently valid review category already supplies that evidence.

### Lifecycle

The lifecycle is:

```text
open -> planned | deferred | resolved | accepted-risk | superseded
planned -> open | deferred | resolved | accepted-risk | superseded
deferred -> open | planned | resolved | accepted-risk | superseded
```

- `open`: explicitly retained without committed delivery disposition;
- `planned`: one or more validated target specs exist, but the finding is not
  resolved;
- `deferred`: retained with rationale and revisit trigger;
- `resolved`: resolution criteria and canonical evidence are satisfied;
- `accepted-risk`: behavior remains under explicit operator acceptance,
  rationale, and revisit policy;
- `superseded`: duplicate, replaced, invalidated, or obsolete with successor or
  reason.

Blocked state is derived from active `blocked_by` relationships. Target-spec
completion creates an attention diagnostic but never resolves a finding.
Terminal findings may reopen only through explicit operator authorization.

### Authority

- The Project Lead may create/promote, plan, defer, resolve with evidence, or
  supersede a finding through semantic controlled operations.
- The operator alone may accept risk or reopen a terminal finding.
- Implementers may add delegated implementation evidence to target specs but
  do not change finding lifecycle.
- Every semantic mutation is locked, atomic, audited, preservation-aware, and
  fail-closed.

### Review closeout and history

“Review addressed” and “finding resolved” are independent statements.

An accepted review may close the originating spec while a promoted finding
remains open, planned, or deferred, provided routing is explicit. The review’s
body, original verdict, event timeline, and metrics remain historically true.
Promotion does not rewrite or reclassify them.

The finding carries the canonical origin link. Reverse relations shown in
review/spec context are derived from the finding index, so normal promotion
does not require rewriting the originating review. If a future workflow adds a
physical backlink, that optional multi-artifact write must be one tested atomic
transaction.

SPEC-037 must be revised: `addressed` means no further action is required
inside the originating spec’s corrective cycle. It may coexist with an active
promoted finding, and it must never imply that the cross-spec obligation is
resolved. Review statistics continue to use the preserved original outcome.

### Migration and rollback

Legacy review prose is inventoried read-only. Candidate detection may expose
stable-form local entries and final routing language, but labels every
inference and never creates artifacts. The operator/Lead selects promotions
explicitly.

Old LMBrain versions preserve but ignore `findings/`; rollback never deletes
finding history. Workspace open, refresh, diagnostics, and migration preview
remain non-mutating.

## Alternatives considered

### Make every review bullet a first-class finding

Rejected. XenoMark contains more than one hundred local tokens, many repeated
and already resolved inside their review. Automatic explosion would create
noise and false debt.

### Keep STATUS/BACKLOG as the lifecycle source

Rejected. XenoMark’s planned debt, known limitation, and design observations
already require forensic reconciliation across several prose pages.

### Consider a target spec or done status sufficient resolution

Rejected. Planning work is not evidence that the underlying claim became
false, safe, or accepted.

### Rewrite reviews with promotion backlinks

Rejected as the default. Canonical origin data on the finding permits derived
reverse joins while preserving review history and avoiding unnecessary
cross-artifact transactions.

### Model unresolved verification gates as findings automatically

Rejected. Verification requirements, diagnostics, review findings, and
cross-spec findings have different authority and resolution semantics.

## Consequences

- [[ADR-013-address-corrective-reviews-during-atomic-spec-closeout]] is
  superseded if this ADR is accepted.
- [[SPEC-037-close-related-reviews-when-a-spec-is-done]] must be revised before
  either review closeout or promoted findings implementation proceeds.
- The kit gains a new additive artifact family, semantic MCP tools, indexes,
  diagnostics, migration guidance, and desktop route.
- Review history and finding lifecycle can disagree legitimately: an accepted
  review may point to active project debt.
- Migration remains partly human because legacy prose cannot establish global
  identity or final disposition safely.
- The 3.1.0 release must include both the core lifecycle and usable desktop
  surface; issue #12 is release-blocking by operator direction.

## Review conditions

Revisit if findings require nested sub-findings, partial resolution of one
canonical finding, cross-repository identity, remote issue synchronization, or
transactional storage replacing Markdown files.
