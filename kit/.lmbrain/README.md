# LMBrain Project Brain

This directory is the portable, versioned source of truth for a project's knowledge, planning, design mockups, agent handoffs, and reviews.

**Kit version:** read from `VERSION` (canonical)

## Quick start

1. Copy `.lmbrain/` into the root of the target repository.
2. Open the repository in LMBrain so it can register `lmbrain-mcp` for supported agent hosts.
3. Give the Project Lead the bootstrap request in `templates/project-lead-bootstrap-prompt.md`.
4. The Project Lead personalizes the root documents and creates the first specs.
5. For each approved piece of work, either start the recommended specialist manually or explicitly authorize the Project Lead to dispatch the named spec using its estimated model tier.
6. When the work is complete, explicitly ask the Project Lead for a review.

## Principles

- Markdown files are the source of truth; the app is a read-oriented operational view over them.
- Durable cross-spec observations live in governed `DEBT-*` artifacts; ordinary review findings remain local.
- The Project Lead analyzes, specifies, recommends, and reviews. It never implements or dispatches agents without explicit, bounded operator authorization.
- Specialist agents are started manually by the user or through an operator-authorized Lead dispatch, and write implementation evidence only.
- Claude Code and Codex can both use the same repository-scoped `lmbrain-mcp` tools after LMBrain registers the workspace.
- New agent profiles and MCPs are proposed before they are made active.
- Design mockups live under `design/` as operator-loaded files; design specialists use the same proposal/profile workflow as every other agent.
- A Project Lead can write a validated session handoff for the next Project Lead session.
- Operator-facing Lead communication uses concise plain language in the operator's language; dense technical shorthand stays in artifacts and specialist handoffs.
- The Lead autonomously maintains `reports/lmbrain-kit-feedback.md` with evidence-backed observations about LMBrain itself for later delivery to the LMBrain team.

Start with `OPERATOR.md` for the human workflow. Read `CONTRACT.md` for the complete data contract, `QUALITY.md` for the mandatory production standard, and `AGENT.md` for the Project Lead's operating rules.

Use `UPGRADING.md` when upgrading between released kit versions.
