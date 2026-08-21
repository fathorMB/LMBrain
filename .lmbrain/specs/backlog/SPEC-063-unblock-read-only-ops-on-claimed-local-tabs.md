---
id: SPEC-063
title: "Unblock read-only DOM and screenshot operations on claimed user-opened file:// tabs"
status: backlog
kind: bugfix
priority: high
area: browser-integration
milestone: M-08
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/37]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-37, browser-skill, claimed-tabs, KIT-NOTE-011]
activity:
  - date: 2026-07-31
    action: "created"
---
# Unblock read-only DOM and screenshot operations on claimed user-opened file:// tabs

## Objective
Allow read-only operations (DOM snapshot, logs, screenshot) on user-opened `file://` tabs that have been explicitly claimed via `claimTab`.

## Context
Reported in `KIT-NOTE-011` (v3.1.2): When an operator opens a local HTML file in the integrated browser and an agent claims the tab via `browser.user.claimTab`, subsequent read-only calls (`domSnapshot`, `dev.logs`, `screenshot`) are still blocked by the URL policy because the tab URL is `file://`.

## Scope
### Included
- Adjust URL policy checks during read-only operations (`domSnapshot`, `screenshot`, `logs`) on tabs successfully claimed by the user.
- Alternatively, enforce policy checks at `claimTab` time so claimed tabs are guaranteed to be usable.
- Add test coverage for claimed local workspace tabs.

### Excluded
- Allowing active write/navigation actions on untrusted external URIs.

## Acceptance criteria
- [ ] Claimed user-opened local tabs allow `domSnapshot`, `screenshot`, and `logs` read calls.
- [ ] If a tab cannot be acted upon, `claimTab` fails immediately with an explicit error instead of returning an unusable tab handle.

## Required verification
- `cargo test --workspace`
- `pnpm test`
