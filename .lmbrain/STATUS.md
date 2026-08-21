---
title: Project pulse
updated: 2026-07-31
---

# Project Pulse

## Current focus

M-08 / LMBrain 3.1.3 maintenance and kit feedback planning is now materialized in backlog as
[[SPEC-060-kit-file-realignment-and-drift-diagnostics]]
through [[SPEC-068-add-active-sessions-close-confirmation-dialog]]. The plan
covers GitHub issues #34 through #42 based on 3.1.2 kit feedback from `E:\Git\XenoMark`.
The issues cover kit file realignment, Windows Node REPL kernel path creation, Browser skill
`file://` policy alignment and claimed-tab read ops, `spec_attest_lead` checklist completion,
waived criteria with linked FINDINGs at `spec_done`, async 3.1.x background loading for UI
responsiveness, Lead remediation verification events, and window close confirmation for active sessions.

Operator-reported desktop freezes were traced to [[SPEC-066-make-3-1-x-background-loading-asynchronous]].
At tag `v3.1.3`, the twelve-command workspace refresh, Pulse statistics aggregation, and diagnostics
remain synchronous Tauri commands executed on the main thread. Pulse's 3.1.1
`InsightReliability` panel independently repeats the whole-project statistics scan on each mount,
while watcher refreshes have no in-flight exclusion. Commit `28a22fb` added the issue #40 fix claim
to the changelog without a corresponding data-loading source change. The bug is therefore unresolved
and the 3.1.3 release note is not supported by implementation evidence.

The operator authorized an urgent Project Lead corrective takeover on branch
`codex/3.1.4-async-workspace-loading`. The branch now uses one off-main-thread workspace snapshot,
coalesces refresh bursts, reuses statistics across Pulse and Insights, and aligns release sources at
3.1.4. Full frontend and Rust gates pass. [[REVIEW-043-independent-verification-of-spec-066-asynchronous-workspace-loading]]
found no code issue; final acceptance is pending a packaged/manual responsiveness smoke because the
installed 3.1.3 process remained active and was not interrupted.

M-07 / LMBrain 3.1.0 issue-driven planning is materialized in backlog as
[[SPEC-049-fix-specification-detail-navigation-and-breadcrumb-semantics]]
through [[SPEC-059-integrate-qualify-and-release-lmbrain-3-1-0]]. The plan
covers open GitHub issues #10 through #18, split into diagnostic, review
lifecycle, verification, spec lifecycle, findings, and release-integration
handoffs.


Repository analysis used `origin/main` / tagged `v3.0.2` as the target
baseline. The local checkout remains at `v3.0.1`, one commit behind, with
user-owned untracked `.pi/`, `ISSUE-4-PLAN.md`, and `ISSUE-8-PLAN.md`
preserved. No pull, source-code edit, dependency change, or implementation
work was performed during planning.

Operator direction on 2026-07-29 makes GitHub issue #12 mandatory for 3.1.0:
the release must include both the governed `FINDING-*` core/MCP lifecycle and
the usable desktop/migration experience. A fixes-only intermediate release is
not planned. Read-only analysis of `E:\Git\XenoMark` validated the planned-debt,
resolved-limitation, targetless-design-observation, duplicate-local-ID, review
cycle, and verification-gate migration cases. Proposed
[[ADR-014-promoted-findings-have-independent-lifecycle-while-reviews-preserve-historical-outcome]]
records the resulting lifecycle and historical-integrity decision; operator
acceptance remains required before SPEC-057 can become ready.

Planning for M-05 / LMBrain 2.9.0 is now in backlog. The release train keeps
[[SPEC-036-verification-transcript-integrity-and-kit-executed-gates]] as the
execution-provenance foundation, adds
[[SPEC-038-context-complete-handoffs-and-structured-verification-gate-contracts]]
for implementer-side context completeness,
[[SPEC-039-governed-agent-improvement-recommendations-proposal-application-and-effectiveness-metrics]]
for the operator-governed learning loop, and
[[SPEC-040-lmbrain-2-9-0-release-integration-migration-and-astranexus-regression-qualification]]
for migration, production-regression qualification, packaging, and release.
The three new specs remain `backlog` until the operator explicitly approves
them; no implementation handoff is authorized by this planning work.
Repository validation also reports a pre-existing duplicate `SPEC-036` between
the ready verification-provenance spec and a Windows-installer remediation in
review. That collision must be resolved under operator authority before M-05
spec approval so dependency resolution is deterministic.

M-03 v3 workflow and workspace ergonomics is active. The operator approved the v3 governance ADR and implementation specs for context/token economy, granular agents, tabbed sessions, and milestone intelligence: [[ADR-008-controlled-agent-self-improvement]], [[SPEC-023-v3-context-economy]], [[SPEC-024-v3-agent-taxonomy-and-improvement-loop]], [[SPEC-025-v3-session-tabs]], and [[SPEC-026-v3-milestone-intelligence]]. [[SPEC-026-A-remove-ui-approval-actions]] is approved and ready to remove direct app-side approval actions for specs and agent profiles. [[SPEC-027-kit-migration-detection-and-project-lead-prompt]] has been drafted in backlog for kit version mismatch detection and Project Lead migration prompts.

[[SPEC-029-add-pi-agent-sessions-through-ollama]] is in corrective implementation after [[REVIEW-031-review-spec-029-pi-agent-sessions-through-ollama]]. The operator authorized Project Lead takeover and accepted [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]]. Compilation-only verification is active while another production instance is running; automated and runtime smoke tests remain deferred to a safe window.

[[SPEC-030-fix-session-scrollback-and-clipboard-interaction]] was added to the same protected corrective cycle after the operator reported lost scrollback on tab/view return and unreliable clipboard interaction. Source remediation and compile-only checks are complete; manual terminal/clipboard verification remains deferred.

[[SPEC-031-add-workspace-preparation-progress-and-pi-dependency-bootstrap]] is working after the operator requested automatic exact-pin Pi dependency preparation and visible workspace-open progress during local testing. Accepted [[ADR-010-bootstrap-pinned-pi-mcp-dependency-during-workspace-preparation]] supersedes the manual-install direction in [[ADR-009-pi-mcp-through-a-pinned-project-local-extension]].

[[SPEC-032-make-review-history-and-insight-reliability-actionable]] is in review under the same operator-authorized pre-release escalation. It removes the low-value temporal review panel and replaces the raw diagnostic path grouping with full-width insight-reliability checks, expandable detail, and shared copyable fix prompts, without changing backend contracts. Independent [[REVIEW-034-final-verification-of-spec-032-insight-panels]] found no issues and recommends acceptance; operator acceptance remains pending.

Release `2.6.0` was published by the operator. A Codex alternate-buffer scrolling regression was then reported and remediated for patch `2.6.1`; package, Tauri crate, Cargo lockfile, and bundled kit are aligned, and the migration requires no artifact rewrite. All 111 frontend tests, lint, production build, Rust workspace checks with tests, version alignment, and diff checks pass. Runtime confirmation against the `2.6.1` build remains pending before SPEC-030 can be accepted.

[[SPEC-033-add-explicit-current-view-data-refresh]] is in review with a passing [[REVIEW-035-review-spec-033-current-view-refresh]]. Patch `2.6.1` now includes a failure-aware header refresh that reloads shared and current-view data while preserving session terminals; the full gate now covers 114 frontend tests.

[[SPEC-034-manage-local-agent-harness-installations-and-updates]] is in review with passing [[REVIEW-036-review-spec-034-local-harness-management]]. Version `2.7.0` adds the Local Harnesses page and guarded user-level self-updates; full Rust tests and 118 frontend tests pass, and a real read-only probe found all three operator harnesses. No updater was executed.

Completed v3 work:

- [[SPEC-023-v3-context-economy]] is done after [[REVIEW-016-spec-023-v3-context-economy-remediation]].
- [[SPEC-024-v3-agent-taxonomy-and-improvement-loop]] is done after [[REVIEW-018-spec-024-v3-agent-taxonomy-remediation]].
- [[SPEC-025-v3-session-tabs]] is done after [[REVIEW-022-spec-025-v3-session-tabs-final-remediation]].

Next v3 handoff: [[SPEC-026-v3-milestone-intelligence]].

## Current milestone

M-03 - LMBrain v3 workflow and workspace ergonomics. M-01 deliverables are accepted; M-02 remains partially open with older non-v3 specs still requiring separate triage.

## Awaiting remediation

No v3 specs are currently awaiting remediation.

Older non-v3 review items remain on the board, including [[SPEC-022-fix-in-app-claude-mcp-resolution]], and should be handled separately from the v3 milestone flow.

## In progress

No v3 implementation is currently in progress. [[SPEC-026-v3-milestone-intelligence]] is ready for the next coding-agent handoff.

## Blockers and risks

- `v3.1.3` still performs artifact parsing and diagnostic aggregation on the Tauri main thread; repeated
  Pulse mounts or watcher-triggered refreshes can freeze the desktop UI. Treat
  [[SPEC-066-make-3-1-x-background-loading-asynchronous]] as a high-priority unresolved regression,
  despite the current changelog claim.
- Release/version narrative still needs reconciliation before a coordinated v3 release.
- Several older non-v3 specs remain in `review`, `ready`, or `backlog` and should be triaged separately to reduce board noise.
- The granular v3 profiles from [[SPEC-024-v3-agent-taxonomy-and-improvement-loop]] remain proposed until the operator explicitly activates the selected profiles.

## Next recommended actions

1. Hand [[SPEC-026-v3-milestone-intelligence]] to [[AGENT-FULLSTACK-DESKTOP]] for the milestone intelligence / milestones UI work.
2. Hand [[SPEC-026-A-remove-ui-approval-actions]] to [[AGENT-FULLSTACK-DESKTOP]] for direct approval UI removal.
3. Approve [[SPEC-027-kit-migration-detection-and-project-lead-prompt]] to `ready` when the operator wants migration detection/prompting implemented.
4. Review [[SPEC-022-fix-in-app-claude-mcp-resolution]] and, if accepted, mark it done.
5. Triage older non-v3 specs in `review`, `ready`, and `backlog` so M-03 progress is easier to read.
6. Reconcile release/version narrative before cutting a coordinated v3 release.

## Recent decisions

- [[ADR-008-controlled-agent-self-improvement]] (accepted - v3 agent improvement governance)
- [[ADR-003-reject-as-first-class-status]] (accepted - Contract v0.2; `rejected` everywhere; resolves SPEC-014 D-1/D-2)
- [[ADR-002-in-app-artifact-status-writes]] (accepted - unblocks M-02)
- [[ADR-001-desktop-first-tauri]]
