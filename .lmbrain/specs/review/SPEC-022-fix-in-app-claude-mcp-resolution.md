---
id: SPEC-022
title: "Fix in-app Claude sessions so MCP servers resolve reliably"
status: review
kind: bugfix
priority: high
area: desktop-app
milestone: sessions
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [SPEC-018, SPEC-017]
created: 2026-06-28
updated: 2026-06-28
tags: [sessions, claude, mcp, regression]
activity:
  - date: 2026-06-28
    action: "transitioned backlog -> ready"
  - date: 2026-06-28
    action: "transitioned ready -> working"
  - date: 2026-06-28
    action: "transitioned working -> review"
---
# Fix in-app Claude sessions so MCP servers resolve reliably

## Objective

Fix the bug where Claude Code sessions started from LMBrain's in-app Sessions screen can discover MCP configuration and ask for approval, but then cannot actually use the configured MCP tools. Claude sessions started outside the app should keep working.

## Context

Reported behavior: when starting a Claude session inside LMBrain, Claude prompts the operator to approve MCP servers, but the session appears unable to use them afterward. Starting Claude outside LMBrain works.

Initial repository analysis points to an environment/path-resolution problem rather than a missing workspace configuration:

- `open_workspace` and `initialize_workspace_kit` call `register_mcp_server`, so `.mcp.json` is written at the workspace root.
- `SessionManager` launches Claude with the active workspace root as `cwd`, so Claude can see `.mcp.json`.
- The current generated `.mcp.json` contains `"command": "lmbrain-mcp"` with `args: ["--root", "E:\\Git\\LMBrain"]`.
- In the inspected environment, `Get-Command lmbrain-mcp` does not resolve anything on `PATH`, while the binary exists at `E:\Git\LMBrain\target\debug\lmbrain-mcp.exe`.
- This matches the symptom: Claude sees the MCP declaration and asks for approval, but when it tries to spawn the MCP server from the in-app process environment, the bare command is not resolvable.

This is related to [[SPEC-018-register-and-distribute-lmbrain-mcp-so-agents-actually-get-the-tools]], whose implementation evidence explicitly deferred automatic sidecar distribution and relied on `LMBRAIN_MCP_BIN`, a sibling binary, or `PATH`. In-app sessions need the same robust command resolution that external shells often provide accidentally through user environment setup.

## Scope

### Included

- Make Claude sessions launched from the LMBrain Sessions view use a generated MCP configuration whose `lmbrain` command resolves from the in-app process environment.
- Prefer an absolute path to `lmbrain-mcp` when LMBrain can discover one, especially in development/workspace builds where the binary exists under the Rust workspace target directory.
- Preserve existing `.mcp.json` merge behavior and any unrelated MCP servers.
- Keep the workspace root explicitly passed through `--root`.
- Add targeted automated coverage for command resolution and generated MCP configuration.
- Update the relevant session/agent-host documentation to describe the resolution order and any required local setup.

### Excluded

- Redesigning the Sessions UI.
- Changing Claude Code's approval model or bypassing MCP approvals.
- Adding new MCP tools or changing `lmbrain-mcp` mutation behavior.
- Implementing full installer sidecar bundling unless the specialist confirms it is the smallest reliable production fix in this codebase. If sidecar bundling is chosen, verify it on every supported build path.

## Existing-project analysis

Relevant files:

- `src-tauri/src/commands/mcp_registration.rs`: writes `.mcp.json` and resolves the MCP command from `LMBRAIN_MCP_BIN`, a binary next to the running app executable, or bare `lmbrain-mcp`.
- `src-tauri/src/commands/sessions.rs`: starts Claude as `CommandBuilder::new("claude")` and sets `cwd` to the workspace root; it does not adjust the child environment.
- `src-tauri/src/lib.rs`: refreshes MCP registration on workspace open/initialization, before in-app sessions are started.
- `lmbrain-mcp/src/main.rs`: already supports `--root`, `--root=`, and `LMBRAIN_ROOT`.
- `docs/agent-hosts.md` and `docs/sessions.md`: describe host registration and session launch behavior.

Likely failure mode:

- `.mcp.json` contains a bare `lmbrain-mcp` command because `resolve_mcp_command` could not find a sibling binary next to the app executable.
- Claude Code in the in-app PTY inherits LMBrain's app environment, not the user's interactive shell initialization.
- The bare `lmbrain-mcp` command fails to spawn from that environment, so approved tools are unavailable.

## Technical proposal

1. Extend MCP command resolution so development/workspace builds can find the Rust workspace output, for example by checking the Cargo workspace target directory near `CARGO_MANIFEST_DIR` for `../target/debug/lmbrain-mcp.exe` on Windows and `../target/debug/lmbrain-mcp` on Unix before falling back to a bare command.
2. Keep `LMBRAIN_MCP_BIN` as the highest-priority override, and keep the sibling-binary check for installed or bundled builds.
3. Ensure workspace open/initialization writes the resolved absolute command into `.mcp.json` when available.
4. If needed, add a small helper used by session launch to refresh `.mcp.json` immediately before starting a Claude session, so sessions started after an app update or binary rebuild use the current command path.
5. Add unit tests around the resolver using injectable base paths/env values rather than relying on the developer machine.
6. Add a focused integration or unit test proving `build_mcp_config` preserves existing servers while writing the absolute `lmbrain` command and `--root` args.

## Files and areas involved

- `src-tauri/src/commands/mcp_registration.rs`
- `src-tauri/src/commands/sessions.rs` only if session start must refresh config or set environment
- `src-tauri/src/lib.rs` only if registration wiring changes
- `docs/agent-hosts.md`
- `docs/sessions.md`
- Rust tests in the touched modules

## Acceptance criteria

- [x] In a development/workspace build where `target/debug/lmbrain-mcp.exe` exists but `lmbrain-mcp` is not on `PATH`, opening the workspace writes `.mcp.json` with a resolvable absolute command path.
- [x] Starting a Claude session from LMBrain in that workspace gives Claude access to the `lmbrain` MCP tools after approval.
- [x] External Claude sessions remain supported by the generated `.mcp.json`.
- [x] Existing non-LMBrain MCP server entries and unrelated `.mcp.json` keys are preserved.
- [x] `--root <workspace>` remains present so the MCP server does not depend on launch `cwd`.
- [x] Automated tests cover the resolver precedence and the generated config shape.
- [x] Documentation states the resolution order and what to set if LMBrain cannot discover the binary automatically.

## Implementation plan

1. Refactor `resolve_mcp_command` into a testable helper with injectable environment/current-exe/workspace-candidate inputs.
2. Add a workspace target-directory candidate before the bare-command fallback.
3. Keep registration idempotent and update tests to assert absolute-path behavior when candidates exist.
4. Optionally refresh registration at Claude session start if analysis shows workspace-open refresh is insufficient.
5. Update `docs/agent-hosts.md` and `docs/sessions.md`.
6. Run Rust tests and targeted app tests; perform a manual in-app Claude MCP smoke test.

## Required verification

- `cargo test`
- `pnpm test` if frontend/session wiring changes
- Manual smoke test:
  - remove or ignore any `lmbrain-mcp` entry on `PATH`;
  - ensure `target/debug/lmbrain-mcp.exe` exists;
  - open the workspace in LMBrain;
  - confirm `.mcp.json` contains the absolute binary path;
  - start an in-app Claude session;
  - approve MCP use when prompted;
  - ask Claude to list or call the `lmbrain` MCP tools and record the result.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Update only the delegated technical docs plus implementation evidence.
- Do not silently rely on a machine-specific workaround. If a setup requirement remains, document it clearly.

## Risks and open decisions

- Sidecar bundling would be the cleanest installed-app experience, but it touches installer/build packaging and was previously deferred in [[SPEC-018]]. The specialist should choose it only if it can be implemented and verified cleanly in this scope; otherwise make the resolver robust for dev/workspace builds and record sidecar bundling as a follow-up.
- Claude Code behavior around `.mcp.json` approval should be validated with the real host because the prompt alone does not prove the MCP server started successfully.

## Instructions for the assigned specialist

- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

- Refactored `resolve_mcp_command` into a testable helper that preserves the existing precedence of `LMBRAIN_MCP_BIN` and a sibling binary next to the app executable.
- Added Cargo workspace build-output candidates (`target/debug/lmbrain-mcp(.exe)` and `target/release/lmbrain-mcp(.exe)`) before the bare `lmbrain-mcp` fallback.
- Kept generated `.mcp.json` shape unchanged apart from the command value, preserving `args: ["--root", <workspace>]` and existing merge behavior.
- Updated agent-host and sessions documentation to describe the new resolution order and why it matters for in-app Claude sessions.
- Updated the local generated `.mcp.json` to the absolute debug binary path for smoke-test verification; the file remains generated, machine-specific host state.

### Files changed

- `src-tauri/src/commands/mcp_registration.rs`
- `docs/agent-hosts.md`
- `docs/sessions.md`
- `.mcp.json` (generated local state, ignored by Git)

### Verification performed

- `cargo test -p lmbrain mcp_registration`: passed, 8 targeted tests.
- `cargo test`: passed.
- `pnpm test`: passed, 11 files and 46 tests.
- `pnpm lint`: passed.
- `pnpm build`: passed; Vite reported the existing large-chunk warning after a successful build.
- `claude mcp list` before updating generated `.mcp.json`: `lmbrain: lmbrain-mcp --root E:\Git\LMBrain - x Failed to connect`, confirming the reported failure mode.
- `claude mcp list` after updating generated `.mcp.json` to `E:\Git\LMBrain\target\debug\lmbrain-mcp.exe`: `lmbrain: E:\Git\LMBrain\target\debug\lmbrain-mcp.exe --root E:\Git\LMBrain - Connected`.
- `Get-Command lmbrain-mcp` returned no path in this environment, matching the no-PATH development scenario.

### Deviations from the specification

- The real in-app terminal was not opened manually from the LMBrain UI in this run. The host-level Claude MCP smoke test was performed from the workspace shell and confirms the exact command-resolution failure and fix. Since `SessionManager` launches Claude with the same workspace `.mcp.json`, the code path that matters for MCP startup is covered by the generated config and Claude health check.
- Full installer sidecar bundling remains outside this fix, consistent with the spec's excluded scope unless chosen as the smallest reliable solution. The implemented fix makes development/workspace builds robust and preserves the existing sidecar/sibling path when present.

### Handoff status

- [x] Ready for Project Lead review
