---
id: SPEC-010
title: Test round 1 findings catalog
status: discarded
kind: bug
priority: medium
area: desktop-app, kit
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [ADR-001, SPEC-011]
created: 2026-06-22
updated: 2026-06-22
tags: [bootstrap, scaffolding, line-endings, wiki, ux, testing]
---

# Test round 1 findings catalog

## Objective

> **Closed/archived (2026-06-22).** All findings remediated via [[SPEC-011-remediate-test-round-1-findings]] (accepted). F-1 (CRLF) end-to-end verification is now confirmed by the operator — a scaffold from the 1.0.6 installer produces an LF `.lmbrain/`. F-2 passed with no work. This document is retained as the evidence/analysis record for round 1.

Capture and remediate the defects found while validating LMBrain during manual test round 1. This spec is the **findings catalog** for round 1: each confirmed finding is recorded as a numbered entry below and carried into the acceptance criteria. Findings may be appended until the operator approves the catalog. Findings are heterogeneous; at approval time each remediation finding is expected to be **split into a focused follow-up spec** for a clean specialist handoff (one cohesive change per handoff per [[QUALITY]] and the LMBrain workflow). This document tracks the round, not a single cohesive change.

## Context

Test setup: a separate repository (`E:\Git\SnippetVault`) was scaffolded **through the desktop app's bootstrap feature**, then validated by the Project Lead against the canonical source kit at `kit/.lmbrain/`. A second agent will act as Project Lead inside the test repository; this Project Lead (for LMBrain itself) validates that agent and triages bugs found in LMBrain (app and kit).

The bootstrap feature is the first feature under test.

## Findings

### F-1 — Bootstrap output uses CRLF while the canonical kit is LF (confirmed)

**Severity:** medium. **Status:** confirmed.

**Observed:**
- Structure and content of the scaffolded `.lmbrain/` are correct: 60/60 files present, no missing or extra files, every status directory and `.gitkeep` in place, `VERSION` = `1.0.5`, `PROJECT.md` is the empty template, agent registry contains only `project-lead` (no LMBrain-specific profiles). Content is byte-identical to source **except for line endings**.
- The canonical source `kit/.lmbrain/` is **LF**; the scaffolded output is **CRLF** in 59/60 files (the file without CR is `VERSION`).

**Root-cause analysis:**
- The app copy path is byte-faithful: [`copy_directory`](src-tauri/src/commands/workspace.rs:231) uses `std::fs::copy`, which does **not** convert line endings. The CRLF therefore did **not** originate in the copy logic — the source the app read was already CRLF.
- The bundled kit source resolves via [`bundled_kit_path`](src-tauri/src/lib.rs:74): in dev it is `../kit/.lmbrain` (LF on disk), in production it is `resource_dir/kit/.lmbrain` (bundled at build time).
- `core.autocrlf=true` is set locally and there is **no `.gitattributes`** in the LMBrain repo, the `kit/`, or the scaffolded repo. Under `autocrlf=true`, a fresh checkout (e.g. on the CI runner that builds installers) converts working-tree files to CRLF, so the **bundled production kit resource ships as CRLF** even though the committed source is LF.

**Confirmed:** the operator ran the **installed 1.0.5 production build**, so the kit source was the bundled `resource_dir/kit/.lmbrain` resource — not the LF working-tree `../kit`. This confirms the bundled production resource ships as CRLF (produced under `autocrlf` during the CI checkout/bundle). The fix **must** cover the bundling step, not only the committed source.

**Why it matters:** LMBrain markets itself as git-friendly. Non-deterministic line endings across OSes and build environments produce spurious git diffs and noise in every scaffolded repo, and risk a latent parser bug if the Rust contract parser was only exercised on LF input.

### F-2 — Operator-confirmation gate works (verified, no action)

The app correctly detected the absence of a `.lmbrain/` brain and waited for the operator to click the scaffold trigger before writing. The "refuse if `.lmbrain` already exists" guard is present at [workspace.rs:206](src-tauri/src/commands/workspace.rs:206). No remediation required; recorded as a passing checkpoint.

### F-3 — Wiki tree is static; folder nodes cannot be collapsed/expanded (confirmed)

**Severity:** medium. **Status:** confirmed.

**Observed:** in the Wiki view the `.lmbrain` tree renders fully expanded and is non-interactive at the folder level. Folders cannot be collapsed or expanded; the whole hierarchy is always shown.

**Root-cause analysis (code-confirmed):** the [`TreeNode`](src/components/Wiki/WikiView.tsx:368) component always renders all children recursively (`node.children.map(...)`) with no expansion state — there is no `useState` for collapsed/expanded, no disclosure/chevron affordance, and folder clicks are inert (`cursor: "default"`; `onClick={() => !isFile || onSelect(node)}` is a no-op for directories). Only file nodes are interactive.

**Why it matters:** for a real project the brain has many files across `specs/`, `tasks/`, `reviews/`, etc.; a permanently fully-expanded tree becomes unusable for navigation. Collapse/expand is baseline behaviour for a file tree.

**Proposed fix:** add per-directory expand/collapse state in `TreeNode` (local `useState`, default sensible — e.g. top-level expanded, deeper levels collapsed), a disclosure affordance (chevron) and click/keyboard toggling on folder rows, while preserving file-node selection behaviour. Keep it accessible (button semantics, `aria-expanded`).

### F-4 — Flat artifact loaders ingest `README.md` as a phantom "UNKNOWN" artifact (confirmed)

**Severity:** medium. **Status:** confirmed (code + observed in the SnippetVault Pulse).

**Observed:** the Project Pulse listed three decisions — `ADR-001`, `ADR-002`, and a third row `UNKNOWN` with status `proposed` — although `decisions/` contains only two ADRs plus `README.md`.

**Root-cause analysis (code-confirmed):** [`build_adrs`](src-tauri/src/commands/contract.rs:231) enumerates every `*.md` in `decisions/` with no filename or content filter. `decisions/README.md` has no frontmatter, so `id` falls back to `"UNKNOWN"` ([contract.rs:252](src-tauri/src/commands/contract.rs:252)) and `status` falls back to `"proposed"` ([contract.rs:256](src-tauri/src/commands/contract.rs:256)). The kit ships a `README.md` in every flat-loader directory, so each becomes a phantom artifact.

**Scope of the bug (which loaders share the pattern):**
- `build_adrs` (`decisions/`) — **visible now** (decisions/README.md → phantom ADR).
- `build_agents` (`agents/profiles/`) — pattern present but **not triggered**: the kit ships no `README.md` in `agents/profiles/`.
- The same flat-scan-without-filter pattern affects the MCP and agent-proposal loaders, whose kit directories **do** ship a `README.md` (`mcp/specs/`, `mcp/proposals/`, `agents/proposals/`) — latent phantom entries there too, not yet surfaced in this round.

Spec/task/review loaders are not affected because their artifacts live in status subdirectories while `README.md` sits at the parent.

**Why it matters:** every scaffolded brain ships these READMEs, so a phantom `UNKNOWN/proposed` decision (and latent MCP/proposal phantoms) appear out of the box, corrupting counts and lists. Independent of the agent's malformed file in F-5.

**Proposed fix:** filter flat loaders to genuine artifacts — by required ID-prefix on the filename (`ADR-`, `MCP-`, `AGENT-PROP-`) and/or by skipping files that lack a parseable `id` in frontmatter. Apply consistently across all flat loaders (`build_adrs`, `build_mcp_records`, MCP/agent-proposal loaders). Add a regression test that a directory containing a `README.md` yields zero phantom artifacts.

### F-5 — Agent wrote `[[wikilinks]]` inside YAML frontmatter (kit effectiveness / conformance)

**Severity:** low-medium. **Status:** confirmed.

**Observed:** the test agent's `agents/proposals/AGENT-PROP-001-...md` frontmatter contains `recommended_for: [[SPEC-001]], [[SPEC-002]]` and `links: [[ADR-001], [ADR-002]]`, which is invalid YAML — correctly flagged by the app's diagnostic ("did not find expected key at line 6 column 30").

**Analysis:** this is **not** a template defect — `templates/agent-proposal.md` ships correct frontmatter (`recommended_for: []`, `links: []`, IDs expected) and `CONTRACT.md` states "references use IDs in frontmatter and `[[wikilinks]]` in prose". The agent deviated on its own. Recorded as evidence of kit effectiveness: the contract rule alone did not prevent a plausible mistake.

**Note on the app:** the diagnostic behaved correctly here (named the file and the exact position) — a positive checkpoint; no app fix required for the detection itself.

**Proposed kit hardening (optional):** add an inline reminder in `templates/agent-proposal.md` (and the other reference-bearing templates) that frontmatter reference fields take bare IDs, not `[[wikilinks]]`. Kit documentation change, pending operator approval.

## Scope
### Included
- Make scaffolded line endings deterministic and consistent with the canonical kit (LF).
- Add line-ending governance (`.gitattributes`) so the canonical source, the bundled resource, and every scaffolded repo stay normalized regardless of `core.autocrlf`.
- Verify the Rust contract parser tolerates both LF and CRLF input defensively.
- (F-3) Add collapse/expand interaction to the Wiki tree folder nodes.
- (F-4) Filter flat artifact loaders so non-artifact files (`README.md`) are not ingested as phantom `UNKNOWN` artifacts.
- (F-5) Optional kit hardening: reinforce in templates that frontmatter reference fields use bare IDs, not `[[wikilinks]]`.

### Excluded
- Any change to the bootstrap UX flow or the operator-confirmation gate (verified working in F-2).
- Reformatting existing LMBrain application source for line endings beyond what governance requires.

## Existing-project analysis

- Copy logic: [`copy_directory`](src-tauri/src/commands/workspace.rs:231) (byte-faithful, correct — keep).
- Kit source resolution: [`bundled_kit_path`](src-tauri/src/lib.rs:74).
- Contract parsing entry: [`src-tauri/src/commands/parser.rs`](src-tauri/src/commands/parser.rs) and [`contract.rs`](src-tauri/src/commands/contract.rs) (frontmatter, wikilink, diagnostics) — must be confirmed CRLF-safe.

## Technical proposal

1. Add a `.gitattributes` enforcing `* text=auto eol=lf` (at minimum for `.lmbrain/**` and `kit/**`) in the LMBrain repo so the committed source and any fresh checkout/CI bundle stay LF.
2. Ship a `.gitattributes` (or equivalent normalization) **inside the kit template** so every scaffolded repository inherits deterministic LF and does not drift under a contributor's `autocrlf=true`.
3. Re-normalize existing working-tree files to LF and confirm the bundled resource is LF after a clean build.
4. Add/confirm a parser unit test that feeds CRLF input and asserts identical parse results to LF (defense-in-depth, independent of governance).

The clean fix is normalization governance, not a runtime conversion in `copy_directory` (keeping the copy byte-faithful is correct; the inputs should be canonical).

## Files and areas involved

- `.gitattributes` (new, repo root)
- `kit/.gitattributes` and/or `kit/.lmbrain/.gitignore` area (new normalization for scaffolded repos)
- `src-tauri/src/commands/parser.rs`, `src-tauri/src/commands/contract.rs` (verification only)
- Possibly the Tauri bundle/resource configuration if the bundling step is found to introduce CRLF
- `src/components/Wiki/WikiView.tsx` — `TreeNode` (F-3)
- `src-tauri/src/commands/contract.rs` — `build_adrs`, `build_agents`, `build_mcp_records` and the MCP/agent-proposal loaders (F-4)
- `.lmbrain/templates/agent-proposal.md` and other reference-bearing templates in `kit/` (F-5, optional)

## Acceptance criteria
- [ ] A freshly scaffolded `.lmbrain/` is LF in every text file (parity with `kit/.lmbrain/`), confirmed by byte comparison.
- [ ] `.gitattributes` enforces deterministic LF for `.lmbrain/**` and `kit/**` in the LMBrain repo.
- [ ] Scaffolded repositories inherit line-ending normalization so they do not drift under `core.autocrlf=true`.
- [ ] The Rust contract parser is shown (by test) to produce identical results for CRLF and LF input.
- [ ] The bundled production resource (`resource_dir/kit/.lmbrain`, confirmed as the CRLF source via the 1.0.5 build) is corrected to LF, and a fresh installed build scaffolds LF.
- [ ] No regression to the operator-confirmation gate or the "refuse if `.lmbrain` exists" guard.
- [ ] (F-3) Wiki tree folders can be collapsed and expanded with a visible affordance; file selection still works; behaviour is keyboard-accessible and covered by a test.
- [ ] (F-4) A `decisions/` (and `mcp/specs/`, `mcp/proposals/`, `agents/proposals/`) directory containing only its kit `README.md` plus valid artifacts yields zero phantom `UNKNOWN` entries; covered by a regression test.
- [ ] (F-5) Reference-bearing templates make it explicit that frontmatter reference fields take bare IDs, not `[[wikilinks]]` (if the operator approves the optional kit change).

## Implementation plan
1. Reproduce: scaffold into a throwaway dir from a clean production build and confirm CRLF; inspect the bundled `resource_dir/kit/.lmbrain` bytes.
2. Add repo `.gitattributes`, re-normalize, rebuild, and confirm the bundled resource is LF.
3. Add kit-level normalization so scaffolded repos stay LF.
4. Add the CRLF/LF parser parity test.
5. Re-run the full scaffold validation (structure + byte parity) and the quality gates.

## Required verification
- `pnpm lint`, `pnpm test`, `pnpm build`.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (native Rust — currently blocked: `cargo`/`rustc` are not installed in this environment; must be run where the toolchain is available).
- Manual: scaffold a fresh repo and byte-compare against `kit/.lmbrain/`.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- **Open decision:** should the kit ship a `.gitattributes` into every scaffolded repo (opinionated, but guarantees git-friendliness), or only normalize internally? Operator input requested.
- **Risk:** re-normalizing existing working-tree files will produce a one-time large diff.
- **Confirmed root cause:** CRLF enters at CI checkout/bundle time under `autocrlf` and ships in the 1.0.5 installed resource; the runtime copy is faithful. Remediation must make the bundled resource deterministic (LF), verified against a clean installed build.

## Instructions for the assigned specialist
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Deviations from the specification

### Handoff status
- [ ] Ready for Project Lead review
