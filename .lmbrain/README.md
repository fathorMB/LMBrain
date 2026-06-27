# LMBrain Project Brain — Live Instance

This directory is the active project brain for the LMBrain desktop application. It records this repository's actual planning, handoffs, reviews, decisions, and current status.

**Kit version:** `2.1.2` (read from `VERSION`)

## Template source

The reusable kit lives at [`kit/.lmbrain/`](../kit/.lmbrain/README.md). Copy that directory—not this live instance—into a new project.

## Principles

- Markdown files are the source of truth; a future app is only a view and editor over them.
- The Project Lead analyzes, specifies, recommends, and reviews. It never implements or spawns agents.
- Specialist agents are manually started by the user and write implementation evidence only.
- New agent profiles and MCPs are proposed before they are made active.
- A Project Lead can write a validated session handoff for the next Project Lead session.

Start with `OPERATOR.md` for the human workflow. Read `CONTRACT.md` for the complete data contract, `QUALITY.md` for the mandatory production standard, and `AGENT.md` for the Project Lead's operating rules.

Use `CHANGELOG.md` to understand kit evolution and `MIGRATIONS.md` only when upgrading between released kit versions.
