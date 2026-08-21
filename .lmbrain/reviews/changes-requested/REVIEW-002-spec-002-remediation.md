---
id: REVIEW-002
title: Re-review of SPEC-002 — desktop MVP remediation
status: changes-requested
spec: SPEC-002
reviewer: AGENT-LEAD
review_requested_by: user
implementation_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-001, TASK-002]
links: [SPEC-001, SPEC-002, TASK-001, TASK-002, REVIEW-001, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [review, tauri, desktop, markdown]
---

# Re-review of SPEC-002 — desktop MVP remediation

## Outcome

Changes requested. The remediation resolves several prior findings, but the remaining defects prevent reliable project-state rendering and complete verification.

## Resolved since REVIEW-001

- The broad filesystem plugin and its capabilities were removed.
- Task `depends_on` and review `spec` frontmatter keys are now read correctly.
- The parser now produces diagnostics for unclosed and malformed YAML frontmatter.
- `pnpm lint`, `pnpm test`, and `pnpm build` pass in the review environment.
- Roadmap view, cancelled-task column, root `.gitignore`, directory validation, and a trailing watcher debounce were added.

## Verification performed

| Check | Result |
| --- | --- |
| `pnpm lint` | Passed with no output. |
| `pnpm test` | Passed: 2 files, 13 tests. No new frontend contract/workspace regression coverage was found. |
| `pnpm build` | Passed. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Not run: `cargo` remains unavailable in the current review environment. |
| SPEC-002 implementation evidence | Missing: sections are empty and handoff was not marked ready. |

## Findings

### F-1 — [P1] Build backlinks from actual parsed wikilinks

`computeBacklinksFromTree` identifies a backlink only when the target page name happens to appear in another file's path. It never reads that file's content or its wikilinks, so it produces false positives and misses genuine backlinks. Build an index of parsed Markdown wikilinks in the native layer or a bounded read-only UI service, resolve targets consistently, and test real cross-page links.

### F-2 — [P1] Make in-document `[[wikilinks]]` navigable

The Markdown renderer safely renders standard Markdown, but Obsidian-style `[[wikilinks]]` remain ordinary text inside the document body. The sidebar can list outgoing links, but the requirement is for project-local wikilinks to be clickable in the rendered document. Add a safe preprocessing/rendering strategy and a navigation callback that uses the existing resolver.

### F-3 — [P1] Parse the actual STATUS.md structure used by the kit

`extract_focus` and `extract_milestone` only accept lines containing both the word and a colon. The kit uses heading-based sections (`## Current focus`, followed by content), so Project Pulse returns no focus/milestone for the installed LMBrain kit. Parse the documented heading structure, keep graceful fallbacks, and add fixtures based on the real kit.

### F-4 — [P1] Surface malformed-frontmatter and status-mismatch diagnostics in the workspace

The parser now creates diagnostics, but contract builders discard them and workspace validation does not scan artifacts for malformed frontmatter or directory/frontmatter status disagreement. Implement an aggregated diagnostics path that Project Pulse/repository picker can show without preventing valid artifacts from rendering.

### F-5 — [P2] Complete or explicitly defer content search

The Roadmap view is now implemented, but the Search route is still a placeholder and the command palette has no input/filtering or content index. Either implement the limited `.lmbrain` content search required by SPEC-002 or obtain explicit scope approval and record the deferral before acceptance.

### F-6 — [P2] Validate and document native checks, with regression tests

The spec requires Rust test/build/lint commands and regression coverage for parser diagnostics, contract mapping, workspace safety, watcher coalescing, and read-only behavior. The current evidence is empty; the visible test suite is still only 13 frontend tests, and native checks cannot be reproduced here because Rust is unavailable. Run these checks in a Rust-enabled environment, record actual results, and add the missing focused tests.

### F-7 — [P2] Set a restrictive CSP before release

`tauri.conf.json` still sets `csp` to `null`. Even with the filesystem plugin removed and sanitized Markdown, a local desktop app that exposes native commands should ship with a restrictive content-security policy. Define the minimal policy needed by the built UI and verify it does not break required assets.

### F-8 — [P2] Validate before mutating the approved workspace root

`open_workspace` still assigns `PathGuard` before calling `validate_workspace`. A rejected non-directory path can therefore replace the currently approved root. Validate first, then set the canonical approved root only after success; add a regression test that a failed open preserves the prior workspace boundary.

## Final decision

Do not mark SPEC-002 or SPEC-001 accepted. Resolve F-1 through F-4, provide truthful completion evidence, and re-submit for re-review. F-5 through F-8 must be resolved or explicitly approved as deferrals before release.
