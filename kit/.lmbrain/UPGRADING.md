# Upgrading LMBrain Kit

LMBrain 5.0+ uses tool-driven kit upgrades.

## Upgrading via MCP Tools or Desktop Application

Kit upgrades are automated, verified, and atomic:

1. **Preview:** Use `kit_migration_preview` or the LMBrain desktop application to preview incoming template and contract updates.
2. **Review:** Inspect the digest-bound plan. Kit-owned governance files (`CONTRACT.md`, `AGENT.md`, `QUALITY.md`, `UPGRADING.md`, `contract/*.md`, `templates/*.md`, `VERSION`) are realigned while project-owned artifacts (`PROJECT.md`, `STATUS.md`, `specs/`, `reviews/`, `decisions/`, `debts/`, `reports/`, `agents/`, etc.) are strictly preserved.
3. **Execute:** Confirm the upgrade via `kit_migrate` or the desktop application. The migration is validated in staging and swapped atomically.

## Release Notes & History

Full release notes, migration guides, and historical changelogs are maintained upstream in the LMBrain product repository and documentation:

- Repository: `https://github.com/fathorMB/LMBrain`
- Documentation: `docs/CHANGELOG.md` and `docs/MIGRATIONS.md`
