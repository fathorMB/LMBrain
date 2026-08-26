# LMBrain Markdown Contract v0.3

**Kit version:** read from `VERSION` (canonical).

The `VERSION` file at the root of `.lmbrain/` is the canonical, machine-readable kit version. Breaking contract changes increment the major version; backward-compatible additions increment the minor version; clarifications and fixes increment the patch version. Read `UPGRADING.md` for upgrade instructions and upstream documentation for released changes.

## General Rules

- Every operational artifact has an immutable, unique ID.
- Frontmatter holds queryable metadata; Markdown body holds human context and evidence.
- Dates use `YYYY-MM-DD`.
- References use IDs in frontmatter and `[[wikilinks]]` in prose.
- The filesystem location and `status` frontmatter must agree.
- `lmbrain-core` is the executable source of truth for creation, transitions, invariant checks, atomic writes, and audit trails. Agents invoke it through the repository-scoped `lmbrain-mcp` server rather than editing managed frontmatter by hand.
- The desktop application is a read-only consultation surface with a dedicated Operations page for operator verification triage; it exposes no ungoverned status mutation.

## IDs & Locations

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
| Skill | `SKILL-` | `skills/<status>/` |
| Debt | `DEBT-` | `debts/<status>/` |

## Shared Frontmatter

```yaml
id: SPEC-012
title: Concise human title
status: ready
created: 2026-06-22
updated: 2026-06-22
tags: []
links: []
```

- Required fields: `id`, `title`, `status`, `created`, `updated`, `tags`, `links`.
- Optional shared fields: `area`, `milestone`, `priority`, `owner`.
- Priority values: `critical`, `high`, `medium`, `low`.

## Core Lifecycles & Statuses

| Artifact | Allowed Statuses |
| --- | --- |
| **Spec** | `backlog`, `ready`, `working`, `review`, `done`, `discarded` |
| **Review** | `pending`, `accepted`, `changes-requested`, `blocked`, `superseded` |
| **ADR** | `proposed`, `accepted`, `rejected`, `superseded`, `deprecated` |
| **Agent profile** | `proposed`, `active`, `inactive`, `retired` |
| **Agent proposal** | `proposed`, `approved`, `rejected` |
| **MCP proposal** | `proposed`, `approved`, `rejected`, `implemented`, `blocked` |
| **MCP spec** | `specified`, `active`, `inactive`, `deprecated` |
| **Session handoff**| `ready`, `consumed`, `superseded`, `archived` |
| **Skill** | `proposed`, `active`, `retired` |
| **Milestone** | `proposed`, `active`, `completed` |

### Spec Acceptance Criteria Markers

A spec`s `## Acceptance criteria` section recognizes exactly three markers:
- `- [ ]`: Declared and not satisfied.
- `- [x]`: Satisfied, with verifiable evidence in `## Implementation evidence`.
- `- [~] <criterion> | waived=DEBT-xxx`: Impeded and consciously waived against an existing debt.

Any other marker is treated as unsatisfied and reported by name.

### Review Finding Taxonomy v1

New reviews use canonical finding categories: `accessibility`, `compatibility`, `correctness`, `documentation`, `localization`, `maintainability`, `metrics-integrity`, `performance`, `provenance`, `requirements-completeness`, `robustness`, `schema-conformance`, `security-boundary`, `test-quality`, `usability`, `verification-integrity`.

## Invariants

- A spec reaches `done` only with its acceptance criteria satisfied or validly waived, evidence recorded, and an accepted review.
- Reviews start as `pending`. Verdicts use semantic MCP verbs (`review_accept`, `review_changes_requested`, `review_block`, `review_supersede`) and append immutable `review_events`.
- Implementation attributions on reviews must resolve to existing `AGENT-*` profiles. Corrections use `review_set_implementation_agent`.
- Supersession is mutual and atomic (`adr_supersede`, `debt_supersede`, `handoff_supersede`).
- At most one `ready` session handoff exists in `handoffs/active/`.
- All implementation and review work complies with `QUALITY.md` unless an explicit human exception is recorded.

## Authority

| Artifact | Project Lead | Specialist | Operator |
| --- | --- | --- | --- |
| Project, roadmap, status | maintain | no | approve / edit |
| Specs | create / maintain | implementation evidence only | approve / edit |
| Reviews | create on request | remediation evidence | request / verdict |
| ADRs | propose / maintain | propose only | approve / edit |
| Agent & MCP registries | maintain proposals | no | approve / edit |
| Skills | propose / maintain | follow active procedures | approve / edit |
| Session handoffs | create / consume | no | inspect / edit |
| Code & Configuration | no (except escalation) | only assigned scope | full authority |

## Capability Modules

Specialized capabilities are documented in modular contract files. The modules are part of this contract's mandatory read: agents read all of them together with `CONTRACT.md`. A module applies whenever its configuration or artifacts exist in the workspace; an absent trigger makes the module dormant, never unread:

- [[contract/verification.md|Verification Gates]]: Manifest-driven gate execution, transcripts, and attestations.
- [[contract/harness.md|Harness Manifest]]: Host environment configuration and drift tracking.
- [[contract/debts.md|First-Class Debts]]: Durable cross-spec debt tracking and lifecycle.
- [[contract/dependencies.md|Dependencies & Parking]]: Hard prerequisite DAG and spec parking.
- [[contract/effort_tags.md|Effort Tiers & Tags]]: Spec sizing heuristics and tag taxonomy.
- [[contract/feedback.md|Kit Feedback]]: Upstream LMBrain feedback notes and resolution.
- [[contract/dreams.md|Dreaming]]: Operator-invited grounded ideation.
- [[contract/branching.md|Branching Strategy]]: Declarative Git branching policies.
