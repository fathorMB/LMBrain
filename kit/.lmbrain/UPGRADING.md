# Upgrading LMBrain Kit

LMBrain 5.0+ uses tool-driven kit upgrades.

## Upgrading via MCP Tools or Desktop Application

Kit upgrades are automated, verified, and atomic:

1. **Preview:** Use `kit_migration_preview` or the LMBrain desktop application to preview incoming template and contract updates.
2. **Review:** Inspect the digest-bound plan. Kit-owned governance files (`CONTRACT.md`, `AGENT.md`, `QUALITY.md`, `UPGRADING.md`, `contract/*.md`, `templates/*.md`, `VERSION`) are realigned while project-owned artifacts (`PROJECT.md`, `STATUS.md`, `specs/`, `reviews/`, `decisions/`, `debts/`, `reports/`, `agents/`, etc.) are strictly preserved.
3. **Check the classification of every kit-owned item.** The kit records the digests it shipped in `.lmbrain/.kit-baseline.json`, so the preview can tell an edited file apart from one that is merely older:
   - `kit-owned` — the workspace copy is exactly what the kit installed; realignment loses nothing.
   - `kit-owned-modified` — the copy was edited after installation. The plan lists it under `locally_modified`, and realigning replaces that edit. Port the change to a project-owned artifact first, or recover it from the backup afterwards.
   - `kit-owned-unverified` — no baseline covers the file (a workspace installed before 5.0, or a file the kit did not ship at that version), so a local edit cannot be ruled out. Diff it before confirming.

   The classification is part of the preview digest: editing a kit-owned file after taking a preview invalidates it, and the migration fails closed rather than applying a plan the operator never saw.
4. **Execute:** Confirm the upgrade via `kit_migrate` or the desktop application. The migration is validated in staging and swapped atomically. The previous `.lmbrain` is retained at the reported `backed_up_to` path, which is the recovery route for anything a realignment replaced.

## Release Notes & History

Full release notes, migration guides, and historical changelogs are maintained upstream in the LMBrain product repository and documentation:

- Repository: `https://github.com/fathorMB/LMBrain`
- Documentation: `docs/CHANGELOG.md` and `docs/MIGRATIONS.md`
