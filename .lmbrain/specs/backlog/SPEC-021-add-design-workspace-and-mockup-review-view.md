---
id: SPEC-021
# Note: Quote the title if it contains a colon
title: "Add design workspace and mockup review view"
status: backlog
kind: feature
priority: high
area: design-workflow
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-06-27
updated: 2026-06-27
tags: [design, agents, kit, ui]
activity:
  - date: 2026-06-27
    action: "created"
---
# Add design workspace and mockup review view

## Objective
Add first-class support for design-agent output in LMBrain: a bootstrap-created `.lmbrain/design/` area for manually loaded self-contained HTML/CSS/JS mockups, a desktop view for browsing and previewing those mockups, and updated kit guidance so the Project Lead can recommend manual design-agent work before or alongside implementation handoffs.

## Context
LMBrain already models Project Lead analysis, implementation specs, review artifacts, manual specialist profiles, and interactive agent sessions. It does not currently have a dedicated place for design artifacts or a UI for inspecting visual mockups produced outside the app.

The operator wants LMBrain to support web-app development workflows where the Project Lead may decide that a design specialist should produce mockups before implementation. Those mockups are expected to be created through Claude Design, then manually copied by the operator into a `design` folder created by the LMBrain kit bootstrap.

## Scope
### Included
- Add `kit/.lmbrain/design/` to the clean kit scaffold, with a README that defines the expected manual upload convention for self-contained mockup packages.
- Do not add or rely on `.lmbrain/design/` in the repository's local dogfooding state; this feature is delivered through the bundled kit and application code/docs, not through residual local project-brain files.
- Add a `Design` app view and sidebar entry that scans the selected workspace's `.lmbrain/design/` folder.
- Display design entries with enough metadata for operators and implementers to identify the mockup: folder/file name, modified time, entry point, size, and any README/manifest summary when present.
- Preview self-contained HTML mockups inside a sandboxed webview/iframe-like surface without executing them in the main React app context.
- Provide actions to open the mockup package/file externally or reveal/copy the path, depending on platform-safe Tauri capabilities already available.
- Update project/product/kit/session docs and LMBrain kit guidance so Project Lead handoffs can reference design artifacts and recommend a design specialist when warranted.
- Add kit-level support for a recurring manual design specialist using the exact same agent proposal/profile workflow already used for all LMBrain agents. Do not introduce a special design-agent mechanism.
- Add tests for backend scanning/path safety and frontend rendering/empty/error states.

### Excluded
- Automatic Claude Design invocation or upload.
- LMBrain-generated design mockups.
- Editing mockups in-app.
- Treating design mockups as managed Markdown lifecycle artifacts with statuses.
- Auto-spawning design or implementation agents.
- External network access, telemetry, cloud sync, or design-host integration.

## Existing-project analysis
- The app is a Tauri 2 + React 19 desktop app. View routing is centralized in `src/types/index.ts` (`AppView`), `src/components/Layout/Sidebar.tsx`, and `src/components/Layout/AppShell.tsx`.
- Workspace data loading lives in `src/context/WorkspaceContext.tsx` and command wrappers in `src/lib/commands.ts`.
- Backend commands are wired in `src-tauri/src/lib.rs`; path safety is enforced by `PathGuard` in `src-tauri/src/commands/filesystem.rs`.
- The reusable kit is copied from `kit/.lmbrain/` by `WorkspaceService::initialize_kit`, so adding a directory there is enough for future bootstraps.
- The Wiki tree only scans Markdown under `.lmbrain/`; it is not suitable for previewing HTML/JS/CSS mockups.
- Current artifact parsing and controlled mutations apply to specs/reviews/ADRs/agents/MCP/handoffs. Design mockups should remain regular files to avoid overloading the managed artifact contract.
- Existing agent profiles are `AGENT-LEAD` and `AGENT-FULLSTACK-DESKTOP`. There is no active design specialist profile.
- `AGENT.md` already allows the Project Lead to recommend specialist profiles, but it should explicitly mention design handoffs and design artifact references.

## Technical proposal
Use `.lmbrain/design/` as an unmanaged asset area, not a new status-managed artifact type.

Recommended folder convention:

```text
.lmbrain/design/
  README.md
  <mockup-slug>/
    index.html
    README.md        optional summary/notes
    manifest.json    optional metadata
    assets/          optional local assets
```

Backend:
- Add `DesignMockup` and optional `DesignManifest` models.
- Add `get_design_mockups` to scan only `.lmbrain/design/`, identify packages with an `index.html` or standalone `.html`, collect safe metadata, and ignore hidden/system files.
- Reuse `PathGuard` for all reads; never serve arbitrary absolute paths outside the selected workspace.
- Add either a `read_design_file`/asset URL strategy or a safe local-file preview strategy that works in Tauri without widening path access beyond `.lmbrain/design/`.

Frontend:
- Add `"design"` to `AppView`, add a sidebar item with a design-oriented icon, and mount `DesignView`.
- Use a utilitarian two-pane layout: list/grid of mockups on the left/main area and preview/details on selection.
- Sandbox preview execution as tightly as feasible. If Tauri/local-file constraints prevent reliable sandboxing for asset packages, fall back to an explicit external-open action and document the limitation in the implementation evidence.
- Include empty state, missing-entry-point state, malformed manifest handling, and file-read errors.

Kit and process:
- Update `kit/.lmbrain/AGENT.md`, `OPERATOR.md`, `README.md`, `CONTRACT.md` if needed, and bootstrap prompt guidance to say the Project Lead may request a design-agent pass when UI/UX uncertainty is material.
- Update `docs/product.md`, `docs/kit.md`, `docs/architecture.md`, and `docs/agent-hosts.md` or `docs/sessions.md` as relevant.
- Add the chosen design-specialist proposal/profile to the distributed kit through the existing agent artifact structure, not to this repository's local dogfooding `.lmbrain/` state.
- Preserve existing agent governance exactly: proposals live under `agents/proposals/`, profiles live under `agents/profiles/`, registry entries are maintained consistently, every profile uses `activation: manual`, and activation/approval remains operator authority.

## Files and areas involved
- `kit/.lmbrain/design/README.md`
- `kit/.lmbrain/AGENT.md`
- `kit/.lmbrain/OPERATOR.md`
- `kit/.lmbrain/README.md`
- `kit/.lmbrain/templates/project-lead-bootstrap-prompt.md`
- `kit/.lmbrain/agents/proposals/AGENT-PROP-*.md`
- `kit/.lmbrain/agents/profiles/*.md` only if the normal operator-approved path decides the profile should ship active in the kit
- `kit/.lmbrain/agents/registry.md`
- `src/types/index.ts`
- `src/lib/commands.ts`
- `src/context/WorkspaceContext.tsx`
- `src/components/Layout/Sidebar.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/components/Design/DesignView.tsx`
- `src-tauri/src/models/design.rs`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/commands/design.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- frontend tests under `src/__tests__/`
- backend tests under `src-tauri/tests/`
- `docs/product.md`, `docs/kit.md`, `docs/architecture.md`

## Acceptance criteria
- [ ] Fresh LMBrain kit initialization creates `.lmbrain/design/` with clear README guidance for manually loaded self-contained mockups.
- [ ] The app has a `Design` sidebar entry and view, with stable empty/loading/error states.
- [ ] The Design view lists valid mockup packages/files under `.lmbrain/design/` and shows name, path, modified time, entry point, size, and optional README/manifest summary.
- [ ] Selecting a mockup previews its HTML in a sandboxed or equivalently isolated surface, or presents a safe external-open fallback with the limitation documented.
- [ ] Path handling prevents traversal and never reads/previews files outside the current workspace's `.lmbrain/design/` subtree.
- [ ] Project Lead and operator guidance documents explain when to request design-agent work and how to reference uploaded design mockups in specs/handoffs.
- [ ] Design-specialist proposal/profile handling uses the same artifact locations, frontmatter fields, registry maintenance, manual activation rule, and operator approval authority as existing LMBrain agents.
- [ ] The distributed kit contains the approved design-specialist support artifact in the normal agent location; no local dogfooding `.lmbrain/` proposal/profile is part of the deliverable.
- [ ] Automated tests cover design scanning, path-safety rejection, frontend list/empty/error rendering, and preview/fallback behavior.
- [ ] Existing `pnpm lint`, `pnpm test`, and Rust tests pass, or any native-Rust limitation is reported with CI delegated evidence.

## Implementation plan
1. Define the `.lmbrain/design/` package convention and add the scaffold to the bundled kit only.
2. Add Rust models and Tauri commands for scanning design packages with strict path safety.
3. Add frontend types/command wrappers/context state for design mockups.
4. Build `DesignView` and wire it into `AppView`, `Sidebar`, and `AppShell`.
5. Implement safe preview or explicit external-open fallback; test with a self-contained local HTML fixture.
6. Update Project Lead/operator/docs guidance and add the chosen kit-level design-specialist support artifact using the existing agent proposal/profile workflow.
7. Add focused backend/frontend tests and run available quality gates.

## Required verification
- `pnpm lint`
- `pnpm test`
- `cargo test` for the workspace, or explicit CI delegation if local Rust tooling is unavailable.
- Manual smoke: initialize a throwaway workspace, confirm `.lmbrain/design/` exists, copy in a sample self-contained mockup, open Design view, inspect metadata, and preview/open it.

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
- Previewing arbitrary HTML/JS has a real isolation/security dimension. The implementation must prefer sandboxing and avoid giving mockups privileged access to the app context.
- Tauri local-file/CSP behavior may constrain inline asset previews. The implementer should choose the safest reliable mechanism and document any fallback.
- Open decision: whether `manifest.json` should be required or optional. Recommendation: optional for first version, with deterministic inference from folder/index.html.
- Open decision: whether design mockups should be visible in Wiki. Recommendation: no; keep Wiki Markdown-only and use the dedicated Design view.
- Open decision: whether specs should gain a structured `design_refs` frontmatter field. Recommendation: defer; use `links`/body references first unless the operator wants queryable design dependencies.
- Open decision: whether the design specialist should ship as an active manual profile in the clean kit or start as a normal agent proposal. This decision must follow the same approval process as every other agent profile; no design-specific exception is allowed.
- Implementation packaging constraint: the production patch must not include local dogfooding `.lmbrain/` state changes from this repository. Any local LMBrain planning artifacts are operator/session notes only and are not part of the deliverable.

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
