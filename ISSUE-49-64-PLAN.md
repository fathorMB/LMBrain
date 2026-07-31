# LMBrain 4.0.0 — Governed spec metadata: tags (#49) and effort tiers (#64)

Joint discovery. #64 states it "should be designed alongside the governed
spec-tag work in #49", and the audit below confirms why: both add
Project Lead-owned metadata to the same artifact, through the same mutation
path, validated by the same rules, displayed on the same Board card. Designed
separately they would build the same machinery twice.

The design is approved (§8). Implementation follows the breakdown in §7.

Measured against `codex/4.0.0-bugs`. Tag statistics come from the 70 specs in
this repository's own `.lmbrain/specs/`.

---

## 1. What exists today

### 1.1 The contract

`kit/.lmbrain/CONTRACT.md` makes `tags` a **required** frontmatter field on every
artifact, alongside `id`, `title`, `status`, `created`, `updated`, and `links`.
It defines no vocabulary, no validation, no ownership, and no meaning. `tags` is
required to exist and free to contain anything.

`Spec.tags` is already parsed end to end: `lmbrain-core` reads it
(`document.string_array("tags")`), `src-tauri/src/models/spec.rs` carries it, and
`src/types/index.ts` exposes it to the frontend.

### 1.2 What the field actually holds

299 tag instances across 66 of 70 specs — **150 distinct values**, averaging 4.5
per tagged spec. The distribution is a long tail: the most common value appears
11 times, and most appear once.

| Kind of value | Examples | Instances |
|---|---|---|
| Release/milestone markers | `3.1.0`, `3.1.3`, `2.8.0`, `v3`, `milestone-m02` | 36 (12 %) |
| Technology/area | `rust`, `tauri`, `ui`, `ux`, `mcp`, `markdown` | ~90 |
| Feature surface | `wiki`, `sessions`, `roadmap`, `reviews`, `agents` | ~70 |
| Work character | `testing`, `regression`, `migration`, `security`, `remediation` | ~50 |
| Ad-hoc | `round-3`, `write`, `modal` | remainder |

Two problems are visible without any judgment call:

1. **12 % of tags restate structured fields.** `3.1.0` and `milestone-m02`
   duplicate `milestone`; `rust`, `ui`, and `tauri` largely duplicate `area`.
   These will silently disagree with the real fields the moment one changes.
2. **Nothing consumes them.** The Board neither displays nor filters by tags
   (`src/components/Taskboard/TaskboardView.tsx` has only a three-state
   dependency filter: `all`, `blocked`, `ready-after`). 150 distinct values are
   being written by hand and read by nobody.

### 1.3 The mutation machinery that already exists

This is the important finding for both issues: **the governed-setter path is
already built.** `lmbrain-core/src/transitions.rs::set_field` performs, for any
frontmatter key:

- path-guarded resolution inside `.lmbrain/`;
- an artifact mutation lock, with an identity re-check after acquiring it;
- a caller-supplied validity predicate;
- typed audit fields (mutation reasons live in frontmatter, never duplicated
  into the body — an invariant added earlier in this milestone);
- `require_force_reason` for audited overrides.

`set_recommended_agent` and `set_agent_mnemonic_name` are thin wrappers over it.
Tags and effort tiers should be two more wrappers, not new infrastructure.

Likewise, **controlled-vocabulary normalization already exists**:
`lmbrain-core/src/taxonomy.rs` normalizes review finding categories through a
canonical alias map with a declared `FINDING_TAXONOMY_VERSION`. The same shape
applies here.

---

## 2. The core design decision

The two issues pull in opposite directions, and conflating them is the main risk:

- #49 wants tags that are **descriptive** — flexible planning vocabulary.
- #64 wants an estimate that is **governed** — mandatory, machine-readable,
  drives handoff recommendations, and must be "distinguishable from ordinary
  descriptive tags".

**Proposal: two planes, one mechanism.**

| | Descriptive tags | Governed attributes |
|---|---|---|
| Field | `tags: []` (existing) | `capability_tier`, `thinking_level` (new) |
| Vocabulary | hybrid: canonical core + free tail | closed, versioned |
| Required | no (may be empty) | yes, at a defined lifecycle point |
| Owner | Project Lead | Project Lead |
| Mutation | `spec_set_tags` | `spec_set_effort` |
| Wrong value | diagnostic, spec still usable | diagnostic + blocks `ready` |
| Board | chips on the card | dedicated badge, always visible |

Effort tiers are therefore **not tags**. They ride the same `set_field`
machinery, the same audit fields, and the same normalization approach, but they
are first-class typed frontmatter. This directly satisfies #64's requirement that
they not become "ordinary free-form tags", and it keeps #49 free to stay
flexible.

---

## 3. Descriptive tags (#49)

### 3.1 Vocabulary model — hybrid

Neither extreme works. Fully free-form is what produced the 150-value tail;
fully closed would need a governance ceremony for every new planning concept.

- A **canonical core** ships in the kit, versioned like the finding taxonomy
  (`SPEC_TAG_TAXONOMY_VERSION`), with an alias map.
- Values outside the core are **allowed** but reported as `unknown-tag`
  informational diagnostics, so the tail stays visible instead of invisible.
- Values that restate a structured field (`milestone`, `area`, `priority`) are
  **rejected** by validation, with the error naming the field to use instead.
  This is the rule that fixes 12 % of today's tags.

### 3.2 Normalization

Lowercase; trim; spaces and underscores to `-`; strip a leading `#`; collapse
duplicates after normalization; reject empty values. Length 2–32 characters,
matching `^[a-z0-9][a-z0-9-]*$`. Order is preserved as written but comparison is
order-insensitive.

### 3.3 Lifecycle

Tags are descriptive, so they are never mandatory, never inherited, and never
copied on status transitions. They survive lifecycle changes untouched. Changing
them is a governed mutation that records actor and reason in the audit fields —
the same as any other Lead-owned edit.

### 3.4 Board display and filtering

Cards show at most 3 chips plus a `+N` overflow, so density stays within the
system defined in `ISSUE-50-PLAN.md`. Chips are not interactive on the card; the
filter is the single control surface.

The existing dependency filter is folded into one coherent filter model rather
than sitting beside a second unrelated control:

| Axis | Semantics |
|---|---|
| Tags — include | any-of (default) or all-of, operator-selectable |
| Tags — exclude | any-of |
| Untagged | explicit toggle, since "no tags" is not expressible as a tag |
| Dependency | existing `all` / `blocked` / `ready-after` |
| Effort tier | see §4 |

Axes combine with AND; values within an axis combine as stated. Column headers
show `shown / total` whenever any filter is active, so a filtered empty column is
never mistaken for an empty status.

Filter state persists per workspace for the session. It is not written to the
artifacts and not shared — it is a view preference, not project state.

---

## 4. Effort and capability tiers (#64)

### 4.1 Two axes, not one composite

`capability_tier` answers *how much code the work touches*; `thinking_level`
answers *how carefully it must be reasoned about*. They are genuinely
independent: a two-file change to a governed invariant is small in footprint and
high in delicacy, and a large mechanical refactor is the reverse. A composite
value would need `3 × N` labels and could not express either case. Two fields
also keep each axis independently queryable and independently revisable.

### 4.2 Proposed capability taxonomy

The names are the operator's (`Luna`, `Terra`, `Sol`). The boundary is the
**expected change footprint**: how much of the codebase the work is expected to
touch. Thresholds are anchored on the fifteen commits actually delivered in this
milestone, so a Lead can calibrate against work that exists rather than against
an abstract scale.

| Tier | Expected footprint | Structure | Measured examples from this milestone |
|---|---|---|---|
| **Luna** | ≤ 2 files, ≲ 120 changed lines | No new module; edits inside existing structures | Remove the notifications bell (2 files, 33 lines); exclude reports from the wiki (2 files, 71); kit version filter (2 files, 79); focus trap in finding detail (2 files, 78) |
| **Terra** | 3–8 files, ≲ 600 changed lines | May add one module or shared helper; designs its own tests | Feedback JSON export (4 files, 124); verification gate diagnostics (4 files, 180); modal close controls (5 files, 123); remediation verification (5 files, 153); branch commit graph (8 files, 498); grouped reviews (2 files, 398) |
| **Sol** | > 8 files or > 600 changed lines, **or** spanning more than one layer (frontend + Rust core + MCP + Markdown contract) | Introduces or reshapes shared abstractions across the codebase | Unread badges (7 files, 992 lines); the layout and density system (46 files, 2 706) |

Rules that make the scale usable:

- **Files and lines are alternatives, not a conjunction.** Whichever is higher
  decides. Grouped reviews touched 2 files but rewrote 398 lines: Terra, not
  Luna. Unread badges touched 7 files with 992 lines: Sol, not Terra.
- **Layer span overrides both counts.** Work crossing frontend, `lmbrain-core`,
  the MCP server, or `CONTRACT.md` is Sol regardless of size, because the
  footprint is measured in artifacts on users' disks, not in diff lines.
- The estimate is made **before** the work, so it is a forecast. The measured
  numbers above are the calibration anchors, not a post-hoc audit; a spec is not
  re-tiered because the diff came out larger.

**A known limitation of a size-calibrated scale:** footprint does not capture
delicacy. Small changes to invariants, security-relevant paths, or migration
behaviour can carry more risk than a large mechanical refactor. That risk is
expressed on the other axis — a Luna-sized change to a governed invariant is
`Luna` + `extended`, not an inflated tier. §4.3 makes this the explicit reason
the two axes exist.

### 4.3 Thinking level

`minimal` · `standard` · `extended` · `maximum`, mapped to whatever the target
harness exposes. **The artifact never names a provider or model.** Tiers and
levels are capability classes; the mapping to concrete models lives in
machine-local configuration alongside the existing harness manifest, so a spec
committed to a repository does not encode one vendor's product names.

Each tier carries a **default level**, which the Lead may override with a
recorded reason. Requiring two explicit decisions per spec is friction that
produces filled-in-at-random values:

| Tier | Default level | Override upward when |
|---|---|---|
| Luna | `minimal` | the change is small but touches an invariant, a security path, or migration behaviour |
| Terra | `standard` | the design space is genuinely open, or acceptance criteria are ambiguous |
| Sol | `extended` | compatibility with existing on-disk artifacts is at stake |

This is where footprint and delicacy separate: the tier answers *how much code*,
the level answers *how carefully*. Constrained combinations: Sol may not be
`minimal`; Luna may not be `maximum` without a recorded reason.

### 4.4 When it becomes mandatory

**At `ready`, not at creation.** A backlog spec is often not understood well
enough to be estimated, and blocking creation would push Leads toward a
meaningless default. `spec_ready` gains a precondition: a valid tier and level
must be present. This mirrors how `depends_on` prerequisites already gate
`ready`.

Legacy specs already past `ready` are never rewritten: they report a
`missing-effort-estimate` diagnostic and are treated as "unknown", never as a
silently assumed tier.

### 4.5 Estimate versus outcome

#64 asks how a specialist reports that the estimate was wrong. These must not be
the same field, or feedback would silently rewrite Lead-owned metadata.

- `capability_tier` / `thinking_level` — the Lead's *recommendation*. Only the
  Lead changes it, through `spec_set_effort`, with a reason.
- `effort_observations` — append-only entries recorded by the implementing
  specialist, stating the tier it actually needed and why.

Observations are evidence for a future Lead revision, and derived views may
report disagreement rates. They never change the recommendation automatically.

### 4.6 What the tier must not do

A tier is a recommendation, not a guarantee and not a launcher. It must never
auto-select or auto-start a model — the kit invariant that LMBrain never spawns
agents holds unchanged.

---

## 5. Migration and compatibility

- `tags` stays a required field; no existing spec becomes invalid.
- The 36 field-restating tags in this repository are **not** auto-rewritten.
  Validation rejects them on the next governed tag mutation, and a diagnostic
  lists them so the Lead can clean them deliberately.
- New effort fields are optional in the parser and mandatory only at the `ready`
  transition, so every existing artifact keeps parsing.
- Unknown tags, unknown tiers, and malformed values degrade to diagnostics. The
  Board never hides a spec because its metadata is unrecognized.
- The kit `VERSION` and `MIGRATIONS.md` gain an entry describing the new fields
  and the `ready` precondition.

---

## 6. Technical impact

| Area | Change |
|---|---|
| `kit/.lmbrain/CONTRACT.md` | tag vocabulary and rules; `capability_tier`, `thinking_level`, `effort_observations`; the `ready` precondition; new invariants |
| `lmbrain-core/src/taxonomy.rs` | spec-tag canonical map + tier/level vocabularies, versioned |
| `lmbrain-core/src/transitions.rs` | `set_spec_tags`, `set_spec_effort` over the existing `set_field`; append-only `effort_observations` |
| `lmbrain-core/src/invariants.rs` | tag validation; effort validity; `ready` gate |
| `lmbrain-core/src/diagnostics.rs` | `unknown-tag`, `field-restating-tag`, `missing-effort-estimate` |
| `lmbrain-core/src/context.rs` | expose tier and level in spec context and handoff prompts |
| `lmbrain-mcp/src/main.rs` | `spec_set_tags`, `spec_set_effort`, `spec_record_effort_observation` |
| `src-tauri/src/models/spec.rs`, `commands/contract.rs` | parse and expose the new fields |
| `src/types/index.ts` | `Spec.capability_tier`, `thinking_level`, `effort_observations` |
| `src/components/Taskboard/TaskboardView.tsx` | chips, tier badge, unified filter model |

No change to lifecycle statuses, authority model, or the mutation-lock design.

---

## 7. Implementation breakdown

| # | Issue | Size | Acceptance criteria |
|---|---|---|---|
| 1 | Tag vocabulary, normalization, diagnostics in core | M | Canonical map versioned; normalization deterministic; field-restating tags rejected with the field named; unknown tags are informational; legacy specs still parse |
| 2 | `spec_set_tags` mutation + MCP verb | M | Lead-only; audited reason; atomic; rejects invalid values; idempotent on unchanged input |
| 3 | Effort taxonomy + fields in core | M | Tier and level vocabularies versioned; two independent axes; constrained combinations enforced; provider-neutral |
| 4 | `spec_set_effort` + `ready` precondition | M | `spec_ready` fails closed without a valid estimate; forced transitions retain blocker details; legacy specs diagnose rather than block |
| 5 | `effort_observations` append-only + verb | S | Specialist-recorded; never mutates the recommendation; attributable |
| 6 | Board tags, tier badge, unified filter | M | ≤3 chips + overflow; include-any/all, exclude, untagged; composes with dependency filter; counts reflect filters; accessible |
| 7 | Context packs and handoff prompts | S | Tier and level appear with the reason they apply; no model name in the artifact |
| 8 | Contract, migration entry, documentation | S | `CONTRACT.md`, `MIGRATIONS.md`, kit `VERSION`, and `docs/kit.md` describe vocabulary, ownership, and compatibility |

**Status: all eight implemented on `codex/4.0.0-bugs`.**

| # | Delivered |
|---|---|
| 1 | `taxonomy.rs`: `SPEC_TAG_TAXONOMY_VERSION`, normalization, alias map, starter vocabulary, `validate_spec_tags`; `diagnostics.rs`: `field-restating-tag`, `invalid-spec-tag`, `unknown-spec-tag` |
| 2 | `transitions.rs::set_spec_tags` + `spec_set_tags` MCP verb, over the shared governed-setter body |
| 3 | `EFFORT_TAXONOMY_VERSION`, tier and level vocabularies, tier-derived defaults, constrained combinations |
| 4 | `set_spec_effort` + `spec_set_effort`; `invariants::spec_effort_is_declared` gating the `ready` transition; `missing-effort-estimate` diagnostic for legacy specs |
| 5 | `record_effort_observation` + `spec_record_effort_observation`, append-only |
| 6 | `src/lib/boardFilters.ts` + Board tag chips, tier badge, unified include/exclude/untagged/tier/dependency filter, `shown/total` counts |
| 7 | `SpecContext` carries tags, tier, level, and a provider-neutral `effort_rationale`; handoff markdown renders the estimate |
| 8 | `CONTRACT.md`, `MIGRATIONS.md` (4.0.0 entry), kit `VERSION`, spec template, `docs/kit.md` |

Coverage: 9 taxonomy unit tests, 8 core integration tests, 14 filter-model tests,
8 Board UI tests.

---

## 8. Operator decisions

The design above is approved. The blocking questions were decided as follows:

1. **Tier boundary — expected change footprint.** Size, not consequence of a
   wrong guess. Thresholds are calibrated on this milestone's fifteen commits
   (§4.2), with layer span as the one override: work crossing frontend,
   `lmbrain-core`, MCP, or `CONTRACT.md` is Sol at any size. The known limitation
   — footprint does not capture delicacy — is handled on the `thinking_level`
   axis rather than by inflating tiers.
2. **Field-restating tags — rejected in validation.** The mutation fails and
   names the field to use instead. The 36 existing tags of this kind in this
   repository are never auto-rewritten: they become invalid on their next
   governed tag mutation, and a diagnostic lists them for deliberate cleanup.
3. **Mandatory at `ready`.** `spec_ready` fails closed without a valid estimate,
   mirroring the existing `depends_on` prerequisite gate. Backlog specs need no
   estimate; legacy specs already past `ready` diagnose rather than block.
4. **Four thinking levels with a tier-derived default** (§4.3), overridable with
   a recorded reason. Constrained combinations enforced: Sol is never `minimal`,
   Luna is never `maximum` without a reason.
5. **Canonical tag core — seeded from observed usage.** The kit ships a starter
   vocabulary built from the values already in use that do not restate a
   structured field, rather than an invented taxonomy or an empty map that would
   make every real tag fire an `unknown-tag` diagnostic on day one.
6. **Filter state — session-scoped per workspace.** It is a view preference, not
   project state, and unlike the read-state in #47 there is no cost to losing it
   on restart.

Decisions 5 and 6 follow the recommendations in this document and were not
separately contested; either can be revisited before the issues in §7 start.
