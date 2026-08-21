---
id: SPEC-062
title: "Align Browser skill specification and integrated browser URL policy for local files"
status: backlog
kind: bugfix
priority: high
area: browser-integration
milestone: M-08
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: [https://github.com/fathorMB/LMBrain/issues/36]
created: 2026-07-31
updated: 2026-07-31
tags: [3.1.3, github-issue-36, browser-skill, url-policy, KIT-NOTE-010]
activity:
  - date: 2026-07-31
    action: "created"
---
# Align Browser skill specification and integrated browser URL policy for local files

## Objective
Reconcile the contradiction between the Browser skill description (which claims `file://` support) and the integrated browser URL policy (which blocks `file://` navigation).

## Context
Reported in `KIT-NOTE-010` (v3.1.2): Navigating to local `file:///...` targets via `tab.goto` fails with a URL policy rejection despite the Browser skill listing `file://` among supported local targets.

## Scope
### Included
- Audit URL policy enforcement for workspace-restricted `file://` URIs.
- Enable safe `file://` navigation for files located within the trusted workspace root, OR update the Browser skill contract to explicitly document the user-opened tab requirement.
- Provide clear error diagnostic feedback explaining whether `file://` navigation is prohibited by policy or constrained to workspace boundaries.

### Excluded
- Allowing arbitrary `file://` access outside workspace boundaries.

## Acceptance criteria
- [ ] Browser skill description and URL policy rules are consistent regarding `file://` handling.
- [ ] Safe `file://` URLs within the workspace root are either supported or cleanly documented with actionable user-opened tab instructions.

## Required verification
- `cargo test --workspace`
- `pnpm test`
