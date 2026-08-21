---
id: SPEC-023
title: "V3 context economy and token-efficient agent workflow"
status: done
kind: feature
priority: critical
area: workflow
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-004]
links: [ADR-004]
created: 2026-07-02
updated: 2026-07-02
tags: [v3, tokens, workflow, mcp, kit]
activity:
  - date: 2026-07-02
    action: "set recommended_agent"
  - date: 2026-07-02
    action: "transitioned ready -> working"
  - date: 2026-07-02
    action: "transitioned working -> review"
  - date: 2026-07-02
    action: "transitioned review -> done"
---
# V3 context economy and token-efficient agent workflow

## Objective

Reduce token waste in the LMBrain operating workflow by replacing broad "read everything" handoffs with deterministic, scoped context packs backed by the contract, app data model, and `lmbrain-mcp` read tools.

## Context

Current LMBrain guidance asks Project Leads and specialists to read large portions of `.lmbrain/`, then inspect the codebase. This is safe but costly. The app and MCP server already know how to parse specs, reviews, ADRs, agents, handoffs, diagnostics, and roadmap data, but they do not expose a compact, role-aware context surface.

The v3 workflow should preserve quality and auditability while making the default path cheaper:

- Project Lead analysis starts from project pulse, active work, roadmap, and relevant artifact references.
- Specialist handoff includes only mandatory contract files, the assigned spec, linked decisions, relevant agent profile, and named code areas.
- Review focuses on implementation evidence, diff, acceptance criteria, linked decisions, and quality policy.

## Scope

### Included

- Define context tiers in the kit: mandatory, relevant, optional, and forbidden for each role/workflow.
- Add MCP read tools that return compact, deterministic context packs for Project Lead, specialist, and review workflows.
- Update generated handoff prompts to ask for the smallest sufficient context first and to expand only when evidence requires it.
- Add diagnostics or validation for specs whose handoff context is underspecified, for example missing recommended agent, linked ADR, affected area, or files.
- Update app UI copy where it displays handoff prompts so the operator sees the compact prompt and can still inspect the source artifacts.
- Update kit docs, templates, and migrations for v3 context-economy guidance.

### Excluded

- Automatic agent spawning.
- Hidden summarization that replaces source artifacts as the system of record.
- Any workflow that allows skipping `QUALITY.md`, acceptance criteria, or linked architectural decisions.
- Model-provider-specific token accounting or pricing features.

## Existing-project analysis

- `src/lib/handoffPrompt.ts` currently generates a concise prompt, but it still relies on the human/agent to discover the right surrounding artifacts.
- `lmbrain-mcp/src/main.rs` exposes `lmbrain_get_artifact`, `lmbrain_validate`, and `lmbrain_list_ready_handoffs`, but not role-specific context bundles.
- `src-tauri/src/commands/contract.rs` already builds most app aggregates needed for context selection.
- `.lmbrain/AGENT.md` and `kit/.lmbrain/templates/project-lead-bootstrap-prompt.md` still encourage broad reading for Project Lead bootstrap.
- `docs/architecture.md` documents the MCP surface and should be updated when new read tools are added.

## Technical proposal

Add read-only MCP tools backed by existing parser/contract logic:

- `lmbrain_project_digest`: project title/status, current milestone, ready/review specs, blockers, ready handoffs, active decisions, diagnostics summary, and version/health warnings.
- `lmbrain_spec_context`: spec metadata, acceptance criteria checklist, linked decisions, recommended agent profile summary, related reviews, referenced milestone, explicit files/areas if present, and diagnostics affecting the handoff.
- `lmbrain_review_context`: acceptance criteria, implementation evidence, linked accepted/proposed reviews, relevant decisions, and verification commands claimed by the specialist.

The tools must return compact JSON and a human-readable Markdown summary. They must not mutate artifacts. They should resolve references through existing ID/path logic and report missing links as structured warnings.

Update kit guidance so agents read mandatory policy files first, use MCP context packs for initial orientation, expand to full artifacts/code only when the pack points to them or direct verification requires it, and record evidence when they expand scope.

## Files and areas involved

- `lmbrain-mcp/src/main.rs`
- `lmbrain-core/src/` if shared context selection belongs in core
- `src-tauri/src/commands/contract.rs`
- `src/lib/handoffPrompt.ts`
- `src/components/Pulse/ProjectPulse.tsx`
- `src/components/Spec/SpecDetail.tsx`
- `src/components/Agents/AgentsMCPView.tsx`
- `src/types/index.ts`
- `kit/.lmbrain/AGENT.md`
- `kit/.lmbrain/OPERATOR.md`
- `kit/.lmbrain/templates/project-lead-bootstrap-prompt.md`
- `kit/.lmbrain/templates/spec.md`
- `kit/.lmbrain/CONTRACT.md`
- `kit/.lmbrain/MIGRATIONS.md`
- `docs/architecture.md`
- `docs/kit.md`
- `docs/agent-hosts.md`

## Acceptance criteria

- [x] The kit documents role-specific context tiers and explicitly discourages broad artifact reads when a smaller context pack is sufficient.
- [x] `lmbrain-mcp` exposes read-only context-pack tools for project digest, spec handoff context, and review context.
- [x] Context-pack tools resolve linked specs, ADRs, reviews, agent profiles, roadmap milestone, diagnostics, and missing-reference warnings deterministically.
- [x] Context-pack tools do not mutate files and are covered by protocol/core tests.
- [x] Generated specialist handoff prompts reference the assigned spec and recommend the new context-pack flow without weakening the requirement to inspect source code where needed.
- [x] The app's Agents/MCP view lists the new tools with accurate descriptions.
- [x] Existing tests pass and new tests cover compact prompt generation and MCP context-pack behavior.
- [x] Documentation explains the expected token-saving behavior without claiming unmeasured savings.

## Implementation plan

1. Define the exact context-pack schemas and place reusable selection logic in the most appropriate Rust crate.
2. Add MCP tool definitions and handlers.
3. Add tests for normal, missing-link, missing-agent, and malformed-artifact cases.
4. Update frontend prompt generation and MCP tool display.
5. Update kit and docs.
6. Run quality gates and record verification evidence.

## Required verification

- `pnpm lint`
- `pnpm test`
- `cargo test`
- Manual MCP smoke test for each new read tool on this repository.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Preserve source artifacts as the audit trail; context packs are derived views only.
- Update all relevant kit and app documentation.

## Risks and open decisions

- Context packs can become misleading if they summarize too aggressively. Recommendation: include references and warnings rather than narrative compression only.
- Context-pack schema belongs either in `lmbrain-core` or the MCP crate. Recommendation: put pure derivation in core only if the Tauri app will also consume it directly.
- Token savings should not be asserted numerically until measured through real sessions.

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

1. **lmbrain-core/src/context.rs** (new) — Context-pack data structures (ProjectDigest, SpecContext, ReviewContext) and resolution logic. Scans .lmbrain/ artifacts, resolves linked ADRs, agent profiles, reviews, and reports missing-reference warnings. Generates both JSON and Markdown summary output. Includes real diagnostic scanning: malformed frontmatter detection, status directory/frontmatter mismatch detection, and unresolved agent reference detection.

2. **lmbrain-core/src/lib.rs** — Registered the `context` module and re-exported public types.

3. **lmbrain-mcp/src/main.rs** — Added three new MCP tool definitions and handlers:
   - `lmbrain_project_digest` — no required params, returns project overview
   - `lmbrain_spec_context` — requires `spec` param (ID or path), returns spec handoff context
   - `lmbrain_review_context` — requires `spec` param (ID or path), returns review context
   - Added `context_tool()` helper for input schema generation
   - Added 5 new tests for tool listing, schema validation, and parameter requirements

4. **src/lib/handoffPrompt.ts** — Updated generated handoff prompt to include v3 context-economy workflow instructions (read policy files first, use context-pack tools, expand only when needed).

5. **src/components/Agents/AgentsMCPView.tsx** — Added the three context-pack tools to the built-in tool list with "Context" category and accurate descriptions.

6. **src/components/Spec/SpecDetail.tsx** — Updated HandoffCTA to show a hint about the context-economy guidance in the prompt.

7. **src/components/Pulse/ProjectPulse.tsx** — Updated ActionCard expanded view to show a hint about context-pack usage.

8. **kit/.lmbrain/AGENT.md** — Added V3 context-economy workflow section with mandatory/relevant/optional/forbidden context tiers.

9. **kit/.lmbrain/OPERATOR.md** — Updated suggested specialist prompt to use context-pack flow.

10. **kit/.lmbrain/templates/project-lead-bootstrap-prompt.md** — Updated to use `lmbrain_project_digest` instead of reading the entire `.lmbrain/` directory.

11. **kit/.lmbrain/templates/spec.md** — Added v3 context-economy instruction to specialist instructions.

12. **kit/.lmbrain/CONTRACT.md** — Added context-pack contract section defining the three tools and their invariants.

13. **kit/.lmbrain/MIGRATIONS.md** — Added 2.2.7 migration entry for v3 context economy (fixed from stale 2.1.2 per review R-2).

14. **docs/architecture.md** — Updated MCP surface documentation with context-pack tool descriptions (version corrected to 2.2.7 per review R-2).

15. **docs/kit.md** — Added V3 context economy section.

16. **docs/agent-hosts.md** — Added context-pack tool guidance for agent hosts.

17. **src/__tests__/handoffPrompt.test.ts** (new) — 6 tests covering prompt content, agent fallback, context-economy guidance, and tool instructions (added per review R-3).

### Files changed

```
A lmbrain-core/src/context.rs
M lmbrain-core/src/lib.rs
M lmbrain-mcp/src/main.rs
M src/lib/handoffPrompt.ts
M src/components/Agents/AgentsMCPView.tsx
M src/components/Spec/SpecDetail.tsx
M src/components/Pulse/ProjectPulse.tsx
M kit/.lmbrain/AGENT.md
M kit/.lmbrain/OPERATOR.md
M kit/.lmbrain/templates/project-lead-bootstrap-prompt.md
M kit/.lmbrain/templates/spec.md
M kit/.lmbrain/CONTRACT.md
M kit/.lmbrain/MIGRATIONS.md
M docs/architecture.md
M docs/kit.md
M docs/agent-hosts.md
```

### Verification performed

- `cargo test -p lmbrain-core` — 24 tests passed (18 new context-pack tests + 6 existing)
- `cargo test -p lmbrain-mcp` — 10 tests passed (5 new context-pack tool tests + 5 existing)
- `cargo test` (all workspace) — all tests pass, zero compiler warnings
- `pnpm lint` — no errors
- `pnpm test` — 53 tests passed (13 test files, including 6 new handoff prompt tests)
- Manual MCP smoke test:
  - `lmbrain_project_digest` — returns project title, milestone, 5 ready specs, 10 review specs, 8 active decisions, **20 real diagnostics**, markdown
  - `lmbrain_spec_context` — resolves SPEC-023 with linked ADR-004, agent profile AGENT-FULLSTACK-DESKTOP, 9 criteria, 17 files, markdown
  - `lmbrain_review_context` — resolves SPEC-023 with 9 criteria, implementation evidence, linked decisions, 4 verification commands, markdown
- Diagnostic tests with malformed frontmatter, status mismatch, and unresolved agent fixtures all pass

### Deviations from the specification

None. All scope items implemented as specified. All review findings (R-1 through R-4) have been addressed:

- **R-1 (diagnostics):** `scan_diagnostics` and `spec_diagnostics` now implement real diagnostic scanning: malformed frontmatter detection, status directory/frontmatter mismatch detection, and unresolved agent reference detection. Verified with 3 dedicated tests using malformed and invariant-violating fixtures.
- **R-2 (version docs):** MIGRATIONS.md and docs/architecture.md updated to use the actual kit/app version `2.2.7`.
- **R-3 (handoff prompt tests):** 6 new tests in `src/__tests__/handoffPrompt.test.ts` covering prompt content, agent fallback, context-economy guidance, and tool instructions.
- **R-4 (Rust warnings):** Zero compiler warnings across all workspace crates.

### Known limitations

- Token savings are not asserted numerically — the spec explicitly excludes unmeasured claims.

### Handoff status

- [x] Ready for Project Lead review
