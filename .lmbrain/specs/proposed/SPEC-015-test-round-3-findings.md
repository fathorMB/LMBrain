---
id: SPEC-015
title: Test round 3 findings catalog
status: proposed
kind: bug
priority: medium
area: desktop-app, kit
milestone: M-02
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-002, ADR-003]
links: [SPEC-012, SPEC-013, SPEC-014, ADR-002, ADR-003]
created: 2026-06-23
updated: 2026-06-23
tags: [testing, round-3]
---

# Test round 3 findings catalog

## Objective

Catalog of defects and enhancement requests found during manual test round 3, validating LMBrain **1.1.0** (M-02 approve/reject workflow + Contract v0.2 reject + round-2 fixes) against the scaffolded `E:\SnippetVault` brain. Same convention as the previous rounds ([[SPEC-012-test-round-2-findings]]): each confirmed finding is a numbered entry (R3-F1, R3-F2, …); at approval time, actionable findings are split into a focused remediation handoff. Scope decisions for each finding are recorded under "Operator decisions".

## Context

The app under test is now read-write for operator-driven status changes (approve/reject). Findings may touch the new write workflow, the kit, or earlier surfaces. For each report, useful detail: **where** (app or kit), **how to reproduce**, and whether on SnippetVault or a fresh scaffold.

## Findings

### R3-F1 — Expose and copy the handoff prompt from recommended-action cards

- **Type:** Enhancement
- **Area:** Desktop app — `Next recommended actions`
- **Observed:** Cards such as **“Start AGENT-FS on SPEC-001”** state that the handoff prompt should be copied and the agent launched manually, but do not expose that prompt or provide a direct way to copy it.
- **Requested behavior:** Add an action on these cards that opens/displays the generated handoff prompt and includes a one-click **Copy to clipboard** control.
- **Acceptance criteria:**
  - The operator can view the exact generated prompt before copying it.
  - A single interaction copies the full prompt to the system clipboard.
  - The UI gives clear success feedback after the copy operation.
  - The action remains a manual handoff: it does not launch an agent or write project state.

### R3-F2 — ADR action controls do not refresh after approval or rejection

- **Type:** Bug
- **Area:** Desktop app — ADR approval/rejection workflow
- **Observed:** After confirming approval of an ADR, its status changes correctly but the **Approve** button remains visible. Reject was not manually exercised in this round and must be covered as the symmetric transition.
- **Expected behavior:** Once an ADR is approved or rejected, the UI must refresh its available actions so the completed transition cannot be invoked again.
- **Acceptance criteria:**
  - After a successful approval, the ADR no longer exposes the **Approve** action.
  - After a successful rejection, the ADR no longer exposes the **Reject** action.
  - The rendered status and available actions are consistent without requiring a manual page refresh.

### R3-F3 — Quick Links for STATUS.md and ROADMAP.md are non-functional

- **Type:** Bug
- **Area:** Desktop app — Quick Links
- **Observed:** The Quick Links panel displays `STATUS.md` and `ROADMAP.md`, but selecting either link does not open its target document.
- **Expected behavior:** Each Quick Link opens the corresponding project Markdown document in the existing read-only artifact-detail modal. This is a read-only navigation feature: it must not expose approve/reject or any other write control.
- **Acceptance criteria:**
  - Selecting `STATUS.md` opens the project's status document in the modal.
  - Selecting `ROADMAP.md` opens the project's roadmap document in the modal.
  - The modal renders the complete document, is keyboard-accessible, and remains read-only.
  - A missing or unreadable document is surfaced with a clear, actionable error state rather than a silent no-op.

### R3-F4 — Roadmap must not display temporal milestone representations

- **Type:** Enhancement
- **Area:** Desktop app — Roadmap / milestone cards
- **Observed:** Milestone cards display a `Target` field (for example, `2026-Q4`). Roadmaps must not currently contain or represent temporal commitments.
- **Requested behavior:** Remove temporal fields from the active roadmap source and do not render targets, ETAs, estimates, dates, quarters, or equivalent timeline representations in the Roadmap UI.
- **Acceptance criteria:**
  - The active `ROADMAP.md` has no temporal target fields for milestones.
  - Roadmap milestone cards render no temporal target/ETA/estimate representation.
  - A roadmap document that contains an obsolete temporal field does not cause the UI to display it.
  - The milestone outcome, status, risks, mapped artifacts, and unmapped-artifact handling remain available.

## Operator decisions

- **R3-F1 (decided 2026-06-23):** applies to recommended manual-handoff cards. The operator can inspect and copy the proposed prompt, but LMBrain never starts an agent automatically.
- **R3-F2 (decided 2026-06-23):** treat the stale action as a workflow-refresh regression and verify both ADR transitions, even though only approval was manually observed.
- **R3-F3 (decided 2026-06-23):** Quick Links open their corresponding Markdown document in the existing read-only detail modal, rather than navigating to a derived application view or launching an external editor.
- **R3-F4 (decided 2026-06-23):** roadmaps must not contain or render temporal representations at this time. The active roadmap's `target` fields have been removed; the UI must ignore any obsolete temporal field it encounters.
