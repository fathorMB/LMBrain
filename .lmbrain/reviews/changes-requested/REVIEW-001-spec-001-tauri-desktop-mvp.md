---
id: REVIEW-001
title: Review of SPEC-001 — Tauri read-only desktop MVP
status: changes-requested
spec: SPEC-001
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-001]
links: [SPEC-001, TASK-001, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, tauri, desktop, read-only]
---

# Review of SPEC-001 — Tauri read-only desktop MVP

## Outcome

Changes requested. The implementation has a promising structural base, but it does not yet satisfy the read-only security boundary, Markdown/contract behavior, or required quality gates.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm test` | Passed: 2 files, 13 tests. Coverage is narrow and does not exercise core workspace behavior. |
| `pnpm build` | Passed: TypeScript build and Vite bundle completed. |
| `pnpm lint` | Failed: 6 errors and 8 warnings, including hook ordering/dependencies and explicit `any`. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Not run: `cargo` is unavailable in the current review environment. |
| Git/worktree review | `node_modules/` and `dist/` are untracked; root `.gitignore` must be checked before staging. |

## Findings

### F-1 — [P1] Remove the broad filesystem plugin capability

`src-tauri/capabilities/default.json` grants `fs:default` in addition to read permissions. This bypasses the intended model in which the frontend can reach the filesystem only through the Rust commands guarded by `PathGuard`. Remove the filesystem plugin and its broad capabilities unless a narrow, justified, scoped permission is genuinely required. The selected-workspace read boundary must be enforced by architecture, not only by frontend convention.

### F-2 — [P1] Parse the documented frontmatter keys exactly

The task contract uses `depends_on`, but `build_tasks` reads `dependencies`; task dependencies will therefore disappear. The review contract uses `spec`, but `build_reviews` reads `spec_id`; reviews will not associate with their specs. Align the parser/models/UI with `CONTRACT.md`, add regression tests for both fields, and surface directory/frontmatter status mismatches as diagnostics.

### F-3 — [P1] Report malformed frontmatter instead of silently accepting it

The parser uses `serde_yaml::from_str(...).unwrap_or_default()`. Invalid YAML becomes an empty frontmatter map without a diagnostic, contrary to the requirement for actionable diagnostics on malformed or partial kits. Return structured parse diagnostics, preserve valid body content, and make the diagnostics visible in the workspace UI.

### F-4 — [P1] Implement actual safe Markdown/Wikilink rendering and backlinks

The Wiki currently exposes raw Markdown body text as `content_html` and renders it with `whiteSpace: pre-wrap`; it does not render headings, lists, tables, callouts, or links. Backlinks are always returned as an empty vector. Implement safe Markdown rendering, local wikilink resolution, backlinks, and tests; do not render untrusted raw HTML.

### F-5 — [P1] Restore the required quality gate and correct implementation evidence

`pnpm lint` currently fails with six errors, including `WorkspaceContext` callbacks used before declaration and `any` values in `AgentsMCPView`. The implementation evidence says lint and Rust checks passed, but the current review cannot reproduce that claim. Resolve all lint errors/warnings required by the project configuration, run the actual commands, and replace the evidence with exact, truthful results.

### F-6 — [P2] Make watcher lifecycle and debounce reliable

Opening a second workspace starts another watcher without first stopping/disposing the prior watcher. The watcher also drops all events arriving within 500ms after the first one, rather than emitting a debounced final refresh. This can leave the UI stale after external edits. Ensure exactly one watcher is active per workspace, dispose it on switch/unmount, and use a trailing debounce/coalesced refresh.

### F-7 — [P2] Complete the required read-only views and contract statuses

Roadmap and Search currently render a generic placeholder. The taskboard omits the contract's `cancelled` state. Implement the required Roadmap view and the promised search/command-palette content search, then decide and document how cancelled tasks are represented without silently hiding them.

### F-8 — [P2] Tighten workspace validation and repository hygiene

`open_workspace` accepts any existing path before making it the approved root; require a directory before setting the root. Validate the documented required root files, not only `VERSION` and `STATUS.md`. Ensure `.gitignore` excludes `node_modules/`, `dist/`, and `src-tauri/target/`; remove unused Vite starter assets and `README.vite.md` if they have no project purpose.

## Root-level files

The root `package.json`, lockfile, TypeScript configs, `vite.config.ts`, `index.html`, ESLint config, `src/`, and `src-tauri/` are normal for a Tauri + Vite application. They are not duplicated application source.

`node_modules/` and `dist/` are generated artifacts and must not be committed. `README.vite.md` and unused starter assets are likely template residue and should be removed or replaced with project documentation.

## Required follow-up

Create a focused fix handoff covering F-1 through F-5 before broadening UI scope. F-6 through F-8 should be completed in the same remediation if practical, or placed in a linked follow-up spec with explicit acceptance criteria.

## Final decision

Do not mark SPEC-001 accepted. Re-submit for review only after all P1 findings are resolved and the full verification evidence is reproducible.
