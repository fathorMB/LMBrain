# LMBrain Project Brain

This directory is the portable, versioned source of truth for a project's knowledge, planning, design mockups, agent handoffs, and reviews.

**Kit version:** `2.2.2` (read from `VERSION`)

## Quick start

1. Copy `.lmbrain/` into the root of the target repository.
2. Open the repository in LMBrain so it can register `lmbrain-mcp` for supported agent hosts.
3. Give the Project Lead the bootstrap request in `templates/project-lead-bootstrap-prompt.md`.
4. The Project Lead personalizes the root documents and creates the first specs.
5. For each approved piece of work, manually start the recommended specialist with its `SPEC-*.md` file.
6. When the work is complete, explicitly ask the Project Lead for a review.

## Principles

- Markdown files are the source of truth; a future app is only a view and editor over them.
- The Project Lead analyzes, specifies, recommends, and reviews. It never implements or spawns agents.
- Specialist agents are manually started by the user and write implementation evidence only.
- Claude Code and Codex can both use the same repository-scoped `lmbrain-mcp` tools after LMBrain registers the workspace.
- New agent profiles and MCPs are proposed before they are made active.
- Design mockups live under `design/` as operator-loaded files; design specialists use the same proposal/profile workflow as every other agent.
- A Project Lead can write a validated session handoff for the next Project Lead session.

Start with `OPERATOR.md` for the human workflow. Read `CONTRACT.md` for the complete data contract, `QUALITY.md` for the mandatory production standard, and `AGENT.md` for the Project Lead's operating rules.

Use `CHANGELOG.md` to understand kit evolution and `MIGRATIONS.md` only when upgrading between released kit versions.
