---
id: REVIEW-025
title: "Review SPEC-026-A remove UI approval actions"
status: changes-requested
spec_id: SPEC-026-A
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, governance, approvals]
links: [SPEC-026-A]
---
# Review SPEC-026-A remove UI approval actions

## Verdict

Changes requested.

The implementation adds the right governance prompt surface for backlog specs and proposed agent profiles, and the frontend gates pass. However, the approval UI is only suppressed for those exact statuses; direct `Approve` actions can still appear for other specs and agent profiles, which does not satisfy the requested governance change.

## Findings

### [P1] Direct approval UI still appears for non-backlog specs and non-proposed agent profiles

`getTargetStatuses` suppresses spec approval only when `status === "backlog"` and agent profile activation only when `status === "proposed"`:

- `src/components/Layout/ArtifactDetailModal.tsx:8`
- `src/components/Layout/ArtifactDetailModal.tsx:21`

The modal then explicitly enables transition controls for every `SPEC-*` and `AGENT-*` regardless of status:

- `src/components/Layout/ArtifactDetailModal.tsx:231`
- `src/components/Layout/ArtifactDetailModal.tsx:233`

So a ready/working/review/done spec can still render an `Approve` button targeting `ready`, and inactive/active agent profiles can still render an `Approve` button targeting `active`. The spec asks to remove app UI approval actions for specs and agent profiles, not merely hide them for the most common starting states.

Required remediation:

- Do not render any direct `Approve` action for `SPEC-*` artifacts from the app UI.
- Do not render any direct `Approve` / activation action for `AGENT-*` profile artifacts from the app UI.
- Keep governance prompts for the actionable request states (`backlog` specs and `proposed` agent profiles).
- Preserve or explicitly document reject/deactivate behavior according to the spec.
- Add tests proving `Approve` is absent for at least:
  - backlog spec;
  - ready spec;
  - proposed agent profile;
  - inactive agent profile.

### [P2] Debug logging remains in production click handlers

The modal includes `console.log("reject button clicked!")` in both reject button handlers:

- `src/components/Layout/ArtifactDetailModal.tsx:507`
- `src/components/Layout/ArtifactDetailModal.tsx:528`

This is not appropriate production UI code and should be removed.

## Acceptance Criteria Assessment

- App no longer renders direct spec approval button/action for `backlog -> ready`: pass for backlog only.
- App no longer renders direct agent-profile approval/activation button/action for `proposed -> active`: pass for proposed only.
- Artifact status/details remain visible: pass.
- Copyable Project Lead prompt for spec approval: pass.
- Copyable Project Lead prompt for agent activation: pass.
- Prompt states operator explicit request and references governance docs: pass.
- Controlled mutation tools remain available: pass.
- Tests cover absence of direct approve UI: partial; only backlog/proposed states covered.
- Tests cover prompt content: pass.
- Unaffected artifact behavior preserved: pass for ADR test.
- `pnpm lint`, `pnpm test`, and `pnpm build`: pass.

## Verification Performed

- `pnpm lint` - pass.
- `pnpm test` - pass, 85 tests / 14 files.
- `pnpm build` - pass; existing Vite large chunk warning remains.
- Static review of `ArtifactDetailModal` and `ArtifactDetailModal.test.tsx`.

## Required Remediation

1. Suppress direct `Approve` UI for all spec artifacts in `ArtifactDetailModal`.
2. Suppress direct `Approve` / activation UI for all agent profile artifacts in `ArtifactDetailModal`.
3. Add missing tests for ready specs and inactive agent profiles so the approval UI cannot reappear through adjacent statuses.
4. Remove debug `console.log` calls from reject handlers.
5. Re-run `pnpm lint`, `pnpm test`, and `pnpm build`.
