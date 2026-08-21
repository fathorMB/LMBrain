---
id: REVIEW-028
title: "Review SPEC-026-A Project Lead mitigation"
status: accepted
spec_id: SPEC-026-A
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, governance, approvals, mitigation]
links: [SPEC-026-A, REVIEW-025, REVIEW-026, REVIEW-027]
---
# Review SPEC-026-A Project Lead mitigation

## Verdict

Accepted.

The REVIEW-027 regression has been fixed. Agent proposals are no longer accidentally treated as agent profiles, so their existing direct approve/reject behavior is preserved while SPEC-026-A governance suppression still applies to specs and real agent profiles.

## Verification

- `src/components/Layout/ArtifactDetailModal.tsx` now distinguishes:
  - `SPEC-*` artifacts;
  - agent profiles matching `AGENT-*` but not `AGENT-PROP-*`;
  - agent proposals matching `AGENT-PROP-*`, which remain in the standard proposed-artifact transition path.
- `src/__tests__/ArtifactDetailModal.test.tsx` includes a regression test for proposed `AGENT-PROP-*` artifacts, asserting both `Approve` and `Reject` remain visible and the agent profile activation prompt is not shown.
- Existing governance tests still cover:
  - backlog spec prompt with no direct `Approve`;
  - proposed agent profile prompt with no direct `Approve`;
  - ready spec with no direct `Approve` and no misleading prompt;
  - inactive agent profile with no direct `Approve` and no misleading prompt;
  - ADR approve/reject behavior preserved.

## Gates

- `pnpm lint` - pass.
- `pnpm test` - pass, 88 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.

## Result

SPEC-026-A is accepted from review. It can be moved to `done` after the implementation commit/evidence step required by the LMBrain workflow.
