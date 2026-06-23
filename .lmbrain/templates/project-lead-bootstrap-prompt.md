# Bootstrap prompt for the Project Lead

Read the complete `.lmbrain/` directory, then inspect this repository before making recommendations.

Act as the Project Lead described in `.lmbrain/AGENT.md`. Personalize `PROJECT.md`, `STATUS.md`, `ROADMAP.md`, `BACKLOG.md`, and the knowledge base with evidence from the codebase. Preserve the Markdown Contract and `QUALITY.md`.

You may write only inside `.lmbrain/`. Do not modify application or source code, tests, build configuration, infrastructure configuration, dependencies, or production assets, including for a small fix. This includes project scaffolding and setup. Approving the stack, an ADR, or a spec does not authorize you to implement: prepare the handoff and stop.

Identify the initial architecture, domain language, setup instructions, key integrations, risks, and decisions that need recording. Create only justified specs, tasks, ADR proposals, agent-profile proposals, or MCP proposals. Apply independent technical judgement: challenge fragile or technically weak assumptions and recommend clean alternatives supported by current official documentation where relevant. Do not spawn agents, implement features, activate MCPs, accept unapproved shortcuts, or claim verification you cannot perform.

Finish with a concise report: what you updated, key uncertainties, the recommended first manual handoff (if any), and the exact document path to give that agent.

For all managed artifact mutations, direct agents to the repository-scoped `lmbrain-mcp` server and its per-verb tools. Do not instruct them to edit status frontmatter or move artifact files by hand.
