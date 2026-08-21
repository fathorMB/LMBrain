---
id: SPEC-070
# Note: Quote the title if it contains a colon
title: "Add operator-invoked Project Lead dreaming sessions and Dream Journal"
status: backlog
kind: feature
priority: high
area: project-lead-dreaming
milestone: M-09
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
depends_on: []
dependency_events: []
parking_events: []
related_tasks: []
related_decisions: []
links: []
created: 2026-08-10
updated: 2026-08-10
tags: [dreaming, project-lead, technical-debt, design-debt, desktop]
activity:
  - date: 2026-08-10
    action: "created"
  - date: 2026-08-10
    action: "set tags"
---
# Add operator-invoked Project Lead dreaming sessions and Dream Journal

## Objective

Let an operator explicitly invite the Project Lead to enter a bounded reflective
session and capture useful, project-grounded technical or design debt without
mistaking tentative ideas for verified findings or approved work. Make the
captured material easy to inspect in a dedicated, accessible **Dream Journal**
page in the desktop sidenav.

## Context

This is the planned implementation handoff for GitHub issue #104. The requested
phrases (for example, “fatti un pisolino” or “vatti a riposare”) are natural
language commands to the Project Lead, not commands the desktop application can
reliably parse. They must therefore be defined as an explicit Lead capability,
with an auditable controlled artifact mutation for its durable output.

## Scope
### Included

- Define a new, separate `DREAM-*` artifact family with an append-only lifecycle:
  `captured -> triaged -> promoted | discarded`.
- Add controlled core and MCP verbs to capture, triage, promote, and discard a
  dream; validate ID allocation, status-directory agreement, required data,
  typed references, concurrent mutation safety, and append-only events.
- Capture a concise hypothesis, classification (`technical-debt` or
  `design-debt`), confidence, rationale, impact/affected area, concrete source
  artifact references, and suggested next disposition.
- Add a documented Project Lead dreaming protocol: explicit operator invitation,
  bounded context selection, a concise operator-facing outcome, and zero or
  more governed `DREAM-*` captures.
- Add models, workspace loading/refresh, commands, command-palette destination,
  sidebar navigation, unread integration where appropriate, and a read-only
  Dream Journal with filters, details, provenance, and copyable action prompts.
- Permit explicit governed promotion to a Finding, Spec, ADR, or backlog record;
  preserve the dream and link its promoted destination.

### Excluded

- Keyword recognition or automatic mutations in Tauri, PTY sessions, or terminal
  transcripts.
- Saving raw conversation/terminal transcripts, credentials, personal data, or
  unrelated context as a dream’s evidence.
- Treating a dream as a confirmed defect, a roadmap commitment, or an approved
  specification.
- Direct lifecycle-editing controls in the desktop UI for the first release.
- Autonomous implementation, external research, or external communication from
  a dreaming session.

## Existing-project analysis

- `src-tauri/src/commands/sessions.rs` is a PTY manager: it relays bytes and
  retains a volatile transcript, but it has no semantic conversation-intent
  layer and cannot inspect or steer Project Lead reasoning. Parsing “sleep” in
  that layer would be fragile and would wrongly couple a general terminal to one
  agent’s protocol.
- First-class `FINDING-*` artifacts already have a controlled core/MCP lifecycle
  (`lmbrain-core/src/finding.rs`, `lmbrain-mcp/src/main.rs`). They require
  statement, evidence, impact, severity, resolution criteria, typed references,
  and a validated lifecycle. That is intentionally stronger than an exploratory
  insight, so reusing Findings would misrepresent speculation as evidence.
- The current Findings page (`src/components/Findings/FindingsView.tsx`) proves
  the desired read-only detail/filter/provenance pattern. Its route is wired
  explicitly through `AppView`, `Sidebar`, `App.tsx`, `WorkspaceContext`, Tauri
  commands, TypeScript types, and tests; a Dream Journal needs equivalent
  end-to-end wiring rather than a local React-only page.
- `.lmbrain/` is the source of truth, and controlled mutations belong in
  `lmbrain-core`/`lmbrain-mcp`; the desktop currently deliberately exposes
  Findings as read-only.

## Technical proposal

Create a dedicated `DREAM-*` domain rather than overloading Findings or Kit
Feedback. A dream is a clearly labelled, provisional project observation.

The Lead protocol must accept only deliberate operator invitations and confirm
the bounded operation. It should read the project digest plus relevant existing
artifacts, identify zero or more distinct ideas, and write one record per idea
through a `dream_capture` MCP verb. It must name its source artifact IDs and
state uncertainty; “no worthwhile dream captured” is a valid outcome.

The initial desktop route is read-only. It should support state,
classification, area, confidence, milestone and free-text filters; non-colour
state indicators; keyboard-accessible cards/detail; empty, loading, malformed,
and command-error states; refresh; provenance; lifecycle timeline; and explicit
promotion links. The detail copies a governed prompt rather than mutating.

Promotion requires a second explicit Lead action with a stated target and must
not silently create an implementation commitment. For example, a technically
substantiated item may become `FINDING-*`; a product-shaping choice may become
an ADR proposal; a scoped delivery proposal may become a backlog spec.

## Files and areas involved

- `.lmbrain/AGENT.md`, the Project Lead profile, contract/templates, kit
  documentation, migration guidance, and changelog.
- `lmbrain-core`: artifact kind/path mapping, dream model/parser/validator,
  atomic lifecycle mutations, diagnostics, context/digest inclusion, fixtures,
  and tests.
- `lmbrain-mcp`: `dream_*` schemas, tool routing, authority rules, and protocol
  tests.
- `src-tauri`: models, snapshot/load commands, command registration, statistics
  and contract tests.
- `src/types`, `src/lib/commands.ts`, `WorkspaceContext`, unread state,
  `App.tsx`, `Sidebar`, command palette, and a new `components/Dreams` view.
- Frontend and Rust tests plus relevant product/architecture documentation.

## Acceptance criteria
- [ ] Only an explicit, documented operator invitation starts dreaming; ordinary
  project conversation and terminal traffic cannot create a dream.
- [ ] A dreaming session writes zero or more individually validated `DREAM-*`
  artifacts through controlled MCP operations, with no raw transcript retention.
- [ ] Each dream records classification, confidence, rationale, scope/impact,
  source IDs, timestamp, actor, and a suggested next disposition; unknown or
  unsupported claims are clearly labelled as hypotheses.
- [ ] Core validation enforces globally unique IDs, status-directory consistency,
  required fields, typed relations, optimistic/concurrent mutation safety, and
  append-only lifecycle events.
- [ ] Dreams and Findings have distinct types, routes, metrics, and semantics;
  neither automatic promotion nor automatic roadmap/spec membership exists.
- [ ] Explicit promotion is auditable, preserves the source dream, and links the
  resulting governed artifact without rewriting historical records.
- [ ] Dream Journal is a dedicated, accessible sidenav and command-palette route
  with loading, empty, error, malformed, filtering, sorting, refresh, detail,
  provenance, lifecycle, and non-colour state coverage.
- [ ] The route is read-only: it exposes copyable governed action prompts but no
  direct lifecycle transitions.
- [ ] Existing Findings, Kit Feedback, session behavior, and all public contracts
  remain backward compatible, covered by cross-layer regression tests.

## Implementation plan
1. Specify the Dream contract and Project Lead protocol; add model, paths,
   validation, mutation lock use, lifecycle events, fixtures, and core tests.
2. Add MCP tools with narrow authority and explicit promotion semantics; update
   schemas, help text, and protocol tests.
3. Add Tauri loaders/models/snapshot refresh and TypeScript contracts.
4. Build and test the Dream Journal, navigation, command palette, unread state,
   filters, details, accessibility, and error states.
5. Qualify migration/backward compatibility, update documentation, and run the
   full Rust/frontend/package smoke suite.

## Required verification

- [ ] DREAM-CORE | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Core tests cover allocation, malformed fields, lifecycle transitions, forbidden transitions, references, concurrent writes, and promotion linkage.
- [ ] DREAM-MCP | kind=manual | owner=agent | phase=before-submit | evidence=artifact | MCP schema, routing, and authority tests cover explicit invitation and a no-op dreaming session.
- [ ] DREAM-CONTRACT | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Tauri/TypeScript contracts and workspace snapshot refresh remain aligned.
- [ ] DREAM-UI | kind=manual | owner=agent | phase=before-submit | evidence=artifact | Frontend tests cover navigation, filters, sort, details, keyboard/focus, non-colour state, empty/loading/error/malformed states, and read-only boundaries.
- [ ] DREAM-SMOKE | kind=operator | owner=operator | phase=before-done | evidence=observation | Packaged desktop smoke covers a workspace with Dreams, Findings, and legacy kit artifacts.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

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

### Files changed

### Verification performed

### Deviations from the specification

### Handoff status
- [ ] Ready for Project Lead review
