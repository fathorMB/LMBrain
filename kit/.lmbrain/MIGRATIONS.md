# Kit Migrations

This document describes how to update an existing LMBrain kit between released versions.

## Current policy

The current kit is `1.0.0` and is still pre-release. Do not run migrations or change `VERSION` during this phase.

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
