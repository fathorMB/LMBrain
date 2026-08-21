---
id: REVIEW-036
# Note: Quote the title if it contains a colon
title: "Review SPEC-034 local harness management"
status: pending
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
spec: SPEC-034
reviewer: Project Lead escalation verification
review_requested_by: user
implementation_agent: Project Lead escalation
related_tasks: []
links: []
created: 2026-07-11
updated: 2026-07-11
tags: [harnesses, security, updates, verification]
activity:
  - date: 2026-07-11
    action: "created"
---
# Review SPEC-034 local harness management

## Outcome

Pass within the verified non-mutating scope. The implementation meets the local harness visibility/update contract and places defense-in-depth controls around user-level mutations. Real updater execution remains explicitly operator-controlled.

## Acceptance-criteria compliance

- All three harnesses have typed installed/missing/error probes with exact executable/version/time and fixed official guidance.
- Codex resolution is shared with Sessions and accepts the configured override.
- Updaters use closed host-to-argv mappings; Pi updates only itself and ignores project-local files.
- UI confirmation and backend active-session/concurrency gates are independent and both enforced.
- Update execution is off-thread, outside the workspace, stdin-closed, time/output bounded, process-tree terminated on timeout, and post-probed.
- Missing harnesses cannot enter the update path and are never installed automatically.
- Version `2.7.0`, documentation, migration guidance, and full tests are aligned.

## Code observations

- `HarnessUpdateLease` clears serialized state through `Drop`, including error paths and worker completion.
- Output reader threads drain child pipes but retain at most 64 KiB; channel receive timeout prevents inherited descendant handles from blocking the Tauri response.
- Windows timeout uses fixed `taskkill /T /F`; Unix starts a dedicated process group and terminates that group. No user-controlled shell string is constructed.
- Success logic correctly requires updater exit zero, no timeout, and a successful installed-state post-probe. Unchanged versions are reported as already current.
- Frontend retains the prior/freshly probed card state on update errors and exposes structured logs for completed updater results.

## Tests and verification

- `cargo test --workspace` - passed; harness tests cover parse/argv/lease/active-session/outcome/timeout/output policies.
- Manual ignored read-only harness probe - passed for the operator's Claude, Desktop-resolved Codex, and Pi installations.
- `pnpm test` - passed, 21 files / 118 tests.
- `pnpm lint` and `pnpm build` - passed; existing bundle-size warning only.
- `cargo check --workspace --tests` - passed.
- `node scripts/check-version.mjs` - passed at `2.7.0`.
- `git diff --check` - passed.
- No real updater or missing-harness installer was invoked.

## Production quality and documentation compliance

Compliant with [[QUALITY]]. The feature is typed, bounded, dependency-free, explicit about external mutation, tested across failure policies, and documented with a no-artifact migration. No shortcut or hidden automatic behavior was introduced.

## Findings

None blocking.

## Required follow-up

- Operator should visually verify the page and may choose to run one real updater at a time after closing matching sessions. Such a mutation was deliberately outside automated verification.

## Final decision

Recommend acceptance of [[SPEC-034-manage-local-agent-harness-installations-and-updates]].
