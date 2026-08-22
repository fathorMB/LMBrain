# Operator Guide

This is the human entry point for using LMBrain in a project repository.

LMBrain does not automatically start agents. You retain control: you decide when to start a Project Lead or specialist agent. You may start a specialist manually, or explicitly authorize the Project Lead to dispatch named specs. Dispatch authorization is bounded to those specs and is separate from approving or moving them to `ready`.

For operator-authorized dispatch, the Lead must use each spec's `capability_tier`: Sol selects Opus (or a harness-native `-sol` equivalent), Terra selects Sonnet (or `-terra`), and Luna selects Haiku (or `-luna`). The Lead must pass the model explicitly and stop if the harness cannot provide an unambiguous equivalent. See `contract/effort_tags.md`.

## First use in a new repository

1. Copy `.lmbrain/` into the repository root.
2. Open the repository in LMBrain once so it can register `lmbrain-mcp` for Claude Code and Codex.
3. Start a Project Lead manually in your chosen supported agent.
4. Give it [`templates/project-lead-bootstrap-prompt.md`](templates/project-lead-bootstrap-prompt.md).
5. Read `STATUS.md` and the Project Lead's final report.
6. If it recommends a spec assignment, either start the proposed specialist with the exact `SPEC-*.md` path or explicitly authorize the Lead to dispatch that named spec.

## I need a new feature, fix, or technical change

1. Start the Project Lead manually.
2. State the request in normal language, for example: “I need feature X in the software.”
3. Ask it to analyze the repository and prepare the implementation spec assignment according to `AGENT.md`.
4. Expect a response with:
   - the path to a ready `SPEC-*.md` file;
   - the recommended specialist profile;
   - dependencies, risks, and decisions requiring your approval;
   - any MCP capability that is missing.
5. Review the spec. Approve or request changes.
6. If the recommended specialist profile is `proposed` (not yet `active`), explicitly ask the Project Lead to activate it with the controlled LMBrain MCP tool. Only then start the specialist manually or authorize the Lead to dispatch the named spec.

Suggested specialist prompt (v3 context-economy):

> Read `QUALITY.md`, `CONTRACT.md`, and `AGENT.md` first. Then use `lmbrain_spec_context` for a compact spec assignment context. Expand to the full spec and source code only when the context pack points to them or verification requires it. Implement only the stated scope. Fill the **Implementation evidence** section when done. Report changed files, verification performed, and deviations from the spec. Do not update roadmap, strategic decisions, or project status.

Treat the work as production-grade. Follow `QUALITY.md`; do not submit a POC, placeholder, or knowingly incomplete implementation. Update every technical LMBrain page explicitly delegated by the spec.

## I need design mockups before implementation

1. Ask the Project Lead whether the feature needs a design pass before implementation.
2. If no active profile fits, use the normal agent proposal process in `agents/proposals/`; design specialists are not handled specially.
3. When a design specialist is available and recommended, start it manually with the relevant spec or design request.
4. Copy the finished self-contained HTML/CSS/JS mockup package into `design/<mockup-slug>/`.
5. Ask the Project Lead to reference the design path in the implementation spec.

Design mockups are support material. They do not replace specs, reviews, or implementation evidence.

## A specialist says the work is complete

1. Confirm that the specialist filled in the implementation evidence in the `SPEC-*.md` document.
2. Start the Project Lead manually.
3. Ask: “Review the completed work for `<SPEC-ID>`.”
4. The Project Lead creates a `REVIEW-*.md` artifact and checks the implementation against the specification, `QUALITY.md`, and required LMBrain documentation updates.
5. Read the result:
   - `accepted`: the spec and related work can be considered complete;
   - `changes-requested`: hand the same `specs/review/SPEC-*.md` and the review findings back to the specialist manually; the spec stays in `review` during remediation;
   - `blocked`: resolve the recorded blocker before continuing.

The Project Lead records these outcomes through `review_accept`, `review_changes_requested`, `review_block`, or `review_supersede`. Each operation preserves an append-only event history; legacy reviews without that history remain readable but their earlier cycles are explicitly unknown.

The Project Lead reviews; it does not fix the code itself.

For a checked `owner=operator`, `phase=before-done` requirement, open the **Operations** page (or follow the link from the spec detail verification summary), enter your operator identity and an evidence reference, then choose **Record Attestation**. This records an append-only evidence attestation. It does not accept a review, approve the spec, check the requirement, or move the spec to `done`; closeout remains a separate governed `spec_done` action. Lead-owned requirements have no operator action in the app.

## I need to configure project verification

1. Open **Settings → Verification** and inspect the typed manifest status.
2. Choose **Discover / refresh preview** to inspect bounded suggestions, provenance, exact program and arguments, environment policy, mutation policy, exclusions, and the proposed diff. Discovery does not execute commands.
3. Create or replace the manifest only after reviewing the complete preview. LMBrain preserves the prior manifest for a guarded rollback.
4. Approve the resulting digest separately with `verification_manifest_approve`. The app intentionally has no approval control.
5. Use `spec_verify` only after the manifest reports `approved`. Any material manifest change makes approval `stale`.

Creating, replacing, or rolling back `.lmbrain/verification.toml` does not run verification, approve configuration, attest evidence, accept a review, or change a spec status.

## I need to triage a durable debt

Use **Debts** to inspect active/history counts, severity, owner, origin, targets, blockers, canonical relations, and the typed timeline. The view is read-only. It offers a governed MCP prompt, not lifecycle buttons.

The Board and spec detail show hard dependency blockers, prerequisite-complete filters, and preserved parking history. They are read-only: the app has no approve, park, dependency-edit, or status-change action. Governed dependency changes use `spec_dependencies_set` in backlog; intentional `ready -> backlog` parking uses `spec_park` with a reason, then normal `spec_ready` is required later.

The Project Lead should speak to you in your language using concise, ordinary wording. Technical abbreviations and English tool names may remain where exactness matters, but the Lead must explain their practical meaning without requiring a second request for a “human” translation.

The Lead also maintains `reports/lmbrain-kit-feedback.md` autonomously when it encounters an evidence-backed problem or improvement opportunity in LMBrain itself. This does not change project state and needs no approval. At session end, the Lead reports newly added notes; you can deliver that single file to the LMBrain team. Review it before external sharing if the project is confidential, even though the contract requires minimal, non-sensitive evidence.

- A review can be accepted while a promoted debt remains open, planned, or deferred.
- Planning a target spec does not authorize implementation and does not resolve the debt.
- A done target spec produces an attention diagnostic; closure still requires explicit evidence.
- Only you may authorize `debt_accept_risk` or `debt_reopen`. A superseded debt stays historical.
- Legacy review bullets remain local unless you explicitly select them for promotion after inspecting `debt_candidates`.

## I need to end a Project Lead session and resume later

1. Ask the current Project Lead: “Prepare a session handoff before ending this session.”
2. It creates one `HANDOFF-*.md` document in `handoffs/active/`.
3. In the next Project Lead session, instruct the new agent: “Read and validate the active session handoff before continuing.”
4. The receiving Project Lead reads the handoff, linked artifacts, `STATUS.md`, and relevant repository state; it then marks the handoff consumed or corrects the project documentation.

The handoff preserves context. It does not replace validation of the actual repository and Markdown state.

## I want the Project Lead to reflect on technical or design debt

Explicitly invite a bounded dreaming session, for example: “Fatti un pisolino sul lavoro appena fatto” or “Vatti a riposare e cerca debito di design”. The Lead confirms the scope, examines current project context, and may capture zero or more tentative `DREAM-*` records through its controlled MCP tool.

Open **Dream Journal** in the sidebar to inspect them. Dreams are deliberately read-only in the app and never become Debts, specs, ADRs, roadmap items, or implementation work automatically. Ask the Lead to triage or promote a specific dream only after you have reviewed its provenance and suggestion.

## I need a new type of specialist agent

1. Ask the Project Lead whether an existing profile fits first.
2. If not, ask it to create an `AGENT-PROP-*.md` proposal.
3. Read its expected benefit, responsibilities, boundaries, and cost.
4. Approve, defer, or reject the proposal in the document.
5. Once approved, ask the Project Lead to prepare the profile under `agents/profiles/` and update `agents/registry.md`.
6. Start that specialist manually, or authorize a bounded Lead dispatch, only when a spec recommends it.

Approving a profile makes it available; it never launches an agent.

## I need a new MCP capability

1. Ask the Project Lead to document the need and alternatives.
2. Review the resulting `MCP-PROP-*.md`, especially permissions, data handling, and risk.
3. Approve only if the capability and permissions are acceptable.
4. Ask for an `MCP-*.md` specification.
5. Manually arrange implementation or configuration through an appropriate specialist.
6. Make the MCP active only after documented verification.

External access, credentials, or write permissions always require your explicit approval.

## Supported agent hosts

LMBrain is agent-agnostic at the workflow layer. Claude Code and Codex can both use the same `lmbrain-mcp` controlled-mutation server after the workspace is opened in LMBrain.

- Claude Code registration is written to `.mcp.json` in the repository root.
- Codex registration is written to `.codex/config.toml` in the repository root, and LMBrain adds a missing trusted-project entry to `$CODEX_HOME/config.toml`.
- Sessions are always operator-started. LMBrain can launch native Claude, Claude through Ollama, or native Codex terminals; it never starts them automatically.

## Daily project check

Open these documents in order:

1. `STATUS.md` — current focus, blockers, and recommended action.
2. `ROADMAP.md` — milestone direction and planning.

Roadmap milestones use an H2 or H3 heading with a numeric `M-` ID (for example `M-01`) and status `proposed`, `active`, or `completed`. Keep template examples as non-numeric placeholders such as `M-NN`; fenced examples and placeholders are ignored by LMBrain.
3. `specs/ready/` — work ready for manual spec assignment.
4. `specs/review/` and `reviews/pending/` — completed work waiting for review.
5. `specs/review/` — specs in the review ping-pong.

## Who may change what

| Work | Human operator | Project Lead | Specialist |
| --- | --- | --- | --- |
| Start agents | Yes | No | No |
| Feature analysis and specs | Approves | Yes | No |
| Application code | Can edit | No | Yes, when manually assigned |
| Roadmap and project status | Approves/edits | Yes | No |
| Code review after assignment | Requests | Yes | No |
| Agent/MCP activation | Approves and arranges | No | No |

## Documents to know

| Need | Document |
| --- | --- |
| What is happening now? | `STATUS.md` |
| What are we building? | `PROJECT.md`, `ROADMAP.md` |
| What should an agent implement? | `specs/<status>/SPEC-*.md` |
| Did the implementation pass review? | `reviews/<status>/REVIEW-*.md` |
| What work is on the board? | `specs/<status>/SPEC-*.md` |
| What is an agent allowed to do? | `agents/profiles/AGENT-*.md` |
| Where are design mockups loaded? | `design/` |
| What quality standard applies to every spec assignment? | `QUALITY.md` |
| How does a new Project Lead resume a prior session? | `handoffs/active/HANDOFF-*.md` |
| Why was a technical choice made? | `decisions/ADR-*.md` |
| Is a new capability safe and justified? | `mcp/proposals/MCP-PROP-*.md` |

For the full metadata and state rules, read `CONTRACT.md`.
