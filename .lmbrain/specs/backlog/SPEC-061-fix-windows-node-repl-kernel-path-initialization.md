---
id: SPEC-061
title: "Fix Windows Node REPL kernel path initialization"
status: backlog
kind: bugfix
priority: high
area: browser-integration
milestone: M-08
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/35]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-35, windows, node-repl, KIT-NOTE-009]
activity:
  - date: 2026-07-31
    action: "created"
---
# Fix Windows Node REPL kernel path initialization

## Objective
Ensure the integrated Node REPL plugin correctly creates missing kernel directory paths on Windows before writing kernel assets, preventing `os error 3` startup failures.

## Context
Reported in `KIT-NOTE-009` (v3.1.2): Invocations of the Node REPL plugin fail on Windows workspace setups with `failed to write kernel assets: The system cannot find the path specified. (os error 3)` because parent directories in `%TEMP%` or `%USERPROFILE%\.codex` are not recursively ensured prior to asset writing.

## Scope
### Included
- Implement atomic, recursive directory creation (`fs::create_dir_all` / `mkdirp`) for Node REPL kernel asset target paths.
- Enhance diagnostic error messages to explicitly log the exact target path when creation fails.
- Add setup/pre-flight checks on Windows to verify temp/kernel directory write permissions.

### Excluded
- Re-architecting the Node REPL protocol or kernel execution loop.

## Acceptance criteria
- [ ] Node REPL kernel initialization automatically creates required missing directory paths on Windows.
- [ ] Error messages explicitly include the target file path on initialization failures.
- [ ] Verification tests confirm clean initialization when target temp subdirectories do not yet exist.

## Required verification
- `cargo test --workspace`
- `pnpm test`
