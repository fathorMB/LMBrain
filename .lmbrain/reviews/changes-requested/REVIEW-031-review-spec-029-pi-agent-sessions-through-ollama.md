---
id: REVIEW-031
# Note: Quote the title if it contains a colon
title: "Review SPEC-029 Pi agent sessions through Ollama"
status: changes-requested
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-029
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
links: [SPEC-029]
created: 2026-07-10
updated: 2026-07-10
tags: [review, sessions, pi, ollama, mcp]
activity:
  - date: 2026-07-10
    action: "created"
---
# Review SPEC-029 Pi agent sessions through Ollama

## Outcome

Changes requested.

The implementation establishes the useful `AgentHost + ModelRoute` domain split, validates the supported matrix before opening a PTY, and constructs the intended Pi command with discrete arguments. Both operator-authorized compilation checks pass. The submitted work is nevertheless incomplete against the approved feature: the default Pi selection path cannot load models, Pi has no LMBrain MCP integration, prerequisite/preflight behavior is absent, required test coverage was not added, and the LMBrain spec lifecycle/evidence was skipped.

## Acceptance-criteria compliance

### Substantially implemented

- Rust and TypeScript contracts now separate `host` from `route`.
- Rust validates Claude/native, Claude/Ollama, Codex/native, and Pi/Ollama before PTY creation.
- Pi command construction uses `ollama`, discrete `launch`, `pi`, `--model`, and model arguments with workspace `cwd`.
- Existing in-memory session/tab structures were migrated to `host` and `route` and compilation succeeds.
- Documentation records the currently implemented support matrix and states that installation/upgrades are not LMBrain-owned.

### Not satisfied or not evidenced

- The normal UI path does not successfully load/select a Pi model.
- Pi does not receive `lmbrain-mcp` tools and no `.pi/mcp.json` registration exists.
- Missing Pi/Ollama/MCP prerequisites are not preflighted or reported before launch.
- An arbitrary non-empty model is accepted by the backend without discovered/tool-capability validation.
- Required backend and frontend behavior tests were not added; existing tests only migrate fixture fields.
- Runtime/session lifecycle, local/cloud model, MCP, and regression criteria remain intentionally unverified under the active operator constraint.
- Documentation, troubleshooting, generated-file ownership, and rollback requirements are incomplete because the MCP integration is absent.
- Acceptance criteria remain unchecked and implementation evidence is empty.

## Code observations

The cleanest part of the change is moving `build_command` before PTY allocation and validating the compatibility matrix first. This avoids allocating terminal resources for an invalid combination. Keeping arguments separate also avoids shell interpolation.

The UI representation is directionally correct, but host selection and route-dependent loading are coordinated through independent state updates without a single validated selection transition. The Pi bug below is one consequence; focused interaction tests are required before this pattern is reliable.

## Tests and verification

Executed during review, without starting/stopping LMBrain, Ollama, Pi, Claude, Codex, or any managed session:

- `git diff --check` - pass.
- `cargo check --workspace` - pass.
- `pnpm build` - pass.
- Vite reports the existing large-chunk warning; the main JS chunk is approximately 802 kB after minification.

Not executed because the operator-approved constraint explicitly limits this handoff to compilation checks:

- `cargo test` and targeted Rust tests;
- `pnpm test` and `pnpm lint`;
- Tauri application launch;
- PTY, Ollama local/cloud, Pi, MCP, process termination, and regression smoke tests.

## Production quality and documentation compliance

The current diff is a partial implementation, not a production-complete result under [[QUALITY]]. The code compiles, but core acceptance behavior and verification evidence are missing. The documentation honestly states that Pi lacks controlled-mutation parity, which is preferable to a misleading claim, but it also proves the approved scope has not been delivered.

The worktree changes are limited to source, tests, and delegated host/session docs; no generated Pi/npm file or credential is tracked. No application/source changes were made by the reviewer.

## Findings

### [P1] Selecting Pi from the default native state never starts model discovery

`selectHost("pi")` changes the route to `ollama` but does not call `ensureModelsLoaded()`. Because React state updates are asynchronous, `openModal()` has already run while the route was still the default `native`, so it also did not load models. The model list remains empty and Start stays disabled indefinitely unless the operator switches through another Ollama path or manually presses refresh.

- `src/components/Sessions/SessionsView.tsx:56`
- `src/components/Sessions/SessionsView.tsx:58`
- `src/components/Sessions/SessionsView.tsx:63`

Required remediation:

1. Make selecting any host/route combination that requires Ollama trigger discovery immediately, including Pi from the default state.
2. Prefer one transition helper that derives the valid route and its side effects together.
3. Add a frontend test starting from the default modal state, selecting Pi, resolving discovered models, and asserting the Pi/Ollama request payload.

### [P1] The approved LMBrain MCP integration is entirely absent

SPEC-029 states that a Pi terminal without `lmbrain-mcp` is not host parity. The diff contains no Pi registration module, no merge of `.pi/mcp.json`, no pinned/approved MCP-client integration, no refresh on workspace open or Pi launch, and no extension-readiness diagnostic. The documentation explicitly says Pi is not integrated yet.

- `.lmbrain/specs/ready/SPEC-029-add-pi-agent-sessions-through-ollama.md` acceptance criteria
- `docs/agent-hosts.md:70`
- `src-tauri/src/commands/mod.rs` (no Pi registration module)
- `src-tauri/src/lib.rs:421` (only Claude registration refresh)

Required remediation:

1. Resolve and document the approved MCP-client direction and exact pin, or create/complete the first-party prerequisite agreed by the operator.
2. Implement atomic, idempotent, preserving project-level Pi MCP registration with the resolved absolute `lmbrain-mcp` command.
3. Refresh registration at workspace open and immediately before Pi launch.
4. Surface unavailable MCP integration as a blocking or explicitly degraded readiness state, as specified.
5. Add config merge, idempotence, path, and failure tests.

### [P1] Launch relies on Ollama's side-effecting Pi bootstrap instead of preflighting prerequisites

The Pi branch immediately runs `ollama launch pi --model ...`. There is no deterministic check for the Ollama executable/daemon, Pi availability, or the required MCP extension. Official Ollama behavior can install/configure Pi as part of this command, which the spec explicitly says LMBrain must not rely on as an implicit feature. Failures arrive only after PTY spawn and may present an installer/configuration prompt rather than an actionable LMBrain error.

- `src-tauri/src/commands/sessions.rs:245`
- `src-tauri/src/commands/sessions.rs:252`

Required remediation:

1. Add non-mutating readiness checks that do not start/stop services or install anything.
2. Return actionable errors before PTY allocation for missing executables, unreachable daemon, and missing MCP prerequisite.
3. Keep setup/install actions operator-controlled and outside automatic session start.

### [P2] Required tests were not implemented

The Rust tests only adapt the existing label fixtures and do not exercise Pi labels, the compatibility matrix, command arguments, invalid routes, or validation failures. The React tests only rename fixture fields and do not cover Agent/Connection selection, model discovery, disabled states, request payloads, or error rendering. This omission allowed the Pi model-loading regression to compile unnoticed.

- `src-tauri/src/commands/sessions.rs:595`
- `src/__tests__/SessionsView.test.tsx:64`

Required remediation: add the focused Rust and frontend tests listed by SPEC-029. They do not need to be executed during the protected production window, but they must be present and later run before acceptance.

### [P2] The spec lifecycle and implementation evidence were skipped

SPEC-029 remains in `ready`; the implementer did not run `spec_start` or `spec_submit`. Its acceptance criteria are unchecked and its implementation evidence is empty. The Project Lead cannot perform the reserved `ready -> working` transition on the implementer's behalf.

- `.lmbrain/specs/ready/SPEC-029-add-pi-agent-sessions-through-ollama.md`

Required remediation:

1. The implementer must restore the controlled lifecycle using the supported MCP verbs; do not move files or edit managed frontmatter manually.
2. Record changed files, actual checks, deferred checks, deviations, and handoff status.
3. Check criteria only when evidence supports them and submit the spec to `review` when remediation is complete.

### [P2] Agent-host documentation contradicts itself about Pi MCP capability

The Pi section correctly says Pi lacks controlled mutation parity, but the same document later says all supported agent hosts can use the LMBrain context-pack MCP tools. With Pi presented as supported, this is internally inconsistent.

- `docs/agent-hosts.md:70`
- `docs/agent-hosts.md:82`

Required remediation: either deliver Pi MCP parity as approved or qualify the later statement so it accurately describes the capability matrix during any explicitly approved degraded phase.

## Required follow-up

Hand this same review-state work back to the implementation specialist. Fix the three P1 findings first, then add the missing automated coverage, reconcile documentation, and complete the controlled spec lifecycle/evidence. Respect the active no-start/no-stop constraint; runtime and test execution remains deferred until the operator provides a safe window.

### Project Lead corrective takeover (2026-07-10)

The operator explicitly authorized the Project Lead to take over the remediation after review. The operator separately approved `pi-mcp-extension@1.5.0` as an exact pinned, manually installed prerequisite. LMBrain may generate and validate project-scoped Pi MCP configuration but must not install or update Pi, Ollama, models, or Pi packages.

Corrective scope:

- fix Pi model discovery from the default modal state;
- add non-mutating Pi/Ollama/MCP readiness checks before PTY allocation;
- add atomic, preserving `.pi/mcp.json` registration for `lmbrain-mcp`;
- add focused Rust and frontend regression tests;
- reconcile agent-host/session documentation and implementation evidence.

Verification plan for the protected production window:

- inspect diffs and run `git diff --check`;
- run `cargo check --workspace` and `pnpm build` only;
- do not run test suites, LMBrain/Tauri, Ollama, Pi, Claude, Codex, or session lifecycle commands;
- leave runtime and automated-test execution explicitly pending for a later operator-approved safe window.

## Final decision

Changes requested. Do not merge, release, or mark SPEC-029 done.

### Corrective takeover verification update (2026-07-10)

The Project Lead implemented the approved bounded remediation after the
operator authorized takeover and exact dependency pin. Source inspection and
compile-only verification confirm that the original P1 code findings are
addressed:

- Pi selection now triggers Ollama model discovery from the default modal state.
- `.pi/mcp.json` registration and exact `pi-mcp-extension@1.5.0` readiness checks
  are present; LMBrain performs no install/update operation.
- Pi/Ollama/model/MCP preflight occurs before PTY allocation and returns
  actionable errors.
- Focused Rust and React regression tests were added.
- Documentation and accepted [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]]
  now describe the capability and dependency contract consistently.

Verified without starting/stopping application or agent processes:

- `cargo check --workspace` - pass.
- `cargo check --workspace --tests` - pass.
- `cargo build -p lmbrain-mcp` - pass (compilation only).
- `pnpm build` - pass with the existing large-chunk warning.
- `git diff --check` - pass.

This review remains `changes-requested`, not accepted: automated test execution,
lint, real Pi/MCP/local/cloud behavior, existing-host regressions, and process
lifecycle checks are deferred by the active production-instance constraint.
The unchecked runtime/evidence criteria in SPEC-029 must pass in a later safe
window before acceptance, merge/release recommendation, or `done`.
