---
id: REVIEW-026
title: "Re-review SPEC-026-A remove UI approval actions remediation"
status: changes-requested
spec_id: SPEC-026-A
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, governance, approvals, remediation]
links: [SPEC-026-A, REVIEW-025]
---
# Re-review SPEC-026-A remove UI approval actions remediation

## Verdict

Changes requested.

The REVIEW-025 findings are mostly addressed: direct `Approve` is now suppressed for all spec artifacts and all agent profile artifacts, adjacent-status tests were added, and debug logging was removed. One governance-copy regression remains.

## Findings

### [P1] Governance prompt is shown for non-actionable spec/profile states with hard-coded wrong transition

`showGovernancePrompt` is true for every `SPEC-*` and `AGENT-*` artifact because `getTargetStatuses` now returns `approve: null` for all of them:

- `src/components/Layout/ArtifactDetailModal.tsx:227`
- `src/components/Layout/ArtifactDetailModal.tsx:232`

The rendered prompt then always uses the hard-coded approval/activation generators:

- `src/components/Layout/ArtifactDetailModal.tsx:445`

Those generators always say:

- spec current status is `backlog` and requested transition is `backlog -> ready`;
- agent profile current status is `proposed` and requested transition is `proposed -> active`.

See:

- `src/components/Layout/ArtifactDetailModal.tsx:45`
- `src/components/Layout/ArtifactDetailModal.tsx:60`

This means a ready/review/done spec can display a Project Lead prompt asking to approve it from backlog to ready, and an inactive/active profile can display a prompt claiming it is proposed. That can mislead the operator and Project Lead.

Required remediation:

- Show the Project Lead approval prompt only for backlog specs.
- Show the Project Lead activation prompt only for proposed agent profiles.
- For other spec/profile statuses, keep direct `Approve` suppressed but do not show a misleading approval/activation prompt. A neutral governance note is acceptable, but it must not request the wrong transition.
- Add tests proving:
  - ready spec has no direct `Approve` and no backlog-to-ready prompt;
  - inactive agent profile has no direct `Approve` and no proposed-to-active prompt;
  - backlog spec still has the spec approval prompt;
  - proposed agent profile still has the activation prompt.

## Resolved Items

- REVIEW-025 P1 direct `Approve` suppression for all specs/profiles: resolved.
- REVIEW-025 P2 debug logging: resolved.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 87 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- Static review of `ArtifactDetailModal` and `ArtifactDetailModal.test.tsx`.

## Required Remediation

1. Restrict action-specific Project Lead prompts to the actual actionable states: backlog spec and proposed agent profile.
2. Keep `Approve` suppressed for all spec/profile statuses.
3. Add negative prompt tests for ready specs and inactive agent profiles.
4. Re-run `pnpm lint`, `pnpm test`, and `pnpm build`.
