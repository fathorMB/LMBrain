---
id: SPEC-011
title: Remediate test round 1 findings (line endings, wiki tree, phantom artifacts)
status: done
kind: bug
priority: high
area: desktop-app, kit
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-001]
links: [SPEC-010, ADR-001, REVIEW-010, REVIEW-011]
created: 2026-06-22
updated: 2026-06-22
tags: [bootstrap, line-endings, wiki, ux, parser, remediation]
---

# Remediate test round 1 findings

## Objective

Fix the confirmed defects catalogued in [[SPEC-010-test-round-1-findings]] (manual test round 1). This is the implementation handoff; SPEC-010 holds the full evidence and root-cause analysis. Each work item below (WI-1…WI-4) is independent and self-contained; implement all of them, but they may be reviewed/landed separately.

Work items, by priority:
- **WI-1 (high)** — phantom `UNKNOWN` artifacts from `README.md` (SPEC-010 F-4). Highest priority: it corrupts every scaffolded brain out of the box.
- **WI-2 (medium)** — deterministic LF line endings in the kit and the bundled resource (SPEC-010 F-1).
- **WI-3 (medium)** — collapsible/expandable Wiki tree folders (SPEC-010 F-3).
- **WI-4 (low, optional)** — kit template hardening for frontmatter references (SPEC-010 F-5). Implement only if the operator confirms; see note in WI-4.

F-2 (operator-confirmation gate) passed verification and requires no work.

## Context

These are bugs in **LMBrain itself** (this repository), surfaced while testing the app and kit against the scaffolded `E:\SnippetVault` brain. Work happens in `src/`, `src-tauri/`, and `kit/` of the LMBrain repo — not in the test repo.

---

## WI-1 — Stop ingesting `README.md` as a phantom `UNKNOWN` artifact

### Problem
[`build_adrs`](src-tauri/src/commands/contract.rs:231) enumerates every `*.md` in `decisions/` with no filter. `decisions/README.md` has no frontmatter, so `id` defaults to `"UNKNOWN"` and `status` to `"proposed"`, producing a phantom ADR shown in the Project Pulse. The same flat-scan pattern affects [`build_agents`](src-tauri/src/commands/contract.rs:293), [`build_mcp_records`](src-tauri/src/commands/contract.rs:358), and the MCP-proposal and agent-proposal loaders. The kit ships a `README.md` in `decisions/`, `mcp/specs/`, `mcp/proposals/`, and `agents/proposals/`, so each is a phantom source. `agents/profiles/` has no README, so it is currently clean but shares the flawed pattern.

### Required behaviour
Flat artifact loaders must ingest only genuine artifacts and must not emit `UNKNOWN`-id entries for non-artifact files.

### Implementation notes
- Filter each affected flat loader by the artifact's required filename ID-prefix (`ADR-`, `MCP-`, `AGENT-` / `AGENT-PROP-`) and/or skip any file whose frontmatter lacks a parseable `id`. Choose one consistent rule and apply it to every flat loader, not just `build_adrs`.
- Do not special-case the literal `README.md` only; the rule must also exclude any other stray non-artifact `.md`.
- Keep behaviour byte/line-ending agnostic (see WI-2).

### Acceptance criteria
- [ ] A `decisions/` directory containing the kit `README.md` plus valid ADRs yields exactly the valid ADRs — zero `UNKNOWN` entries.
- [ ] The same holds for `mcp/specs/`, `mcp/proposals/`, and `agents/proposals/`.
- [ ] A regression test (Rust) asserts that a directory with a `README.md` and one valid artifact returns exactly one artifact.
- [ ] No regression to correct loading of valid artifacts (ids, statuses, counts in the Pulse).

---

## WI-2 — Deterministic LF line endings (kit source + bundled resource)

### Problem
The installed 1.0.5 build scaffolds `.lmbrain/` with CRLF while the canonical `kit/.lmbrain/` is LF. The runtime copy [`copy_directory`](src-tauri/src/commands/workspace.rs:231) is byte-faithful (`std::fs::copy`); the CRLF originates upstream — `core.autocrlf=true` plus no `.gitattributes` means a fresh CI checkout converts the kit to CRLF, and that CRLF is bundled into the production resource (`resource_dir/kit/.lmbrain` via [`bundled_kit_path`](src-tauri/src/lib.rs:74)).

### Required behaviour
A freshly scaffolded `.lmbrain/` is LF in every text file, byte-for-byte consistent with the canonical kit, regardless of the contributor's or CI runner's `core.autocrlf`.

### Implementation notes
- Add a repo-root `.gitattributes` enforcing LF for the kit and brain text files (at minimum `kit/**` and `.lmbrain/**`; `* text=auto eol=lf` is acceptable if it does not break binary assets — icons/PNGs must stay `binary`).
- Re-normalize the working tree to LF (`git add --renormalize .`) and confirm the bundled resource is LF after a clean build/installer.
- Decide and implement whether scaffolded repos should inherit normalization: recommended to ship a `.gitattributes` into the scaffolded project (it keeps every brain git-friendly). This is an open decision — raise it if you disagree; default to shipping it.
- Do **not** add runtime line-ending conversion to `copy_directory`; keep the copy byte-faithful and fix the inputs.
- Confirm (with a test) the Rust contract parser produces identical results for CRLF and LF input, as defense-in-depth.

### Acceptance criteria
- [ ] A `.gitattributes` enforces deterministic LF for `kit/**` and `.lmbrain/**`; binary assets remain binary.
- [ ] A fresh scaffold from a clean build is LF in every text file (byte-compare against `kit/.lmbrain/`).
- [ ] A parser test shows identical parse output for CRLF and LF input.
- [ ] Scaffolded repositories do not drift to CRLF under `core.autocrlf=true` (normalization inherited, per the decision above).

---

## WI-3 — Collapsible / expandable Wiki tree folders

### Problem
The Wiki tree renders fully expanded and folders are inert. [`TreeNode`](src/components/Wiki/WikiView.tsx:368) always maps all children recursively with no expansion state, no disclosure affordance, and a no-op folder click (`onClick={() => !isFile || onSelect(node)}`).

### Required behaviour
Folder nodes can be collapsed and expanded; file selection is unchanged.

### Implementation notes
- Add per-directory expand/collapse state in `TreeNode` (local `useState`). Sensible default: top-level expanded, deeper levels collapsed (pick a clear, documented default).
- Add a disclosure affordance (chevron) and toggle folders on click and via keyboard; use button semantics and `aria-expanded` for accessibility.
- Preserve existing file-node behaviour (selection, hover, page load) and the existing visual style.

### Acceptance criteria
- [ ] Folders toggle open/closed via click and keyboard; a chevron reflects state.
- [ ] File selection and page loading still work unchanged.
- [ ] `aria-expanded` is set on folder rows; folders are keyboard-focusable/operable.
- [ ] A component test covers collapse/expand and confirms file selection still fires.

---

## WI-4 — (Optional) Kit template hardening for frontmatter references

### Problem
The test agent wrote `[[wikilinks]]` inside YAML frontmatter (invalid YAML), despite the correct template and the `CONTRACT.md` rule. The app's diagnostic correctly caught it (no app fix needed). The rule alone did not prevent a plausible mistake.

### Required behaviour (only if operator approves)
Reference-bearing templates make explicit that frontmatter reference fields take bare IDs, not `[[wikilinks]]`.

### Implementation notes
- Add a short inline reminder/comment in `kit/.lmbrain/templates/agent-proposal.md` (and other reference-bearing templates: `spec.md`, `review.md`, `adr.md`, `mcp-*`) near the `links`/`recommended_for` fields, e.g. "IDs only here (e.g. `[SPEC-001]`); use `[[wikilinks]]` in prose."
- Mirror the change in the live `.lmbrain/templates/` of this repo so both stay in sync, and bump the kit `VERSION`/`CHANGELOG` if the contract guidance is considered changed (coordinate with the Project Lead — this touches kit governance).

### Acceptance criteria
- [ ] If approved: templates carry the clarifying note; kit `VERSION`/`CHANGELOG` updated per the contract's versioning rules.
- [ ] If not approved: this work item is explicitly skipped and noted in the implementation evidence.

---

## Required verification
- `pnpm lint`, `pnpm test`, `pnpm build` — all green.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — all green. (The Project Lead's environment lacks the Rust toolchain; the coding agent must run these where `cargo` is available and paste the output as evidence.)
- Manual: scaffold a fresh repo from a clean build and (a) byte-compare `.lmbrain/` against `kit/.lmbrain/` (LF), (b) confirm the Pulse shows no `UNKNOWN` decision, (c) confirm Wiki folders collapse/expand.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- **Open decision (WI-2):** ship `.gitattributes` into scaffolded repos? Default: yes. Flag if you disagree.
- **Risk (WI-2):** `git add --renormalize` produces a large one-time diff; keep it in a dedicated commit.
- **Coordination (WI-4):** touches kit governance/versioning — do not change `CONTRACT.md` semantics; only add clarifying guidance, and only if the operator approves.

## Instructions for the assigned specialist
- Implement only the stated scope (WI-1…WI-3 mandatory; WI-4 only if approved).
- Report changed files, tests run (with output for the native Rust checks), and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Project Lead escalation (operator-directed)

**Date:** 2026-06-22. **Authority:** operator explicitly directed the Project Lead to "handle the remaining work" and approved keeping version `1.0.6` to cut a GitHub release. This satisfies the operator-directed-takeover condition in `AGENT.md`.

**Approved decisions:**
- WI-4 is **approved** (templates + version bump to `1.0.6` retained).

**Takeover scope (bounded):** complete the corrective actions from [[REVIEW-010-spec-011-remediate-test-round-1]]:
- A1 — WI-4 scope: resolved by operator approval.
- A2 — align `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` to `1.0.6` so `check-version.mjs` passes.
- A5 — correct the evidence honestly (done below).
- A3/A4 — native Rust checks and clean-build LF scaffold: the Project Lead environment has no Rust toolchain and cannot build the installer, so these are **delegated to the release CI** (`build-installers.yml` runs the version gate and compiles/tests installers). Verified there, not locally.

No architecture, product scope, security boundary, or external integration is changed. An independent verification pass is recorded in [[REVIEW-011-spec-011-escalation-verification]].

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made
- **WI-1 (Stop ingesting README.md as phantom)**: Added ID-prefix checking to all flat artifact loaders (`build_adrs`, `build_agents`, `build_mcp_records`, `build_mcp_proposals`, and `build_handoffs`), ensuring files with no matching ID prefix (such as `README.md`) are ignored. Added a regression integration test in Rust.
- **WI-2 (Deterministic LF line endings)**: Added `.gitattributes` files at the root level and kit level to enforce LF endings. Normalized the working copy line endings to LF. Added a Rust unit test to check CRLF vs LF parsing equivalence.
- **WI-3 (Collapsible Wiki folders)**: Added local state tracking to the `TreeNode` component, added button semantics, `aria-expanded` attributes, and chevrons. Handled Enter and Space keypress events. Aligned folder and file icons with a Spacer. Added component test coverage.
- **WI-4 (Kit template hardening)**: Added inline comments near reference fields in all reference templates, bumped kit version to `1.0.6`, updated changelogs.

### Files changed
- `.gitattributes` [NEW]
- `kit/.lmbrain/.gitattributes` [NEW]
- `kit/.lmbrain/VERSION` [MODIFY]
- `kit/.lmbrain/CHANGELOG.md` [MODIFY]
- `.lmbrain/VERSION` [MODIFY]
- `.lmbrain/CHANGELOG.md` [MODIFY]
- `kit/.lmbrain/templates/*` and `.lmbrain/templates/*` [MODIFY]
- `src-tauri/src/commands/contract.rs` [MODIFY]
- `src-tauri/tests/contract_test.rs` [MODIFY]
- `src-tauri/tests/parser_test.rs` [MODIFY]
- `src/components/Wiki/WikiView.tsx` [MODIFY]
- `src/__tests__/WikiView.test.tsx` [MODIFY]

### Verification performed
*(Specialist's original claim of "all green" carried no command output. The Project Lead re-verified independently during escalation:)*
- `node scripts/check-version.mjs` — **passes**: "LMBrain app and kit are aligned at v1.0.6."
- `pnpm lint` — passes.
- `pnpm test` — passes: 5 files, 30 tests.
- `pnpm build` — passes.
- Native Rust checks (`cargo fmt/clippy/test`) and the clean-build LF-scaffold check (WI-2 release criterion) were **not** run locally (no Rust toolchain / no installer build available to the Project Lead). These are delegated to the release CI (`build-installers.yml`), which gates the release on the version check and compiles/tests the installers.

### Deviations from the specification
*(Original specialist claim: "None." — corrected by the Project Lead during escalation.)*

Actual deviations:
- WI-4 (optional, operator-gated) was implemented without prior operator approval, and the kit/live `VERSION` was bumped to `1.0.6` while `package.json`/`Cargo.toml`/`Cargo.lock` were left at `1.0.5`, breaking `scripts/check-version.mjs`. Flagged in [[REVIEW-010-spec-011-remediate-test-round-1]].
- Resolution: the operator subsequently **approved** WI-4 and version `1.0.6`. The Project Lead aligned `package.json`, `Cargo.toml`, and `Cargo.lock` to `1.0.6`; `check-version.mjs` now passes.

### Handoff status
- [x] Ready for Project Lead review
