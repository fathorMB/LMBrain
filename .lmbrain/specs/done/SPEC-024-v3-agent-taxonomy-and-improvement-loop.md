---
id: SPEC-024
title: "V3 agent taxonomy and controlled improvement loop"
status: done
kind: feature
priority: critical
area: agents
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-008]
links: [ADR-008, ADR-004]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, agents, kit, governance]
activity:
  - date: 2026-07-02
    action: "transitioned ready -> working"
  - date: 2026-07-02
    action: "transitioned working -> review"
  - date: 2026-07-02
    action: "accepted by REVIEW-018 and transitioned review -> done"
---
# V3 agent taxonomy and controlled improvement loop

## Objective

Make LMBrain's agent recommendations more granular and project-aware, while adding a controlled self-improvement loop that lets evidence from project work improve future agent profiles without allowing autonomous profile mutation.

## Context

The clean kit currently ships a Project Lead and limited specialist support. This is too coarse for v3: frontend UI, Tauri/Rust backend, MCP/contract, kit/release, design, and review work require different expertise and different context. The operator also wants agents to become better on the project over time.

The improvement loop must stay compatible with LMBrain governance: all activation remains manual, the Project Lead recommends but does not spawn agents, and operator approval is required before a new profile is active or an existing profile is materially changed.

## Scope

### Included

- Extend the kit's agent model with optional metadata for domains, primary files, review focus, context-pack preference, and known constraints.
- Add a v3 set of granular agent proposals/profiles for recurring bounded work: frontend UI, Tauri/Rust backend, MCP/contract, kit/docs/release, product reviewer/QA, and design if not already accepted through the existing design workflow.
- Add app parsing/types/UI support for the optional metadata without breaking existing profiles.
- Add a controlled improvement artifact or proposal flow for lessons learned from accepted reviews, remediation loops, and implementation evidence.
- Update Project Lead guidance to recommend existing granular profiles before proposing new ones.
- Update diagnostics for unresolved recommended agents and optionally for specs whose area does not match the selected agent domain.

### Excluded

- Autonomous agent spawning.
- Agent profiles that rewrite themselves.
- Automatic profile activation without operator approval.
- Model-provider-specific routing.
- A database specialist profile unless the product introduces a database-backed subsystem.

## Existing-project analysis

- `src-tauri/src/models/agent.rs` and `src/types/index.ts` expose only basic agent fields.
- `src/components/Agents/AgentsMCPView.tsx` lists profiles/proposals but does not expose domain or fit information.
- `kit/.lmbrain/templates/agent-profile.md` has no structured specialization metadata.
- `kit/.lmbrain/AGENT.md` already states that profiles are manual and operator-approved.
- `lmbrain-core` and `lmbrain-mcp` already validate that a spec's `recommended_agent` resolves to a profile.

## Technical proposal

Adopt ADR-008's governance model: agents may generate improvement proposals from evidence, but only operator-approved artifacts change agent behavior.

Add optional frontmatter fields to agent profiles:

```yaml
domains: [frontend, ui, tauri, rust, mcp, docs]
primary_files: [src/components, src-tauri/src]
review_focus: [accessibility, path-safety, contract-invariants]
context_pack: spec
constraints: []
```

Preferred improvement mechanism: extend existing `agents/proposals/` so proposals can target either a new specialist profile or an update to an existing specialist profile/template. Add a new managed artifact family only if the existing proposal model cannot express the need cleanly.

## Files and areas involved

- `kit/.lmbrain/templates/agent-profile.md`
- `kit/.lmbrain/templates/agent-proposal.md`
- `kit/.lmbrain/agents/profiles/`
- `kit/.lmbrain/agents/proposals/`
- `kit/.lmbrain/agents/registry.md`
- `kit/.lmbrain/AGENT.md`
- `kit/.lmbrain/CONTRACT.md`
- `kit/.lmbrain/OPERATOR.md`
- `src-tauri/src/models/agent.rs`
- `src-tauri/src/commands/contract.rs`
- `src/types/index.ts`
- `src/components/Agents/AgentsMCPView.tsx`
- `lmbrain-core/src/invariants.rs`
- `lmbrain-mcp/src/main.rs`
- `docs/kit.md`
- `docs/architecture.md`
- `docs/product.md`

## Acceptance criteria

- [x] Agent profiles support optional structured specialization metadata while existing v2 profiles continue to parse.
- [x] The clean kit includes or proposes granular specialist profiles for recurring v3 work, all with `activation: manual`.
- [x] Project Lead guidance explains when to recommend each granular profile.
- [x] Agents/MCP UI displays specialization metadata in a scannable way.
- [x] A controlled improvement flow exists for proposing profile/template changes from implementation evidence and reviews.
- [x] The improvement flow requires operator approval before behavior-affecting profile/template changes become active.
- [x] Diagnostics continue to detect unresolved `recommended_agent` values and do not break on optional metadata.
- [x] Tests cover profile parsing, backward compatibility, UI rendering, and any new invariant/proposal behavior.

## Implementation plan

1. Confirm ADR-008 or update this spec if the operator rejects that governance model.
2. Extend agent profile/proposal templates and parser models with optional metadata.
3. Add granular v3 agent artifacts to the clean kit using the normal profile/proposal locations.
4. Add UI rendering for domains, review focus, and profile fit.
5. Add or extend a controlled improvement proposal mechanism.
6. Update kit/docs and diagnostics.
7. Add tests and run quality gates.

## Required verification

- `pnpm lint`
- `pnpm test`
- `cargo test`
- Manual app check of Agents/MCP view with old and new profile files.
- Manual validation that no profile is auto-activated by the improvement loop.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Do not weaken agent governance to satisfy the "self-improving" goal.
- Document any new artifact fields in the kit contract and migration notes.

## Risks and open decisions

- Too many profiles can make recommendations noisy. Recommendation: ship a small curated set tied to recurring work.
- A new artifact family may be overkill. Recommendation: extend agent proposals unless the implementation proves it cannot represent profile updates cleanly.
- If ADR-008 is rejected, this spec must be revised before implementation.

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

1. **src-tauri/src/models/agent.rs** — Extended `AgentProfile` with optional v3 fields: `domains`, `primary_files`, `review_focus`, `context_pack`, `constraints`. Extended `AgentProposal` with `proposal_type` and `target_profile` fields. All fields are optional for backward compatibility.

2. **src-tauri/src/commands/parser.rs** — Added `fm_string_array_opt` helper for parsing optional string arrays that distinguish "absent" from "empty".

3. **src-tauri/src/commands/contract.rs** — Updated `build_agents` and `build_agent_proposals` to read the new optional fields. Added domain/area matching diagnostics: warns when a spec's `area` does not match the recommended agent's `domains`.

4. **src/types/index.ts** — Updated `AgentProfile` and `AgentProposal` TypeScript interfaces with the new optional fields.

5. **src/components/Agents/AgentsMCPView.tsx** — Updated `AgentCard` to display domains (blue pills) and review_focus (gray pills) when present. Updated `AgentProposalCard` to show proposal type ("Improvement proposal" vs "New-profile proposal") and target profile, with distinct styling for improvement proposals.

6. **kit/.lmbrain/templates/agent-profile.md** — Added v3 specialization metadata fields (domains, primary_files, review_focus, context_pack, constraints) as optional frontmatter.

7. **kit/.lmbrain/templates/agent-proposal.md** — Added `proposal_type` and `target_profile` fields, plus an "Evidence" section for improvement proposals.

8. **kit/.lmbrain/agents/profiles/** — Added 6 granular v3 agent profiles:
   - `frontend-ui-specialist.md` (AGENT-FRONTEND-UI)
   - `tauri-backend-specialist.md` (AGENT-TAURI-BACKEND)
   - `mcp-contract-specialist.md` (AGENT-MCP-CONTRACT)
   - `kit-docs-specialist.md` (AGENT-KIT-DOCS)
   - `product-reviewer.md` (AGENT-REVIEWER)
   - `design-specialist.md` (AGENT-DESIGN)
   All use `activation: manual` and `status: proposed`.

9. **kit/.lmbrain/agents/proposals/improvement-example.md** (new) — Example improvement proposal demonstrating the controlled improvement loop.

10. **kit/.lmbrain/agents/registry.md** — Updated registry table with all new profiles and domains. Added V3 controlled improvement loop section.

11. **kit/.lmbrain/AGENT.md** — Added step 9 to "When receiving a feature request": guidance on recommending granular profiles based on spec area and files. Updated per review R-1: now explicitly says to use only **active** profiles for handoff and to ask the operator to approve/activate proposed profiles first.

12. **kit/.lmbrain/OPERATOR.md** — Updated per review R-1: step 6 now instructs the operator to activate proposed profiles before starting the specialist.

13. **kit/.lmbrain/agents/registry.md** — Added activation guard section: proposed profiles are not ready for handoff; operator must set `status: active` first.

14. **docs/kit.md** — Added V3 agent taxonomy section with profile table and improvement loop description.

15. **docs/architecture.md** — Added agent profile specialization metadata and improvement proposal documentation.

16. **src-tauri/tests/contract_test.rs** — Added 6 new tests per review R-2:
    - `test_build_agents_parses_v3_metadata_fields` — verifies domains, primary_files, review_focus, context_pack, constraints parsing
    - `test_build_agents_backward_compatible_without_v3_fields` — verifies legacy profiles parse with None for all v3 fields
    - `test_build_agent_proposals_parses_v3_fields` — verifies proposal_type and target_profile parsing
    - `test_build_agent_proposals_backward_compatible_without_v3_fields` — verifies legacy proposals parse with None
    - `test_build_diagnostics_area_domain_mismatch` — verifies warning when spec area doesn't match agent domains
    - `test_build_diagnostics_area_domain_match_stays_quiet` — verifies no warning when domains match

17. **src/__tests__/AgentsMCPView.test.tsx** — Added 3 new tests per review R-2:
    - Renders domain chips (frontend, ui, react) for agents with specialization metadata
    - Renders review focus chips (accessibility, state-management) for agents with specialization metadata
    - Renders improvement proposal with target profile label and "Improvement proposal" type indicator

### Files changed

```
M src-tauri/src/models/agent.rs
M src-tauri/src/commands/parser.rs
M src-tauri/src/commands/contract.rs
M src-tauri/tests/contract_test.rs
M src/types/index.ts
M src/components/Agents/AgentsMCPView.tsx
M src/__tests__/AgentsMCPView.test.tsx
M kit/.lmbrain/templates/agent-profile.md
M kit/.lmbrain/templates/agent-proposal.md
A kit/.lmbrain/agents/profiles/frontend-ui-specialist.md
A kit/.lmbrain/agents/profiles/tauri-backend-specialist.md
A kit/.lmbrain/agents/profiles/mcp-contract-specialist.md
A kit/.lmbrain/agents/profiles/kit-docs-specialist.md
A kit/.lmbrain/agents/profiles/product-reviewer.md
A kit/.lmbrain/agents/profiles/design-specialist.md
A kit/.lmbrain/agents/proposals/improvement-example.md
M kit/.lmbrain/agents/registry.md
M kit/.lmbrain/AGENT.md
M kit/.lmbrain/OPERATOR.md
M docs/kit.md
M docs/architecture.md
```

### Verification performed

- `cargo test` — all workspace tests pass (zero warnings). Contract tests: 21 passed (6 new)
- `pnpm lint` — no errors
- `pnpm test` — 56 tests pass (13 test files, 3 new UI tests)
- Manual check: existing v2 profiles parse correctly with the new optional fields (backward compatible)
- Manual check: new profiles have `activation: manual` and `status: proposed` — no auto-activation
- Manual check: AGENT.md guidance now requires operator approval before recommending proposed profiles

### Deviations from the specification

None. All scope items implemented as specified. All review findings (R-1, R-2) have been addressed:

- **R-1 (activation guard):** AGENT.md, OPERATOR.md, and registry.md updated to require operator approval/activation before proposed profiles are recommended for handoff.
- **R-2 (test coverage):** 6 new Rust tests (profile/proposal parsing, backward compatibility, area/domain diagnostics) and 3 new frontend tests (domain chips, review focus chips, improvement proposal labeling) added.

### Handoff status

- [x] Ready for Project Lead review

### Final review

Accepted by [[REVIEW-018-spec-024-v3-agent-taxonomy-remediation]] on 2026-07-02 and closed as done. The review verified the original findings were remediated, all acceptance criteria were satisfied, and the required frontend/Rust verification passed.
