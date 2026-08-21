---
id: SPEC-047
# Note: Quote the title if it contains a colon
title: "LMBrain 3.0.1 tranche 4: persist repository GitHub PAT in native credential stores"
status: review
kind: feature
priority: medium
area: 
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-XXX
related_tasks: []
related_decisions: []
links: []
created: 2026-07-18
updated: 2026-07-18
tags: []
activity:
  - date: 2026-07-18
    action: "created"
activity:
  - date: 2026-07-18
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-18
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-18
    action: "transitioned working -> review"
---
# LMBrain 3.0.1 tranche 4: persist repository GitHub PAT in native credential stores

## Objective
Make Repository-page GitHub PAT saves durable across credential entry instances and application restarts on every packaged platform.

## Context
Operator testing showed that Save appeared to succeed but the page immediately returned to `NO TOKEN`. Keyring 3 has no default native features; `keyring = "3"` therefore selected its in-memory mock backend. Each fresh entry was empty, so the UI lost the apparent save on reload.

## Scope
### Included
- Enable Windows Credential Manager, Apple Keychain, and Linux Secret Service backends.
- Reject empty PAT values and trim surrounding whitespace.
- Verify save durability through a freshly opened credential entry before returning success.
- Distinguish a missing credential from credential-store read failures.
- Make deletion of an already-missing token idempotent.
- Add native Windows round-trip coverage using a temporary credential that is removed by the test.

### Excluded
- PAT validation against GitHub or changes to token permission requirements.
- Displaying, logging, or exporting stored secret contents.

## Existing-project analysis
The frontend correctly invoked `save_github_pat` and refreshed state. The backend's dependency configuration selected keyring's mock store, which accepts writes but does not persist them across separately constructed entries. Existing frontend mocks could not expose this platform integration defect.

## Technical proposal
Enable the keyring-native feature set recommended by keyring 3 for the supported targets. Centralize entry construction, return typed missing-vs-error behavior, and perform read-after-write verification without exposing the token.

## Files and areas involved
- `src-tauri/Cargo.toml`
- `Cargo.lock`
- `src-tauri/src/commands/github_integration.rs`
- `src-tauri/src/lib.rs`
- `src/__tests__/RepositoryView.test.tsx`
- `docs/repository.md`
- `kit/.lmbrain/CHANGELOG.md`

## Acceptance criteria
- [x] Windows builds use Credential Manager rather than the mock keyring.
- [x] macOS and Linux builds select their native credential-store integrations.
- [x] Save returns success only after a fresh entry reads back the same PAT.
- [x] No PAT value is logged or included in an error.
- [x] Missing credentials are treated as unconfigured; store failures remain errors.
- [x] A temporary native Windows credential round trip passes and cleans up after itself.

## Implementation plan
1. Correct keyring feature selection for packaged platforms.
2. Harden PAT read/save/delete semantics.
3. Add native persistence regression coverage.
4. Update Repository documentation and release notes.
5. Run Rust, frontend, build, and diff gates.

## Required verification
- `cargo test -p lmbrain native_windows_credential_store_persists_across_entries -- --nocapture`
- `cargo test --workspace`
- `cargo check -p lmbrain --all-targets`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
Linux Secret Service requires the user's desktop secret-service session at runtime; failures are now reported instead of silently degrading to an in-memory credential. Native cross-platform installer builds remain the release-level compilation proof.

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
- Enabled keyring's Apple, Windows, and synchronous Secret Service features.
- Added read-after-write persistence verification and actionable credential-store errors.
- Added an isolated Windows Credential Manager round-trip test.

### Files changed
- `src-tauri/Cargo.toml`
- `Cargo.lock`
- `src-tauri/src/commands/github_integration.rs`
- `src-tauri/src/lib.rs`
- `src/__tests__/RepositoryView.test.tsx`
- `docs/repository.md`
- `kit/.lmbrain/CHANGELOG.md`

### Verification performed
- Native Windows credential round trip: 1 passed.
- `cargo check -p lmbrain --all-targets`: passed.
- `cargo test --workspace`: passed (application 71 passed / 3 ignored; all core, MCP, integration, protocol, and doc-test suites passed).
- `pnpm lint`: passed.
- `pnpm test`: 26 files / 139 tests passed.
- `pnpm build`: passed with the existing Vite chunk-size advisory only.
- `git diff --check`: passed.

### Verification transcript
```text
$ cargo test -p lmbrain native_windows_credential_store_persists_across_entries -- --nocapture
test commands::github_integration::tests::native_windows_credential_store_persists_across_entries ... ok
test result: ok. 1 passed; 0 failed

$ cargo check -p lmbrain --all-targets
Finished dev profile
exit code: 0

$ cargo test --workspace
lmbrain: 71 passed; 0 failed; 3 ignored
contract tests: 29 passed
design tests: 6 passed
parser tests: 18 passed
path safety tests: 9 passed
workspace tests: 7 passed
lmbrain-core: 62 passed
transition tests: 15 passed
lmbrain-mcp: 15 passed
protocol tests: 3 passed
doc-tests: passed
exit code: 0

$ pnpm lint
$ eslint .
exit code: 0

$ pnpm test
Test Files  26 passed (26)
Tests       139 passed (139)
exit code: 0

$ pnpm build
320 modules transformed
built in 322ms
exit code: 0

$ git diff --check
exit code: 0
```

### Deviations from the specification
- None.

### Handoff status
- [x] Ready for Project Lead review
