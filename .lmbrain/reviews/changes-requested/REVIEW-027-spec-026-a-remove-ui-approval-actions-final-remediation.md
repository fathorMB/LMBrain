---
id: REVIEW-027
title: "Re-review SPEC-026-A remove UI approval actions final remediation"
status: changes-requested
spec_id: SPEC-026-A
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, governance, approvals, remediation]
links: [SPEC-026-A, REVIEW-025, REVIEW-026]
---
# Re-review SPEC-026-A remove UI approval actions final remediation

## Verdict

Changes requested.

The REVIEW-026 finding is resolved: governance prompts are now limited to backlog specs and proposed agent profiles, while ready specs and inactive agent profiles no longer show misleading approval prompts. However, the remediation introduced a regression for an explicitly unaffected artifact kind.

## Findings

### [P1] Agent proposals are accidentally treated as agent profiles and lose direct approval

`isGovernanceControlled` currently classifies every `AGENT-*` artifact as governance-controlled:

- `src/components/Layout/ArtifactDetailModal.tsx:227`

That prefix also matches `AGENT-PROP-*`. As a result, an agent proposal still receives the correct transition mapping from `getTargetStatuses`:

- `src/components/Layout/ArtifactDetailModal.tsx:16`

but the footer branches on `isGovernanceControlled` and renders only the reject/deactivate-style single action, suppressing the `Approve` button:

- `src/components/Layout/ArtifactDetailModal.tsx:498`
- `src/components/Layout/ArtifactDetailModal.tsx:503`

This violates SPEC-026-A, which explicitly excludes agent proposals from the change unless a broader change is documented and reported. It also conflicts with the implementation evidence claiming agent proposals retain existing approve/reject behavior.

Required remediation:

1. Distinguish agent profiles from agent proposals before applying governance suppression, for example by using an `isAgentProfile` predicate that excludes `AGENT-PROP-*`.
2. Keep direct `Approve` suppressed for `SPEC-*` artifacts and real agent profiles.
3. Preserve existing direct approve/reject behavior for `AGENT-PROP-*`.
4. Add a regression test proving a proposed `AGENT-PROP-*` artifact still renders both `Approve` and `Reject`.

## Resolved Items

- REVIEW-026 P1 misleading prompts for non-actionable states: resolved.
- REVIEW-025 P1 direct `Approve` suppression for all specs/profiles: still resolved for the intended artifact classes.
- REVIEW-025 P2 debug logging: resolved.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 87 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- Static review of `ArtifactDetailModal` and `ArtifactDetailModal.test.tsx`.

## Required Remediation

Fix the `AGENT-PROP-*` prefix collision, add the unaffected-kind regression test, and re-run `pnpm lint`, `pnpm test`, and `pnpm build`.
