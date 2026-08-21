---
id: SPEC-041
title: "Fix Windows installer gate for the OpenCode resolver test"
status: review
kind: bugfix
priority: high
area: desktop-ci
milestone: M-03
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-12
updated: 2026-07-12
tags: [windows, github-actions, rust, opencode, installer]
activity:
  - date: 2026-07-16
    action: "corrected duplicate historical ID SPEC-036 -> SPEC-041 on explicit operator authority"
  - date: 2026-07-12
    action: "transitioned backlog -> ready"
  - date: 2026-07-12
    action: "transitioned ready -> working"
  - date: 2026-07-12
    action: "transitioned working -> review"
---
# Fix Windows installer gate for the OpenCode resolver test

## Objective

Restore the Windows installer pipeline by replacing the environment-dependent OpenCode resolver test with deterministic coverage of the Windows npm-shim resolution behavior, without weakening the production resolver or skipping the Windows assertion.

## Context

GitHub Actions run [29173384006](https://github.com/fathorMB/LMBrain/actions/runs/29173384006) failed in the `Windows installer` job during `Run Rust tests`. The installer build and asset upload were skipped as downstream steps.

The only failing test was `commands::sessions::tests::resolves_native_opencode_binary_behind_windows_npm_shim` at `src-tauri/src/commands/sessions.rs:942`. It called `resolve_opencode_command()` against the runner's real `PATH` and panicked with `OpenCode is not installed`. The workflow does not install OpenCode. Linux passed because the test returns early outside Windows.

This test was introduced by commit `c43342245f4c36f884e23b9960a509d58f751a1d`. The production behavior being checked is valid, but the test currently verifies developer/runner provisioning rather than resolver logic.

## Scope

### Included

- Refactor the OpenCode command-resolution logic only as needed to accept an explicit resolved shim or search context in tests while preserving the existing production entry point.
- Build an isolated temporary Windows-like npm layout containing an `opencode.cmd` shim and a fake native `opencode.exe` at the package path expected by the resolver.
- Cover the architecture-specific package path and the x64 baseline fallback where applicable.
- Cover the fallback behavior when a shim exists but no native package binary is present.
- Keep real OpenCode installation checks out of the automated unit-test gate.
- Re-run the Windows Rust tests and the installer workflow.

### Excluded

- Installing OpenCode in GitHub Actions solely to satisfy this unit test.
- Ignoring, conditionally skipping, or deleting the Windows resolver assertion.
- Changing supported OpenCode routes, provider configuration, installer contents, or release version.
- Broad refactoring of session launching or harness management.

## Existing-project analysis

- `command_on_path` scans the process `PATH` and returns the first matching executable or Windows shim.
- `resolve_opencode_command` converts an npm `.cmd`/`.bat` shim into the native executable path under `node_modules/opencode-ai/node_modules/<platform-package>/bin/opencode.exe`, otherwise returning the resolved command unchanged.
- The failing test invokes the production zero-argument resolver directly, so its result depends on software preinstalled on the host.
- The release workflow correctly gates installer creation on Rust tests; bypassing that gate would reduce release quality.

## Technical proposal

Extract a small pure/path-driven helper from `resolve_opencode_command`, for example one that receives the already-resolved OpenCode command and the target architecture/package preference. Keep `resolve_opencode_command()` as the production wrapper that calls `command_on_path("opencode")` and delegates to the helper.

Unit tests should create the required tree under `tempfile::TempDir` and pass the fake shim path directly. They must not mutate the process-wide `PATH`, because Rust tests can run concurrently and environment mutation would introduce another source of flakiness. A fake file is sufficient because the resolver checks `is_file`; no external process should be launched.

## Files and areas involved

- `src-tauri/src/commands/sessions.rs`
- `.github/workflows/build-installers.yml` only if verification reveals a separate workflow defect; no workflow change is expected for the diagnosed failure.
- Relevant session or testing documentation only if the resolver contract changes.

## Acceptance criteria

- [x] Windows unit tests do not require OpenCode to be installed or present on the runner `PATH`.
- [x] A deterministic test proves that an npm `.cmd` or `.bat` shim resolves to the expected native `opencode.exe` when the architecture-specific package exists.
- [x] Deterministic coverage proves the intended baseline-package fallback and the no-native-binary fallback behavior.
- [x] The production `resolve_opencode_command()` still searches the real `PATH` and preserves its current runtime behavior and actionable preflight error.
- [x] Tests do not mutate process-global `PATH` or depend on an operator-installed harness.
- [x] The complete Rust workspace test suite passes on Windows.
- [ ] The `Windows installer` job reaches and passes `Build installers` and uploads the expected release assets on a rerun/new run.
- [ ] Linux installer and existing frontend quality gates remain green.

## Implementation plan

1. Extract a path-driven helper while retaining the production wrapper.
2. Replace the host-dependent test with temporary-filesystem fixtures for shim, platform binary, baseline fallback, and missing-native fallback cases.
3. Run formatting, linting, targeted tests, and the full Rust workspace suite locally where supported.
4. Push through the normal release/CI path and verify both installer jobs and uploaded artifacts.
5. Record exact commands, run URLs, and any limitations in implementation evidence.

## Required verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Targeted Rust test(s) for the OpenCode resolver.
- `cargo test --workspace` on Windows.
- Existing frontend lint/test gate if any non-Rust file changes.
- A GitHub Actions installer run showing successful Windows and Linux jobs, installer build, and asset upload.

## Production quality and documentation

- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Avoid using `std::env::set_var` for `PATH` in a parallel test suite; this would replace one environmental dependency with a race.
- Do not install an external harness in CI unless a separate integration-test requirement is approved; it adds network/provisioning risk and does not improve unit coverage of path resolution.
- GitHub-hosted Windows runner image changes are not the root cause: the test contract itself incorrectly assumes external software availability.

## Project Lead corrective takeover

- **Authorized by:** human operator on 2026-07-12 with the explicit instruction to proceed directly with implementation.
- **Rationale:** this is a narrow, technically understood correction to the single environment-dependent acceptance failure blocking the Windows installer.
- **Bounded scope:** resolver extraction and deterministic Rust regression tests in `src-tauri/src/commands/sessions.rs`; no product, architecture, security, integration, workflow, or release-version change.
- **Verification plan:** formatting, targeted resolver tests, clippy, full workspace tests, then a separate diff/acceptance-criteria review. The remote Windows installer run remains required after the correction is committed and pushed.
- **Release-trigger extension authorized 2026-07-12:** the operator explicitly requested the patch-version increment needed by the existing release gate. Scope expands only to aligning the app manifests/lockfile at `2.7.3`, running the version gate, and committing/pushing that version-only change; no workflow or product behavior change is authorized.

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

### Changes made

- Extracted `resolve_windows_opencode_command`, a deterministic path-driven helper used by the Windows production resolver after the existing real-`PATH` lookup.
- Replaced the host-provisioning test with isolated temporary-filesystem fixtures.
- Added regression coverage for the architecture package, x64 baseline fallback, and missing-native fallback to the original npm shim.

### Files changed

- `src-tauri/src/commands/sessions.rs`

### Verification performed

- `rustfmt --edition 2021 --check src-tauri/src/commands/sessions.rs` — passed.
- Three targeted resolver tests — passed.
- `cargo clippy -p lmbrain --all-targets --no-deps -- -D warnings` — passed.
- `cargo test --workspace` — passed, including 58 app tests (55 passed, 3 intentionally ignored) and all core/MCP/integration tests.
- `git diff --check` — passed.
- `cargo fmt --all -- --check` — blocked by pre-existing formatting drift in unrelated files including `lmbrain-core/src/frontmatter.rs` and several other `src-tauri` modules; the changed file passes independently.
- `cargo clippy --workspace --all-targets -- -D warnings` — blocked by pre-existing `lmbrain-core/src/context.rs` warnings (`collapsible_if` and `too_many_arguments`); the changed crate passes independently.

### Deviations from the specification

- No source-scope deviation. Remote installer build/upload verification remains pending because the local change has not been committed or pushed.

### Handoff status

- [x] Ready for Project Lead review
