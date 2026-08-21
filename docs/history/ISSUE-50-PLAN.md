# LMBrain 4.0.0 — Layout and density discovery for issue #50

Discovery and visual-system definition. This document does **not** authorize an
application-wide CSS rewrite: it ends with a prioritized breakdown that becomes
separate implementation issues once the operator approves the direction.

Everything in the audit is measured against the current tree (`codex/4.0.0-bugs`),
not estimated. The default window is 1440×900 (`src-tauri/tauri.conf.json`), the
sidebar is a fixed 236 px (`src/components/Layout/Sidebar.tsx`), so the content
region is **1204 px at default size** and **1684 px on a 1920 display**.

---

## 1. Problem statement

Every page invents its own layout with inline styles. There is no layout
vocabulary anywhere in the product: `src/styles/global.css` defines colours,
fonts, scrollbars, and animations — and not one spacing, width, radius, or type
token. The result is three distinct problems, in order of severity:

1. **Unexplained side voids.** Most workspace pages centre a fixed narrow column
   inside a much wider region, so a third of a wide screen renders nothing.
2. **No shared scale.** Spacing, radii, and type sizes are picked per component,
   so comparable elements on different pages are visibly different sizes.
3. **No responsive or contrast contract.** Two media queries exist in the entire
   application, and the most-used secondary text colour fails WCAG AA.

---

## 2. Measured audit

### 2.1 Content width and side voids

| Page | Shell | Max width | Void per side @1440 | Void per side @1920 |
|---|---|---|---|---|
| Pulse | centred | 1320 | 0 | 182 |
| Insights | centred | 1180 | 12 | 252 |
| Roadmap | centred | 1100 | 52 | 292 |
| Harnesses | centred | 1040 | 82 | 322 |
| Skills | centred | 980 | 112 | 352 |
| Reviews | centred | 920 | 142 | **382** |
| Decisions | centred | 920 | 142 | **382** |
| Agents | centred | 920 | 142 | **382** |
| MCP | centred | 920 | 142 | **382** |
| Spec detail | centred | 880 | 162 | 402 |
| Findings | full-bleed | — | 0 | 0 |
| Kit Feedback | full-bleed | — | 0 | 0 |
| Board | full-bleed | — | 0 | 0 |
| Settings | full-bleed | — | 0 | 0 |
| Wiki | split pane | — | 0 | 0 |
| Design | split pane | — | 0 | 0 |
| Repository | grid + own CSS | — | 0 | 0 |
| Sessions | full-bleed | — | 0 | 0 |

**Eight distinct content widths** (880, 920, 980, 1040, 1100, 1180, 1320, plus
full-bleed) for what are really three kinds of page. On a 1920 display, Reviews,
Decisions, Agents, and MCP leave **45 % of the content region empty** while
rendering single-column card lists that would comfortably hold three columns.

Findings and Kit Feedback are the counter-example: same card-list content, no max
width at all. Two pages built from the same pattern sit at opposite extremes.

### 2.2 Gutters and vertical rhythm

Four different page paddings for the same job:

| Padding | Pages |
|---|---|
| `24px 36px 70px` | Reviews, Decisions, Agents, MCP, Skills, Insights, Roadmap, Harnesses |
| `22px 28px 70px` | Findings, Kit Feedback |
| `26px 30px 60px` | Pulse |
| `22px 36px 70px` | Spec detail |
| `20px 24px 14px` + `16px 24px` | Board (header / columns) |
| `22px 30px 0` | Settings |
| `15px 11px` / `18px 14px` | Wiki / Design side panes |

The 60–70 px bottom padding is a scroll-comfort hack applied inconsistently; it
is roughly three times the top padding on every page that has it.

### 2.3 Scales that do not exist

Counted across `src/components/**/*.tsx`:

- **Type:** 14 distinct sizes — 10, 10.5, 11, 11.5, 12, 12.5, 13, 13.5, 14, 15,
  16, 17, 18, 24. The half-pixel steps (10.5/11.5/12.5/13.5) are the third, fifth,
  and sixth most common sizes in the product; 11.5 and 12 are used ~80 times each
  for the same kind of content.
- **Spacing:** 11 distinct `gap` values (4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16)
  and ~40 distinct `padding` strings, the most common being `10px 12px`,
  `7px 11px`, `3px 8px`, and `2px 6px` — four different paddings for chips and
  badges that look like the same component.
- **Radius:** 10 distinct values (4, 5, 6, 7, 8, 9, 10, 11, 12, 13) plus `999`.
  A 5 px and a 7 px radius are not a design decision at this scale; they are
  noise.

### 2.4 Responsive and accessibility findings

- **Two media queries in the whole application**, both in
  `src/components/Repository/RepositoryView.css` (1120 px, 700 px). Every other
  page is fixed-layout: below ~1100 px the multi-column grids (Insights uses
  `repeat(5, minmax(0, 1fr))`) compress rather than reflow.
- **Contrast.** Using the WCAG formula against `--bg-primary` (#0b0a0d):
  - `--text-secondary` #9a949f ≈ **6.7:1** — passes AA.
  - `--text-tertiary` #6c6671 ≈ **3.6:1** — **fails AA for normal text**, and it
    is the colour of every page description at 13–13.5 px.
  - `--text-muted` #56525b ≈ **2.7:1** — fails AA and AA-large; used for section
    labels such as the sidebar "Workspace" caption.

  These are computed values and should be re-verified in the browser during
  implementation, but the ranking is not in doubt: the two lowest-contrast text
  tokens carry most of the product's supporting copy.
- **Focus.** There is no shared focus-visible treatment; only
  `.modal-close-button` defines one (`global.css`). Denser layouts make missing
  focus rings materially worse.

### 2.5 Prioritized inconsistencies

| # | Issue | Severity | Evidence |
|---|---|---|---|
| P1 | Card-list pages waste 30–45 % of a wide screen | High | §2.1 |
| P2 | Identical content, opposite treatments (Findings/Feedback vs Reviews/Decisions) | High | §2.1 |
| P3 | `--text-tertiary` body copy below AA contrast | High | §2.4 |
| P4 | No spacing/type/radius scale; ~40 ad-hoc paddings | Medium | §2.3 |
| P5 | Four page paddings and an arbitrary 70 px bottom gap | Medium | §2.2 |
| P6 | No responsive behaviour outside Repository | Medium | §2.4 |
| P7 | No shared focus-visible style | Medium | §2.4 |
| P8 | Page header markup duplicated in 14 components | Low | §3.3 |

---

## 3. Proposed system

### 3.1 Tokens (`src/styles/global.css`)

```css
:root {
  /* Spacing — 4 px base, 7 steps, no in-between values */
  --space-0: 2px;  --space-1: 4px;  --space-2: 8px;   --space-3: 12px;
  --space-4: 16px; --space-5: 24px; --space-6: 32px;  --space-7: 48px;

  /* Radius — 3 steps + pill */
  --radius-sm: 6px; --radius-md: 9px; --radius-lg: 12px; --radius-pill: 999px;

  /* Type — 7 steps, integers only */
  --text-2xs: 10px; --text-xs: 11px; --text-sm: 12px; --text-md: 13px;
  --text-lg: 15px;  --text-xl: 18px; --text-2xl: 24px;

  /* Layout */
  --page-reading: 780px;   /* prose / single-artifact detail */
  --page-wide: 1600px;     /* dense workspace surfaces */
  --page-wide-xl: 2000px;  /* relaxed cap above 2200px viewports, see §3.6 */
  --page-gutter: var(--space-5);       /* ≤1280px */
  --page-gutter-wide: var(--space-6);  /* >1280px */
  --page-top: var(--space-5);
  --page-bottom: var(--space-7);
}
```

The type scale collapses 14 sizes onto 7. The mapping is mechanical: 10 → 2xs,
10.5/11/11.5 → xs, 12/12.5 → sm, 13/13.5/14 → md, 15/16 → lg, 17/18 → xl,
24 → 2xl.

### 3.2 Page archetypes

**A — Reading column** (`--page-reading`, 780 px): prose and single-artifact
detail, where line length is the constraint. Members: Spec detail, Wiki content
pane, artifact detail modal body.

**B — Dense workspace** (fluid to `--page-wide`): the default. Cards flow into a
responsive grid instead of a centred single column:

```css
grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
```

Members: Reviews, Decisions, Findings, Kit Feedback, Agents, MCP, Skills,
Insights, Roadmap, Harnesses, Pulse. At the 1440 default this yields 3 columns;
at 1920, 4; at 1024, 2; below 760 px, 1. The void disappears without any page
becoming less readable, because the readable unit is the card, not the column.

**C — Full-bleed surface** (no max width): pages whose panes own their own
scrolling. Members: Board, Sessions, Wiki shell, Design shell, Repository,
Settings. These are the justified exceptions and stay as they are structurally.

### 3.3 Primitives (`src/components/Shared/`)

| Primitive | Replaces |
|---|---|
| `<PageShell archetype="reading\|dense\|full">` | the 18 hand-written outer `<div>`s with per-page `maxWidth`/`padding` |
| `<PageHeader title description actions>` | the h1 + p + action-row block duplicated in 14 views |
| `<PageSection title description>` | ad-hoc section headings and `marginBottom` values |
| `<Toolbar>` | the filter/search rows in Findings, Feedback, Board, Skills |
| `<CardGrid minColumnWidth>` | `flexDirection: column, gap: 9\|10\|14` card lists |

### 3.4 Wireframes

Archetype B, dense workspace at 1684 px — today (left) vs proposed (right):

```
 today                                    proposed
┌──────┬──────────────────────────────┐  ┌──────┬──────────────────────────────┐
│ nav  │        ╎ Reviews    ╎        │  │ nav  │ Reviews          [filters]   │
│      │  382px ╎ ┌────────┐ ╎ 382px  │  │      │ ┌──────┐ ┌──────┐ ┌──────┐   │
│      │  void  ╎ │ card   │ ╎ void   │  │      │ │ card │ │ card │ │ card │   │
│      │        ╎ ├────────┤ ╎        │  │      │ ├──────┤ ├──────┤ ├──────┤   │
│      │        ╎ │ card   │ ╎        │  │      │ │ card │ │ card │ │ card │   │
│      │        ╎ └────────┘ ╎        │  │      │ └──────┘ └──────┘ └──────┘   │
└──────┴──────────────────────────────┘  └──────┴──────────────────────────────┘
   ~45 % of the region unused                 same cards, 3 per row
```

Archetype A stays a centred column, deliberately:

```
┌──────┬──────────────────────────────┐
│ nav  │      ┌────────────────┐      │   780px reading measure
│      │      │ SPEC-12 title  │      │   ≈ 70–80 characters
│      │      │ prose body …   │      │   void here is intentional
│      │      └────────────────┘      │
└──────┴──────────────────────────────┘
```

### 3.5 Density targets and constraints

- Card padding `--space-3`; card gap `--space-3`; section gap `--space-5`.
- Body copy `--text-md` (13 px) minimum for anything the operator reads
  continuously; `--text-xs` only for metadata chips and monospace ids.
- Interactive targets ≥ 28×28 px, never below 24×24 px.
- Reading measure 60–85 characters, enforced by archetype A only.
- Every focusable element gets the shared `:focus-visible` ring
  (`outline: 2px solid var(--accent-light); outline-offset: 2px`).
- Body text must reach 4.5:1. `--text-tertiary` is lightened to approximately
  #8b8592 (decision 1 in §7), which changes the product's visual tone on every
  page. The exact value is confirmed by measurement during issue 6.

### 3.6 Responsive matrix

Column counts below are computed from the 360 px card minimum against the content
region (viewport − 236 px sidebar − gutters), capped by `--page-wide`.

| Viewport | Content region | Cap applied | Columns | Void per side |
|---|---|---|---|---|
| 2560 | 2324 | `--page-wide-xl` 2000 | 5 | 162 |
| 1920 | 1684 | `--page-wide` 1600 | 4 | 42 |
| 1440 (default) | 1204 | none | 3 | 0 |
| 1280 | 1044 | none | 2 | 0 |
| 1024 | 788 | none | 2 | 0 |
| 768 | 532 | none | 1 | 0, split panes collapse to a single pane with a toggle |
| <768 | — | — | out of scope: the desktop shell defaults to 1440×900 and has no mobile target |

**Why two caps.** A single 1600 cap leaves 362 px of void per side at 2560 —
almost exactly the problem this issue exists to fix, just moved to a larger
display. Relaxing to `--page-wide-xl` (2000 px) above a 2200 px viewport keeps
the void at 162 px and adds a fifth column, without letting a 1440 screen turn
into a sparse grid. This is the one place where the layout system has two width
rules instead of one; the alternative is accepting the void at 2560.

---

## 4. Migration strategy

Incremental, one archetype at a time, each step shippable and independently
reversible:

1. **Tokens only.** Add the CSS variables. No component changes. Zero visual diff.
2. **Primitives.** Add `PageShell`/`PageHeader`/`CardGrid` and adopt them on
   **one** page (Decisions — the smallest card-list view, 160 lines) as the
   reference.
3. **Archetype B rollout**, one page per commit, in ascending complexity:
   Decisions → Skills → Reviews → Agents → MCP → Findings → Kit Feedback →
   Harnesses → Roadmap → Insights → Pulse.
4. **Archetype A** (Spec detail, Wiki content, artifact modal).
5. **Archetype C audit**: keep the structure, only align gutters, radii, and type
   to the tokens.
6. **Contrast and focus pass** across everything.

Steps 3–5 are mechanical once step 2 lands. Any page can stop at its current
state without breaking the others.

---

## 5. Technical impact

- `src/styles/global.css` — tokens, shared `:focus-visible`, contrast changes.
- `src/components/Shared/` — new primitives (~5 files) and their tests.
- 18 page components — outer shell and header markup replaced; internal content
  untouched in steps 1–3.
- `src/components/Repository/RepositoryView.css` — its two media queries fold
  into the shared breakpoints.
- No backend, contract, MCP, or data-model change. No change to information
  architecture, which stays with issues #46, #48, and #49.

**Verification.** There is no visual-regression tooling in the repository today,
and adding one is a larger decision than this issue should make. Proposal:
(a) unit tests asserting each page renders through `PageShell` with the expected
archetype, so a page cannot silently regress to a hand-rolled container;
(b) a documented manual comparison matrix at 1280×800, 1440×900, 1920×1080, and
2560×1440, captured once before and after each rollout step.

---

## 6. Implementation breakdown (follow-up issues)

| # | Issue | Size | Acceptance criteria |
|---|---|---|---|
| 1 | Layout tokens + shared focus ring | S | **4.0.0.** Tokens exist in `global.css`; a shared `:focus-visible` rule applies to all focusable elements; no visual diff on any page beyond focus rings |
| 2 | Layout primitives + Decisions reference | M | **4.0.0.** `PageShell`, `PageHeader`, `CardGrid`, `Toolbar` exist with tests; Decisions renders through them; Decisions shows 3 card columns at 1440 and 4 at 1920 |
| 3 | Archetype B rollout | L | The 11 archetype-B pages use `PageShell`; no page leaves >120 px unexplained void per side at 1440; column counts match §3.6 |
| 4 | Archetype A pages | S | Spec detail, Wiki content, artifact modal use the reading archetype; measure stays 60–85 characters |
| 5 | Archetype C alignment | M | Board, Sessions, Wiki, Design, Repository, Settings use token gutters/radii/type; scroll and modal layering unchanged |
| 6 | Contrast and type-scale sweep | M | No body text below 4.5:1; the 14 font sizes collapse to the 7 tokens; no half-pixel sizes remain |
| 7 | Layout documentation | S | `docs/product.md` (or a new `docs/design-system.md`) records archetypes, tokens, exceptions, and the viewport matrix |

Issues 1 and 2 are prerequisites for everything else and ship in 4.0.0. Issues
3–7 carry over to 4.1.0; issue 6 can run in parallel with 3–5.

**Status.** All seven issues are implemented on `codex/4.0.0-bugs`:

| # | Delivered |
|---|---|
| 1 | Tokens and the shared focus ring in `src/styles/global.css` |
| 2 | `src/components/Shared/PageLayout.tsx` + `layout.css`; Decisions as the reference page |
| 3 | All 11 archetype-B pages migrated to `PageShell`/`CardGrid` |
| 4 | Spec detail, Wiki content pane, and the artifact modal on the reading measure |
| 5 | Board, Sessions, Wiki, Design, Repository, Settings aligned to token gutters/radii/type |
| 6 | 451 font sizes collapsed onto the 7 type tokens; `--text-tertiary` and `--text-muted` raised above 4.5:1; 84 hardcoded text hexes routed through tokens |
| 7 | `docs/design-system.md`, linked from `docs/README.md` |

Enforced by `src/__tests__/layoutContract.test.ts`.

---

## 7. Operator decisions (approved)

The direction in §3–§6 is approved. The five open questions were decided as
follows:

1. **Contrast vs tone — lighten.** `--text-tertiary` moves to approximately
   #8b8592 so supporting copy clears 4.5:1. The tonal change across every page is
   accepted deliberately: density makes low-contrast body text worse, and this
   product is adding density. Exact value to be confirmed by measurement in the
   browser during issue 6.
2. **`--page-wide` — 1600 px**, with the `--page-wide-xl` relaxation above
   2200 px viewports documented in §3.6. Fully fluid was rejected: it turns a
   reading list into a browsing grid on large displays.
3. **Card minimum width — 360 px.** Review and finding titles are long enough
   that 320 px would trade wrapped titles for one extra column.
4. **Milestone scope — all seven issues in 4.0.0.** Initially split (1–2 in
   4.0.0, the rollout in 4.1.0), then revised by the operator after reviewing the
   Decisions reference page in a running build: the whole system ships in 4.0.0.
   The accepted trade is that every page changes inside a milestone that already
   carries 12 issues awaiting validation; the mitigation is the per-page rollout
   order in §4, where each step is independently reversible.
5. **Visual regression — manual matrix.** The documented comparison matrix in §5
   is accepted for now. Screenshot-based tooling is a larger decision than this
   issue should make, and the `PageShell` archetype tests block structural
   regressions regardless.

**Consequence for 4.0.0:** only issues 1 and 2 of §6 are in scope. Their
acceptance criteria are unchanged. The remaining five carry over to 4.1.0 and
should be opened as GitHub issues against that milestone when it is created.
