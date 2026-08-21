---
id: SPEC-066
title: "Make 3.1.x background data operations asynchronous to prevent UI stalls"
status: ready
kind: bugfix
priority: high
area: desktop-ui
milestone: M-08
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/40]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-40, desktop-ui, async-loading, KIT-NOTE-014]
activity:
  - date: 2026-07-31
    action: "created"
  - date: 2026-07-31
    action: "transitioned backlog -> ready"
---
# Make 3.1.x background data operations asynchronous to prevent UI stalls

## Objective
Ensure background data fetching and state reconciliation introduced in 3.1.x run asynchronously without blocking the main UI thread.

## Context
Reported in `KIT-NOTE-014` (v3.1.2): After updating to 3.1.x, the desktop app experiences blocking loader loops that freeze the UI thread during normal operator navigation and attestation workflows.

## Diagnostic analysis (2026-07-31)

### Confirmed root cause

The frontend's use of promises does not make the Rust work asynchronous. `fetchWorkspaceData()` launches twelve IPC requests with `Promise.all`, but every corresponding Tauri handler is a synchronous `#[tauri::command]`. Tauri executes commands without `async` on the main thread, so the requests serialize expensive filesystem parsing on the UI thread.

The regression is amplified by redundant full-workspace scans:

- `get_pulse_data` reparses specs, reviews, ADRs, and handoffs even though the same refresh also calls the dedicated collection commands.
- `get_diagnostics` scans and parses the complete `.lmbrain` Markdown tree.
- The Pulse `InsightReliability` panel added in 3.1.1 mounts without statistics props and independently calls `get_project_statistics`.
- `get_project_statistics` reparses all artifact families and runs the complete diagnostics scan again.
- Switching away from Pulse and back remounts the view and repeats the statistics request. The explicit refresh key also forces the same remount.
- React StrictMode re-runs the mount effect once more in development, exposing two uncancelled backend statistics computations even though the component ignores the first result after cleanup.
- Watcher events call `loadAllData()` without in-flight exclusion, cancellation, or frontend coalescing. The backend watcher debounces filesystem notifications for 500 ms, but a later event may enqueue another twelve-command refresh while the previous refresh is still running.

The 3.1.3 release commit `28a22fb` adds the claim "Asynchronous 3.1.x Background Loading (#40)" to the bundled changelog but contains no relevant source change: its application change is limited to the active-session close handler in `src/App.tsx`. All affected data commands remain synchronous at tag `v3.1.3`.

### User-visible sequence

1. Opening or refreshing a workspace queues twelve synchronous filesystem commands.
2. Pulse renders, mounts `InsightReliability`, and queues a second whole-project aggregation.
3. Each synchronous handler occupies the Tauri main thread; `Promise.all` increases queued work rather than parallelizing it off-thread.
4. Navigation, window input, paint, and loader progress stall until the queue drains.
5. A watcher event or Pulse remount starts the sequence again, which presents as recurring or continuous loading.

### Scale factor

The cost grows with project artifact count and document size. On the current repository there are 163 `.lmbrain` Markdown files, including 70 specs and 43 reviews. Review parsing also reads and parses each review a second time for lifecycle analysis, making the duplicated statistics path particularly expensive.

## Scope
### Included
- Audit data loading hooks and IPC invocations added in 3.1.x releases (diagnostic reconciliation, finding indexes, verification gate checks).
- Move blocking filesystem and parsing work off the Tauri main thread using async commands with an appropriate blocking-worker boundary.
- Consolidate shared workspace collection, Pulse, statistics, and diagnostics reads into a coherent snapshot or cache so one refresh does not repeatedly parse the same artifact families.
- Prevent overlapping watcher/manual refreshes and discard or supersede stale refresh results.
- Stop Pulse remounts from independently recomputing whole-project statistics when the same snapshot already contains the required reliability values.
- Ensure non-blocking loading spinners or skeleton UI states are shown while retaining desktop UI responsiveness.

### Excluded
- Changing data fetch schemas or backend query contracts.
- Disabling React StrictMode or the file watcher as a workaround.
- Hiding loaders without removing the main-thread work.

## Acceptance criteria
- [x] All workspace read/aggregation commands that perform filesystem I/O or non-trivial parsing execute off the Tauri main thread.
- [x] One logical workspace refresh parses each artifact family and diagnostics input at most once, with shared results reused by Pulse and Insights.
- [x] Mounting or revisiting Pulse does not start an additional whole-project statistics scan when current snapshot data is available.
- [x] React StrictMode development mounting does not leave duplicate backend computations running.
- [x] Watcher bursts are coalesced and at most one refresh pipeline is active; a newer request cannot allow an older result to overwrite it.
- [ ] UI input, navigation, painting, and session terminals remain responsive while a representative large workspace refresh is in progress.
- [x] Background loading is presented locally and non-blockingly; existing data remains visible until a successful replacement snapshot is committed.
- [x] A failed or stale refresh produces bounded error handling and never causes an automatic retry loop.
- [x] Regression coverage records command invocation counts for workspace open, watcher burst, Pulse revisit, and explicit refresh.
- [x] The release changelog claims issue #40 fixed only after the responsiveness and invocation-count evidence passes.

## Required verification
- `pnpm test`
- `pnpm lint`
- `pnpm build`
- `cargo test --workspace`
- Instrumented desktop test with a representative large `.lmbrain` fixture, recording main-thread responsiveness and per-command invocation counts.

## Investigation evidence

- `src/context/WorkspaceContext.tsx:307-350` — twelve collection commands are issued by `Promise.all`.
- `src/context/WorkspaceContext.tsx:390-397` — every `file-changed` event starts `loadAllData()` with no in-flight guard.
- `src/components/Pulse/ProjectPulse.tsx:225-238` — Pulse mounts `InsightReliability` without precomputed statistics.
- `src/components/Shared/InsightReliability.tsx:23-50` — a mount effect calls `getProjectStatistics`; cleanup suppresses state updates but cannot cancel backend work.
- `src/components/Layout/AppShell.tsx:35-77,137-147` — navigation and explicit refresh remount the active view.
- `src/main.tsx:7-11` — root StrictMode causes the development-only extra Effect cycle.
- `src-tauri/src/lib.rs:223-446` — Pulse, artifact, statistics, and diagnostics handlers are synchronous Tauri commands.
- `src-tauri/src/commands/contract.rs:793-878` — statistics rebuild every artifact family and diagnostics.
- `src-tauri/src/commands/contract.rs:90-139` — review lifecycle analysis re-reads and reparses every review.
- `src-tauri/src/commands/watcher.rs:70-119` — backend notification debounce exists, but it does not serialize frontend refresh pipelines.
- `git show --stat 28a22fb` — the 3.1.3 issue #40 claim has no corresponding data-loading implementation.

## Project Lead corrective takeover

- **Authorized by:** human operator on 2026-07-31, with an explicit request for an urgent structural fix on a 3.1.4 feature branch.
- **Reason:** the released desktop application is practically unusable because the main-thread loading regression remains present despite the 3.1.3 fix claim.
- **Scope boundary:** only the loading architecture, refresh coordination, Pulse/Insights statistics reuse, targeted regression coverage, version 3.1.4 alignment, and directly affected documentation.
- **Technical direction:** create one coherent backend workspace snapshot; execute blocking filesystem/parsing work outside the Tauri main thread; serialize/coalesce refresh requests; prevent stale commits; and remove view-mount-triggered whole-project rescans.
- **Verification plan:** frontend invocation/concurrency tests, Rust snapshot/command tests, full frontend and Rust quality gates, version alignment checks, and a separate final verification pass against every acceptance criterion.
- **Not authorized:** product-scope changes, disabling the watcher or StrictMode, hiding loaders, unrelated refactors, new external integrations, or dependency additions without renewed operator approval.

## Implementation evidence

### Changes made

- Added a typed Rust/TypeScript `WorkspaceSnapshot` contract that returns Pulse, collections, diagnostics, and project statistics from one backend build.
- Refactored statistics construction to reuse already parsed collections and diagnostics.
- Replaced the twelve-command frontend refresh with one `get_workspace_snapshot` invocation.
- Marked filesystem, Git, aggregation, verification, harness, and lifecycle Tauri commands asynchronous; intentionally kept only ordered low-cost session writes/resizes/lists and watcher status synchronous.
- Added a trailing single-flight refresh coordinator: concurrent requests share one promise, any burst produces at most one trailing refresh, failures do not retry automatically, and every waiter receives the newest result.
- Removed mount effects from Pulse reliability and Insights; both now consume snapshot statistics.
- Added a non-blocking `Syncing workspace` TopBar status while preserving the previous snapshot.
- Aligned application, Tauri crate, lockfile, bundled kit, changelog, README, and migration guidance at 3.1.4.

### Verification transcript

```text
node scripts/check-version.mjs
PASS — LMBrain app and kit are aligned at v3.1.4.

pnpm lint
PASS

pnpm build
PASS — TypeScript and production Vite build.

pnpm test
PASS — 34 files, 181 tests.

cargo test --workspace
PASS — 316 passed, 0 failed, 3 ignored manual/long-running harness tests.

cargo test -p lmbrain --test contract_test workspace_snapshot -- --nocapture
PASS — coherent snapshot test plus 250-spec representative fixture; 2 passed.

git diff --check
PASS
```

### Separate verification pass

- Confirmed no frontend view invokes `getProjectStatistics`; only the compatibility command wrapper remains.
- Confirmed the workspace context no longer calls the twelve individual artifact/diagnostic commands.
- Confirmed all non-trivial Tauri commands use `#[tauri::command(async)]` or an `async fn`; synchronous exceptions are limited to low-cost ordered session I/O/listing and watcher status.
- Confirmed coalesced callers share one request and receive only the trailing snapshot in both coordinator and provider-level tests.
- Confirmed a failed in-flight refresh rejects once, clears coordinator state, and requires an explicit later request.
- Confirmed the full frontend and Rust suites pass after the final loading-indicator and large-fixture additions.

### Remaining runtime verification

The installed production LMBrain process was active during verification (`C:\Users\moren\AppData\Local\LMBrain\lmbrain.exe`, PID 33852). It was not stopped or replaced without operator direction. A packaged/manual 3.1.4 smoke against this real workspace remains required to check the UI-input/session-terminal responsiveness criterion before release acceptance.
