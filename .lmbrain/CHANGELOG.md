# Changelog

All notable changes to the LMBrain kit are recorded here.

The `VERSION` file is the canonical, machine-readable kit version.

## 1.3.1 — 2026-06-23

### Fixed

- CI: point the installer and MCP-binary artifact paths at the workspace-root `target/` (the cargo workspace relocated build output from `src-tauri/target/`), and make the `create` test's path assertion platform-independent so Rust tests pass on Windows.

## 1.3.0 — 2026-06-23

### Added

- Controlled-mutation engine (SPEC-017 / [[ADR-004-controlled-mutation-engine-mcp]]): a tauri-free `lmbrain-core` crate (per-artifact state-machine transitions, shared invariants, surgical frontmatter editing, atomic writes, progressive ID allocation, `force`+`reason` audit) and an `lmbrain-mcp` server exposing per-verb tools to agents. The app's `set_artifact_status` and the kit diagnostics now run on the shared core.

## 1.2.6 — 2026-06-23

### Changed

- `AGENT.md` and the Project Lead bootstrap prompt now state explicitly that initial scaffolding, setup, and bootstrapping are implementation work, and that approving an ADR/spec/technical direction does not authorize the Lead to implement — its next step is the handoff, then stop.

## 1.2.5 — 2026-06-23

### Changed

- Task lifecycle is now explicit. New tasks start in `backlog` (template default changed from `planned`); the `backlog → planned → in-progress → review → done` flow and its owners are documented in `AGENT.md` and `tasks/README.md`.
- The generated handoff prompt instructs the implementer to move the linked task(s) to `in-progress` when starting and to `review` when finished.

### Added

- Diagnostics warn when a task is `planned` but has no ready spec backing it (missing/nonexistent/not-yet-ready spec), so it can be kept in `backlog`.

## 1.2.4 — 2026-06-23

### Fixed

- Made the `set_artifact_status` integration-test path assertions platform-independent (compare canonicalized paths), so the Rust tests pass on the Windows CI runner. Completes the CI Rust-test wiring from 1.2.3.

## 1.2.3 — 2026-06-23

### Changed

- CI release builds now run the Rust integration tests (`cargo test`) alongside the frontend lint and tests.

## 1.2.2 — 2026-06-23

### Added

- Diagnostics now warn when a spec's `recommended_agent` does not resolve to an existing agent profile (including the `AGENT-XXX` template placeholder), surfacing it as a missing reference in the Project Pulse.

## 1.2.1 — 2026-06-23

### Fixed

- `[[wikilinks]]` now render as clickable links instead of raw `[[...]]` text in the Roadmap milestone titles/outcomes and the Project Pulse blockers and recommended actions, completing the inline rendering added in 1.2.0.

## 1.2.0 — 2026-06-23

### Changed

- The Taskboard column now follows the task's frontmatter `status:` (source of truth), so a status change moves the card; the folder is expected to agree and a divergence is surfaced as a warning badge on the card.

### Fixed

- Project Pulse "Copy prompt" / "View prompt" buttons now match the app's button styling.
- Project Pulse breadcrumb, current focus, and milestone now render `**bold**` and `[[wikilinks]]` as formatted text / clickable links instead of raw markup.

## 1.1.1 — 2026-06-23

### Fixed

- Recommended manual-handoff cards now expose a viewable, copyable prompt without launching an agent.
- `STATUS.md` and `ROADMAP.md` Quick Links now open their Markdown source in the read-only detail modal.
- Artifact-detail actions refresh after an approve/reject transition, including flat ADR files.
- Roadmaps no longer model or display temporal targets.

## 1.1.0 — 2026-06-23

### Added

- Contract v0.2: `rejected` is now a first-class terminal status on all proposable artifacts (Spec, ADR, Agent proposal, MCP proposal), and Agent proposals have an explicit status set (`proposed`/`approved`/`rejected`). See [[ADR-003-reject-as-first-class-status]].
- `specs/rejected/` directory for rejected specifications.

## 1.0.6 — 2026-06-22

### Added

- Inline reminders in templates to clarify that frontmatter reference fields take bare IDs, not `[[wikilinks]]`.
- `.gitattributes` shipped in the kit to enforce LF line endings in scaffolded repositories.

### Fixed

- Flat artifact loaders (ADRs, agents, MCP records/proposals, handoffs) no longer ingest `README.md` as a phantom `UNKNOWN` artifact.
- Wiki tree folders can now be collapsed and expanded.
- Line endings are normalized to LF across the repository and kit via `.gitattributes`.

## 1.0.5 — Unreleased

### Fixed

- Release workflow installer artifacts to target only final files, avoiding duplicate asset name failures.

## 1.0.4 — Unreleased

### Fixed

- Release publishing uploads only downloaded files, not intermediate artifact directories.

## 1.0.3 — Unreleased

### Fixed

- Release publishing checks out the repository before invoking GitHub CLI, allowing the release command to resolve the repository context.

## 1.0.2 — Unreleased

### Added

- Version-gated installer builds and GitHub Release publishing, with versioned artifact names and release assets.

## 1.0.1 — Unreleased

### Added

- Desktop bootstrap support: the application can initialize the clean kit in a selected repository after explicit operator confirmation.
- Version-alignment guard for the desktop application, Rust package, and distributable kit.
- Windows and Linux installer build workflow.

## 1.0.0 — Unreleased

### Added

- Canonical Markdown contract for project, task, specification, review, decision, agent, MCP, and session-handoff artifacts.
- Human operator guide and Project Lead operating contract.
- Production-quality policy: Project Lead is documentation-only; specialists deliver production-grade work with evidence.
- Independent technical-judgement policy: agents challenge weak assumptions, use current official documentation for material technical choices, and require explicit approval for shortcuts.
- Operator-authorized Project Lead escalation for narrowly scoped, repeatedly missed corrective work.
- Manual specialist handoff and formal Project Lead review workflow.
- Session-handoff workflow for continuing Project Lead context across agent sessions.
- Agent and MCP registries, profiles, proposals, and templates.
- Read-only desktop-first Tauri architecture decision.
- Version marker at `.lmbrain/VERSION`.
- First desktop MVP implementation spec, task, and Fullstack Desktop Specialist profile.
- Migration guidance for future released kit updates.

### Deliberately deferred

- Multi-writer/concurrency protocols and branching-strategy workflows.
- Automatic migrations or application-driven kit updates.
- Remote sync, cloud accounts, and external coordination.
