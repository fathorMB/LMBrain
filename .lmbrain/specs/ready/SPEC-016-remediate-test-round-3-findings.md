---
id: SPEC-016
title: Remediate test round 3 findings (handoff actions, modal links, workflow refresh, timeless roadmap)
status: ready
kind: bug
priority: high
area: desktop-app, kit
milestone: M-02
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-002, ADR-003]
links: [SPEC-015, SPEC-013, SPEC-014, ADR-002, ADR-003]
created: 2026-06-23
updated: 2026-06-23
tags: [remediation, round-3, pulse, modal, roadmap, workflow]
---

# Remediate test round 3 findings

## Objective

Implement the confirmed, operator-decided findings in [[SPEC-015-test-round-3-findings]]: make recommended manual handoffs actionable without starting agents, refresh the detail modal after an ADR transition, make Quick Links open the underlying Markdown read-only, and remove temporal representations from roadmap data and UI.

This is a corrective M-02 handoff. It must preserve the operator-controlled, manual-agent model and the status-transition invariants defined by [[ADR-002-in-app-artifact-status-writes]] and [[ADR-003-reject-as-first-class-status]].

## Scope

### Included

- R3-F1: a viewable and copyable prompt on recommended **manual-handoff** action cards.
- R3-F2: reliable refresh of the artifact detail modal after ADR approval or rejection.
- R3-F3: functional `STATUS.md` and `ROADMAP.md` Quick Links that open the source Markdown in the existing read-only detail modal.
- R3-F4: remove temporal roadmap fields from the app data model/parser and Roadmap view, while retaining all non-temporal roadmap information.
- Targeted automated regression coverage and the verification below.

### Excluded

- Starting, spawning, or otherwise automating an agent.
- New artifact status transitions, free-form status editing, reopening rejected artifacts, or bulk actions.
- Changing the Markdown contract, versioning, release notes, or application architecture.
- Restoring a roadmap target/ETA/date/quarter field under a different label.

## Work items

### WI-1 — Recommended manual-handoff prompt (R3-F1)

`build_pulse_data` already emits `action_type: "handoff"` cards for ready specs, but `ActionCard` in `ProjectPulse.tsx` only renders static text. Add an accessible action for cards of this type that reveals the proposed prompt in a read-only, selectable control and provides a one-click clipboard copy with visible success/failure feedback.

The prompt must identify the recommended profile and exact ready spec path. Reuse or centralize the existing prompt construction in `SpecDetail.tsx` rather than maintaining divergent handoff wording. This is strictly a copy aid: it must never invoke an agent, alter an artifact, or imply that a handoff has been launched.

### WI-2 — Refresh transition-dependent controls in the artifact modal (R3-F2)

`ArtifactDetailModal.tsx` derives its eligible controls from the parsed document's `status`. After `setArtifactStatus` succeeds, the modal changes to the returned path but can retain stale parsed state long enough to keep an already-completed action visible. Make the post-write state deterministic: after a successful backend transition, reload the returned document and derive status/action visibility from that refreshed document only.

For a proposed ADR, `approve → accepted` and `reject → rejected` remain the only legal transitions. After either result, the modal must show the persisted status and no approve/reject control; the backend remains the authority for all transition legality.

### WI-3 — Open Quick Link Markdown in the read-only modal (R3-F3)

`QuickLink` currently renders a pointer-styled `div` without a click handler. Make `STATUS.md` and `ROADMAP.md` real, keyboard-accessible controls. Resolve their paths from the current workspace and open them with the existing `SET_DETAIL_ARTIFACT` flow.

The existing `ArtifactDetailModal` is the destination. It must render the complete Markdown and stay read-only for these documents: no approve/reject or other write affordance may appear. If the document is missing or cannot be parsed, use the modal's visible error state; do not silently do nothing. Preserve the ready-handoff Quick Link's existing display unless it is deliberately wired and covered separately.

### WI-4 — Timeless roadmap model and view (R3-F4)

The product decision is that roadmaps do not currently contain or represent temporal commitments. The active `ROADMAP.md` has already had its `target` entries removed. Remove the `target` property from the Rust `Milestone` model, the frontend `Milestone` type, and the Roadmap view. The roadmap parser must ignore an obsolete `target` entry if one is encountered, so it can never reappear in the UI.

Keep parsing and rendering `status`, `outcome`, `specs`, `decisions`, `risks`, `depends_on`, progress, and the explicit Unmapped Artifacts section. Do not replace `target` with an ETA, estimate, timeframe, date, quarter, or comparable timeline representation.

## Files and areas involved

- `src/components/Pulse/ProjectPulse.tsx` — handoff-card action and Quick Links.
- `src/components/Spec/SpecDetail.tsx` (or a small shared frontend helper) — reuse/centralize canonical handoff-prompt generation.
- `src/components/Layout/ArtifactDetailModal.tsx` — refresh lifecycle after a successful status write; read-only detail behavior.
- `src/__tests__/ProjectPulse.test.tsx`, `src/__tests__/ArtifactDetailModal.test.tsx` — component regressions; add a Roadmap view test if needed.
- `src-tauri/src/models/roadmap.rs`, `src-tauri/src/commands/contract.rs`, `src/types/index.ts`, `src/components/Roadmap/RoadmapView.tsx` — remove the temporal roadmap representation and parser/UI use.
- `src-tauri/tests/contract_test.rs` — update roadmap parser fixtures/assertions for the removed `target` field and legacy-field ignore behavior.

## Acceptance criteria

- [ ] Every Pulse action with `action_type: "handoff"` exposes a read-only proposed prompt and a one-click copy action; successful copying receives visible feedback, and clipboard failure receives usable feedback.
- [ ] The handoff prompt names the recommended agent and the exact ready spec path, and is consistent with the existing Spec-detail handoff prompt.
- [ ] The recommended-action control never launches an agent or writes project/artifact state.
- [ ] After an ADR approval, the detail modal displays `accepted` and no longer shows Approve or Reject without a manual reload.
- [ ] After an ADR rejection, the detail modal displays `rejected` and no longer shows Approve or Reject without a manual reload.
- [ ] `STATUS.md` and `ROADMAP.md` Quick Links are semantic, keyboard-accessible controls that open their exact workspace files in `ArtifactDetailModal`.
- [ ] Those Quick Link documents render in the modal with no status-write controls; missing/unreadable files produce a visible modal error.
- [ ] `Milestone` has no `target` property in Rust or TypeScript; roadmap cards render no temporal field.
- [ ] A legacy `target` line in a roadmap is ignored and never appears in the Roadmap UI.
- [ ] The Roadmap continues to render outcome, status, risks, mapped artifacts/progress, and Unmapped Artifacts correctly.

## Required verification

- `pnpm lint`, `pnpm test`, and `pnpm build` pass.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` pass; paste actual output in implementation evidence.
- `node scripts/check-version.mjs` passes; do not alter versions, `CHANGELOG.md`, or `CONTRACT.md`.
- Component tests cover: handoff-prompt reveal/copy and clipboard failure; successful ADR approve and reject refreshes; both Quick Links dispatch the exact source-document path and stay read-only; Roadmap does not render a legacy target.
- Rust tests cover: a roadmap without temporal data parses correctly, and an obsolete `target` field is ignored.
- Manual: verify the four flows in a scaffolded brain, including keyboard activation of each Quick Link and a modal error for a deliberately unavailable document where practical.

## Production quality and documentation

- Follow [[QUALITY]] and preserve the backend validation/atomic-write behavior from SPEC-014.
- Keep the implementation local-first and dependency-free unless a new dependency is demonstrably necessary and approved.
- Update only this spec's **Implementation evidence** and any technical documentation explicitly delegated here. Do not update roadmap, status, decisions, contract semantics, or release/version files.
- Report changed files, tests run, and deviations honestly.

## Instructions for the assigned specialist

- Implement only WI-1 through WI-4. Do not broaden the M-02 workflow or revive the withdrawn agent-proposal request.
- Treat the modal's freshly parsed document as the source for UI status after a write; do not paper over the issue by merely hiding a button optimistically.
- Do not automatically start agents. A copied prompt remains a manual handoff under `OPERATOR.md` and `CONTRACT.md`.
- Do not encode or render roadmap timing under a new name.
- Fill in the Implementation evidence when complete. Do not change the specification's status; the Project Lead manages its review transition.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

- Added a shared handoff-prompt builder and used it from both the Spec detail CTA and Pulse handoff actions. Pulse handoff cards now reveal a read-only prompt and support clipboard copy with success/error feedback.
- Made `STATUS.md` and `ROADMAP.md` Quick Links keyboard-accessible buttons that open their exact workspace documents in the existing detail modal.
- Added an explicit modal reload after status writes, including flat ADR files whose path does not change; stale document state is suppressed while the refreshed document loads.
- Removed the roadmap `target` property from Rust and TypeScript models, the parser, and the Roadmap UI. Legacy `target` entries are ignored.

### Files changed

- `src/lib/handoffPrompt.ts`
- `src/components/Spec/SpecDetail.tsx`
- `src/components/Pulse/ProjectPulse.tsx`
- `src/components/Layout/ArtifactDetailModal.tsx`
- `src/components/Roadmap/RoadmapView.tsx`
- `src/types/index.ts`
- `src-tauri/src/models/roadmap.rs`
- `src-tauri/src/commands/contract.rs`
- `src-tauri/tests/contract_test.rs`
- `src/__tests__/ProjectPulse.test.tsx`
- `src/__tests__/ArtifactDetailModal.test.tsx`
- `src/__tests__/RoadmapView.test.tsx`

### Verification performed

- `pnpm test` — passed: 8 test files, 41 tests.
- `pnpm lint` — passed.
- `pnpm build` — passed.
- `node scripts/check-version.mjs` — passed: app and kit aligned at `v1.1.0`.
- `cargo fmt --check` — passed.
- `cargo clippy -- -D warnings` — passed.
- `cargo test` — passed: 11 contract tests, 18 parser tests, and 9 path-safety tests (plus empty unit/doc-test suites).

### Deviations from the specification

- None.

### Handoff status

- [x] Ready for Project Lead review
