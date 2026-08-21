# Issue #48 — Decisions view: discovery and implementation plan

Milestone 4.0.0. Companion to `ISSUE-50-PLAN.md` (layout and density) and
`ISSUE-49-64-PLAN.md` (governed spec metadata), whose primitives this plan
consumes rather than reinventing.

---

## 1. Problem statement

The Decisions page renders every ADR as an identical card in one flat grid. It
answers "which decision records exist" and nothing else. The issue frames this
as an information-architecture problem, and it is — but the audit found that the
architecture problem sits on top of a **data-integrity problem**, and fixing the
layout without fixing the data would produce a better-looking page that is still
wrong.

### 1.1 What the audit measured

| Observation | Evidence |
| --- | --- |
| `supersedes` and `superseded_by` are declared by the ADR template and read by nobody | Zero occurrences of either identifier in any `.rs` file in the workspace |
| The `Adr` model cannot carry supersession | `src-tauri/src/models/adr.rs` has 12 fields; neither is present |
| Two live supersession claims contradict the status field | ADR-010 declares `supersedes: [ADR-009]` and ADR-014 declares `supersedes: [ADR-013]`; **ADR-009 and ADR-013 are both still `accepted`** with `superseded_by: []` |
| No ADR in the workspace has ever reached `superseded` | `grep '^status:' .lmbrain/decisions/*.md` → 11 `accepted`, 3 `proposed`, 0 everything else |
| Nothing technical prevented the correct state | The transition matrix already allows `accepted → superseded` (`transitions.rs:1103`) |
| A proposal has already retired a decision | ADR-014 is `proposed` yet declares `supersedes: [ADR-013]` |
| `rejected` renders as if pending | `AdrStatus::Rejected` exists in Rust; `STATUS_COLORS` in `DecisionsList.tsx:6` has no `rejected` key, so it falls through to the `proposed` grey |
| Status is displayed twice per card | The badge at `DecisionsList.tsx:112` and the subtitle at `:121` |
| The inbound relationship graph already exists and is unused | Specs and findings carry `related_decisions`, resolved in `context.rs` for MCP context; the Decisions page reads none of it |

The operational consequence: **the page today tells the operator that 11
decisions are authoritative when at most 9 are.** Two retired decisions are
presented with the same green `ACCEPTED` badge as live ones. That is the
concrete harm behind the issue's abstract complaint, and it is the reason the
supersession model leads this plan rather than following it.

### 1.2 Why the data drifted

The relationship is written on one side by hand. An agent authoring ADR-010
records `supersedes: [ADR-009]` in its own frontmatter — the file it is already
editing — and has no reason, and no mechanism, to reach into ADR-009 and retire
it. There is no verb that makes the two sides move together, so they don't.

This is the same failure shape as the tasks/specs drift that ADR-005 retired:
**two artifact layers the agents must keep in sync, which they handle poorly.**
The remedy that worked there applies here — remove the possibility of drift
rather than instruct against it.

---

## 2. Operator workflows

Four workflows, in descending frequency:

1. **"What is authoritative right now?"** — the dominant read. Answering it
   requires trusting the status field, which today cannot be trusted.
2. **"What needs my attention?"** — proposals awaiting an accept/reject
   decision, and integrity problems.
3. **"Why is this the way it is?"** — arriving from a spec, a finding, or a
   review at the ADR that governs it, then reading the reasoning.
4. **"What did this replace, and what replaced it?"** — following a supersession
   chain backwards or forwards to understand how the position evolved.

Workflow 3 is the only one the current page partly serves, and only by accident:
the operator can find the card and click into the artifact detail panel.

---

## 3. Recorded decisions

Answered by the operator on 2026-08-01.

### 3.1 Supersession → governed verb plus diagnostic

A new `adr_supersede` verb writes both sides; a diagnostic surfaces the
inconsistencies that already exist. Read-only exposure was rejected because it
leaves the drift mechanism in place, and the two existing contradictions prove
the mechanism fires in practice.

**Atomicity.** Two files cannot be written atomically without a journal, which
this kit does not have and does not need. The design instead makes the failure
mode benign:

- Both artifacts are locked before either is read, **acquired in lexicographic
  ID order** so two concurrent supersessions can never deadlock.
- All validation runs before any write.
- The **superseding** ADR is written first, the **superseded** one second.

The ordering is deliberate. A crash between the two writes leaves the
superseding ADR claiming `supersedes: [X]` while X is still accepted — which is
precisely today's state, is detected by the new diagnostic, and is repaired by
re-running the verb. The opposite order would leave a decision stripped of its
authority with no successor recorded on the other side: a silent loss, invisible
to any check. Between two imperfect partial states, choose the one that is loud
and self-healing.

The verb is **idempotent**: re-running it against an already-consistent pair is
a no-op that succeeds, so the repair path is simply "run it again".

**Supersession takes effect at acceptance.** A `proposed` ADR may *declare*
`supersedes` — that is a legitimate pending claim, and ADR-014 is a real example
of one. But the verb requires the superseding ADR to be `accepted`, because a
proposal that has not been approved cannot retire anything. The diagnostic
distinguishes the two cases: a pending claim from a proposal is silent; an
accepted ADR whose declared predecessor is still accepted is a `Warning`.

### 3.2 Page structure → attention band plus grouped list

An attention band at the top carries what requires operator action; below it,
the ADRs grouped by lifecycle. Split view was rejected because it duplicates the
artifact detail panel that already exists and complicates deep-linking; a pure
timeline was rejected because it answers the historical question well and the
"what holds now" question badly.

### 3.3 Grouping → by authority

Three groups rather than five status buckets:

| Group | Contains | Behaviour |
| --- | --- | --- |
| **Authoritative** | `accepted` | Expanded, listed first |
| **Awaiting decision** | `proposed` | Expanded, second |
| **Historical** | `superseded`, `deprecated`, `rejected` | Collapsed by default, count in the header |

Five single-status groups would produce groups of one or two at the current
collection size. Chronological ordering was rejected because it interleaves
retired and live decisions in the same scan.

### 3.4 Scope → discovery and implementation together in 4.0.0

The issue reserves implementation pending operator approval of the UX
direction. This document plus the operator's answers constitute that approval,
following the precedent set by issue #50.

---

## 4. Information architecture

```
┌─ Decisions ─────────────────────────────────────────────────────────┐
│  Architecture decision records in .lmbrain/decisions/               │
│                                                                     │
│  ┌─ Needs attention ───────────────────────────────── 3 items ──┐   │
│  │  ⚠ ADR-009 is marked accepted but ADR-010 supersedes it      │   │
│  │  ⚠ ADR-013 is marked accepted but ADR-014 supersedes it      │   │
│  │  ○ ADR-006, ADR-007 await an accept or reject decision       │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  [ search ]  [ status ▾ ]  [ tag ▾ ]  [ sort: recent ▾ ]            │
│                                                                     │
│  Authoritative · 9                                                  │
│  ┌────────────────────┐ ┌────────────────────┐                      │
│  │ ADR-010  ACCEPTED  │ │ ADR-005  ACCEPTED  │                      │
│  │ Bootstrap pinned…  │ │ Retire tasks…      │                      │
│  │ 2026-07-14 · lead  │ │ 2026-06-23 · user  │                      │
│  │ ⤿ supersedes 009   │ │ ← 3 specs          │                      │
│  └────────────────────┘ └────────────────────┘                      │
│                                                                     │
│  Awaiting decision · 3                                              │
│  …                                                                  │
│                                                                     │
│  ▸ Historical · 2                                       (collapsed) │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.1 Attention band

Present only when it has content — an always-visible empty band trains the
operator to ignore it. Three item classes, in severity order:

1. **Integrity** — supersession claims that contradict the status field, and
   references to ADR IDs that do not exist.
2. **Malformed** — records whose frontmatter did not parse.
3. **Pending** — proposals awaiting an accept or reject decision.

Each item deep-links to the ADR it concerns. The band derives from workspace
state and the diagnostics already computed by `build_diagnostics`; it invents no
new severity vocabulary.

### 4.2 Card content

Ordered by what the operator scans for:

- **Authority row** — ID, status badge, and (when applicable) a malformed or
  integrity marker. `rejected` gets its own colour; the duplicate status
  subtitle is removed.
- **Title.**
- **Provenance** — decision date and decider.
- **Relationship footer** — supersession links, and a count of inbound
  references from specs and findings. Rendered only when non-empty.

### 4.3 Relationship model

| Direction | Source | Availability |
| --- | --- | --- |
| ADR → superseded ADR | `supersedes` | New: parsed by this work |
| ADR → superseding ADR | `superseded_by` | New: parsed by this work |
| ADR ↔ ADR/spec | `links` | Exists, untyped |
| spec → ADR | `related_decisions` | Exists in the model, unused by this page |
| finding → ADR | `related_decisions` | Exists in the model, unused by this page |

The inbound index (ADR → the specs and findings that cite it) is **computed in
the frontend**. `state.specs`, `state.findings` and `state.adrs` all live in the
same store, so the reverse map is a `useMemo` over data already loaded. No
backend work, no new command, no extra I/O.

### 4.4 Search, filter, sort, density

Search matches ID and title. Status and tag filters compose; sort offers
recent-first (default, on `decision_date` falling back to `updated`) and by ID.
The page keeps the `dense` archetype and `CardGrid` from issue #50, so density
and responsive behaviour need no page-specific work.

### 4.5 Degenerate states

| State | Treatment |
| --- | --- |
| No ADRs at all | Existing empty state, unchanged |
| A group is empty | Group header omitted entirely, not shown with a zero |
| Every ADR filtered out | "No decisions match these filters", with a clear-filters action |
| Malformed record | Card renders with whatever parsed, marked malformed, and is listed in the attention band |
| Reference to a nonexistent ADR | Rendered as plain text, not a link, and reported in the attention band |
| Supersession cycle | The chain renderer stops at the first repeated ID rather than recursing |

### 4.6 Accessibility

Groups are `<section>` elements with an `aria-labelledby` heading; the
collapsible historical group is a `<button aria-expanded>` controlling a region.
Status badges carry text, not colour alone. Cards remain a single focusable
control with an accessible name covering ID, title and status. The attention
band is a labelled region placed before the list in DOM order, so it is reached
first by both screen reader and keyboard. Focus rings come from the zero-
specificity `:focus-visible` rule added in #50.

### 4.7 Read state

The page is already registered in `UNREAD_PAGES`. The unread signature is
`status|updated|malformed`, so an ADR moving to `superseded` marks it unread —
which is the correct behaviour: authority changed. No change needed.

---

## 5. Technical impact

**Kit contract** (breaking-adjacent; the kit is already at 4.0.0)

- `supersedes` and `superseded_by` become read fields with an invariant:
  the two sides must agree, and a superseded ADR must not be `accepted`.
- New verb `adr_supersede`.

**`lmbrain-core`**

- `transitions.rs` — `supersede_adr`, a two-artifact governed mutation reusing
  `PathGuard`, `ArtifactMutationLock`, the identity re-check and the
  re-read-before-write concurrency check, generalized to a locked pair.
- `invariants.rs` — `supersession_is_consistent`.
- `diagnostics.rs` — `diagnose_decisions`, emitting `dangling-supersession`,
  `supersession-not-mutual`, and `unknown-decision-reference`.

**`lmbrain-mcp`** — `adr_supersede` schema and dispatch arm.

**`src-tauri`** — `supersedes` and `superseded_by` on the `Adr` model, parsed in
`build_adrs`.

**Frontend** — `src/lib/decisionIndex.ts` (pure, tested: grouping, inbound
index, attention items, chain walking with cycle protection) and a rewritten
`DecisionsList.tsx` consuming it.

**Backwards compatibility.** Both fields are optional. ADRs lacking them parse
as empty and land in the authoritative group as before. No artifact is
rewritten by this change; the two known inconsistencies surface as diagnostics
and are repaired by running the verb.

---

## 6. Implementation breakdown

| # | Item | Acceptance criteria |
| --- | --- | --- |
| 1 | Parse supersession | `Adr` carries both fields; existing ADRs without them parse to empty; contract test covers both shapes |
| 2 | `supersede_adr` in core | Locks both artifacts in ID order; rejects a non-accepted superseder, a self-supersession, and an unknown target; idempotent on an already-consistent pair; `force` audited |
| 3 | Invariant and diagnostics | `dangling-supersession` fires on ADR-009 and ADR-013; a proposal's pending claim stays silent; unknown ADR references reported |
| 4 | MCP verb | `adr_supersede` listed, schema-validated, dispatched |
| 5 | `decisionIndex.ts` | Grouping, inbound index, attention items, chain walking with cycle protection; unit-tested |
| 6 | Page rewrite | Attention band, three groups, search/filter/sort, relationship footer, `rejected` colour, duplicate subtitle removed |
| 7 | Docs | Contract, migrations, template comments, `docs/kit.md` |

---

## 7. Status

| # | Item | State |
| --- | --- | --- |
| 1 | Parse supersession | done — `Adr` model, `build_adrs`, TS type |
| 2 | `supersede_adr` in core | done — `transitions.rs`, 5 integration tests |
| 3 | Invariant and diagnostics | done — `supersession_is_consistent`, `diagnose_decisions`, 3 integration tests |
| 4 | MCP verb | done — `adr_supersede` schema and dispatch |
| 5 | `decisionIndex.ts` | done — 20 unit tests |
| 6 | Page rewrite | done — 12 component tests |
| 7 | Docs | done — contract, migrations, template, `docs/kit.md`, `docs/architecture.md` |

### Left for the operator

The workspace's own ADR-009 and ADR-013 are untouched. They now raise
`dangling-supersession` and appear in the attention band, which is the correct
first behaviour: repairing them changes project records, and that is the
operator's call. Running `adr_supersede` on each pair clears both.
