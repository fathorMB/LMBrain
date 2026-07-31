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

## Project harness manifest

`.lmbrain/HARNESSES.json` is the optional, versioned source of project harness intent. Schema version 1 contains a `hosts` object keyed by `claude-code`, `codex`, `pi`, or `open-code`. Each host may declare `enabled`, portable `required_tools`, non-secret `environment` values, and—only where supported—an `lsp.required` policy.

The manifest is strict: unknown fields, arbitrary commands, scripts, hooks, absolute machine paths, traversal, secret-like environment keys, and unsupported host capabilities are invalid. Machine-local executable selection and credentials never belong in it. Repository intent does not authorize effects: materialization requires a separate operator approval bound to the canonical manifest digest and workspace identity.

Controlled MCP access uses `harness_config_get`, `harness_config_validate`, and `harness_config_set`. The setter replaces the complete validated manifest atomically and records digest-only audit evidence; it never grants approval or materializes native harness files.

Operator approval is machine-local application state keyed by canonical workspace fingerprint and canonical manifest digest. Approval requires the exact previewed digest, becomes stale after any material manifest change, and is not reused when a workspace identity changes. Corrupt approval state is quarantined and fails closed.

## Verification gates

`.lmbrain/verification.toml` is an optional strict, versioned registry of named verification gates. Specs reference gates with `verification_gates`; MCP `spec_verify` accepts a spec identity, never an ad-hoc command. Execution requires machine-local operator approval bound to the canonical workspace identity and manifest digest. Gates use direct program/argv execution, confined cwd, a minimal environment, time/output limits, and process-tree termination. Generated transcripts record real green or red results, manifest digest, workspace fingerprint, tool version, and content hash.

A `Required verification` item with `kind=executable` declares a gate that the kit could execute only when its ID is present in the approved manifest. Manual or operator evidence is distinct: a checked transcript or checklist item is self-reported evidence and never proves kit execution. Validation and project digest diagnostics report declared executable-gate count, manifest coverage, and approval state; they never treat a checked item as executed.

Verification onboarding is explicit and reversible. `verification_manifest_status` reports `absent`, `invalid`, `unsafe`, `unapproved`, `approved`, `stale`, or `approval-invalid` with a next action. `verification_manifest_init` produces a bounded deterministic preview from supported repository metadata; it never executes discovered commands and never imports shell bodies from package scripts or CI. `verification_manifest_validate` validates complete TOML without writing. `verification_manifest_set` atomically replaces the complete manifest with optimistic digest checking and preserves one recoverable previous version; `verification_manifest_rollback` restores it with the same stale-write protection. Every create, replace, or rollback requires a separate approval of the resulting digest.

The application may expose status, discovery preview, complete-manifest replacement, and rollback. It must not expose verification approval or couple any of those actions to an artifact lifecycle transition. Approval remains a separate operator action through `verification_manifest_approve`.

A gate that intentionally emits build artifacts may declare `fingerprint_exclude` with workspace-relative output paths; those paths are skipped by the pre/post snapshot fingerprints and later freshness checks for the executed gate set. Exclusions reject absolute paths, traversal, and anything under `.lmbrain`, and they are part of the canonical manifest digest: declaring or changing one invalidates existing approval, so a gate cannot self-exclude its own mutations without operator review.

Inside `### Verification transcript`, `spec_verify` owns only the region delimited by `lmbrain-generated-verification:start` and `lmbrain-generated-verification:end` comments. Agent-authored evidence outside that region is immutable from the verifier's perspective. The verifier merges into the latest unchanged artifact identity and gate contract; it fails closed if the spec moves or the gate contract changes during execution.

Every `working -> review` transition requires a non-empty fenced `### Verification transcript` nested inside `## Implementation evidence`. Hand-authored evidence passes only the structural gate and is unauthenticated. Generated evidence becomes stale after relevant workspace or manifest changes; force remains an explicit reasoned and audited operator override.

Planning is read-only and deterministic. It reports the effective host configuration, supported capability keys, required-tool readiness, LMBrain-owned native paths, and whether each target would be added, changed, preserved, or blocked by a structural conflict. Planning never grants approval or writes a native host file.

Application requires a currently approved digest and uses the same workspace mutation lock as manifest replacement. All changed native files are staged before replacement; structural conflicts stop before writing, and a failed batch restores the prior files. Successful application records machine-local content hashes for read-only drift reporting. Repeating an unchanged application is a no-op.

## First-class findings

A review-local finding remains evidence inside one review. Promote it to a global `FINDING-*` only when the underlying observation survives the originating spec, spans artifacts, records a durable limitation/risk, or is retained before a spec is ready. A finding records what remains true; it is not a spec, work authorization, ADR, diagnostic, GitHub issue, verification requirement, or agent-performance signal.

Findings live under `findings/<status>/` with globally allocated IDs. Creation is `open` only. Normal transitions are `open|planned|deferred -> planned|open|deferred|resolved|accepted-risk|superseded` as allowed by the semantic operation. `resolved`, `accepted-risk`, and `superseded` are terminal; only the operator may reopen resolved/accepted-risk, and superseded history is never reopened. Blocked state is derived from `blocked_by`.

Canonical relationships are flat arrays: `related_specs`, `related_reviews`, `related_decisions`, `target_specs`, `blocked_by`, and `resolution_refs`. `origin_artifact + origin_ref` identifies a promoted review-local finding; two active findings cannot claim the same pair. Direct observations may omit both origin fields but require explicit statement and provenance. References must resolve to allowed families, self-links and blocker cycles fail closed, and linking or completing a target spec never resolves a finding.

`planned` requires an existing target spec. `deferred` requires rationale and a revisit condition. `resolved` requires canonical resolution references, body evidence, and a reasoned typed event. `accepted-risk` is operator-only and requires rationale plus an explicit revisit condition or no-revisit statement. `superseded` requires a successor or explicit obsolescence rationale. All mutations are locked, atomic, status-directory preserving, and append a typed `finding_events` entry.

Semantic MCP operations are `finding_create`, `finding_plan`, `finding_defer`, `finding_resolve`, `finding_accept_risk`, `finding_supersede`, and `finding_reopen`. `finding_context` and `finding_candidates` are read-only; candidate inventory never infers disposition or creates artifacts. The app exposes only read-only finding lists, relations, and copyable governed prompts—never ungoverned lifecycle controls.

## Hard spec dependencies and parking

`depends_on: [SPEC-*]` is the only first-class hard-prerequisite field for specs. Entries must be unique, resolve to specs, exclude self, and form an acyclic graph. Legacy prose and wikilinks are advisory; `spec_dependency_candidates` may report explicit hard-dependency language but never writes or enforces a candidate. `spec_dependency_context` returns direct prerequisites/dependents, a deterministic bounded transitive view, and blocking chains.

Normal `spec_ready` and `spec_start` require every direct hard prerequisite to be `done`. Missing, discarded, malformed, cyclic, and otherwise non-done prerequisites block with the complete direct set and bounded transitive chains. A reasoned force override retains the exact unmet dependency evidence in `mutation_overrides`. Completing prerequisites makes dependents eligible without rewriting them.

`spec_dependencies_set` is the only dependency replacement operation. It requires actor, reason, the exact source digest returned by dependency context, graph validation, and an append-only `dependency_events` record. Dependency edits are allowed only in `backlog`; an approved spec must first be parked so its readiness cannot survive a changed contract.

`spec_park` is the only legal `ready -> backlog` operation. It requires actor and reason, accepts an optional revisit condition, moves atomically, and appends `parking_events` with `readiness_invalidated: true`. It is not discard, rejection, remediation rollback, or an agent-failure signal. Re-entry uses normal `spec_ready` and preserves all parking history. The app displays dependency and parking state read-only and exposes no approve, parking, dependency mutation, or other status-changing action.

## LMBrain kit feedback report

`reports/lmbrain-kit-feedback.md` is the portable, append-only field report for evidence-backed observations about LMBrain itself. It is not project status, backlog, a review, a diagnostic, a `FINDING-*`, or implementation authority. Its fixed identity is `LMBRAIN-KIT-FEEDBACK`, schema version 1, with typed `notes`.

Each note has a stable `KIT-NOTE-*` ID, timestamp, LMBrain version, category, severity, summary, observed behavior, expected behavior, impact, evidence, actor, and optional workaround, suggested improvement, or `related_note`. Categories are `bug`, `usability`, `workflow`, `documentation`, `compatibility`, `performance`, and `improvement`. Severities are `blocking`, `high`, `medium`, `low`, and `info`.

The Project Lead may call `lmbrain_feedback_record` autonomously. The operation is locked and atomic, appends rather than rewrites history, validates relations and taxonomy, and never changes an artifact status. `lmbrain_feedback_report` is read-only and does not create an absent report. Notes must omit credentials, personal data, proprietary source excerpts, and unnecessary project content; evidence uses the minimum reproducible context required by the LMBrain product team.

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
| Skill | `SKILL-` | `skills/<status>/` |
| Finding | `FINDING-` | `findings/<status>/` |

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

Agent-profile optional field: `mnemonic_name`. It is a short human conversational label for an agent profile. It does not replace `id`, `title`, `role`, or authority metadata.

Agent-proposal optional field: `proposed_mnemonic_name`. It records the intended `mnemonic_name` before a profile is materialized.

Spec optional fields include `depends_on`, `dependency_events`, and `parking_events`. They record hard prerequisites and append-only governed mutation history; they are never edited casually in the app.

Spec and agent-profile optional field: `skills`. It records `SKILL-*` procedure references that should be considered during spec assignment or role-specific operation.

Skill optional fields: `scope`, `kind`, `risk`, `applies_to`, `domains`, `commands`, and `requires_operator_approval`. Skills are documented project procedures; command entries are instructions for agents, not app-executed automation.

Priority values: `critical`, `high`, `medium`, `low`.

### Spec tags

`tags` is descriptive planning vocabulary owned by the Project Lead and assigned
through `spec_set_tags`. Values are normalized to lowercase, with spaces and
underscores replaced by `-`, a leading `#` stripped, duplicates collapsed, and a
2–32 character `^[a-z0-9][a-z0-9-]*$` shape.

A tag must not restate a structured field. Values equal to the spec's own
`milestone`, `area`, or `priority`, values shaped like a release (`3.1.0`,
`v2.8`), and values starting with `milestone-` are rejected: set the field
instead. Existing artifacts are never rewritten automatically; they surface a
`field-restating-tag` diagnostic until the next governed tag mutation.

The kit ships a canonical starter vocabulary. Values outside it stay usable and
report an informational `unknown-spec-tag` diagnostic.

### Spec implementation estimate

`capability_tier` and `thinking_level` are the Project Lead's implementation
estimate, assigned through `spec_set_effort`.

| Field | Values | Meaning |
| --- | --- | --- |
| `capability_tier` | `luna`, `terra`, `sol` | Expected change footprint |
| `thinking_level` | `minimal`, `standard`, `extended`, `maximum` | Expected deliberation |

`luna` is roughly two files and a change known before starting; `terra` is
several files in one layer; `sol` is a large footprint, or any work crossing the
frontend, `lmbrain-core`, MCP, and this contract, at any size.

`thinking_level` defaults from the tier (`luna`→`minimal`, `terra`→`standard`,
`sol`→`extended`) and may be raised or lowered with a recorded reason. A `sol`
spec is never `minimal`; a `luna` spec is never `maximum` without a reason.

`effort_observations` is an append-only list written by implementation
specialists through `spec_record_effort_observation`. It records the tier the
work actually required and never modifies the Lead's recommendation.

### Decision supersession

`supersedes` and `superseded_by` are the two sides of one relationship, written
together by `adr_supersede`. The verb sets the successor's `supersedes`, and the
predecessor's `superseded_by` plus its status.

The superseding decision must already be `accepted`: a proposal may *declare*
`supersedes` as a pending intent, but supersession only takes effect when the
successor is approved.

The verb locks both artifacts in ID order before reading either, and writes the
successor first. A crash between the two writes therefore leaves a one-sided
claim, which the `dangling-supersession` diagnostic reports and re-running the
verb repairs; the verb is idempotent on an already-consistent pair. Writing the
predecessor first would instead strip a decision of its authority with no
successor recorded anywhere.

Existing artifacts are never rewritten: one-sided relationships written before
this verb existed surface as diagnostics until the verb is run.

## Declared branching strategy

`.lmbrain/BRANCHING.json` is the optional, versioned source of project Git branching intent. Schema version 1 defines top-level properties: `topology` (`main-only`, `github-flow`, `git-flow`, `custom`), `default_branch`, `protected_branches`, `development_branch`, `branch_naming`, `authority`, and `commit_triggers`.

The strategy is declarative guidance for the Project Lead and implementing agents; LMBrain never executes automated `git` checkout, branch creation, merge, or push commands.

Controlled MCP access uses `branching_strategy_get` and `branching_strategy_set`. Mutating the strategy requires operator authority (`actor: operator`) and is written atomically with audit logging in `.lmbrain/BRANCHING.audit.jsonl`. An unconfigured repository reports status `absent`. Kit scaffolding initializes the `main-only` default strategy upon explicit initialization.

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
| Skill | `proposed`, `active`, `retired` |

## Context packs (v3 context economy)

Context packs are read-only, derived views of the artifact directory. They are not the system of record:

- `lmbrain_project_digest` — compact project overview for Project Lead bootstrap and pulse.
- `lmbrain_spec_context` — spec assignment context for specialist orientation.
- `lmbrain_review_context` — review context for reviewer orientation.

Context packs resolve linked specs, ADRs, reviews, agent profiles, roadmap milestones, required verification, applicable skills, and diagnostics deterministically. Spec/review packs preserve the canonical verification source, distinguish executable/manual/operator gates and lifecycle owners, include profile path/digest plus bounded operational guidance, and expose skill path/digest plus body-command fallback. They report lossy legacy syntax and missing references as structured warnings. They never mutate files.

Agents must read mandatory policy files (`QUALITY.md`, `CONTRACT.md`, `AGENT.md`) before relying on context packs. They must expand to full source artifacts when a context pack warning indicates a missing or unresolved reference.

## Review finding taxonomy v1

New review writes use one of these canonical `finding_categories`: `compatibility`, `correctness`, `documentation`, `localization`, `maintainability`, `metrics-integrity`, `performance`, `provenance`, `requirements-completeness`, `robustness`, `schema-conformance`, `security-boundary`, `test-quality`, `usability`, `verification-integrity`.

Legacy aliases are normalized at read time while their raw value remains visible. Unknown values are not merged into recurrence signals: context packs and dashboards report them as data-quality warnings. Review lifecycle metrics prefer valid structured events, fall back to explicit legacy cycle/count fields, and treat status-only history as uncertain rather than first-pass success.

## Diagnostics and project orientation

Diagnostics use a versioned core record with a stable ID, code, severity, artifact ID/path, message, safe next action, and fixability. `lmbrain_validate`, `lmbrain_project_digest`, and the desktop app consume the same rule engine. The digest is bounded: every list carries total and omitted counts, and compatibility warning strings are derived from the same findings rather than counted separately.

`STATUS.md` is the declared narrative state. Lifecycle counts and the current working milestone are derived separately from governed specs and reconciled with `ROADMAP.md`; conflicts are reported and neither source is silently rewritten or chosen as universal truth.

## Invariants

- A spec reaches `done` only with its acceptance criteria checked, evidence recorded, and an accepted review.
- A spec cannot normally become `ready` or `working` until every `depends_on` prerequisite is `done`; dependency graph errors fail closed.
- A spec cannot become `ready` without a valid `capability_tier` and `thinking_level`; the gate fails closed and forced transitions retain the blocker in the audit trail. Specs already past `ready` without an estimate are never rewritten: they surface a `missing-effort-estimate` diagnostic.
- Spec tags are descriptive metadata and never carry authority. A tag that restates `milestone`, `area`, or `priority` is rejected by the governed tag mutation.
- An effort observation is specialist evidence. It is append-only and never changes the Lead-owned recommendation, and no tier ever selects or starts an agent.
- A ready spec returns to backlog only through `spec_park`; a parked spec cannot start until normal re-approval and its parking history is retained.
- Kit feedback is append-only and informational. Recording it never authorizes a project mutation, LMBrain implementation, external submission, or lifecycle transition.
- Verification authority is distinct: `agent`/`kit` requirements belong to `before-submit`; `lead`/`operator` requirements belong to `before-done`. Lead and operator requirements need both an already-checked checklist item and a fresh typed attestation from the matching authority. Attestation records evidence only: it never approves a spec, checks the item, or changes lifecycle status. Lead uses `spec_attest_lead`; the human operator uses the desktop verification panel. Normal `spec_submit`/`spec_done` report every blocker; forced transitions retain the blocker details in the audit trail. Legacy completed specs remain completed and surface unresolved gates as diagnostics.
- Reviews are created only as `pending`. Verdicts use the semantic `review_accept`, `review_changes_requested`, `review_block`, or `review_supersede` MCP verb; each mutation moves the file and appends one versioned `review_events` entry atomically. Negative verdicts require a rationale. Missing legacy history is reported as unknown and is never reconstructed from prose.
- A spec reaches `review` only with a structurally valid Verification transcript; stale generated evidence is rejected unless explicitly forced with an audited reason.
- Improvement signals are read-only derived views. Profile changes require an evidence-linked proposal, explicit operator approval, a matching target digest, and additive atomic application; agents never self-approve.
- `rejected` is a terminal "declined at proposal/decision time" status available on every proposable artifact (Spec, ADR, Agent proposal, MCP proposal). It is distinct from `changes-requested` (a review asking for revision and resubmission) and from `archived`/`superseded`/`deprecated` (retiring something that was once active). A rejected artifact records the rejection rationale in its body and is not silently reopened.
- An `active` MCP needs a documented spec, permissions, and verification evidence.
- An agent profile always has `activation: manual`; LMBrain never spawns agents.
- An agent profile may have a `mnemonic_name`; when present it is display/context metadata only and never grants authority.
- A skill is procedural knowledge, not an executable capability. LMBrain must not auto-run skill commands.
- Skill references from specs, agent profiles, or `applies_to` must resolve to existing `SKILL-*` or `AGENT-*` artifacts where applicable; `applies_to: [all]` is allowed.
- An ADR is not rewritten to change history: create a replacement ADR and mark the old one `superseded`.
- Supersession agrees on both sides or it is reported. When an `accepted` decision declares `supersedes`, the named decision must be `superseded` and must record the successor in `superseded_by`; a mismatch raises `dangling-supersession` or `supersession-not-mutual`. A `proposed` decision's declaration is a pending claim and raises nothing. A decision never supersedes itself.
- The Project Lead may write only inside `.lmbrain/` during ordinary work. It may alter application code only through the narrowly scoped, operator-authorized escalation process in `AGENT.md`.
- All implementation and review work complies with `QUALITY.md` unless a human-approved exception is recorded.
- A session handoff is a context snapshot and must be validated by the receiving Project Lead before it drives project decisions or status changes.
- At most one `ready` session handoff may exist in `handoffs/active/`.
- The application should warn about duplicate IDs, broken links, directory/status mismatches, missing references, and circular dependencies.

Mutation reasons are stored in typed frontmatter audit fields and are not
duplicated into the Markdown body. Existing historical `## Mutation override`
sections remain readable and are not rewritten automatically.

Review remediation lifecycle events are append-only. `review_remediation` is an
implementation-specialist event and must identify `remediation_agent`.
`review_remediation_verified` is a Project Lead-only event: it preserves review
status, requires at least one non-empty `evidence_refs` value, and is legal only
immediately after a remediation event. A second consecutive verification is
rejected; the next remediation starts a new verification cycle. Existing review
events are never rewritten.

## Authority

| Artifact | Project Lead | Specialist | User |
| --- | --- | --- | --- |
| Project, roadmap, status | maintain | no | approve/edit |
| Specs | create/maintain | implementation evidence only | approve/edit |
| Reviews | create on request | no | request/edit |
| ADRs | propose/maintain | propose only | approve/edit |
| Agent and MCP registries | maintain proposals | no | approve/edit |
| Skills | propose/maintain | follow active procedures and suggest improvements | approve/edit |
| LMBrain kit feedback report | record evidence autonomously | suggest observations | review/deliver externally |
| Session handoffs | create/consume | no | request/edit |
| Application code and configuration | no, except qualified escalated corrective work | only when manually assigned by user | edit |
