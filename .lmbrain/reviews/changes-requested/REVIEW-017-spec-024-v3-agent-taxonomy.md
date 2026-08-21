---
id: REVIEW-017
title: "Review SPEC-024 v3 agent taxonomy"
status: changes-requested
spec_id: SPEC-024
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, agents]
links: [SPEC-024, ADR-008]
---

# Review SPEC-024 v3 agent taxonomy

## Verdict

Changes requested.

The implementation adds the expected metadata fields, UI rendering, proposed granular profiles, and an improvement-proposal shape. The quality gates pass. However, two acceptance criteria are not satisfied yet: Project Lead guidance currently tells the Lead to prefer proposed profiles as if they were ready for handoff, and the new parsing/UI/diagnostic behavior lacks direct automated coverage.

## Findings

### R-1 - Guidance recommends proposed profiles without an approval/activation guard

Severity: blocking

Evidence:

- The newly added kit guidance says to "Prefer granular profiles (AGENT-FRONTEND-UI, AGENT-TAURI-BACKEND, AGENT-MCP-CONTRACT, AGENT-KIT-DOCS, AGENT-REVIEWER, AGENT-DESIGN) over the generic fullstack specialist when the work is bounded to one area."
- The new granular profile files are all `status: proposed`, for example `AGENT-FRONTEND-UI`.

References:

- `kit/.lmbrain/AGENT.md:23`
- `kit/.lmbrain/agents/profiles/frontend-ui-specialist.md:4`

Why this blocks acceptance:

ADR-008 and the LMBrain contract keep profile approval/activation under operator control. A proposed profile is not an active specialist profile for immediate handoff. The guidance should either instruct the Lead to recommend active granular profiles only, or to ask the operator to approve/activate the proposed profile before using it for implementation handoff.

Required correction:

Update Project Lead/operator/kit guidance so proposed granular profiles are treated as pending profiles. The recommended handoff path should be explicit: use an already active matching profile, or ask the operator to approve/activate the proposed granular profile before recommending it. Do not imply proposed profiles are directly ready for implementation assignment.

### R-2 - Required automated coverage for new agent metadata/proposals is missing

Severity: blocking

Evidence:

- Existing frontend test `AgentsMCPView.test.tsx` still covers only hiding materialized approvals. It does not assert rendering of `domains`, `review_focus`, improvement proposal labeling, or `target_profile`.
- Existing Rust tests still cover unresolved/resolved `recommended_agent` and status mismatch diagnostics, but not parsing of optional profile metadata, proposal metadata, backward compatibility with absent fields, or the new area/domain mismatch diagnostic.
- The implementation evidence reports manual checks for backward compatibility and proposed/manual activation status, but SPEC-024 requires tests.

References:

- `src/__tests__/AgentsMCPView.test.tsx:84`
- `src-tauri/tests/contract_test.rs:58`
- `src-tauri/tests/contract_test.rs:93`

Why this blocks acceptance:

SPEC-024 acceptance criteria explicitly require tests covering profile parsing, backward compatibility, UI rendering, and new invariant/proposal behavior. The current automated suite passes, but it does not exercise the new feature surface.

Required correction:

Add focused tests for:

1. `build_agents` parses `domains`, `primary_files`, `review_focus`, `context_pack`, and `constraints`, while old profiles without these fields still parse.
2. `build_agent_proposals` parses `proposal_type` and `target_profile`, while old proposals without these fields still parse.
3. `build_diagnostics` emits an area/domain mismatch warning for a mismatched active/recommended agent and stays quiet for a matching domain.
4. `AgentsMCPView` renders domain/review-focus chips and improvement proposal target/profile labeling.

## Acceptance criteria assessment

- [x] Agent profiles support optional structured specialization metadata while existing v2 profiles continue to parse at the model level.
- [x] The clean kit includes or proposes granular specialist profiles for recurring v3 work, all with `activation: manual`.
- [ ] Project Lead guidance explains when to recommend each granular profile. It currently omits the approval/activation guard for proposed profiles.
- [x] Agents/MCP UI implements specialization metadata display.
- [x] A controlled improvement flow exists for proposing profile/template changes from implementation evidence and reviews.
- [ ] The improvement flow requires operator approval before behavior-affecting profile/template changes become active. Governance docs say this, but handoff guidance currently risks bypassing it for proposed granular profiles.
- [x] Diagnostics continue to detect unresolved `recommended_agent` values and do not break on optional metadata.
- [ ] Tests cover profile parsing, backward compatibility, UI rendering, and new invariant/proposal behavior.

## Verification performed

- `cargo test` - passed for the full Rust workspace.
- `pnpm lint` - passed.
- `pnpm test -- --runInBand` - passed, 53 tests across 13 files.
- `pnpm build` - passed; Vite emitted the existing large-chunk warning.
- Inspected new agent profile/proposal artifacts, parser/model changes, diagnostics changes, Agents/MCP UI changes, and kit/docs guidance.

## Required remediation

Keep remediation inside SPEC-024:

1. Add the approval/activation guard to Project Lead/operator guidance for proposed granular profiles.
2. Add automated tests for the new metadata, proposal, UI, and area/domain diagnostic behavior.
3. Update SPEC-024 implementation evidence with the new verification results.

Resubmit SPEC-024 for review after those changes.
