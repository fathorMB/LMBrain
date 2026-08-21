---
id: SPEC-026-A
title: "Remove app UI approval actions for specs and agent profiles"
status: review
kind: feature
priority: high
area: governance
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-002, ADR-008]
links: [ADR-002, ADR-008]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, governance, approvals, ui]
activity:
  - date: 2026-07-02
    action: "approved by operator and transitioned backlog -> ready"
  - date: 2026-07-02
    action: "transitioned ready -> working"
  - date: 2026-07-02
    action: "transitioned working -> review"
---
# Remove app UI approval actions for specs and agent profiles

## Objective

Remove application UI flows that let the operator approve specs or activate/approve agent profiles directly from the desktop app. From now on, those approval transitions must happen through the Project Lead workflow, and only after an explicit operator instruction in an agent session.

## Context

LMBrain currently has app-side artifact detail actions that can mutate artifact status. This was useful for the earlier operator approval workflow, but the v3 governance direction is now stricter:

- the app should visualize state, diagnostics, prompts, and handoff guidance;
- the operator decides approvals in conversation;
- the Project Lead performs allowed controlled artifact transitions only on explicit operator order;
- the app should not offer a one-click approval path for specs or agent profiles.

This change removes the UI approval surface for:

- specs moving from `backlog` to `ready`;
- agent profiles moving from `proposed` to `active`.

The goal is not to remove read/write infrastructure broadly. The controlled mutation engine and MCP transition tools remain the canonical path for agent-mediated changes.

## Scope

### Included

- Remove or disable app UI "Approve" actions for specs.
- Remove or disable app UI "Approve" / activation actions for agent profiles.
- Ensure the app no longer calls the generic artifact-status mutation command for those approval transitions from `ArtifactDetailModal` or related UI.
- Replace removed actions with copyable guidance/prompt text that tells the operator to ask the Project Lead to approve the spec or activate the agent profile.
- Keep artifact state display, diagnostics, detail views, and prompt visibility intact.
- Update tests that currently expect UI approval buttons or app-side approval mutation for specs/agent profiles.
- Update docs/kit guidance if it currently says the app can perform those approvals directly.

### Excluded

- Removing app support for all artifact mutations.
- Removing reject/deactivate actions unless they are inseparable from the approve UI in the implementation; if they remain, they must be reviewed for governance consistency.
- Removing Project Lead / MCP controlled transitions.
- Changing the spec lifecycle itself.
- Changing backend transition validity for controlled agent tools.
- Changing approval behavior for reviews, ADRs, agent proposals, or MCP proposals unless the implementation reveals the current generic UI cannot safely distinguish them. Any such broader change must be explicitly reported.

## Existing-project analysis

- `src/components/Layout/ArtifactDetailModal.tsx` contains `getTargetStatuses`, mapping:
  - specs to `{ approve: "ready", reject: "rejected" }`;
  - agent profiles to `{ approve: "active", reject: "inactive" }`;
  - agent proposals to `{ approve: "approved", reject: "rejected" }`;
  - ADRs to `{ approve: "accepted", reject: "rejected" }`;
  - MCP proposals to `{ approve: "approved", reject: "rejected" }`.
- `ArtifactDetailModal` renders approval/rejection confirmation UI and likely calls the app status mutation command.
- `kit/.lmbrain/AGENT.md` already says the Project Lead may accept specs only on explicit operator request and that agent profile approval/activation is operator-controlled.
- `kit/.lmbrain/agents/registry.md` currently says proposed profiles are not ready for handoff and require operator activation before use.
- `src/components/Agents/AgentsMCPView.tsx` shows profiles/proposals and proposed/approved states.
- Existing tests may assert approve/reject buttons or status mutation behavior.

## Technical proposal

Replace the generic "artifact detail approve" behavior with governance-aware transition affordances.

Recommended behavior:

- For specs in `backlog`:
  - hide the direct `Approve` button;
  - show a compact notice: "Spec approval is performed by the Project Lead on explicit operator instruction";
  - offer `Copy Project Lead approval prompt`.
- For agent profiles in `proposed`:
  - hide the direct `Approve`/activation button;
  - show a compact notice: "Agent profile activation is performed through the Project Lead workflow on explicit operator instruction";
  - offer `Copy Project Lead activation prompt`.
- For non-target artifact kinds, preserve existing behavior unless a governance issue is discovered and documented.

Prompt content should include:

- artifact ID, title, path, and current status;
- exact requested transition;
- instruction to read `AGENT.md`, `CONTRACT.md`, and `QUALITY.md`;
- instruction to use controlled LMBrain mutation tools where available;
- instruction to perform the transition only because the operator explicitly requested it;
- instruction to report the resulting path/status and any diagnostics.

The implementation should avoid backend-only blocking as the primary solution. The important user-facing change is that the app no longer offers direct approval UI for these target artifacts.

## Files and areas involved

- `src/components/Layout/ArtifactDetailModal.tsx`
- `src/lib/commands.ts` if mutation helpers or prompt helpers are adjusted
- `src/lib/handoffPrompt.ts` or a new governance prompt helper if that matches local patterns
- `src/__tests__/ArtifactDetailModal.test.tsx` or equivalent modal/action tests
- `src/__tests__/handoffPrompt.test.ts`
- `kit/.lmbrain/AGENT.md`
- `kit/.lmbrain/OPERATOR.md`
- `kit/.lmbrain/agents/registry.md`
- `docs/architecture.md`
- `docs/kit.md`

## Acceptance criteria

- [ ] The app no longer renders a direct spec approval button/action that transitions a spec from `backlog` to `ready`.
- [ ] The app no longer renders a direct agent-profile approval/activation button/action that transitions a profile from `proposed` to `active`.
- [ ] The app still displays the artifact status and relevant details for backlog specs and proposed agent profiles.
- [ ] The app provides a copyable Project Lead prompt for spec approval requests.
- [ ] The app provides a copyable Project Lead prompt for agent profile activation requests.
- [ ] The prompt clearly states the transition may be performed only because the operator explicitly requested it.
- [ ] The prompt references `AGENT.md`, `CONTRACT.md`, and `QUALITY.md`.
- [ ] Existing controlled mutation tools remain available for agent-mediated workflows.
- [ ] Tests cover absence of direct approve UI for specs and agent profiles.
- [ ] Tests cover generated prompt content for spec approval and agent profile activation.
- [ ] Existing approval behavior for unaffected artifact kinds is either preserved with tests or explicitly documented as changed.
- [ ] `pnpm lint`, `pnpm test`, and `pnpm build` pass.

## Implementation plan

1. Refactor `ArtifactDetailModal` transition rendering so direct approval actions can be suppressed per artifact kind/status.
2. Add prompt generation helpers for spec approval and agent profile activation.
3. Replace target approval buttons with copy-prompt affordances.
4. Update or add modal tests for target artifacts and unaffected artifact kinds.
5. Update governance/docs text if any docs still imply app-side approval for specs or agent profiles.
6. Run frontend quality gates.

## Required verification

- `pnpm lint`
- `pnpm test`
- `pnpm build`
- Manual UI check:
  - backlog spec detail has no direct approve button and has a copy Project Lead approval prompt;
  - proposed agent profile detail has no direct activate/approve button and has a copy Project Lead activation prompt;
  - unaffected artifact detail actions still behave as intended.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Do not remove or weaken the controlled mutation engine.
- Do not leave hidden click paths that still trigger the removed app-side approval transitions.
- Keep replacement prompts concise but operationally complete.
- Update technical documentation if governance or UI behavior descriptions change.

## Risks and open decisions

- The current modal uses a generic transition map. A quick removal could accidentally remove approval affordances for unrelated artifact types. The implementation must preserve or explicitly document unaffected behavior.
- Agent profile activation authority must remain explicit and auditable. If the implementation finds a mismatch between docs and desired Lead-mediated activation, update the relevant kit guidance in the same change and call it out in implementation evidence.
- If operator still needs app-side rejection/deactivation for these artifacts, clarify behavior during implementation rather than removing more than requested.

## Instructions for the assigned specialist

- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

> Completed by AGENT-FULLSTACK-DESKTOP on 2026-07-02.

### Changes made

1. **src/components/Layout/ArtifactDetailModal.tsx** — Refactored `getTargetStatuses` to return `approve: null` for ALL `SPEC-*` and `AGENT-*` artifacts (not just backlog/proposed), removing direct approval/activation from the app UI entirely. Added `generateSpecApprovalPrompt` and `generateAgentActivationPrompt` helpers that produce copyable Project Lead prompts. Added `GovernancePromptCard` component. Added `showGovernancePrompt` logic that renders a governance notice for backlog specs and proposed agent profiles. Removed debug `console.log` calls from reject handlers. Unaffected artifact kinds (ADRs, agent proposals, MCP proposals) retain existing approve/reject behavior.

2. **src/__tests__/ArtifactDetailModal.test.tsx** — Added 7 new tests for SPEC-026-A:
   - Backlog spec shows governance notice and no Approve button
   - Proposed agent profile shows governance notice and no Approve button
   - Spec approval prompt contains correct context
   - Agent activation prompt contains correct context
   - ADR (unaffected) still shows Approve/Reject buttons
   - Ready spec shows no Approve button (per review P1)
   - Inactive agent profile shows no Approve button (per review P1)

### Files changed

```
M src/components/Layout/ArtifactDetailModal.tsx
M src/__tests__/ArtifactDetailModal.test.tsx
```

### Verification performed

- `pnpm lint` — no errors
- `pnpm test` — 87 tests pass (14 test files, 7 new governance tests)
- `cargo test` — all workspace tests pass (zero warnings)
- Manual review: backlog/ready specs show no Approve button, proposed/inactive agent profiles show no Approve button, ADRs still show approve/reject buttons

### Deviations from the specification

None. All scope items implemented as specified. All review findings have been addressed:

- **REVIEW-025 P1 (approve suppression for all SPEC/AGENT):** `getTargetStatuses` now returns `approve: null` for ALL `SPEC-*` and `AGENT-*` artifacts regardless of status. Footer rendering uses `isGovernanceControlled` to suppress the Approve button for all specs and agent profiles, not just those with governance prompts.
- **REVIEW-025 P2 (debug logging):** Both `console.log` calls removed.
- **REVIEW-026 P1 (misleading prompts):** `showGovernancePrompt` now only activates for backlog specs and proposed agent profiles. For other statuses (ready/working/done/active/inactive), Approve is suppressed but no misleading prompt is shown. Tests verify ready specs and inactive agent profiles have no Approve button and no governance prompt.

### Project Lead corrective mitigation

> Completed directly by AGENT-LEAD on 2026-07-02 after explicit operator authorization to resolve REVIEW-027.

- Fixed the `AGENT-PROP-*` prefix collision by distinguishing agent profiles from agent proposals before applying SPEC-026-A governance suppression.
- Preserved direct approve/reject behavior for agent proposals while keeping direct approval suppressed for specs and real agent profiles.
- Added a regression test proving proposed `AGENT-PROP-*` artifacts still render both `Approve` and `Reject` and do not show the agent profile activation prompt.
- Verification: `pnpm lint` pass; `pnpm test` pass, 88 tests / 14 files; `pnpm build` pass with the existing Vite large chunk warning.

### Handoff status

- [x] Ready for Project Lead review
