# Changelog

All notable changes to the LMBrain kit are recorded here.

The `VERSION` file is the canonical, machine-readable kit version.

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
- Version marker at `.lmbrain/VERSION`.
- Migration guidance for future released kit updates.

### Deliberately deferred

- Multi-writer/concurrency protocols and branching-strategy workflows.
- Automatic migrations or application-driven kit updates.
- Remote sync, cloud accounts, and external coordination.
