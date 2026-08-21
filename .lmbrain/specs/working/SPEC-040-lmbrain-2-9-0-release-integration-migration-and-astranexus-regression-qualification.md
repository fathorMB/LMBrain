---
id: SPEC-040
# Note: Quote the title if it contains a colon
title: "LMBrain 2.9.0 release integration, migration, and AstraNexus regression qualification"
status: working
kind: release
priority: high
area: release-and-migration
milestone: M-05
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-008, ADR-012]
links: [SPEC-036, SPEC-038, SPEC-039]
created: 2026-07-16
updated: 2026-07-16
tags: [2.9.0, release, migration, qualification, astranexus]
activity:
  - date: 2026-07-16
    action: "created"
  - date: 2026-07-16
    action: "transitioned backlog -> ready"
  - date: 2026-07-16
    action: "transitioned ready -> working"
---
# LMBrain 2.9.0 release integration, migration, and AstraNexus regression qualification

## Objective

Integrate [[SPEC-036]], [[SPEC-038]], and [[SPEC-039]] into one coherent 2.9.0
contract and release, migrate existing projects without overwriting local
knowledge, and qualify the result against the production failure patterns that
motivated the work.

## Context

Release 2.8.0 shipped governed harness environments, while the ready
[[SPEC-036]] verification-provenance work and the context/improvement gaps found
in AstraNexus remain outstanding. Shipping these pieces independently would
leave mismatched schemas, incomplete migrations, and misleading claims about
self-learning. Version 2.9.0 is the coordinated release boundary.

## Scope
### Included

- Require accepted reviews for SPEC-036, SPEC-038, and SPEC-039 before release.
- Reconcile all overlapping contract, template, model, MCP, Tauri, UI, and docs
  changes into a single backward-compatible 2.9.0 schema.
- Provide an additive 2.8.x -> 2.9.0 migration guide and Project Lead prompt.
- Detect and explain legacy paragraph-only verification, empty skill command
  metadata, stale profile primary files, unresolved gates, ownerless manual
  gates, and unapplied improvement proposals.
- Preserve project-specific profiles, skills, specs, ADR amendments, and local
  approval state rules; never overwrite them with bundled defaults.
- Add a reusable anonymized AstraNexus regression fixture covering the seven
  recent production spec shapes and their fast-fail/remediation patterns.
- Demonstrate omission blocking, authentic red/green kit transcripts, stale
  evidence rejection, complete implementer context, operator-gate separation,
  and governed profile improvement application.
- Align package, Cargo, lockfile, bundled kit `VERSION`, changelog, migrations,
  docs, installer metadata, and release notes at 2.9.0.
- Perform Windows packaged qualification and explicitly coordinate any test that
  would start/stop a running LMBrain instance.
- Define a post-release observation report, after enough new specs exist, for
  first-pass, fast-fail, cycle, recurrence, and escalation metrics.

### Excluded

- Silently migrating project behavior or activating profiles/proposals.
- Claiming historical AstraNexus review counts will be retroactively corrected.
- Blocking release on a statistically significant productivity improvement;
  correctness and observability ship first, outcome measurement follows.
- Remote CI attestation, sandboxing, or autonomous agent orchestration.

## Existing-project analysis

- The canonical package and bundled kit are currently 2.8.0, but project
  roadmap/status text still contains older milestone/release residue.
- SPEC-036 is `ready` and contains the main execution security boundary.
- Repository validation currently fails `unique_ids` because the verification
  spec and an unrelated Windows-installer remediation both use `SPEC-036`. The
  exact prerequisite intended here is
  `.lmbrain/specs/ready/SPEC-036-verification-transcript-integrity-and-kit-executed-gates.md`.
  The collision must be corrected under operator authority before any 2.9.0
  feature approval or release sequencing relies on the ID.
- SPEC-038 and SPEC-039 are new 2.9.0 backlog specs and must not bypass the
  normal operator approval, implementation, review, and closeout lifecycle.
- Migration logic is additive by policy, which is essential because AstraNexus
  contains locally amended Lead rules and specialist profiles.

## Technical proposal

Treat 2.9.0 as a release train rather than one large implementation handoff:

1. SPEC-036: trusted execution provenance and stale-evidence enforcement.
2. SPEC-038: complete structured handoff/context contract.
3. SPEC-039: governed improvement signals, proposal application, and metrics.
4. SPEC-040: integration, migration, fixture qualification, packaging, and release.

Each feature spec is independently reviewed. This release spec owns only
cross-cutting reconciliation and release evidence, not feature implementation
that belongs in its prerequisites.

## Files and areas involved

- Versioned schemas and all cross-crate/app generated contracts
- `kit/.lmbrain/` templates, bundled profiles/skills, VERSION, CHANGELOG,
  MIGRATIONS, policies, and operator guidance
- Release/version files in package, Cargo, lockfile, Tauri, and installer config
- Migration prompts/detection, diagnostics, and regression fixtures
- CI/release workflows and Windows packaging tests
- `docs/` architecture, kit, agent-host, verification, and release documentation

## Acceptance criteria

- [ ] SPEC-036, SPEC-038, and SPEC-039 are `done` with accepted reviews before
      the 2.9.0 version is finalized.
- [ ] `lmbrain_validate` reports unique IDs and the former SPEC-036 collision has
      an audited operator-governed resolution before prerequisite closeout.
- [ ] One documented 2.9.0 schema consistently represents executable provenance,
      structured manual requirements, context completeness, and governed profile
      improvements across core, MCP, Tauri, TypeScript, UI, and kit artifacts.
- [ ] A 2.8.x project opens read-only without migration and receives accurate,
      actionable diagnostics rather than parser failures or silent omissions.
- [ ] The migration is additive and tests prove customized profiles, skills,
      specs, ADRs, and Lead amendments are preserved byte-semantically except for
      explicitly operator-approved changes.
- [ ] Migration never carries machine-local digest approval across a materially
      changed verification manifest or workspace identity.
- [ ] AstraNexus regression fixtures prove all seven recent first-cycle omission
      shapes are either blocked before review or surfaced completely in the
      implementer context.
- [ ] Operator/playtest gates are shown as `before-done` and cannot trigger an
      implementer transcript fast-fail unless explicitly assigned before submit.
- [ ] A real end-to-end sample proves an approved improvement reaches the next
      handoff while an unapproved/stale proposal cannot alter the profile.
- [ ] Insights expose baseline and post-release observation fields without
      claiming improvement before sufficient new data exists.
- [ ] All version-bearing files equal `2.9.0`; version-alignment checks, changelog,
      migration note, and release notes are complete.
- [ ] Full Rust/frontend/contract/migration/security gates and Windows packaged
      qualification are green with kit-generated provenance where available.
- [ ] No known process-tree leak, stale-evidence bypass, context gate omission,
      or unauthorized profile mutation is waived for release.

## Implementation plan

1. Verify prerequisite reviews and freeze the combined 2.9.0 schema.
2. Reconcile generated contracts, diagnostics, UI copy, and bundled kit assets.
3. Implement/test additive migration and preservation guarantees.
4. Run AstraNexus-derived end-to-end qualification and remediate cross-feature gaps.
5. Align versions/changelog/docs and build signed/packaged release candidates.
6. Run coordinated Windows qualification, publish only after operator approval,
   and create the post-release observation checklist.

## Required verification

- Full prerequisite accepted-review and lifecycle audit.
- Clean 2.8.x fixture open, diagnostic, migration-plan, migration, reopen, and
  preservation comparison.
- AstraNexus-derived end-to-end scenarios for omission, synthesis/provenance,
  stale evidence, context completeness, operator gates, and improvement apply.
- Full workspace Rust checks/tests, frontend lint/tests/build, contract and
  version alignment, migration snapshots, installer build, and packaged Windows
  smoke on an operator-coordinated instance.
- Release artifact/version/hash inventory and rollback documentation.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- SPEC-036 is security-sensitive and may dominate schedule; do not reduce its
  process, environment, freshness, or approval controls to meet a release date.
- Migrating freeform historical reviews into categories would invent facts;
  leave them uncategorized unless a human curates them.
- AstraNexus is a valuable regression corpus but not the only supported project
  shape; retain generic and non-Git fixtures.
- Release publication, signing, and starting/stopping installed instances remain
  operator-coordinated external actions.

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

- Aligned package, Tauri/Cargo lock, bundled kit version, changelog, migration, templates, contracts, UI contracts, and technical docs at 2.9.0.
- Added seven AstraNexus-derived context regression shapes and cross-feature verification/improvement tests.
- Corrected the historical duplicate verification/installer spec identity under operator authorization; validation now reports unique IDs.

### Files changed

- Version-bearing package/Cargo/kit files, bundled kit assets, docs, core/MCP/Tauri/frontend integration, and regression tests.

### Verification performed

- `node scripts/check-version.mjs`: LMBrain app and kit aligned at v2.9.0.
- `lmbrain_validate`: `unique_ids: true`.
- Full Rust workspace and frontend canonical gates passed.
- Packaged build attempted without starting/stopping the app; blocked by Windows file access held by the running instance.

### Verification transcript

```text
$ node scripts/check-version.mjs
LMBrain app and kit are aligned at v2.9.0.

$ lmbrain_validate
{"unique_ids":true}

$ pnpm tauri build
FAILED: tauri-build could not access the in-use sidecar path (Windows error 5). Existing LMBrain instance was not started, stopped, or disturbed.
```

### Deviations from the specification

- Windows packaged build/smoke and release finalization remain open until the separate running instance can be coordinated. No attempt was made to stop it or replace its in-use sidecar.
- Publication/signing remain external operator actions.

### Handoff status
- [ ] Ready for Project Lead review — blocked only on coordinated packaged Windows qualification
