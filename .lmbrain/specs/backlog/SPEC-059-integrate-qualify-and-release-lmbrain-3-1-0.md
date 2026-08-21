---
id: SPEC-059
# Note: Quote the title if it contains a colon
title: "Integrate qualify and release LMBrain 3.1.0"
status: backlog
kind: release
priority: critical
area: release-integration
milestone: M-07
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/10, https://github.com/fathorMB/LMBrain/issues/11, https://github.com/fathorMB/LMBrain/issues/12, https://github.com/fathorMB/LMBrain/issues/13, https://github.com/fathorMB/LMBrain/issues/14, https://github.com/fathorMB/LMBrain/issues/15, https://github.com/fathorMB/LMBrain/issues/16, https://github.com/fathorMB/LMBrain/issues/17, https://github.com/fathorMB/LMBrain/issues/18]
created: 2026-07-29
updated: 2026-07-29
tags: [3.1.0, release, migration, regression]
activity:
  - date: 2026-07-29
    action: "created"
---
# Integrate, qualify, and release LMBrain 3.1.0

## Objective
Integrate the issue-driven 3.1.0 tranche on top of 3.0.2, migrate the bundled kit additively, qualify real project regressions, align every version surface, and publish only from reproducible green release gates.

## Context
The local checkout is currently at tagged `v3.0.1` and one commit behind `origin/main`/`v3.0.2`; user-owned untracked `.pi/`, `ISSUE-4-PLAN.md`, and `ISSUE-8-PLAN.md` must be preserved. GitHub has nine open issues (#10–#18) with no milestone assignment. This release spec closes only after the approved leaf specs are implemented and reviewed.

## Scope
### Included
- Rebase/integrate leaf work on the exact 3.0.2 baseline without overwriting local user work.
- Require accepted reviews for every included leaf spec and reconcile GitHub issue coverage.
- Apply explicit additive kit migrations for new schemas/artifacts/tools; never auto-rewrite customized projects.
- Qualify migration/rollback against LMBrain dogfooding and sanitized XenoMark/AstraNexus-shaped fixtures.
- Run frontend, Rust workspace, MCP contract, migration, installer, packaged Windows, version-alignment, and diff gates.
- Update package/Tauri/Cargo/kit versions, changelog, migrations, README/docs, installer workflow, and release notes consistently.
- Verify that older apps preserve unknown Markdown and that 3.1.0 diagnoses unsupported/new state safely.

### Excluded
- Shipping unapproved leaf scope or hiding known failures.
- Closing GitHub issues without evidence that their acceptance criteria passed.
- Automatic migration during workspace open.

## Existing-project analysis
3.0.2 changes `lmbrain-core/src/verification.rs` with digest-compatible build-output exclusions; all current issue root causes remain present on `origin/main`. The repository’s own `.lmbrain` project/status documentation is stale and has many historical review-state artifacts, making it a useful but noisy migration fixture.

## Technical proposal
Treat release integration as its own reviewed handoff. Freeze schemas before version bumps, merge in dependency order, run leaf gates after every tranche, then execute one clean release candidate qualification from a clean worktree and packaged application. Record exact evidence and unresolved deviations; do not convert warnings to success.

## Files and areas involved
- all leaf-spec outputs
- `package.json`, Cargo manifests/lock, Tauri config, kit VERSION/CHANGELOG/MIGRATIONS
- scripts, CI/installers, README and public docs
- release and migration fixtures/evidence

## Acceptance criteria
- [ ] Every included GitHub issue maps to an approved leaf spec and accepted review.
- [ ] Work is based on 3.0.2 and preserves unrelated local/user changes.
- [ ] Kit/app/MCP schemas, versions, docs, migrations, and rollback guidance are aligned.
- [ ] No workspace-open/read/refresh path performs new automatic mutation.
- [ ] Sanitized real-project fixtures cover review cycles, verification gates, digest conflicts, spec dependencies, parking, and findings.
- [ ] Full canonical and packaged Windows release gates pass from a clean release candidate.
- [ ] Installer assets and sidecar report 3.1.0 consistently.
- [ ] GitHub issues are closed only after evidence is linked and the release decision is explicit.
- [ ] Issue #12 ships completely through accepted SPEC-057 and SPEC-058; a
      core-only or hidden Findings implementation is not a valid 3.1.0 release.

## Implementation plan
1. Confirm approved leaf scope and dependency graph.
2. Integrate bug-foundation and verification tranches on 3.0.2.
3. Integrate spec lifecycle and findings tranches after their ADR decisions.
4. Run migration/rollback and real-project qualification.
5. Align versions/docs and execute clean packaged release gates.
6. Prepare release evidence and issue closeout for operator approval.

## Required verification
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `cargo test --workspace`
- `node scripts/check-version.mjs`
- migration/rollback fixture suite
- Windows installer workflow and installed-app/sidecar smoke
- final `git diff --check` and clean-worktree verification

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- All nine issues are release scope. Issue #12 is explicitly release-blocking;
  reduce concurrency or move the release date rather than shipping an
  incomplete intermediate version.
- Release is blocked by any unresolved authority/UX decision recorded in leaf specs.

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
