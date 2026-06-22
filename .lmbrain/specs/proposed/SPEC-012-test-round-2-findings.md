---
id: SPEC-012
title: Test round 2 findings catalog
status: proposed
kind: bug
priority: medium
area: desktop-app, kit
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [SPEC-010]
created: 2026-06-22
updated: 2026-06-22
tags: [testing, parser, frontmatter, ux, kit]
---

# Test round 2 findings catalog

## Objective

> Round 2 closed. Remediation handed off in [[SPEC-013-remediate-test-round-2-findings]] (R2-F1…R2-F4 + R2-F5 part 1). R2-F5 parts 2–3 (approve/reject writes) are deferred to milestone M-02, gated by [[ADR-002-in-app-artifact-status-writes]].

Catalog of defects found during manual test round 2 (continuing to validate the app and kit against the scaffolded `E:\SnippetVault` brain). Same convention as [[SPEC-010-test-round-1-findings]]: each confirmed finding is a numbered entry (R2-F1, R2-F2, …); at approval time, actionable findings are split into focused remediation specs for clean handoffs.

## Findings

### R2-F1 — A malformed-frontmatter artifact renders as a silently degraded "UNKNOWN" card (confirmed)

**Severity:** medium. **Status:** confirmed (code + observed in the SnippetVault Taskboard).

**Observed:** the Taskboard card for `TASK-004` shows id `UNKNOWN`, title `TASK-004`, and no priority/area/spec/date, unlike the sibling task cards. The file `tasks/planned/TASK-004.md` actually contains complete, intended frontmatter (`id: TASK-004`, a real title, priority, area, spec, etc.).

**Root-cause analysis (code-confirmed):**
- The frontmatter is **invalid YAML**. Line 3 is `title: React UI: list, detail, editor, search, tag filter, copy`. The unquoted `: ` inside the value makes YAML read it as a nested mapping, so `serde_yaml::from_str` fails for the **entire** block.
- On failure, [`parse_frontmatter`](src-tauri/src/commands/parser.rs:54) returns an **empty** frontmatter map (plus a diagnostic). `build_tasks` then falls back: `id` → `"UNKNOWN"`, `title` → the file stem (`TASK-004`), and every other field → empty. Hence the degraded card.
- The detection works: [`build_diagnostics`](src-tauri/src/commands/contract.rs:822) walks all `.md` files and surfaces the parse diagnostic, so a "Malformed YAML frontmatter" warning for `tasks/planned/TASK-004.md` should appear in the Pulse DIAGNOSTICS panel.

**The two facets:**
1. **Trigger (agent conformance + kit hardening).** The agent wrote an unquoted colon in `title:`. Titles containing a colon (`"X: subtitle"`) are extremely common, so this is a high-likelihood recurring mistake. The kit templates show `title: Feature or work item title` with no quoting guidance. This is a stronger, more likely variant of SPEC-010 F-5.
2. **App robustness / UX (the reported symptom).** When frontmatter fails to parse, the **card** degrades silently to a misleading `UNKNOWN`/filename card with no on-card indication that the artifact is malformed. A user reading the board sees "incomplete," not "broken." The Pulse diagnostic exists but is disconnected from the card.

**Scope check:** only `TASK-004` is affected in the current brain; a scan of the whole `.lmbrain/` found no other unquoted-colon frontmatter values.

**Proposed remediation (split at approval):**
- *Kit (prevents recurrence):* quote the `title` field in the kit templates (e.g. `title: "Feature or work item title"`) and/or add a one-line note that frontmatter values containing `:` must be quoted. Low-risk, high-leverage.
- *App (robustness/UX, optional):* when an artifact's frontmatter fails to parse, mark its card as malformed (a warning badge / "malformed frontmatter" affordance and, ideally, the diagnostic message) instead of silently showing an `UNKNOWN` card with partial data.
- *Data (immediate, in the test repo):* the test agent should quote the `TASK-004` title to fix that specific card — this is the SnippetVault Project Lead's job, not an LMBrain change.

**Note:** the parse-failure detection itself is correct (a positive checkpoint); the gap is the card's silent, misleading degradation and the ease of producing the malformed input.

### R2-F2 — Windows `\\?\` verbatim prefix leaks into displayed file paths (confirmed)

**Severity:** low-medium. **Status:** confirmed (code + observed in the Task drawer).

**Observed:** opening a task detail shows SOURCE FILE as `\\?\E:\Git\SnippetVault\.lmbrain\tasks\planned\...002.md`. The leading `\\?\` is confusing (the "question mark" the operator noticed) and looks broken.

**Root-cause analysis (code-confirmed):** `\\?\` is the Windows extended-length / "verbatim" path prefix returned by Rust's `Path::canonicalize()`. [`initialize_kit`](src-tauri/src/commands/workspace.rs:196) calls `root.canonicalize()` and then returns `validate_workspace(&root.to_string_lossy())`, so the workspace path handed to the frontend already carries `\\?\`. Every artifact `path` is then `root.join(...).to_string_lossy()` (e.g. [contract.rs:82](src-tauri/src/commands/contract.rs:82)), inheriting the prefix. `filesystem.rs` also canonicalizes ([filesystem.rs:27](src-tauri/src/commands/filesystem.rs:27), [:55](src-tauri/src/commands/filesystem.rs:55)). Once the canonical path is stored as the workspace root (and likely persisted in the workspace registry), it propagates to all views and survives reopen.

**Why it matters:** Windows-only, but it makes every displayed/copied path look malformed (SOURCE FILE, and any other path surfaced in the UI). It is purely the display/normalization of the path; file operations still work.

**Proposed fix:** strip the verbatim prefix when establishing the workspace root so downstream paths are clean — e.g. use `dunce::canonicalize` (returns non-verbatim paths on Windows) or strip a leading `\\?\` (and `\\?\UNC\`) after `canonicalize()`. Apply at the single boundary where the root is set (`initialize_kit`, and the path-safety canonicalization in `filesystem.rs`) so all artifact paths inherit the clean form. Add a test that a normalized workspace path does not begin with `\\?\`.

### R2-F3 — Roadmap view ignores `ROADMAP.md` and invents milestones from frontmatter (confirmed)

**Severity:** medium. **Status:** confirmed (code + observed).

**Observed:** the Roadmap view shows `M-01` (0/4 tasks, 0%, chip `SPEC-001`) and a second `Unassigned` group (0/1 tasks). It is "not clear at all": no milestone title, outcome, target date, or description — just an identifier, a progress bar, and spec-id chips. A separate `Unassigned` milestone also appears.

**Root-cause analysis (code-confirmed):**
- **`ROADMAP.md` is never parsed.** There is no roadmap command/parser; the backend only checks that the file exists ([workspace.rs:158](src-tauri/src/commands/workspace.rs:158)) and warns if missing. The rich milestone definition in `ROADMAP.md` (outcome, target, risks, associated specs) is never read or surfaced.
- **Milestones are synthesized from frontmatter.** [`RoadmapView`](src/components/Roadmap/RoadmapView.tsx:18) groups specs/tasks by their `milestone` field (`const m = spec.milestone || "Unassigned"`). So the view can only ever show bare milestone identifiers + counts + spec chips — never the roadmap narrative the operator actually wrote.
- **The `Unassigned` group** is produced by artifacts whose `milestone` is empty. Here it is `TASK-004`, whose entire frontmatter fails to parse (see R2-F1), so its milestone is empty and it is bucketed as a phantom `Unassigned` milestone. Fixing R2-F1 removes this specific instance.

**Why it matters:** Roadmap is a primary navigation surface. As built, it duplicates a weak slice of the Taskboard (grouping by milestone) and discards the authored roadmap, so it reads as empty/unclear. And because milestones are invented from data rather than defined by `ROADMAP.md`, any malformed or unmapped artifact silently creates a phantom milestone group.

**Proposed fix (two parts; depth is an operator decision):**
- *Backend:* add a `get_roadmap` command that parses `ROADMAP.md` milestone entries (id, title/outcome, target, status, risks, listed specs).
- *Frontend:* render the defined milestones from `ROADMAP.md` as the source of truth — show outcome/target/status — and associate specs/tasks to them; show artifacts that map to no defined milestone explicitly (e.g. an "unmapped" note) rather than as a phantom milestone. Keep the task progress bar.

**Relationship:** the `Unassigned`/"unknown" group is partly a symptom of R2-F1; the clarity gap is an independent RoadmapView limitation.

### R2-F4 — "Copy fix prompt" action on diagnostics (enhancement, operator-requested)

**Kind:** enhancement. **Priority:** medium. **Status:** requested.

**Request:** when a malformed document is reported in the diagnostics panel, add a button on that diagnostic that opens a copyable text area containing a ready-made prompt asking an agent to fix that specific malformed item. The operator copies it and hands it to the responsible agent.

**Context (code):** diagnostics are rendered in [`ProjectPulse`](src/components/Pulse/ProjectPulse.tsx:254); each `KitDiagnostic` carries `message` (the parser error) and `path` (the offending file). Both are exactly what a fix prompt needs. The data is already present client-side — no backend change required.

**Proposed design:**
- Add a small action (e.g. a "Copy fix prompt" button/icon) on each diagnostic row.
- Clicking it reveals a read-only, selectable text area pre-filled with a generated prompt, plus a one-click "Copy" affordance. Purely local; no external calls.
- The prompt is tailored to the diagnostic type. For malformed frontmatter, generate something like:

  > The LMBrain document `<path>` has malformed YAML frontmatter and the app cannot parse it. Parser error: `<message>`. Fix only this file's frontmatter so it is valid YAML, preserving the intended field values. Follow the LMBrain contract: frontmatter reference fields use bare IDs, not `[[wikilinks]]`; any value containing a colon must be quoted. Do not change the document body or any other file.

  For a status mismatch, generate a prompt describing the directory-vs-frontmatter status conflict and the two ways to resolve it.

**Relationship:** complements R2-F1 (malformed-artifact handling). R2-F1 flags a malformed artifact on its card; R2-F4 gives the operator a one-click remediation prompt from the diagnostic. They can be delivered together.

**Scope (decided 2026-06-22):** type-aware for **all** diagnostic types — malformed frontmatter (fix YAML) and status mismatch (resolve directory-vs-frontmatter conflict).

### R2-F5 — Detail modal + approve/reject for proposed items (enhancement, architecture-affecting)

**Kind:** enhancement. **Priority:** medium. **Status:** requested — needs an architectural decision before the write part.

**Request:** for items in `proposed` state that the operator must approve (ADRs, and by extension specs / agent proposals / MCP proposals), open the **full document in a modal**, with **Approve / Reject** actions by click. Rejected items get the R2-F4 corrective-prompt action.

**Project Lead analysis — this is partly a scope expansion, flagged per [[QUALITY]]:**

The request has three parts with very different impact:

1. **Detail modal (read-only) — compatible with M-01.** Showing the full artifact document in a modal is pure read. It can ship now and is generally useful (not only for proposed items). No architectural concern.
2. **Approve / Reject (write) — beyond M-01's read-only scope.** [[ADR-001-desktop-first-tauri]] *does* permit the app to read **and write** `.lmbrain/` in principle, so this is not an architecture violation. But the current milestone **M-01 is explicitly read-only** ("status changes happen in Markdown"), and the app currently has **no per-artifact write command** (only the one-time bootstrap `initialize_kit`). Adding approve/reject turns the visualizer into an editor — a deliberate product step that should be its own milestone, not folded silently into a bug-fix round.
3. **Rejected → corrective prompt.** Natural reuse of R2-F4; depends on (2).

**What "Approve/Reject" correctly requires (not a one-line status flip):**
- Update `status` frontmatter **and physically move the file** to the matching status directory — the CONTRACT invariant requires filesystem and `status` to agree.
- Bump `updated`.
- Honor cross-artifact invariants: e.g. CONTRACT says "a spec can be `accepted` only when an associated review is `accepted`" — so an app "approve" on a spec cannot blindly set `accepted`. ADRs move `proposed → accepted`/`rejected`-equivalent; ADRs have no `rejected` status (allowed: proposed/accepted/superseded/deprecated), so "reject" semantics must be defined per artifact type.
- Interplay with the file watcher (avoid reload races) and git safety; consider an undo/confirmation.

**Recommendation:**
- Ship **Part 1 (read-only detail modal)** within the round-2 remediation.
- Treat **Parts 2–3 (write/approval workflow)** as a new milestone **M-02**, gated by a **new ADR** that records the read→read-write decision, the per-artifact approve/reject state machine, and the file-move/invariant handling. The Project Lead should not expand established technical direction without an explicit operator decision and a recorded ADR.

## Operator decisions
- **R2-F1 scope (decided 2026-06-22):** full coverage — both (a) kit/template hardening to prevent recurrence **and** (b) the in-card malformed indicator so a malformed artifact is visibly flagged instead of silently degraded. Both will be carried into the round-2 remediation handoff.
- **R2-F3 scope (decided 2026-06-22):** full solution — `ROADMAP.md` becomes the source of truth. Add a backend `get_roadmap` parser and drive the Roadmap view from the defined milestones (outcome, target, status, risks), associating specs/tasks and explicitly surfacing unmapped artifacts (no phantom milestone).
- **R2-F4 scope (decided 2026-06-22):** the "Copy fix prompt" action appears on **all** diagnostic types with a type-aware prompt (malformed frontmatter + status mismatch).
- **R2-F5 scope (decided 2026-06-22):** ship Part 1 (read-only detail modal) in the round-2 remediation. Parts 2–3 (approve/reject writes + corrective prompt on rejected items) are deferred to a new milestone **M-02**, preceded by a new ADR defining the read→read-write decision and the per-artifact approve/reject state machine (status change + file move + cross-artifact invariants). Project Lead follow-up: author that ADR and the M-02 milestone entry before any write-capability handoff.
