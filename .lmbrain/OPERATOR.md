# Operator Guide

This is the human entry point for using LMBrain in a project repository.

LMBrain does not automatically start agents. You retain control: you decide when to start a Project Lead or specialist agent, and you pass the relevant document to that agent manually.

## First use in a new repository

1. Copy `.lmbrain/` into the repository root.
2. Start a Project Lead manually.
3. Give it [`templates/project-lead-bootstrap-prompt.md`](templates/project-lead-bootstrap-prompt.md).
4. Read `STATUS.md` and the Project Lead's final report.
5. If it recommends a handoff, start the proposed specialist and give it the exact `SPEC-*.md` path.

## I need a new feature, fix, or technical change

1. Start the Project Lead manually.
2. State the request in normal language, for example: “I need feature X in the software.”
3. Ask it to analyze the repository and prepare the implementation handoff according to `AGENT.md`.
4. Expect a response with:
   - the path to a ready `SPEC-*.md` file;
   - the recommended specialist profile;
   - dependencies, risks, and decisions requiring your approval;
   - any MCP capability that is missing.
5. Review the spec. Approve or request changes.
6. Start the specialist manually and give it the spec path.

Suggested specialist prompt:

> Read `<spec path>`, its linked knowledge pages, and linked ADRs. Implement only the stated scope. Fill the **Implementation evidence** section when done. Report changed files, verification performed, and deviations from the spec. Do not update roadmap, strategic decisions, or project status.

Treat the work as production-grade. Follow `QUALITY.md`; do not submit a POC, placeholder, or knowingly incomplete implementation. Update every technical LMBrain page explicitly delegated by the spec.

## A specialist says the work is complete

1. Confirm that the specialist filled in the implementation evidence in the `SPEC-*.md` document.
2. Start the Project Lead manually.
3. Ask: “Review the completed work for `<SPEC-ID>`.”
4. The Project Lead creates a `REVIEW-*.md` artifact and checks the implementation against the specification, `QUALITY.md`, and required LMBrain documentation updates.
5. Read the result:
   - `accepted`: the spec and related work can be considered complete;
   - `changes-requested`: hand the follow-up `SPEC-*.md` to a specialist manually;
   - `blocked`: resolve the recorded blocker before continuing.

The Project Lead reviews; it does not fix the code itself.

## I need to end a Project Lead session and resume later

1. Ask the current Project Lead: “Prepare a session handoff before ending this session.”
2. It creates one `HANDOFF-*.md` document in `handoffs/active/`.
3. In the next Project Lead session, instruct the new agent: “Read and validate the active session handoff before continuing.”
4. The receiving Project Lead reads the handoff, linked artifacts, `STATUS.md`, and relevant repository state; it then marks the handoff consumed or corrects the project documentation.

The handoff preserves context. It does not replace validation of the actual repository and Markdown state.

## I need a new type of specialist agent

1. Ask the Project Lead whether an existing profile fits first.
2. If not, ask it to create an `AGENT-PROP-*.md` proposal.
3. Read its expected benefit, responsibilities, boundaries, and cost.
4. Approve, defer, or reject the proposal in the document.
5. Once approved, ask the Project Lead to prepare the profile under `agents/profiles/` and update `agents/registry.md`.
6. Start that specialist manually only when a spec recommends it.

Approving a profile makes it available; it never launches an agent.

## I need a new MCP capability

1. Ask the Project Lead to document the need and alternatives.
2. Review the resulting `MCP-PROP-*.md`, especially permissions, data handling, and risk.
3. Approve only if the capability and permissions are acceptable.
4. Ask for an `MCP-*.md` specification.
5. Manually arrange implementation or configuration through an appropriate specialist.
6. Make the MCP active only after documented verification.

External access, credentials, or write permissions always require your explicit approval.

## Daily project check

Open these documents in order:

1. `STATUS.md` — current focus, blockers, and recommended action.
2. `ROADMAP.md` — milestone direction and planning.
3. `specs/ready/` — work ready for manual handoff.
4. `specs/review/` and `reviews/pending/` — completed work waiting for review.
5. `tasks/blocked/` — work requiring attention.

## Who may change what

| Work | Human operator | Project Lead | Specialist |
| --- | --- | --- | --- |
| Start agents | Yes | No | No |
| Feature analysis and specs | Approves | Yes | No |
| Application code | Can edit | No | Yes, when manually assigned |
| Roadmap and project status | Approves/edits | Yes | No |
| Code review after handoff | Requests | Yes | No |
| Agent/MCP activation | Approves and arranges | No | No |

## Documents to know

| Need | Document |
| --- | --- |
| What is happening now? | `STATUS.md` |
| What are we building? | `PROJECT.md`, `ROADMAP.md` |
| What should an agent implement? | `specs/<status>/SPEC-*.md` |
| Did the implementation pass review? | `reviews/<status>/REVIEW-*.md` |
| What work is on the board? | `tasks/<status>/TASK-*.md` |
| What is an agent allowed to do? | `agents/profiles/AGENT-*.md` |
| What quality standard applies to every handoff? | `QUALITY.md` |
| How does a new Project Lead resume a prior session? | `handoffs/active/HANDOFF-*.md` |
| Why was a technical choice made? | `decisions/ADR-*.md` |
| Is a new capability safe and justified? | `mcp/proposals/MCP-PROP-*.md` |

For the full metadata and state rules, read `CONTRACT.md`.
