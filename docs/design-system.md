# Design system

The layout and density system introduced in 4.0.0. The discovery that produced
it, including the measured audit and the operator decisions, is in
`ISSUE-50-PLAN.md`.

## Tokens

All tokens live in `src/styles/global.css`. Components consume them through
`var(--token)`; new numeric values for spacing, radius, or type do not belong in
component styles.

| Group | Tokens | Notes |
|---|---|---|
| Spacing | `--space-0` 2px … `--space-7` 48px | 4px base, 8 steps, no in-between values |
| Radius | `--radius-sm` 6, `--radius-md` 9, `--radius-lg` 12, `--radius-pill` | |
| Type | `--text-2xs` 10 … `--text-2xl` 24 | 7 steps, integers only |
| Page geometry | `--page-reading` 780, `--page-wide` 1600, `--page-wide-xl` 2000, `--page-gutter`, `--page-gutter-wide`, `--page-top`, `--page-bottom` | |

**Text colours carry a contrast contract.** `--text-secondary` (6.7:1),
`--text-tertiary` (5.5:1), and `--text-muted` (4.9:1) all clear WCAG AA 4.5:1 on
`--bg-primary`. The last two were raised in 4.0.0 from values that failed. Do not
darken them, and do not hardcode their hex values in components: the layout
contract test fails on the legacy hexes.

**Focus.** `global.css` applies one `:focus-visible` ring to every focusable
element at zero specificity, so component-specific focus styles still win where
they exist.

## Page archetypes

Pages compose `PageShell` from `src/components/Shared/PageLayout.tsx`.

**Reading** (`archetype="reading"`) — prose and single-artifact detail, capped at
`--page-reading` so the measure stays 60–85 characters. Used by Spec detail, the
Wiki content pane, and the artifact detail modal body.

**Dense** (`archetype="dense"`) — the default workspace surface. Fluid up to
`--page-wide`, relaxing to `--page-wide-xl` above a 2200px viewport. Cards flow
through `CardGrid` into as many columns as the width allows, instead of centring
a single narrow column. Used by Board-adjacent list pages: Decisions, Reviews,
Findings, Kit Feedback, Agents, MCP, Skills, Insights, Roadmap, Harnesses, Pulse.

**Full** (`archetype="full"`) — full-bleed shells whose panes own their own
scrolling. Board, Sessions, Wiki, Design, Repository, and Settings keep their own
structure and only align gutters, radii, and type to the tokens. These are the
documented exceptions.

## Primitives

| Primitive | Use |
|---|---|
| `PageShell` | the page container and its archetype |
| `PageHeader` | page title, description, and action row |
| `PageSection` | a titled section inside a page |
| `Toolbar` | filter and search rows |
| `CardGrid` | responsive card columns; `minColumnWidth` defaults to 360 |
| `EmptyState` | the shared "nothing here" block |

`CardGrid` takes a wider minimum where cards hold prose or dense metadata — Kit
Feedback and Harnesses use 420, Skills uses 280. Anything else should justify
deviating from 360.

## Responsive behaviour

Column counts follow from the 360px card minimum against the content region
(viewport − 236px sidebar − gutters):

| Viewport | Columns | Void per side |
|---|---|---|
| 2560 | 5 | 162 |
| 1920 | 4 | 42 |
| 1440 (app default) | 3 | 0 |
| 1280 | 2 | 0 |
| 1024 | 2 | 0 |
| 768 | 1 | 0 |

Below 768px is out of scope: the desktop shell defaults to 1440×900 and has no
mobile target.

## Verification

There is no screenshot-based visual regression tooling. Two things stand in for
it:

1. `src/__tests__/layoutContract.test.ts` asserts that every page renders through
   its expected archetype, that no page centres a hardcoded column, that no
   half-pixel font sizes survive, and that the tokens and responsive rules exist.
2. A manual comparison at 1280×800, 1440×900, 1920×1080, and 2560×1440 before and
   after any change to the shared layout.

## Known exceptions

- **Icon glyph sizes** (`material-symbols-outlined`) stay numeric. They are icon
  geometry, not type, and do not follow the text scale.
- **Display numerals** above 24px (KPI values, empty-state glyphs) stay numeric
  for the same reason.
- **xterm** in `SessionTerminal.tsx` requires a numeric `fontSize`; it cannot take
  a CSS token.
- **Pulse** keeps its own main-plus-rail grid: the rail width is content-driven
  rather than card-driven.
