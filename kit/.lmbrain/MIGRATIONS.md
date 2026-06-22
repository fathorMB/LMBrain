# Kit Migrations

This document describes how to update an existing LMBrain kit between released versions.

## Current policy

The current released kit is `1.1.0`.

### 1.1.0 (Contract v0.2 — additive)

`1.1.0` adds the `rejected` status across proposable artifacts and defines Agent-proposal statuses (see [[ADR-003-reject-as-first-class-status]]). It is **additive and backward-compatible**: no existing artifact changes meaning, and no frontmatter must be rewritten. When upgrading an existing brain to `1.1.0`:

1. Add the `specs/rejected/` directory (with a `.gitkeep`).
2. No other file moves or frontmatter changes are required.
3. Existing artifacts remain valid; `rejected` simply becomes an available status.

When the first released version is published, this document will state:

1. the supported source version(s);
2. required file additions, moves, renames, or frontmatter changes;
3. any manual review required from the human operator;
4. validation steps after updating;
5. rollback guidance where applicable.

## Migration principles

- Never silently destroy or overwrite project knowledge.
- Preserve custom project content and unknown Markdown files.
- Prefer additive, backward-compatible changes where possible.
- Use explicit, versioned instructions for breaking contract changes.
- Require human confirmation before a future application performs repository writes for migration.
- Update `VERSION` only after every required migration step and validation check succeed.

## Planned validation after a future migration

- `VERSION` contains the expected released version.
- Required root documents and directories exist.
- Artifact IDs remain unique.
- Status-directory paths and frontmatter status values agree.
- References to specs, tasks, reviews, ADRs, agents, MCPs, and handoffs resolve.
- Git diff is reviewed before committing the update.
