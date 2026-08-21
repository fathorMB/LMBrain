---
id: SPEC-060
title: "Kit-owned file realignment procedure and drift diagnostics during migration"
status: backlog
kind: feature
priority: medium
area: kit-migration
milestone: M-08
recommended_agent: AGENT-KIT-LEAD
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/34]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-34, kit-migration, KIT-NOTE-008]
activity:
  - date: 2026-07-31
    action: "created"
---
# Kit-owned file realignment procedure and drift diagnostics during migration

## Objective
Prevent silent documentation and template drift between project `.lmbrain/` files and bundled kit releases by explicitly distinguishing project-owned artifacts from kit-owned governance files during migrations.

## Context
Reported in `KIT-NOTE-008` (v3.1.2): `MIGRATIONS.md` states "no manual content migration required" for existing workspace artifacts. While true for project artifacts (specs, reviews, ADRs, findings), kit-owned files inside `.lmbrain/` (such as `CHANGELOG.md`, `README.md`, `MIGRATIONS.md`, `reports/README.md`, and `templates/`) accumulate silent drift across multiple releases if not updated.

## Scope
### Included
- Update `MIGRATIONS.md` release template to include an explicit step for auditing and realigning kit-owned governance files.
- Add a read-only diagnostic / report (`kit_file_drift`) comparing project kit-owned files against bundled defaults to surface outdated files without mutating project-specific customizations (like agent/skill registers).
- Add tests verifying detection of outdated kit templates or changelogs.

### Excluded
- Automatic overwriting of user-customized files (registers, custom knowledge).

## Acceptance criteria
- [ ] `MIGRATIONS.md` clearly separates project artifact rules from kit-owned file updates.
- [ ] A read-only `kit_file_drift` check identifies outdated kit-owned files.
- [ ] Customized project files are preserved and never auto-overwritten.

## Required verification
- `cargo test --workspace`
- `pnpm test`
- `pnpm lint`
