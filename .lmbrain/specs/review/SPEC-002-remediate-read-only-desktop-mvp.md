---
id: SPEC-002
title: Remediate read-only desktop MVP review findings
status: review
kind: bugfix
priority: critical
area: desktop
milestone: M-01
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: [TASK-001, TASK-002]
related_decisions: [ADR-001]
links: [SPEC-001, TASK-001, TASK-002, REVIEW-001, ADR-001]
created: 2026-06-22
updated: 2026-06-22
tags: [tauri, review, security, markdown]
---

# Remediate read-only desktop MVP review findings

## Objective

Resolve the blocking findings in [[REVIEW-001-spec-001-tauri-desktop-mvp]] so that the Tauri desktop MVP can be re-reviewed against [[SPEC-001-tauri-read-only-desktop-mvp]].

## Scope

### Included

1. Remove broad Tauri filesystem-plugin permissions and any unused filesystem plugin surface that bypasses the guarded native command boundary.
2. Parse the kit's documented frontmatter names exactly: `depends_on` for tasks and `spec` for reviews.
3. Return visible structured diagnostics for malformed/unclosed YAML frontmatter and status-directory/frontmatter disagreements, while still rendering unrelated valid artifacts.
4. Implement safe Markdown rendering in Wiki and spec/detail content as required by SPEC-001, with local wikilink resolution and backlinks. Do not permit untrusted raw HTML execution.
5. Resolve every frontend lint error and warning required by configured lint rules; remove `any` escapes and repair hook ordering/dependencies.
6. Fix workspace watcher lifecycle so switching repositories leaves exactly one watcher, and coalesce rapid changes into a final refresh.
7. Implement the Roadmap and content search surfaces, or explicitly document and receive approval for deferring them; do not leave required views as placeholders.
8. Render `cancelled` tasks in an intentional, discoverable read-only representation.
9. Require a directory before approving a workspace root, validate documented root files, and add root `.gitignore` rules for generated files.

### Excluded

- Any runtime write to an operator-selected workspace repository.
- New product features outside the review findings.
- Changes to kit contract, project roadmap, or architecture decisions.

## Verification requirements

- `pnpm lint` must pass with no errors or warnings.
- `pnpm test` must pass and include regression coverage for the contract fields, parser diagnostics, workspace path safety, watcher lifecycle/coalescing, and read-only behavior.
- `pnpm build` must pass.
- Run `cargo test --manifest-path src-tauri/Cargo.toml` and the applicable Rust lint/build checks in an environment with Rust installed; report actual commands/results.
- Confirm selected workspace repositories remain unchanged in Git after all app interactions.

## Instructions for the assigned specialist

- Read this spec, [[REVIEW-001-spec-001-tauri-desktop-mvp]], [[SPEC-001-tauri-read-only-desktop-mvp]], `CONTRACT.md`, and `QUALITY.md` before modifying code.
- Preserve all existing work that is not implicated by the review.
- The LMBrain application source repository may be modified. At runtime, selected workspace repositories must remain read-only.
- Fill the implementation evidence truthfully with the exact verification commands and outcomes.

## Copyable manual handoff prompt

> You are the Fullstack Desktop Specialist. Read `.lmbrain/specs/ready/SPEC-002-remediate-read-only-desktop-mvp.md`, `REVIEW-001`, the original `SPEC-001`, `CONTRACT.md`, and `QUALITY.md` in full. Resolve every listed blocking finding with production-grade code and regression coverage. Preserve unrelated work. You may modify this LMBrain application source repository, but the application must never write to an operator-selected workspace repository at runtime. Report exact verification commands/results and fill only this spec's implementation evidence plus strictly necessary technical documentation. Return the work ready for Project Lead re-review.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

### Files changed

### Verification performed

### Documentation updated

### Deviations from the specification

### Handoff status
- [x] Reviewed by Project Lead
- [ ] Changes requested in REVIEW-002 are resolved and ready for re-review
