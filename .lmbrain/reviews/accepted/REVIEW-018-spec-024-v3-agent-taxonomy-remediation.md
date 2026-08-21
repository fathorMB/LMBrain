---
id: REVIEW-018
title: "Accepted review SPEC-024 v3 agent taxonomy remediation"
status: accepted
spec_id: SPEC-024
reviewer: AGENT-LEAD
created: 2026-07-02
updated: 2026-07-02
tags: [review, v3, agents]
links: [SPEC-024, REVIEW-017, ADR-008]
---

# Accepted review SPEC-024 v3 agent taxonomy remediation

## Verdict

Accepted.

The remediation addresses the blocking findings from [[REVIEW-017-spec-024-v3-agent-taxonomy]]. The implementation now preserves the operator approval boundary for proposed profiles and includes automated coverage for the new profile metadata, proposal metadata, diagnostics, and Agents/MCP UI rendering.

## Findings

No blocking findings.

## Review of prior findings

### R-1 - Guidance recommends proposed profiles without an approval/activation guard

Accepted as remediated.

Evidence:

- `kit/.lmbrain/AGENT.md` now says only active profiles may be used for implementation handoff and proposed granular profiles require operator approval/activation first.
- `kit/.lmbrain/OPERATOR.md` now tells the operator to activate a proposed recommended profile before starting the specialist.
- `kit/.lmbrain/agents/registry.md` now includes an activation guard stating proposed profiles are not ready for handoff.

### R-2 - Required automated coverage for new agent metadata/proposals is missing

Accepted as remediated.

Evidence:

- `src-tauri/tests/contract_test.rs` now covers v3 agent metadata parsing, legacy profile backward compatibility, v3 proposal metadata parsing, legacy proposal backward compatibility, area/domain mismatch diagnostics, and matching-domain quiet behavior.
- `src/__tests__/AgentsMCPView.test.tsx` now covers domain chips, review-focus chips, and improvement proposal target/type rendering.

## Acceptance criteria assessment

- [x] Agent profiles support optional structured specialization metadata while existing v2 profiles continue to parse.
- [x] The clean kit includes or proposes granular specialist profiles for recurring v3 work, all with `activation: manual`.
- [x] Project Lead guidance explains when to recommend each granular profile and preserves the active-profile handoff boundary.
- [x] Agents/MCP UI displays specialization metadata in a scannable way.
- [x] A controlled improvement flow exists for proposing profile/template changes from implementation evidence and reviews.
- [x] The improvement flow requires operator approval before behavior-affecting profile/template changes become active.
- [x] Diagnostics continue to detect unresolved `recommended_agent` values and do not break on optional metadata.
- [x] Tests cover profile parsing, backward compatibility, UI rendering, and new diagnostic/proposal behavior.

## Verification performed

- `cargo test` - passed for the full Rust workspace, including 21 contract tests.
- `pnpm lint` - passed.
- `pnpm test -- --runInBand` - passed, 56 tests across 13 files.
- `pnpm build` - passed; Vite emitted the existing large-chunk warning.
- Inspected `kit/.lmbrain/AGENT.md`, `kit/.lmbrain/OPERATOR.md`, `kit/.lmbrain/agents/registry.md`, `src-tauri/tests/contract_test.rs`, and `src/__tests__/AgentsMCPView.test.tsx`.

## Notes

The new granular profiles remain `status: proposed`, which is correct for the approved governance model. They should not be used for implementation handoff until the operator explicitly activates the selected profile.

SPEC-024 is accepted for the reviewed implementation. Before marking the spec `done`, ensure the controlled `spec_done` transition is used and that the spec's acceptance criteria/evidence satisfy the contract invariants.
