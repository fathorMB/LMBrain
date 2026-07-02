# LMBrain Markdown Contract v0.2

**Kit version:** read from `VERSION` (canonical).

The `VERSION` file at the root of `.lmbrain/` is the canonical, machine-readable kit version. Use semantic versioning: breaking contract changes increment the major version; backward-compatible additions increment the minor version; clarifications and fixes increment the patch version. Read `CHANGELOG.md` for released changes and `MIGRATIONS.md` before upgrading a released kit.

## General rules

- Every operational artifact has an immutable, unique ID.
- Frontmatter holds queryable metadata; the Markdown body holds human context and evidence.
- Dates use `YYYY-MM-DD`.
- References use IDs in frontmatter and `[[wikilinks]]` in prose.
- The filesystem and `status` frontmatter must agree where a status directory exists.
- `lmbrain-core` is the executable source of truth for controlled creation, transitions, setters, invariant checks, atomic writes, and audit entries. Agents invoke it through the repository-scoped `lmbrain-mcp` server rather than editing managed frontmatter by hand.

## IDs and locations

| Artifact | Prefix | Location |
| --- | --- | --- |
| Specification | `SPEC-` | `specs/<status>/` |
| Review | `REVIEW-` | `reviews/<status>/` |
| Decision | `ADR-` | `decisions/` |
| Agent profile | `AGENT-` | `agents/profiles/` |
| Agent proposal | `AGENT-PROP-` | `agents/proposals/` |
| MCP specification | `MCP-` | `mcp/specs/` |
| MCP proposal | `MCP-PROP-` | `mcp/proposals/` |
| Session handoff | `HANDOFF-` | `handoffs/active/` |

## Shared frontmatter

```yaml
id: SPEC-012
title: Concise human title
status: ready
created: 2026-06-22
updated: 2026-06-22
tags: []
links: []
```

Required fields are `id`, `title`, `status`, `created`, `updated`, `tags`, and `links`.

Optional shared fields: `area`, `milestone`, `priority`, `owner`.

Priority values: `critical`, `high`, `medium`, `low`.

## Allowed statuses

| Artifact | Values |
| --- | --- |
| Spec | `backlog`, `ready`, `working`, `review`, `done`, `discarded` |
| Review | `pending`, `accepted`, `changes-requested`, `blocked`, `superseded` |
| ADR | `proposed`, `accepted`, `rejected`, `superseded`, `deprecated` |
| Agent profile | `proposed`, `active`, `inactive`, `retired` |
| Agent proposal | `proposed`, `approved`, `rejected` |
| MCP proposal | `proposed`, `approved`, `rejected`, `implemented`, `blocked` |
| MCP | `specified`, `active`, `inactive`, `deprecated` |
| Session handoff | `ready`, `consumed`, `superseded`, `archived` |

## Context packs (v3 context economy)

Context packs are read-only, derived views of the artifact directory. They are not the system of record:

- `lmbrain_project_digest` — compact project overview for Project Lead bootstrap and pulse.
- `lmbrain_spec_context` — spec handoff context for specialist orientation.
- `lmbrain_review_context` — review context for reviewer orientation.

Context packs resolve linked specs, ADRs, reviews, agent profiles, roadmap milestones, and diagnostics deterministically. They report missing references as structured warnings. They never mutate files.

Agents must read mandatory policy files (`QUALITY.md`, `CONTRACT.md`, `AGENT.md`) before relying on context packs. They must expand to full source artifacts when a context pack warning indicates a missing or unresolved reference.

## Invariants

- A spec reaches `done` only with its acceptance criteria checked, evidence recorded, and an accepted review.
- `rejected` is a terminal "declined at proposal/decision time" status available on every proposable artifact (Spec, ADR, Agent proposal, MCP proposal). It is distinct from `changes-requested` (a review asking for revision and resubmission) and from `archived`/`superseded`/`deprecated` (retiring something that was once active). A rejected artifact records the rejection rationale in its body and is not silently reopened.
- An `active` MCP needs a documented spec, permissions, and verification evidence.
- An agent profile always has `activation: manual`; LMBrain never spawns agents.
- An ADR is not rewritten to change history: create a replacement ADR and mark the old one `superseded`.
- The Project Lead may write only inside `.lmbrain/` during ordinary work. It may alter application code only through the narrowly scoped, operator-authorized escalation process in `AGENT.md`.
- All implementation and review work complies with `QUALITY.md` unless a human-approved exception is recorded.
- A session handoff is a context snapshot and must be validated by the receiving Project Lead before it drives project decisions or status changes.
- At most one `ready` session handoff may exist in `handoffs/active/`.
- The application should warn about duplicate IDs, broken links, directory/status mismatches, missing references, and circular dependencies.

## Authority

| Artifact | Project Lead | Specialist | User |
| --- | --- | --- | --- |
| Project, roadmap, status | maintain | no | approve/edit |
| Specs | create/maintain | implementation evidence only | approve/edit |
| Reviews | create on request | no | request/edit |
| ADRs | propose/maintain | propose only | approve/edit |
| Agent and MCP registries | maintain proposals | no | approve/edit |
| Session handoffs | create/consume | no | request/edit |
| Application code and configuration | no, except qualified escalated corrective work | only when manually assigned by user | edit |
