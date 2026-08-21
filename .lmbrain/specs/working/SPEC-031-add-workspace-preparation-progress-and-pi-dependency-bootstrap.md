---
id: SPEC-031
# Note: Quote the title if it contains a colon
title: "Add workspace preparation progress and Pi dependency bootstrap"
status: working
kind: feature
priority: high
area: workspace-sessions
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-009]
links: [SPEC-029, SPEC-030, REVIEW-031]
created: 2026-07-10
updated: 2026-07-10
tags: [workspace, loading, pi, dependencies, ux]
activity:
  - date: 2026-07-10
    action: "created"
activity:
  - date: 2026-07-10
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-10
    action: "transitioned ready -> working"
---
# Add workspace preparation progress and Pi dependency bootstrap

## Objective

Make workspace opening visibly progressive and automatically prepare the exact approved project-local Pi MCP dependency when missing, without blocking access to Pulse on optional integration failure.

## Context

The picker currently sets `state.loading` but renders no loading UI. Folder selection can therefore appear frozen while backend validation, registrations, data loading, session refresh, and watcher startup run. Pi first use also fails late because the pinned MCP extension is only checked at session start.

## Scope
### Included

- Loading overlay with current preparation stage and selected path.
- Backend idempotent check/install of `npm:pi-mcp-extension@1.5.0` in project scope.
- No installation when the exact pin is already ready.
- Non-blocking warning when Pi CLI, network, trust, or package installation fails.
- Preserve Pulse access and existing workspace error behavior.
- Ignore only generated `.pi/npm/`; do not hide `.pi/settings.json`.
- Tests and documentation for preparation result and loading reducer/UI behavior.

### Excluded

- Installing Pi, Ollama, models, or global packages.
- Unpinned upgrades or arbitrary package installation.
- Blocking a valid workspace because optional Pi preparation failed.

## Existing-project analysis

## Technical proposal

## Files and areas involved

## Acceptance criteria

- [x] Workspace selection immediately displays an accessible loading overlay.
- [x] Loading text advances through validation, Pi integration, project data, watcher, and Pulse stages.
- [x] Exact pinned Pi MCP dependency is detected without reinstalling.
- [x] Missing dependency is installed project-locally with exact source/version and explicit approved trust flag.
- [x] Installation failure produces a persistent non-blocking warning and Pulse still opens.
- [x] Pi session preflight succeeds after successful preparation and retains its defensive exact-pin check.
- [x] `.pi/npm/` is ignored while `.pi/settings.json` remains visible to Git.
- [x] No global Pi settings or unrelated `.pi` resources are overwritten.
- [x] Compilation checks pass; runtime install/loading verification is recorded from the active local dev app.
- [x] Documentation and replacement ADR describe the new security/ownership behavior.

## Implementation plan

1. Add structured backend preparation result and exact-pin install path.
2. Expose a Tauri preparation command and call it after workspace validation.
3. Add loading stage/notice state and picker/app UI.
4. Add focused source tests and update docs/evidence.

## Required verification

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions

- Package installation requires network access and may be slow; it runs on a blocking worker while the webview displays progress.
- `--approve` is a deliberate trust decision limited to exact operator-approved source/version and project scope.
- Pi preparation is optional to core workspace access; failure must remain a warning rather than a workspace-open error.
- `.pi/settings.json` may combine LMBrain's pin with user configuration and therefore remains visible/versionable.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
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

- Added project-local exact-pin detection using both `.pi/settings.json` and offline `pi list`.
- Added idempotent automatic `pi install npm:pi-mcp-extension@1.5.0 -l --approve` on a blocking backend worker when the pin is missing.
- Added structured `ready` / `installed` / `unavailable` preparation results and persistent non-blocking warnings.
- Added accessible workspace preparation overlay with validation, Pi, Git, project data, session, watcher, and Pulse stages.
- Retained defensive Pi session preflight after workspace preparation.
- Added source tests for exact project setting detection and picker loading UI.
- Updated `.pi/npm/` ignore policy, docs, changelog, and accepted replacement [[ADR-010-bootstrap-pinned-pi-mcp-dependency-during-workspace-preparation]].

### Files changed

- `src-tauri/src/commands/pi_registration.rs`
- `src-tauri/src/lib.rs`
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/context/WorkspaceContext.tsx`
- `src/components/Picker/RepositoryPicker.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/styles/global.css`
- `src/__tests__/RepositoryPicker.test.tsx`
- `.gitignore`
- `docs/agent-hosts.md`, `docs/sessions.md`, and `docs/development.md`
- `.lmbrain/CHANGELOG.md` and [[ADR-010-bootstrap-pinned-pi-mcp-dependency-during-workspace-preparation]]

### Verification performed

- `cargo check --workspace --tests` - passed on the final async-worker implementation.
- `pnpm build` - passed on the final implementation; existing Vite chunk warning remains (main JS approximately 807 kB).
- `git diff --check` - passed.
- Runtime workspace-open/install behavior is ready for the operator's active local-app test.
- Operator observed the staged loader complete and Pulse open for `E:\Git\AstraNexus`.
- Project `.pi/settings.json` contains exact pin `npm:pi-mcp-extension@1.5.0` and the installed package reports version `1.5.0` under `.pi/npm/node_modules`.
- Initial post-install verification produced a false warning because `pi list` ignores project packages unless invoked with `--approve`; the verifier and session preflight now pass that documented flag.
- `PI_OFFLINE=1 pi list --approve` in AstraNexus reports the exact project package and exits 0.
- Final `cargo check --workspace --tests` and `git diff --check` pass after the verification fix.

### Deviations from the specification

- No unpinned/global dependency operation is permitted. Runtime installation success and loading-stage behavior remain unverified until the local app is reopened and a project is selected.

### Handoff status
- [ ] Compilation complete; waiting for operator workspace-open/install verification.
