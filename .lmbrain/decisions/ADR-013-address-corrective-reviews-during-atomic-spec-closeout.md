---
id: ADR-013
title: "Address corrective reviews during atomic specification closeout"
status: accepted
decision_date: 2026-07-12
decider: operator
supersedes: []
superseded_by: []
links: [SPEC-037]
tags: [architecture, reviews, lifecycle, transactions, history]
created: 2026-07-12
updated: 2026-07-12
activity:
  - date: 2026-07-12
    action: "accepted by operator"
  - date: 2026-07-12
    action: "created"
---

# Address corrective reviews during atomic specification closeout

## Context

A corrective review's verdict remains historically true after remediation, but its findings are no longer actionable once a later accepted review closes the spec. Keeping `status: changes-requested` forever makes resolved work appear open. Converting it to `accepted` falsifies history, while converting it to generic `superseded` erases the verdict from current quality statistics.

Specification completion and review resolution are one logical lifecycle event. Performing them as unrelated writes risks a done spec with apparently open corrective reviews.

## Proposed decision

- Add terminal review status `addressed` for resolved corrective reviews.
- Preserve the original `changes-requested` or eligible `blocked` verdict in an `outcome` field.
- Record the accepted review that resolved the findings through `resolved_by`, plus resolution date and audit activity.
- Route `spec_done` through a dedicated atomic multi-artifact completion operation.
- Keep accepted reviews accepted and reject closeout while related reviews remain pending or ambiguous. A blocked review is addressed only when the selected accepted review explicitly includes its ID in `links`.
- Base historical metrics on review outcome, with backward-compatible inference from legacy status.
- Provide explicit preview/apply reconciliation for existing done specs; never reconcile automatically during workspace open.

## Alternatives considered

### Move corrective reviews to superseded

Rejected because it loses their verdict unless another field is added and makes historical quality metrics silently improve after closeout.

### Change corrective reviews to accepted

Rejected because acceptance is a different verdict and would rewrite history falsely.

### Leave status unchanged and add only a resolved flag

Rejected because status-directory navigation and current UI continue presenting the artifact as actionable; it also creates two competing lifecycle sources.

### Let the Lead close reviews manually after spec_done

Rejected because the exact consistency bug comes from relying on a separate manual cleanup step.

## Consequences

- Review status and original outcome become distinct concepts for addressed reviews.
- `spec_done` changes from a single-file transition to a cross-artifact transaction with locking and crash recovery requirements.
- Statistics and joins must interpret legacy and addressed reviews consistently.
- Existing projects need optional controlled reconciliation and may surface ambiguities requiring operator/Lead resolution.
- The Reviews UI gains a meaningful separation between actionable work and historical remediation cycles.

## Review conditions

Revisit if reviews gain multiple independent finding lifecycles, partial finding resolution, cross-spec reviews, or the artifact store moves from files to a transactional database.
