---
id: SPEC-028
title: "Project-scoped agent skills"
status: backlog
kind: feature
priority: high
area: agents
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-008]
links: [ADR-008, SPEC-023, SPEC-024, SPEC-027]
created: 2026-07-07
updated: 2026-07-07
tags: [agents, skills, kit, context, desktop]
---

# Project-scoped agent skills

## Objective

Add project-scoped LMBrain skills that document reusable operational procedures available to agents, such as build commands, test suites, diagnostics, release checks, and project-specific runbooks.

Skills must be first-class Markdown artifacts in `.lmbrain/`, visible in a dedicated application page, included in relevant context packs, and governed by the Project Lead workflow. They are auditable procedural knowledge, not automatically executed tools.

## Context

LMBrain already models:

- agent profiles and proposals under `.lmbrain/agents/`;
- MCP records and proposals under `.lmbrain/mcp/`;
- context-pack tools for project, spec, and review orientation;
- kit migrations and templates for additive contract evolution.

The operator wants all agents to have access to reusable project procedures, and wants the Project Lead to be able to create project-scoped skills for recurring work such as compiling, running tests, or executing diagnostic batteries.

The new concept must not blur existing safety boundaries:

- MCP records represent technical capabilities and integrations.
- Agent profiles represent roles and authority boundaries.
- Skills represent documented procedures and operating knowledge.

LMBrain must not auto-run skill commands as part of this feature. Agents may read and follow skill procedures during their manually started sessions, then record what they ran in implementation evidence.

## Scope

### Included

- Introduce a new `SKILL-*` artifact family under `.lmbrain/skills/`.
- Add skill templates, registry guidance, contract documentation, and migration instructions to the bundled kit.
- Parse skill artifacts in the Tauri backend and expose them to the frontend.
- Add a dedicated `Skills` page to the app navigation.
- Display active, proposed, and retired skills without crowding the existing `Agents & MCP` page.
- Add optional skill references to specs and agent profiles.
- Include applicable active skills in spec and review context packs.
- Add diagnostics for missing or invalid skill references.
- Extend controlled mutation support so agents can create and transition skill artifacts through LMBrain-controlled paths where available.
- Add tests covering parser, diagnostics, context packs, MCP tool listing/calls, and frontend rendering.

### Excluded

- Automatic execution of skill commands by the app, backend, or MCP server.
- Installing external tools, dependencies, MCP servers, or shell integrations.
- A visual command runner or terminal automation UI.
- Per-user private skills outside `.lmbrain/`.
- Global skills shared across unrelated projects.
- Complex permission enforcement beyond documentation, diagnostics, and existing manual-agent boundaries.

## Existing-project analysis

- `kit/.lmbrain/templates/agent-profile.md` already contains `allowed_mcp` and `knowledge`, but backend models do not currently expose those fields. Skills should be modeled explicitly instead of overloading `knowledge` or MCP.
- `src-tauri/src/models/agent.rs` and `src/types/index.ts` expose agent specialization metadata, but there is no `skills` field yet.
- `src-tauri/src/commands/contract.rs` parses specs, reviews, ADRs, agents, MCP records, handoffs, diagnostics, and wiki content. Skills need a parallel builder and command surface.
- `lmbrain-core/src/context.rs` resolves recommended agent profiles for `lmbrain_spec_context`; this is the right place to include applicable skills.
- `lmbrain-mcp/src/main.rs` currently supports artifact creation, lifecycle transitions, setters, reads, validation, and context packs. Skill lifecycle and setters should follow the same pattern.
- `src/components/Agents/AgentsMCPView.tsx` is already dense. Skills must have their own page instead of being added there.
- `src/context/WorkspaceContext.tsx`, `src/lib/commands.ts`, and app navigation need state and command integration for the new page.
- Kit migration prompts from `src/lib/handoffPrompt.ts` already instruct the Project Lead to merge additive kit changes carefully. The skill migration should follow the same non-destructive pattern.

## Technical proposal

### Artifact layout

Add this bundled kit structure:

```text
.lmbrain/
  skills/
    README.md
    registry.md
    active/
    proposed/
    retired/
```

Add `kit/.lmbrain/templates/skill.md`.

### Skill frontmatter

Use this initial schema:

```yaml
id: SKILL-XXX
title: "Skill title"
status: proposed
scope: project
kind: verification
risk: low
applies_to: []
domains: []
commands: []
requires_operator_approval: false
links: []
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: []
```

Field semantics:

- `id`: immutable `SKILL-*` identifier.
- `status`: `proposed`, `active`, or `retired`.
- `scope`: initially `project`; reserved for future extension.
- `kind`: short category such as `build`, `test`, `diagnostic`, `release`, `review`, `workflow`, or `verification`.
- `risk`: `low`, `medium`, or `high`; describes expected operational risk if an agent follows the procedure.
- `applies_to`: agent profile IDs, `all`, or empty. Empty means generally discoverable but not automatically recommended.
- `domains`: matching tags used for spec/agent context selection.
- `commands`: documented commands that may be useful during the procedure. They are not executed by LMBrain.
- `requires_operator_approval`: whether an agent must ask before following the skill, usually for destructive, expensive, credentialed, or release-affecting workflows.

### Skill body template

Each skill body must include:

```md
## Purpose

## When to use

## Preconditions

## Procedure

## Expected output

## Failure handling

## Evidence to record
```

### Spec and agent references

Add optional frontmatter fields:

```yaml
# specs
skills: []

# agent profiles
skills: []
```

Spec-level skill references are explicit requirements/recommendations for the handoff.

Agent-profile skill references are default operating procedures for that role.

### Applicability rules

For `lmbrain_spec_context`, include active skills when any of these conditions hold:

1. The spec frontmatter explicitly references the skill in `skills`.
2. The recommended agent profile references the skill in `skills`.
3. The skill `applies_to` includes the recommended agent ID.
4. The skill `applies_to` includes `all`.
5. The spec `area` or tags overlap the skill `domains`.

For `lmbrain_review_context`, include active skills when:

1. They are explicitly referenced by the spec.
2. Their `kind` is `review`, `test`, `diagnostic`, or `verification`.
3. They apply to the reviewer profile when that can be determined.

Context packs should include compact skill summaries, not full documents:

- `id`
- `title`
- `status`
- `kind`
- `risk`
- `requires_operator_approval`
- `commands`
- `path`

If a skill requires operator approval, the context markdown must say so clearly.

### Controlled mutation

Extend `lmbrain-core` artifact support with `ArtifactKind::Skill`.

Supported lifecycle:

- create in `.lmbrain/skills/proposed/`;
- `skill_activate`: `proposed -> active`;
- `skill_retire`: `active|proposed -> retired`.

Add setters only if needed for stable references:

- `lmbrain_set_spec_skills`
- `lmbrain_set_agent_skills`

If broad list-setter support would be too large for this spec, implement lifecycle and creation first, then document skill reference edits through templates and migration guidance. Do not add ad hoc unsafe frontmatter editing.

### App page

Add a dedicated `Skills` page in the main application navigation.

The page must:

- load all parsed skill artifacts;
- group or filter by status;
- show compact cards for `active`, `proposed`, and `retired`;
- display `SKILL-*`, title, status, kind, risk, applies-to summary, and command count or command preview;
- open the existing artifact detail modal when a skill is selected;
- show a useful empty state when no skills exist;
- surface skill diagnostics such as missing referenced agents or malformed skill metadata.

Do not add this content to `Agents & MCP` beyond small reference chips if agent profile cards later need them. The dedicated `Skills` page is the primary discovery surface.

### Diagnostics

Add warnings for:

- malformed skill frontmatter;
- skill file path/status mismatch;
- duplicate skill IDs;
- spec `skills` references that do not resolve;
- agent profile `skills` references that do not resolve;
- skill `applies_to` agent IDs that do not resolve, except `all`;
- invalid `risk` or `status` values.

Diagnostics should remain warnings unless the artifact cannot be parsed at all.

### Kit migration

Add an additive migration for the release that includes this feature:

- create `.lmbrain/skills/` directories when absent;
- add `skills/README.md`, `skills/registry.md`, and `templates/skill.md`;
- update `CONTRACT.md`, `AGENT.md`, `QUALITY.md` if needed, `templates/spec.md`, `templates/agent-profile.md`, `CHANGELOG.md`, and `MIGRATIONS.md`;
- preserve all project-specific `.lmbrain/` content;
- do not auto-create active project skills unless the Project Lead validates them against the actual repository and the operator approves.

## Files and areas involved

- `kit/.lmbrain/CONTRACT.md`
- `kit/.lmbrain/AGENT.md`
- `kit/.lmbrain/QUALITY.md`
- `kit/.lmbrain/MIGRATIONS.md`
- `kit/.lmbrain/CHANGELOG.md`
- `kit/.lmbrain/templates/spec.md`
- `kit/.lmbrain/templates/agent-profile.md`
- `kit/.lmbrain/templates/skill.md`
- `kit/.lmbrain/skills/README.md`
- `kit/.lmbrain/skills/registry.md`
- `src-tauri/src/models/`
- `src-tauri/src/commands/contract.rs`
- `src-tauri/tests/contract_test.rs`
- `src/types/index.ts`
- `src/context/WorkspaceContext.tsx`
- `src/lib/commands.ts`
- `src/App.tsx` and navigation components
- `src/components/Skills/`
- `src/__tests__/`
- `lmbrain-core/src/context.rs`
- `lmbrain-core/src/transitions.rs`
- `lmbrain-core/src/invariants.rs`
- `lmbrain-core/tests/`
- `lmbrain-mcp/src/main.rs`
- `lmbrain-mcp/tests/`
- `docs/architecture.md`
- `docs/kit.md`
- `docs/agent-hosts.md`

## Acceptance criteria

- [ ] The bundled kit defines a `SKILL-*` artifact family with registry, README, template, lifecycle states, and migration guidance.
- [ ] The Markdown contract distinguishes skills from MCP integrations and states that skills are documented procedures, not automatically executed tools.
- [ ] Specs and agent profiles support optional `skills: []` references without breaking existing artifacts.
- [ ] The Tauri backend parses skill artifacts and exposes them through a typed command.
- [ ] The app includes a dedicated `Skills` navigation page separate from `Agents & MCP`.
- [ ] The Skills page displays active, proposed, and retired skills, including kind, risk, applies-to, and command summary.
- [ ] Selecting a skill opens the existing artifact detail flow.
- [ ] Diagnostics warn about unresolved skill references and unresolved `applies_to` agents.
- [ ] `lmbrain_spec_context` includes applicable active skills with compact summaries and operator-approval warnings.
- [ ] `lmbrain_review_context` includes relevant verification/review skills.
- [ ] `lmbrain-mcp` lists and handles controlled skill lifecycle tools, at least creation plus activate/retire transitions.
- [ ] Automated tests cover parser/model behavior, diagnostics, context-pack skill inclusion, MCP tool exposure, and Skills page rendering.
- [ ] Existing `Agents & MCP` behavior remains focused on agents and MCP records and is not overloaded with skill management UI.
- [ ] Migration guidance is additive and explicitly preserves project-specific content.

## Implementation plan

1. Extend the kit contract and templates with the new skill artifact family.
2. Add backend Rust models and parsers for skills, including status parsing and command/list frontmatter support.
3. Add frontend TypeScript types, command bindings, workspace state, and navigation entry.
4. Build the dedicated `Skills` page with filtered/grouped cards and artifact-detail integration.
5. Add diagnostics for skill references and path/status mismatches.
6. Extend core context-pack structures and markdown formatting to include applicable skill summaries.
7. Extend controlled mutation support for skill creation and lifecycle transitions.
8. Add MCP tool list/call handling for skill lifecycle.
9. Update docs and migration guidance.
10. Add and run focused automated tests, then broader workspace checks.

## Required verification

- `node scripts/check-version.mjs`
- `pnpm vitest run`
- `pnpm build`
- `cargo test -p lmbrain-core`
- `cargo test -p lmbrain-mcp`
- `cargo test -p lmbrain --test contract_test`
- `cargo test --workspace`
- Manual app check: open a project with no skills and a project with sample skill artifacts; verify navigation, cards, filters/grouping, diagnostics, and artifact detail opening.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Update relevant technical documentation: `docs/architecture.md`, `docs/kit.md`, and `docs/agent-hosts.md`.
- Keep the implementation additive and backward-compatible for existing projects.
- Preserve the security boundary: skill commands are documented instructions, not app-triggered execution.
- Do not add new dependencies unless there is a clear production reason.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Skill applicability can become noisy if domain matching is too broad. Prefer deterministic explicit references and simple matching over clever inference.
- List setters for `skills` may require careful frontmatter update support. If this becomes too broad, defer setters and keep lifecycle/create support in this spec.
- The first release should avoid runnable skill automation. Executable skill workflows should require a future spec and explicit permission model.
- The app navigation may need visual balancing if another top-level page increases sidebar density.

## Instructions for the assigned specialist

- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Keep skills distinct from MCP capabilities in code, UI, and documentation.
- Do not implement automatic command execution.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Deviations from the specification

### Handoff status
- [ ] Ready for Project Lead review
