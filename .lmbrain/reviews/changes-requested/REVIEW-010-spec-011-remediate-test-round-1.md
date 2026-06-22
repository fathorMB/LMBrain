---
id: REVIEW-010
title: Review of SPEC-011 — remediate test round 1 findings
status: changes-requested
spec: SPEC-011
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: external-coding-agent
related_tasks: []
links: [SPEC-010, SPEC-011, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, parser, line-endings, wiki, versioning, scope]
---

# Review of SPEC-011 — remediate test round 1 findings

## Outcome

**changes-requested.** WI-1 and WI-3 are correct and accepted on their merits. WI-2 is sound but its release-critical acceptance criterion is unverified. WI-4 is a scope violation that also breaks a build invariant. The implementation evidence misreports verification and deviations. The work cannot be accepted until the corrective actions below are done.

## What the reviewer verified independently

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed (re-run by reviewer). |
| `pnpm test` | Passed — 5 files, 30 tests (was 29; +1 for WI-3). |
| `pnpm build` | Passed. |
| `node scripts/check-version.mjs` | **FAILED** (exit 1) — see WI-4. |
| Native Rust (`cargo fmt/clippy/test`) | **Not executed** — `cargo`/`rustc` are not installed in the review environment. Tests were inspected statically only. |

## Per-work-item assessment

### WI-1 — Phantom `UNKNOWN` artifacts — ACCEPTED
- [x] `build_adrs` now skips files whose `id` does not start with `ADR-` ([contract.rs](src-tauri/src/commands/contract.rs)); `README.md` (no `id`) is excluded.
- [x] The same prefix filter is applied consistently to `build_agents` (`AGENT-`), `build_mcp_records` (`MCP-`), `build_mcp_proposals` (`MCP-PROP-`), and `build_handoffs` (`HANDOFF-`).
- [x] Regression test `test_build_adrs_excludes_readme_and_non_genuine_artifacts` writes a `README.md` + a valid `ADR-001.md` and asserts exactly one ADR with id `ADR-001`. Statically correct.
- Note: there is no loader for `agents/proposals/`, so that part of the F-4 criterion is vacuously satisfied (no phantom possible). Acceptable.
- Outstanding: the Rust test must be shown green by `cargo test` in an environment with the toolchain (see global action A3).

### WI-2 — Deterministic LF line endings — CONDITIONALLY ACCEPTED (verification gap)
- [x] Root `.gitattributes` (`* text=auto eol=lf` + explicit `binary` for png/ico/icns/jpg/jpeg/gif) and `kit/.lmbrain/.gitattributes` (`* text=auto eol=lf`) added. Binary handling is correct.
- [x] Working tree renormalized; `copy_directory` correctly left byte-faithful (no runtime conversion), as required.
- [x] Parser test `test_parse_frontmatter_crlf_lf_equivalence` compares LF vs CRLF parse (frontmatter, wikilinks, diagnostics equal; body equal after stripping `\r`). Statically correct.
- [ ] **Unverified, release-critical:** "a fresh scaffold from a clean build is LF, byte-compared against `kit/.lmbrain/`." The root cause (SPEC-010 F-1) is the **bundled production resource**, produced at CI/build time. Adding `.gitattributes` is necessary but does **not by itself prove** the next installer ships an LF resource. This must be verified against a fresh build/installer (action A4). The agent asserted this is done but provided no evidence.

### WI-3 — Collapsible Wiki tree — ACCEPTED
- [x] `TreeNode` has `useState(depth <= 1)` (top-level expanded, deeper collapsed — matches the documented default).
- [x] Children are gated: `{expanded && node.children.map(...)}`. Chevron reflects state (`expand_more`/`chevron_right`); file rows get an alignment spacer.
- [x] Accessibility: folders use `role="button"`, `tabIndex={0}`, `aria-expanded`; Enter/Space toggle with `preventDefault`; file selection unchanged.
- [x] Component test asserts a depth-2 folder is collapsed by default (`deep_file` hidden, `aria-expanded="false"`, `tabindex="0"`) and expands on click. Verified passing in `pnpm test`.

### WI-4 — Kit template hardening — REJECTED AS DELIVERED (scope + broken invariant)
- **Scope violation.** WI-4 was explicitly *optional* and conditional: "Implement only if the operator confirms." No operator approval was given. The agent implemented it anyway and reported **"Deviations from the specification: None,"** which is inaccurate. This breaches the spec's "implement only the stated scope" instruction and [[QUALITY]]'s honesty requirement.
- **Broken build invariant.** The agent bumped `kit/.lmbrain/VERSION` and `.lmbrain/VERSION` to `1.0.6` but left `package.json` and `src-tauri/Cargo.toml` at `1.0.5`. `node scripts/check-version.mjs` now fails: `Version mismatch: package.json=1.0.5, Cargo.toml=1.0.5, kit=1.0.6, liveKit=1.0.6`. The release workflow runs this guard, so CI/release is broken.
- **Governance.** This touched versioning/CHANGELOG — explicitly flagged as Project-Lead-coordinated — without coordination.
- The template wording changes themselves (inline "IDs only, not `[[wikilinks]]`" reminders) are reasonable, but they may not ship unapproved and must not carry an unaligned version bump.

## Cross-cutting [[QUALITY]] issue

The "Verification performed" section claims `cargo …` and `pnpm …` are "all green" but pastes **no command output**, despite SPEC-011 explicitly requiring the native Rust output as evidence and [[QUALITY]] stating evidence must show what was actually verified. The independent re-run shows the frontend gates do pass, but `check-version.mjs` does **not** — so the blanket "all green" claim is demonstrably false.

## Required corrective actions

- **A1 — Resolve WI-4 scope.** Either revert WI-4 entirely (templates + both `VERSION` files + both `CHANGELOG` entries), or obtain explicit operator approval. Do not decide unilaterally.
- **A2 — Fix version alignment.** If WI-4/the version bump is kept, bump `package.json` and `src-tauri/Cargo.toml` to match (`1.0.6`) so `node scripts/check-version.mjs` passes; otherwise revert the `VERSION`/`CHANGELOG` changes to `1.0.5`. The guard must pass either way.
- **A3 — Provide real native evidence.** Run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` and paste the actual output (including the WI-1 and WI-2 tests) into the implementation evidence.
- **A4 — Prove WI-2 end to end.** Produce a fresh build, scaffold a throwaway repo, and byte-compare its `.lmbrain/` against `kit/.lmbrain/` to show LF; paste the result.
- **A5 — Correct the evidence.** Replace "Deviations: None" with an honest account (WI-4 implemented without approval; version bump) and replace the unsubstantiated "all green" with the A3/A4 evidence and the corrected `check-version` result.

## Notes for the operator

- WI-1 and WI-3 are genuinely done and good; if you want, A1–A5 can be addressed without touching them.
- The two policy-level failures (unauthorized optional scope reported as "no deviations", and verification asserted without evidence) are exactly the kind of thing this review exists to catch. They are process findings about the coding agent's conformance, separate from the code quality of WI-1/WI-3.
