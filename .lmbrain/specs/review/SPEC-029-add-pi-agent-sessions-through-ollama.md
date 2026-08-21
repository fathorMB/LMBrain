---
id: SPEC-029
# Note: Quote the title if it contains a colon
title: "Add Pi agent sessions through Ollama"
status: review
kind: feature
priority: high
area: sessions
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-006, ADR-007]
links: []
created: 2026-07-10
updated: 2026-07-10
tags: [sessions, pi, ollama, agent-hosts, mcp]
activity:
  - date: 2026-07-10
    action: "created"
activity:
  - date: 2026-07-10
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-10
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-10
    action: "transitioned working -> review"
---
# Add Pi agent sessions through Ollama

## Objective

Allow an operator to start an interactive Pi coding-agent session from LMBrain, using the local Ollama daemon as the model gateway for both local and Ollama cloud-backed, tool-capable models, without regressing Claude Code or Codex sessions.

The feature must preserve LMBrain's user-controlled launch model and controlled-artifact mutation workflow. A Pi terminal that can edit source but cannot access the `lmbrain-mcp` verbs is not considered host parity and must not be presented as fully integrated.

## Context

LMBrain currently exposes three hard-coded session modes:

- `claude` starts the native Claude Code CLI;
- `ollama` starts `ollama launch claude --model <model>`;
- `codex` starts the resolved native Codex CLI.

This means the current `SessionMode` conflates two separate concepts: the coding-agent host and the model transport/runtime. Adding Pi as another one-off mode would work temporarily but would make future combinations (`pi` native, Codex through Ollama, another host through Ollama) increasingly conditional and error-prone.

Current official behavior researched on 2026-07-10:

- Ollama documents `ollama launch pi --model <model>` as its supported Pi integration and states that the quick setup can install Pi and configure its Ollama provider: <https://docs.ollama.com/integrations/pi>.
- Pi supports Ollama and other OpenAI-compatible endpoints through `models.json`, using `http://localhost:11434/v1`, `api: openai-completions`, and a dummy API key: <https://pi.dev/docs/latest/models>.
- Pi reads project `AGENTS.md`, so LMBrain's existing managed root pointer remains the instruction discovery mechanism: <https://pi.dev/docs/latest/usage>.
- Pi intentionally does not ship an MCP client in core. MCP requires an extension: <https://pi.dev/docs/latest/usage>.
- The current candidate `pi-mcp-extension` supports project `.pi/mcp.json`, stdio servers, lifecycle management, cancellation, and MCP tool bridging, but is third-party executable code and therefore requires an explicit dependency/security decision before this spec can become `ready`: <https://pi.dev/packages/pi-mcp-extension>.

## Scope

### Included

- Add Pi as a selectable coding-agent host in the Sessions new-session flow.
- Support Pi through Ollama only in this first release.
- Reuse the existing Ollama discovery path and tool-capability filter for local and cloud-backed models.
- Refactor the session request/info contract so agent host and launch route are represented separately rather than extending the overloaded `ollama` mode.
- Launch Pi in a PTY with the workspace root as `cwd`, the selected model, and the same attach/output/resize/exit/kill lifecycle guarantees as existing sessions.
- Resolve prerequisites deterministically and return actionable errors for missing Ollama, unavailable daemon, missing Pi, unsupported model, invalid configuration, or failed process spawn.
- Preserve project instruction discovery through the existing managed root `AGENTS.md` block.
- Provide Pi access to `lmbrain-mcp` controlled mutation tools through an explicitly approved, pinned, project-scoped MCP-client integration; merge generated Pi MCP configuration without deleting unrelated user configuration.
- Keep all agent starts operator-initiated; do not auto-start Pi, Ollama, models, or agents.
- Update automated tests and the relevant technical documentation (`docs/sessions.md`, `docs/agent-hosts.md`, `docs/development.md` if generated paths change, and release notes/changelog when release policy requires it).

### Excluded

- Native/direct-provider Pi sessions that bypass Ollama.
- Codex-through-Ollama support.
- Automatic installation or silent upgrade of Pi, Ollama, models, or third-party Pi packages.
- Global mutation of `~/.pi/agent` without a separate explicit operator action.
- SDK/RPC embedding of Pi or replacement of the existing PTY terminal surface.
- Autonomous agent spawning, background agents, session persistence across LMBrain restarts, or changes to Pi's own permission/security model.
- Bundling Pi itself into the LMBrain installer.

## Existing-project analysis

### Backend

- `src-tauri/src/models/session.rs` defines the serialized session contract. `SessionMode::{Claude,Ollama,Codex}` currently makes `Ollama` synonymous with Claude-via-Ollama.
- `src-tauri/src/commands/sessions.rs` owns PTY lifecycle, command construction, default labels, Codex resolution, and Ollama model discovery. The generic PTY manager is reusable; command construction and labeling need restructuring.
- `src-tauri/src/lib.rs::session_start` refreshes Claude MCP registration only for native Claude. Host-specific pre-launch preparation is currently embedded in the Tauri command.
- `src-tauri/src/commands/mcp_registration.rs` writes the Claude-compatible root `.mcp.json`; `codex_registration.rs` owns Codex-specific configuration and trust. Pi needs an equivalent host adapter only after its MCP client dependency is approved.

### Frontend

- `src/types/index.ts`, `src/lib/commands.ts`, and `src/context/WorkspaceContext.tsx` mirror the overloaded session contract and derive host-specific fallback labels.
- `src/components/Sessions/SessionsView.tsx` presents flat `Claude / Ollama / Codex` buttons. The model picker only appears for `mode === "ollama"`, so it cannot express `Pi + Ollama` without more conditionals.
- `src/__tests__/SessionsView.test.tsx` covers tab behavior and modal layering, but not mode/route combinations, model selection, request payloads, or error states.

### Operational constraints

- Sessions are in-memory only, so the frontend/backend request schema can be changed atomically without a persisted-data migration.
- `.lmbrain/`, `.mcp.json`, `.codex/`, `.claude/`, and root `AGENTS.md` are generated/ignored today. A generated `.pi/mcp.json` must not cause LMBrain to ignore or overwrite unrelated user-owned `.pi` resources.
- [[ADR-006-session-process-execution]] remains the PTY lifecycle basis. [[ADR-007-codex-agent-host-support]] establishes the pattern of a peer host with scoped configuration and preservation of unrelated settings.

## Technical proposal

### 1. Separate host from launch route

Replace the overloaded mode with an explicit, validated combination at the Rust/TypeScript boundary:

- `AgentHost`: `claude | codex | pi`;
- `ModelRoute`: `native | ollama`;
- `SessionStartRequest`: `host`, `route`, optional `model`, optional `label`, and host-specific executable override only where supported;
- `SessionInfo`: `host`, `route`, `model`, status, and exit code.

For this release, validate a closed compatibility matrix:

| Host | Native | Ollama |
| --- | --- | --- |
| Claude | supported | supported |
| Codex | supported | rejected |
| Pi | rejected | supported |

Reject invalid combinations in Rust before opening a PTY. Keep command construction in a small pure/testable launch-spec function that returns executable, arguments, environment overrides, and label inputs before `portable-pty` is invoked.

### 2. Pi launch path

Initial implementation should use the official Ollama integration command:

`ollama launch pi --model <selected-model>`

Do not rely on Ollama's ability to auto-install Pi as an implicit application feature. Preflight Pi/Ollama readiness and fail with an actionable message directing the operator to the official setup flow. Do not run `ollama launch pi --config`, `pi install`, or package updates silently.

Pass every argument as a discrete `CommandBuilder` argument; never interpolate a shell command. Preserve workspace `cwd` and current PTY lifecycle behavior. A custom label defaults to `Pi via <model>` when absent.

### 3. Model discovery and validation

Reuse `list_ollama_models()` for both Claude-via-Ollama and Pi-via-Ollama. Continue filtering API results to models advertising `tools`; retain the CLI fallback behavior, while documenting that CLI fallback cannot verify tool capability and is therefore lower-confidence.

At session start, trim and require a model for every Ollama route. Treat model names as opaque argument values. Do not infer local/cloud behavior from the name for execution; `cloud` remains display metadata only.

### 4. Pi and LMBrain controlled mutations

Pi core has no MCP client, so full LMBrain integration requires a reviewed Pi extension. Recommended first implementation:

1. Security-review and approve an exact pinned release of `pi-mcp-extension` (candidate at research time: `1.5.0`); record the dependency and update policy in a proposed ADR before moving this spec to `ready`.
2. Require an operator-controlled project-local installation (`pi install -l npm:pi-mcp-extension@<approved-version>`) or an equivalent pre-provisioned installation. LMBrain must not install it silently.
3. Add a Pi registration module that merges only `mcpServers.lmbrain` into `.pi/mcp.json`, using stdio, the resolved absolute `lmbrain-mcp` command, `args: ["--root", <workspace>]`, and eager lifecycle. Preserve all unrelated keys and servers; write atomically and idempotently.
4. Refresh the Pi MCP registration on workspace open and immediately before Pi launch, mirroring the repaired Claude behavior in [[SPEC-022-fix-in-app-claude-mcp-resolution]].
5. Detect/report an unavailable MCP extension as a failed readiness check or clearly degraded integration; do not label the session fully LMBrain-enabled while controlled mutation tools are absent.

If the third-party extension is not approved, split a first-party Pi MCP adapter into a separate prerequisite spec. Do not implement an ad-hoc partial MCP client inside the session launcher.

### 5. UI and diagnostics

Change the modal from flat launch modes to two compact choices:

- agent: Claude, Codex, Pi;
- connection: Native or Ollama, showing only valid options for the selected agent.

Show the model picker whenever the selected route is Ollama. Preserve the selected Ollama model when switching between Claude and Pi. Disable Start while required data is loading or missing. Surface backend error text in the modal and keep it open after failure.

Session tabs should identify the host, while the default label identifies route/model (`Pi via qwen…`). The Settings view may expose a Pi executable override only if runtime evidence shows PATH resolution is unreliable; do not add speculative settings.

## Files and areas involved

- `src-tauri/src/models/session.rs`
- `src-tauri/src/commands/sessions.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/pi_registration.rs` (new, conditional on approved MCP direction)
- `src-tauri/src/lib.rs`
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/context/WorkspaceContext.tsx`
- `src/components/Sessions/SessionsView.tsx`
- `src/__tests__/SessionsView.test.tsx`
- targeted Rust tests in the session/Pi registration modules and integration tests where useful
- `.gitignore` only for the exact generated Pi configuration paths; do not ignore all user-owned `.pi/` content without justification
- `docs/sessions.md`
- `docs/agent-hosts.md`
- `docs/development.md` if generated local state changes
- `.lmbrain/CHANGELOG.md` or release documentation as required by the repository release policy

## Acceptance criteria

- [x] The serialized frontend/backend contract represents agent host and launch route separately and rejects unsupported combinations before PTY creation.
- [ ] Existing Claude-native, Claude-via-Ollama, and Codex-native sessions retain their current command, `cwd`, PTY, labeling, and lifecycle behavior.
- [x] The Sessions modal allows `Pi + Ollama`, requires a tool-capable discovered model, and does not offer `Pi + Native` in this release.
- [x] Starting `Pi + Ollama` constructs `ollama launch pi --model <model>` with discrete arguments and workspace-root `cwd`.
- [x] Local and cloud-backed Ollama models remain selectable without changing execution behavior based on model-name heuristics.
- [x] Missing/invalid model, unsupported host/route pair, missing executable, unreachable Ollama, failed process spawn, and unavailable Pi/MCP prerequisites produce actionable, non-panicking errors visible in the modal.
- [ ] Closing a Pi tab terminates its process; app/workspace shutdown still terminates all managed sessions; exited Pi sessions report their exit code.
- [x] Pi receives the existing root `AGENTS.md` LMBrain instruction pointer.
- [ ] An explicitly approved and pinned Pi MCP client exposes the repository-scoped `lmbrain-mcp` verbs in Pi; generated `.pi/mcp.json` registration is atomic, idempotent, path-correct, and preserves unrelated configuration.
- [x] LMBrain never silently installs or upgrades Pi, Ollama, a model, or a Pi extension/package.
- [x] Generated Pi files do not cause unrelated user-owned `.pi` settings/extensions/skills to be overwritten or broadly ignored.
- [ ] Rust unit tests cover the compatibility matrix, launch-spec arguments, validation, labels, config merge/idempotence, and platform path behavior; frontend tests cover Pi selection, model loading, payload construction, disabled states, and visible launch errors.
- [ ] `cargo test`, `pnpm test`, `pnpm lint`, and `pnpm build` pass, with any pre-existing warning explicitly identified.
- [ ] A real Windows smoke test demonstrates a Pi PTY using one local Ollama model and one Ollama cloud-backed model when available; evidence records exact versions and any unavailable optional case.
- [ ] A real Pi smoke test verifies `lmbrain-mcp` discovery and at least one read-only tool call plus one controlled test mutation in a disposable fixture/workspace; no production project artifact is mutated for the smoke test.
- [x] `docs/sessions.md` and `docs/agent-hosts.md` accurately describe prerequisites, supported matrix, configuration ownership, security implications, troubleshooting, and removal/rollback.

## Implementation plan

1. Revalidate current Pi and Ollama CLI behavior on the implementation date; record versions and any drift from the researched commands.
2. Resolve the MCP dependency gate: audit/approve a pinned Pi MCP extension and create the required proposed ADR, or split and complete a first-party adapter prerequisite before claiming host parity.
3. Introduce the host/route domain model in Rust and TypeScript, with a closed compatibility validator and pure launch-spec tests.
4. Adapt Claude and Codex behavior to the new model without functional changes; run regression tests before adding Pi.
5. Add Pi-via-Ollama command construction, prerequisite diagnostics, labels, and PTY lifecycle coverage.
6. Add safe Pi MCP project registration and exact-path ignore/documentation rules; verify preservation and idempotence against existing config fixtures.
7. Refactor the Sessions modal to agent + connection + model, add accessibility labels, loading/error states, and request-payload tests.
8. Run automated gates and manual Windows PTY/MCP smoke tests using disposable fixtures.
9. Update delegated documentation and implementation evidence, then submit the spec to `review` with exact commands/results.

## Required verification

### Operator-approved constraint for the current implementation handoff (2026-07-10)

Another LMBrain instance is running production work. During this implementation handoff, do not execute commands that start, stop, restart, attach to, or otherwise control LMBrain, Ollama, Pi, OpenCode, Claude, Codex, or any managed session/process. Stop verification at compilation checks: `cargo check --workspace` and `pnpm build`. Do not run `cargo test`, `pnpm test`, the Tauri app, or any manual PTY/MCP smoke test in this window.

This is a verification deferral, not acceptance evidence. Record exactly which checks were skipped and why. The real PTY, Ollama model, MCP discovery/mutation, process termination, and regression smoke tests remain required before the Project Lead may recommend the spec as `done`; execute them later only in an operator-approved safe window.

- Rust targeted tests for session launch specifications and Pi config registration.
- Full `cargo test` workspace suite.
- Frontend interaction tests covering all supported combinations and failure states.
- `pnpm test`, `pnpm lint`, and `pnpm build`.
- Manual PTY checks for Claude native, Claude/Ollama, Codex native, and Pi/Ollama.
- Manual Pi/Ollama checks with a tool-capable local model and, when the operator has access, a cloud-backed model.
- Disposable-workspace MCP verification through Pi, including tool discovery, failure diagnostics, and clean server/process shutdown.
- `git diff --check` and repository status review to ensure generated Pi/npm files and credentials are not accidentally tracked.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- **Resolved dependency decision:** The operator approved exact project-local pin `npm:pi-mcp-extension@1.5.0`; see accepted [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]]. LMBrain verifies but never installs or updates it.
- **Third-party code execution:** Pi extensions run with the user's full permissions. Project-local auto-install after trust is convenient but is still a network/package-install side effect; LMBrain must disclose it and must not trigger it silently.
- **Ollama quick-setup side effects:** `ollama launch pi` may install/configure Pi and web tools. The launcher must not treat those mutations as an invisible preflight. Missing prerequisites should fail safely with setup guidance.
- **Host/route refactor regression:** The contract refactor touches every existing launch path. Preserve behavior with pure launch-spec tests and land the refactor before Pi-specific UI logic.
- **CLI/API drift:** Pi and Ollama are fast-moving. Revalidate command flags, package ownership/name, project trust, and configuration formats at implementation time using current official docs.
- **Tool-capability fallback:** `ollama list` fallback does not expose the same capability metadata as `/api/tags`. Decide whether to show unverified models with a warning or reject them for Pi; do not silently claim tool support.
- **Generated `.pi` ownership:** Pi project settings may contain user-authored extensions, skills, prompts, themes, npm packages, or credentials. Only the narrow MCP file/entry owned by LMBrain may be merged or ignored.
- **Cloud routing/privacy:** A `:cloud` model selected through the local daemon is still remotely processed. Keep the existing cloud badge and document that “through local Ollama” does not imply local inference.
- **Version support floor:** Record the minimum verified Pi and Ollama versions in implementation evidence and docs; avoid speculative semver checks unless behavior requires them.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Corrective takeover evidence (supersedes the original evidence below)

#### Changes made

- Refactored the session boundary to separate agent `host` from model `route`.
- Added `Pi + Ollama` as discrete PTY arguments: `ollama launch pi --model <model>`.
- Fixed default Pi selection so it immediately discovers Ollama models and keeps launch errors visible.
- Added pre-PTY validation for route compatibility, `ollama`/`pi` executables,
  live Ollama API/model tool capability, and exact manual package pin
  `npm:pi-mcp-extension@1.5.0`.
- Added preserving project `.pi/mcp.json` registration for `lmbrain-mcp`, eager
  stdio lifecycle, recoverable replacement, and narrow ignore rules.
- Added `PI_OFFLINE=1`, `PI_SKIP_VERSION_CHECK=1`, and `PI_TELEMETRY=0` to Pi launch.
- Added focused Rust and React regression tests and updated delegated docs.
- Recorded and accepted [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]].

#### Files changed by the complete implementation/remediation

- `.gitignore`
- `src-tauri/src/models/session.rs`
- `src-tauri/src/commands/sessions.rs`
- `src-tauri/src/commands/pi_registration.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/context/WorkspaceContext.tsx`
- `src/components/Sessions/SessionsView.tsx`
- `src/__tests__/SessionsView.test.tsx`
- `src/__tests__/sessionReducer.test.ts`
- `docs/sessions.md`, `docs/agent-hosts.md`, and `docs/development.md`
- `.lmbrain/CHANGELOG.md`, `.lmbrain/STATUS.md`, [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]], and [[REVIEW-031-review-spec-029-pi-agent-sessions-through-ollama]]

#### Verification performed during takeover

- `cargo check --workspace` - passed.
- `cargo check --workspace --tests` - passed; Rust test sources compile but were not executed.
- `cargo build -p lmbrain-mcp` - passed; compilation only.
- `pnpm build` - passed; existing Vite chunk-size warning remains.
- `git diff --check` - passed.
- TypeScript test-source compilation is blocked by a pre-existing
  `tsconfig.test.json` exclusion; exposing it revealed unrelated stale fixtures
  in `TaskboardView.test.tsx` and `WikiView.test.tsx`, so no scope-expanding fix was retained.
- Test execution, lint, PTY/runtime, Ollama/Pi, MCP, and process lifecycle checks
  were skipped under the operator-approved concurrent-production constraint.

#### Remaining verification limitation

No package, model, app, agent, daemon, or session was installed, started,
stopped, or updated. Automated tests, lint, real Pi MCP exposure, existing-host
regressions, process termination, and local/cloud runtime checks remain deferred
until an operator-approved safe window. These unchecked criteria prevent `done`.

### Original specialist evidence (superseded; retained for audit)

#### Original changes made

- Refactored the session boundary to use separate `host` and `route` values.
- Added the supported `Pi + Ollama` launch path as discrete PTY arguments:
  `ollama launch pi --model <model>`.
- Added pre-PTY compatibility validation and Pi/route-aware labels and UI.
- Preserved Ollama model discovery, workspace `cwd`, instruction discovery,
  and existing Claude/Codex lifecycle plumbing.
- Documented the supported matrix and the unresolved Pi MCP dependency gate.

#### Original files changed

- `src-tauri/src/models/session.rs`
- `src-tauri/src/commands/sessions.rs`
- `src-tauri/src/lib.rs`
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/context/WorkspaceContext.tsx`
- `src/components/Sessions/SessionsView.tsx`
- `src/__tests__/SessionsView.test.tsx`
- `src/__tests__/sessionReducer.test.ts`
- `docs/sessions.md`
- `docs/agent-hosts.md`

#### Original verification performed

- `cargo check --workspace` — passed.
- `pnpm build` — passed; Vite emitted only the existing chunk-size warning.
- `git diff --check` — passed.
- Tests, lint, runtime PTY checks, Ollama/Pi smoke checks, and MCP checks were
  intentionally skipped under the operator-approved concurrent-work constraint
  in Required verification.

#### Original deviations from the specification

- The approved/pinned Pi MCP client prerequisite was not implemented. No
  third-party extension was installed or executed without explicit approval;
  Pi controlled-mutation parity remains a prerequisite for acceptance.
- The spec remains `ready` because the repository-scoped LMBrain MCP mutation
  tools were unavailable in this session, so the required `spec_start` and
  `spec_submit` transitions could not be performed through the controlled
  mutation workflow.

#### Original handoff status
- [ ] Ready for Project Lead review (blocked on Pi MCP approval and deferred
  runtime/test verification)
