# Agent Proposals

Use `templates/agent-proposal.md` when the Project Lead identifies recurring specialist work not covered by existing profiles. Approval creates a profile; it never starts an agent.

Shipped proposals, such as the web app design specialist proposal, follow the same process as proposals created during project work: the operator approves, defers, or rejects them before any profile is added to `agents/profiles/` or `agents/registry.md`.

For `proposal_type: improvement`, use deterministic review categories from at least two distinct specs (or one integrity/security escalation), retain every evidence link, and propose only bounded additive fields. `agent_improvement_signals` is read-only. Proposal creation is explicit; apply requires operator approval and the unchanged target-profile digest. Metrics are diagnostic and do not establish causality.
