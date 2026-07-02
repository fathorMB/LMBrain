# Kit Migrations

This document describes how to update an existing LMBrain kit between released versions.

## Current policy

The current released kit is `2.2.7`.

### 2.2.7 (v3 context economy — additive)

`2.2.7` adds context-pack MCP tools (`lmbrain_project_digest`, `lmbrain_spec_context`, `lmbrain_review_context`), granular specialist-profile guidance, and v3 context-economy workflow docs. It is **additive and backward-compatible**: no existing project artifact changes meaning, and no existing artifact frontmatter must be rewritten. When upgrading an existing brain to `2.2.7`:

1. No file moves or frontmatter changes are required.
2. Existing artifacts remain valid.
3. The new MCP tools become available automatically when the app registers `lmbrain-mcp`.
4. The updated handoff prompt includes context-economy guidance; existing prompts still work.
5. Review `AGENT.md`, `CONTRACT.md`, `OPERATOR.md`, and `templates/project-lead-bootstrap-prompt.md` for the updated context-tier guidance.
6. Add missing bundled granular specialist profiles from `agents/profiles/` only when their IDs do not already exist in the project:
   - `AGENT-FRONTEND-UI`
   - `AGENT-TAURI-BACKEND`
   - `AGENT-MCP-CONTRACT`
   - `AGENT-KIT-DOCS`
   - `AGENT-REVIEWER`
   - `AGENT-DESIGN`
7. Add bundled v3 agent proposal examples from `agents/proposals/` only when their IDs or filenames do not already exist. Do not overwrite project-specific proposals.
8. Merge the v3 registry rows and "V3 controlled improvement loop" guidance from `agents/registry.md` additively. Preserve all project-specific active profiles and proposals.
9. Keep existing project-customized agent profiles active/inactive exactly as they are unless the operator explicitly approves a profile status change.
10. Update `.lmbrain/VERSION` to `2.2.7` only after the additive file/registry updates and validation checks succeed.

### 1.1.0 (Contract v0.2 — additive)

`1.1.0` adds the `rejected` status across proposable artifacts and defines Agent-proposal statuses (see [[ADR-003-reject-as-first-class-status]]). It is **additive and backward-compatible**: no existing artifact changes meaning, and no frontmatter must be rewritten. When upgrading an existing brain to `1.1.0`:

1. Add the `specs/rejected/` directory (with a `.gitkeep`).
2. No other file moves or frontmatter changes are required.
3. Existing artifacts remain valid; `rejected` simply becomes an available status.

When any kit-changing version is released, the author MUST document migration guidance for that version in this file. The guidance section (headed by `### <version>`) must include:
1. the supported source version(s);
2. required file additions, moves, renames, or frontmatter edits;
3. any manual review required from the human operator;
4. validation steps to run after upgrading;
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
- References to specs, reviews, ADRs, agents, MCPs, and handoffs resolve.
- Git diff is reviewed before committing the update.
