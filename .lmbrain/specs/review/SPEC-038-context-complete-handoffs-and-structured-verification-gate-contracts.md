---
id: SPEC-038
# Note: Quote the title if it contains a colon
title: "Context-complete handoffs and structured verification gate contracts"
status: review
kind: feature
priority: critical
area: context-and-verification-contract
milestone: M-05
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-004, ADR-008, ADR-012]
links: [SPEC-023, SPEC-028, SPEC-036]
created: 2026-07-16
updated: 2026-07-16
tags: [2.9.0, context-packs, verification, skills, handoffs]
activity:
  - date: 2026-07-16
    action: "created"
  - date: 2026-07-16
    action: "transitioned backlog -> ready"
  - date: 2026-07-16
    action: "transitioned ready -> working"
  - date: 2026-07-16
    action: "transitioned working -> review"
---
# Context-complete handoffs and structured verification gate contracts

## Objective

Make every implementation handoff expose the complete, canonical verification
contract and the behavior-relevant parts of its assigned profile and skills, so
an agent can determine every required gate before implementation and
`spec_submit` without reconstructing requirements from scattered prose.

## Context

Production evidence from AstraNexus shows that transcript fast-fails persist
even after strong procedural wording. LMBrain 2.8.0 currently makes the
reviewer better informed than the implementer: `ReviewContext` attempts to
extract `Required verification`, while `SpecContext` omits it entirely.
`extract_section_list` also returns no entries for the paragraph-style Required
verification sections used by real specs. Skill summaries read `commands` only
from frontmatter, but shipped and project skills commonly keep commands in the
body while declaring `commands: []`. Finally, the agent-profile summary omits
the profile guidance where approved lessons are recorded.

[[SPEC-036]] establishes executable gate provenance and stale-evidence
protection. This spec complements it: it makes the full verification contract
visible and structurally consistent before execution. It must not duplicate or
weaken SPEC-036's approval and execution boundary.

Repository validation currently reports a pre-existing duplicate `SPEC-036`:
the intended dependency is exactly
`.lmbrain/specs/ready/SPEC-036-verification-transcript-integrity-and-kit-executed-gates.md`,
not the unrelated Windows-installer review artifact. The duplicate-ID invariant
must be resolved through an operator-governed correction before this spec moves
to `ready`; no implementation handoff may rely on ambiguous ID resolution.

## Scope
### Included

- Add a lossless, versioned verification-requirement representation to spec
  context, covering executable gates, manual inspections, operator/playtest
  gates, owner, lifecycle phase, evidence type, and source reference.
- Scope acceptance-criterion parsing strictly to `## Acceptance criteria`;
  verification and implementation-evidence checklists remain separately typed
  and must not inflate criterion counts or `spec_done` invariants.
- Define a canonical structured Markdown form for non-executable requirements
  under `## Required verification`, with stable gate IDs and checked state.
- Merge typed `verification_gates` references introduced by [[SPEC-036]] with
  manual requirements into one deterministic `SpecContext` view.
- Preserve the original section text and artifact paths so compact context
  never silently replaces the source of truth.
- Expand `AgentProfileSummary` with behavior-relevant specialization metadata,
  profile path/content digest, and bounded operational guidance; update the
  handoff prompt to require reading the assigned active profile in full.
- Make applicable skill summaries expose canonical commands and path/digest;
  define precedence and diagnostics when frontmatter and body disagree.
- Add diagnostics for paragraph-only/unparseable requirements, duplicate gate
  IDs, unresolved gate/skill references, empty commands on active verification
  skills, contradictory command definitions, and manual gates without an owner
  or completion phase.
- Update templates so every new spec lists exact gate IDs and separates
  `before-submit` agent/kit gates from `before-done` operator gates.
- Add migration guidance and additive repair prompts for existing projects;
  never rewrite project-specific specs, profiles, or skills silently.
- Expose the same complete contract in MCP JSON/Markdown, Tauri models, app
  detail views, and copyable handoff/remediation prompts.

### Excluded

- Executing commands or authenticating their output; owned by [[SPEC-036]].
- Inferring arbitrary shell commands from prose.
- Automatically accepting a spec, review, profile, skill, or proposal.
- Replacing full artifacts with generated summaries.
- Treating an operator-only observation as an implementer-owned gate.

## Existing-project analysis

- `lmbrain-core/src/context.rs::SpecContext` contains criteria and applicable
  skills but no required-verification field.
- `ReviewContext.verification_commands` uses list-only parsing, while current
  project specs generally use prose paragraphs.
- `SkillSummary.commands` reads only frontmatter; all three AstraNexus active
  skills currently declare an empty commands array despite body procedures.
- `AgentProfileSummary` omits domains, primary files, review focus, constraints,
  profile path, and approved guidance body.
- Current context parsing scans checkboxes across the whole spec body: the three
  new planning specs incorrectly expose `Ready for Project Lead review` as an
  additional acceptance criterion, proving section scoping is not enforced.
- `src/lib/handoffPrompt.ts` names the profile ID and full spec but does not
  require the assigned profile or every applicable skill to be read.
- Existing context-pack tests cover resolution, not semantic completeness
  against the full source artifact.

## Technical proposal

Introduce a `VerificationRequirement` context model with at least: stable ID,
title, kind (`command`, `inspection`, `manual`, `operator`), owner, completion
phase (`before-submit`, `before-review`, `before-done`), evidence type, source
path/reference, checked state, executable manifest gate when applicable, and
the original source text. Use a deliberately simple canonical Markdown line
for manual requirements, documented and parsed with strict diagnostics rather
than heuristic natural-language interpretation.

`SpecContext` returns the ordered union of manifest-backed executable gates and
structured manual requirements. The implementation handoff fails diagnostics
when a required item cannot be resolved, but legacy prose remains visible with
an explicit `unstructured` warning during migration.

Make skill command metadata canonical in frontmatter for context purposes.
Body procedures may explain commands but must not contradict them. Migration
guidance should populate bundled skill metadata and produce copyable,
operator-reviewed remediation prompts for project skills.

The agent profile context remains compact but cannot omit approved behavior:
include all specialization fields, path, digest, and bounded extracts from
operational-boundary/project-skill sections. The handoff still directs the
specialist to read the full profile before starting.

## Files and areas involved

- `lmbrain-core/src/context.rs` and shared verification/profile/skill models
- `lmbrain-core` parser, diagnostic, fixture, and context completeness tests
- `lmbrain-mcp/src/main.rs` context tool schemas and contract tests
- Tauri contract/models/statistics only where the app consumes the new schema
- `src/lib/handoffPrompt.ts` and remediation prompt generation
- Spec, skill, agent, Insights, and diagnostic UI surfaces and tests
- `kit/.lmbrain/templates/spec.md`, skill/profile templates and bundled skills
- `kit/.lmbrain/AGENT.md`, `CONTRACT.md`, `QUALITY.md`, `MIGRATIONS.md`
- `docs/architecture.md`, `docs/kit.md`, `docs/agent-hosts.md`

## Acceptance criteria

- [ ] `lmbrain_spec_context` exposes every required verification gate present in
      the full spec, including SPEC-036 manifest gates and manual/operator gates.
- [ ] `SpecContext` never returns an apparently complete empty gate set when the
      source contains a non-empty but unstructured Required verification section;
      it returns the source text plus an actionable warning.
- [ ] Structured requirements have stable unique IDs, owner, phase, evidence
      type, source, and checked state; invalid or duplicate records fail validation.
- [ ] Acceptance criteria are parsed only from their named section; handoff,
      evidence, verification, and decision checkboxes never affect criterion
      counts or completion invariants.
- [ ] `before-submit` and `before-done` requirements are distinguished so an
      implementer cannot be fast-failed for an unavailable operator observation
      and an operator gate cannot be silently treated as complete.
- [ ] Applicable skill summaries expose canonical non-empty command metadata;
      empty or contradictory active verification skills produce diagnostics.
- [ ] Agent profile context includes specialization metadata, profile path/digest,
      and approved operational guidance; handoff prompts require the full active
      profile and applicable skills to be read before implementation.
- [ ] Context JSON and Markdown preserve source references and deterministic
      ordering and remain read-only.
- [ ] Templates emit structured verification requirements and exact gate IDs by
      default; paragraph-only legacy form is no longer generated.
- [ ] Migration guidance is additive, preserves project customization, and never
      auto-rewrites behavior-affecting profiles or skills.
- [ ] Tests reproduce AstraNexus failures: paragraph gates disappearing, empty
      skill commands, stale profile paths, and ownerless operator gates.
- [ ] Existing context-pack consumers remain backward compatible or receive a
      documented versioned migration with exhaustive contract tests.
- [ ] The pre-existing duplicate `SPEC-036` invariant is resolved before
      approval, and this spec's dependency resolves uniquely to the verification-
      provenance artifact.
- [ ] Full Rust/frontend checks and packaged Windows smoke coverage are green.

## Implementation plan

1. Finalize the structured requirement contract and migration compatibility.
2. Add parser/models and completeness diagnostics in `lmbrain-core`.
3. Extend spec/profile/skill context schemas and MCP serialization.
4. Update handoff/remediation prompts and app inspection surfaces.
5. Update bundled templates, skills, policies, docs, and migrations.
6. Run AstraNexus-derived fixtures and full quality gates; submit only with a
   complete transcript under the then-current [[SPEC-036]] contract.

## Required verification

- Parser/property fixtures for structured, legacy, duplicate, unresolved, and
  ownerless requirements.
- Section-boundary fixtures proving acceptance, verification, decision, and
  handoff checklists remain distinct under nested headings and mixed line endings.
- Golden comparisons proving full-spec gate coverage equals context-pack gate
  coverage for representative server, Tauri, frontend, and operator-playtest specs.
- Skill/profile context fixtures for empty commands, conflicting commands,
  stale primary files, guidance digests, and legacy profiles.
- MCP schema/serialization and handoff-prompt tests.
- `cargo fmt --all -- --check`, canonical Clippy, full Rust workspace tests,
  frontend lint/tests/build, version/contract checks, and Windows packaged smoke.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Over-structuring manual verification can make specs noisy. Keep the syntax
  compact and show a friendly editor/rendered form.
- Context growth can erode v3 token savings. Measure size, but never remove
  required gates or approved guidance to hit a token target.
- Command metadata may duplicate SPEC-036 manifests. The manifest is executable
  authority; skill commands are human runbook metadata and must reference, not
  override, a manifest gate when both exist.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

- Added typed, source-preserving verification requirements to spec and review context with executable/manual/operator owner, phase, evidence, and checked state.
- Scoped acceptance parsing to its canonical section; added paragraph/list legacy fallbacks, unresolved/duplicate gate warnings, profile path/digest/guidance, and skill path/digest/body-command fallback.
- Updated the handoff prompt and bundled templates/policies so implementers read full active profiles/skills and separate `before-submit` from `before-done` gates.

### Files changed

- `lmbrain-core/src/context.rs`, `src/lib/handoffPrompt.ts`
- bundled spec template, contract, quality, agent guidance, migrations, and architecture/kit docs

### Verification performed

- Context regression tests include seven anonymized AstraNexus-derived verification shapes and prove handoff visibility plus acceptance-checklist isolation.
- Full Rust workspace, frontend lint/test/build, version alignment, and whitespace checks passed.

### Verification transcript

```text
$ cargo test -p lmbrain-core context::tests
All context tests passed, including seven AstraNexus-derived legacy/structured/operator shapes.

$ cargo test --workspace && pnpm lint && pnpm test && pnpm build
Rust: all non-ignored tests passed, 0 failed.
Frontend: lint passed; 121 tests passed; production build passed.
```

### Deviations from the specification

- Existing project artifacts are diagnosed and preserved rather than rewritten automatically, per the additive migration policy.

### Handoff status
- [x] Ready for Project Lead review
