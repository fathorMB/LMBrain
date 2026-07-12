# Changelog

All notable changes to the LMBrain kit are recorded here.

The `VERSION` file is the canonical, machine-readable kit version.

## 2.7.2 - 2026-07-12

### Fixed

- **OpenCode project discovery.** LMBrain starts OpenCode directly with the absolute workspace positional and a session-scoped Ollama provider, so file `@` autocomplete and project-scoped language tooling cannot lose the selected repository in a nested Windows launcher.
- **Deterministic OpenCode file completion.** Generated configuration exposes the project as the preserved `@workspace/` local reference when no operator-owned alias exists.
- **OpenCode LSP bootstrap.** Generated OpenCode configuration enables built-in LSP integration when no operator policy exists, while preserving explicit disabled or customized LSP configuration.
- **Packaged session scrolling.** The terminal renderer moves to xterm 6, OpenCode embedded sessions avoid nested mouse capture, and every session exposes Page up, Page down, and Bottom controls. OpenCode wheel and toolbar navigation use its documented alternate message bindings instead of unreliable PageUp CSI forwarding through Windows ConPTY.
- **Compact Project Pulse.** The duplicated Current focus body is no longer rendered above the operational cards; the complete status remains available through the `STATUS.md` Quick Link.

## 2.7.1 - 2026-07-11

### Fixed

- **Rust test suite hangs on Linux.** Refactored process-tree termination to use direct system calls and swapped test executable spawns with lightweight standard Unix tools to prevent deadlock in headless CI runners.

## 2.7.0 - 2026-07-11

### Added

- **Local Harnesses management.** A dedicated page probes the exact user-level Claude Code, Codex, Pi, and OpenCode executables, reports versions and paths, and runs only their supported self-updaters after explicit confirmation. Updates are serialized, blocked by matching active sessions, bounded by timeout/output limits, and verified with a post-update probe.
- **OpenCode sessions through Ollama.** LMBrain launches OpenCode with operator-selected Ollama models, requires a preinstalled CLI, and registers `lmbrain-mcp` through OpenCode's native project configuration without an extension dependency.

### Changed

- **Missing harness guidance.** Missing installations show official documentation and copyable user-level install commands; LMBrain never installs a missing harness automatically or guesses a package manager.

## 2.6.1 - 2026-07-10

### Added

- **Explicit current-view refresh.** A header action reloads shared workspace data and view-local queries with visible success/failure feedback while preserving running session terminals.

### Fixed

- **Codex session scrolling.** LMBrain launches Codex with its supported inline `--no-alt-screen` mode so conversation output remains in xterm's normal scrollback; buffer-aware wheel routing remains in place for other full-screen TUIs.

## 2.6.0 - 2026-07-10

### Added

- **Pi sessions through Ollama.** LMBrain can launch operator-controlled Pi sessions through local or cloud-backed Ollama models, register the repository-scoped `lmbrain-mcp` server, and prepare the exact project-local pinned MCP extension during visible workspace opening.
- **Workspace preparation feedback.** Opening a project now reports staged progress; optional Pi preparation failures remain visible without blocking access to Pulse.
- **Actionable Insights reliability.** Insights replaces ambiguous temporal/path summaries with full-width metric-integrity checks, expandable diagnostic detail, and copyable corrective prompts shared with Pulse.

### Fixed

- **Session scrollback and clipboard.** Session terminals preserve scrollback and selection across tab/view changes and expose clear copy/paste controls and platform-standard shortcuts.

## 2.5.1 - 2026-07-07

### Added

- **Project Insights page.** The app now exposes a dedicated Insights view with artifact inventory, spec flow, diagnostics, and review-quality statistics, including the share of reviewed specs that received `changes-requested`.

## 2.5.0 - 2026-07-07

### Added

- **Project-scoped skills.** The kit now includes `SKILL-*` procedure artifacts under `.lmbrain/skills/`, a skill template, a dedicated app page, context-pack inclusion, lifecycle tools, and governance guidance for reusable build, test, diagnostic, release, and review runbooks.

## 2.4.1 - 2026-07-07

### Added

- **Agent mnemonic names.** Agent profiles now support optional `mnemonic_name` metadata, and agent proposals support `proposed_mnemonic_name`. Bundled specialist profiles include human, memorable role-aligned names, and the Project Lead contract now requires a mnemonic name when proposing or creating profiles.
- **Controlled mnemonic-name mutation.** `lmbrain-mcp` exposes `lmbrain_set_agent_mnemonic_name` so existing profiles can receive or update mnemonic names without hand-editing managed frontmatter.

### Fixed

- **Spec closeout invariant false-negative.** `spec_done` now evaluates checked boxes only inside `## Acceptance criteria` and accepts implementation evidence under `## Implementation evidence` or `## Evidence`, avoiding false failures caused by unrelated checklists such as handoff status.
- **Review remediation lifecycle guidance.** Kit docs, templates, handoff prompts, and MCP descriptions now reinforce that only implementers move specs to `working`/`review`, and changes-requested remediation happens while the spec stays in `review`.
- **Windows migration prompt paths.** Bundled kit paths are normalized before they reach migration prompts, avoiding malformed `file:///%3F...` URLs when the installed app resolves paths through the Windows extended-path prefix.

## 2.3.4 - 2026-07-05

### Fixed

- **Design previews execute packaged runtimes again.** The Design view now loads mockup entries through the dedicated `lmbrain-design.localhost` preview origin instead of `srcdoc`, so WebView2 can execute package scripts and resolve local assets without exposing raw `{{ ... }}` template markup.

## 2.3.3 - 2026-07-03

### Fixed

- **Design package previews on Windows/WebView2.** The Design view now renders package previews from backend-bundled HTML with local CSS and JavaScript inlined, avoiding iframe custom-protocol asset failures and showing the actual mockup instead of the app fallback shell.
- **Nucleus-style roadmap milestones.** Roadmaps using milestone IDs such as `M0`/`M4` and inline spec references now populate the Roadmap view correctly.
- **Tabbed session ergonomics.** Session headers and tabs use stable alignment, and terminal wheel input explicitly scrolls xterm scrollback.

## 2.3.2 - 2026-07-02

### Fixed

- **Design package preview loading.** The Design view now reads validated mockup HTML through the backend and injects a package-scoped base URL before rendering in the iframe, so multi-file mockups with relative CSS/JS assets load correctly from `.lmbrain/design/<package>/`.

## 2.3.1 - 2026-07-02

### Fixed

- **Approval governance consistency.** Proposed ADRs no longer expose direct app approve/reject buttons. The detail modal now provides copyable Project Lead prompts, and `lmbrain-mcp` exposes explicit operator-requested ADR decision and agent activation tools so the Lead can execute controlled transitions without frontmatter hand edits. This supersedes the earlier `1.3.4` agent-tool restriction for ADR/profile transitions.

## 2.3.0 - 2026-07-02

### Changed

- **Formal v3 package release.** Publishes the v3 workflow, migration, context-pack, granular-agent, session-tab, milestone-intelligence, and operator-approval changes under a new shared app/kit version so the installer workflow builds release assets instead of skipping unchanged-version pushes.
- **Migration source remains explicit.** Existing `2.2.7` brains can move to `2.3.0` without artifact rewrites; projects older than `2.2.7` should apply the documented v3 additive migration first, validate, then update `VERSION`.

## 2.2.7 - 2026-07-02

### Added

- **V3 context-economy workflow.** The kit documents the `lmbrain_project_digest`, `lmbrain_spec_context`, and `lmbrain_review_context` MCP context packs and updates Project Lead / specialist handoff guidance to use compact context first, expanding to full artifacts only when required.
- **Granular specialist profile taxonomy.** The reusable kit now includes proposed frontend UI, Tauri backend, MCP/contract, kit/docs, product reviewer, and design specialist profiles, plus registry guidance for operator activation and controlled profile-improvement proposals.
- **Manual kit migration guidance.** `MIGRATIONS.md` now describes the additive `2.2.7` migration path, including preserving project-specific files, adding missing bundled v3 profiles, merging registry guidance, and updating `VERSION` only after validation.

## 2.2.2 - 2026-06-27

### Fixed

- **Release workflow no longer depends on Actions artifact storage.** Installer jobs upload built installers and `lmbrain-mcp` binaries directly to the GitHub Release, avoiding `upload-artifact` quota failures after successful test/build steps.

## 2.2.1 - 2026-06-27

### Fixed

- **CI validation for the design release.** Stabilized the Design view preview test by waiting for the async preview frame and made Codex trusted-project path matching recognize Windows-style project keys even when Rust tests run on Linux.

## 2.2.0 - 2026-06-27

### Added

- **Design view and kit workspace.** New workspaces scaffold `.lmbrain/design/` for operator-loaded self-contained HTML/CSS/JS mockups, and the desktop app now has a Design view that lists those mockups, shows metadata, and previews HTML entries in an isolated iframe surface.
- **Normal agent proposal support for design work.** The kit ships a proposed Web App Design Specialist under `agents/proposals/`, and the Agents & MCP view now lists agent proposals alongside profiles so design specialists follow the same approval/profile workflow as every other agent.

## 2.1.2 — 2026-06-27

### Fixed

- **Sessions new-session modal could open behind session windows.** The modal layer now sits above the current highest session-window z-index, so it remains visible and interactive even after repeatedly bringing session windows to the front.
- **Non-Sessions views could lose click and scroll interaction.** The hidden Sessions layer is now mounted only while the Sessions view is active, and the main content, Wiki panes, and Board columns have the flex sizing needed for their internal scroll containers to work reliably.

## 2.1.1 — 2026-06-27

### Fixed

- **Sessions terminals stayed blank.** The PTY reader emitted a session's first output (e.g. a TUI entering the alternate screen) before the xterm terminal had attached its listener, so the opening frame was lost and the terminal never rendered. Output is now buffered per session and replayed on attach (new `session_attach` command), preserving order with no loss or duplication.
- **Terminal content was clipped at the bottom.** With a global `box-sizing: border-box`, padding on the measured container inflated the FitAddon height by ~one row, which overflowed the window. The xterm element is now measured on a padding-free inner container.
- **Session windows could not be dragged** (and clicking the header could black-screen the app). Root cause: `react-draggable` (under `react-rnd`) reads `process.env.*`, which is undefined in the browser and threw `process is not defined` — silently aborting drags, and crashing render once dragging was disabled. Vite now defines the referenced `process.env` values, and window dragging is driven directly from header mouse events with canvas-bounded clamping.

## 2.1.0 — 2026-06-27

### Added

- **Sessions view.** Launch and monitor interactive Claude Code sessions as floating, draggable, resizable terminals inside LMBrain (native `claude`, Claude via `ollama launch claude --model <model>`, and native Codex). Sessions run with `cwd` at the workspace root, persist while the app is open, and are terminated on exit. Ollama models are auto-discovered from the local API and filtered to tool-capable ones. (ADR-006, proposed.)
- **Codex support (agent-agnostic host).** On opening a workspace LMBrain now registers the `lmbrain-mcp` controlled-mutation server for **both** Claude Code (`.mcp.json`) and Codex: it writes a project-scoped `.codex/config.toml` with `[mcp_servers.lmbrain]`, ensures the workspace is a trusted project in `$CODEX_HOME/config.toml` (adds a missing entry only, preserving everything else), and scaffolds a root `AGENTS.md` pointer block to `.lmbrain/AGENT.md`. (ADR-007, proposed.)

### Changed

- `lmbrain-mcp` no longer replies to JSON-RPC notifications (id-less messages such as `notifications/initialized`), for compatibility with stricter MCP clients like Codex.

## 2.0.1 — 2026-06-26

### Fixed

- The controlled-mutation engine's frontmatter parser no longer hangs on `activity:` blocks (nested mappings with inline scalar fields). Reading any transitioned or created artifact could previously trigger an infinite loop, freezing the desktop app and the `lmbrain-mcp` server.

### Changed

- Internal consolidation pass (no behaviour change for artifacts): frontmatter parsing is unified on `lmbrain-core` (`serde_yaml` removed), the desktop artifact loaders were de-duplicated, the engine and MCP server were reformatted for readability, the file "modified" timestamp now reports true elapsed time, and dead code was removed (the workspace is `clippy`-clean).

## 2.0.0 — 2026-06-23

### Changed (breaking)

- **Tasks are retired.** The board now tracks **specs** through `backlog → ready → working → review → done` (plus `discarded` for anything abandoned). Sub-spec granularity lives in each spec's acceptance-criteria checklist; a spec reaches `done` only with its criteria checked, evidence recorded, and an accepted review. The engine, the `lmbrain-mcp` tools (`spec_ready`/`spec_start`/`spec_submit`/`spec_done`/`spec_discard`), the diagnostics, the templates, and the prompts no longer reference tasks. See [[ADR-005-retire-tasks-spec-board]] / SPEC-019. No migration tooling is provided (early development; re-scaffold instead).

## 1.3.5 — 2026-06-23

### Added

- Diagnostics warn when a spec is `ready` / `in-progress` / `review` but has no implementation tasks, so a ready-for-handoff spec with an empty board is visible instead of silent. `AGENT.md` now requires the Project Lead to break a spec into its tasks before handoff.
- The Agents & MCP view lists the built-in `lmbrain-mcp` per-verb tools (registered automatically via `.mcp.json`).

## 1.3.4 — 2026-06-23

### Changed

- Approval authority is enforced at the agent tool surface: `lmbrain-mcp` no longer exposes `adr_accept` (accepting ADRs and approving/activating agent profiles is operator-only). The Project Lead may still accept specs/reviews, but only on the operator's explicit request — documented in `AGENT.md`.

## 1.3.3 — 2026-06-23

### Fixed

- The Roadmap view was empty for valid roadmaps: the parser matched milestone headings at `##` (h2) while the kit template and generated roadmaps use `###` (h3). It now recognizes any heading that names a milestone (`M-<n>`), ignoring section headers.

## 1.3.2 — 2026-06-23

### Added

- The app now auto-registers the controlled-mutation tools: on opening a workspace it writes a host-format `.mcp.json` at the root that launches `lmbrain-mcp --root <workspace>` (idempotent, preserves other servers). `lmbrain-mcp` accepts `--root`/`LMBRAIN_ROOT`, and the command resolves via `LMBRAIN_MCP_BIN` → a binary next to the app → `PATH`. (SPEC-018; addresses agents falling back to hand-editing because the server was never registered.)

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
- `specs/rejected/` directory in the kit for rejected specifications.

## 1.0.6 — 2026-06-22

### Added

- Inline reminders in templates to clarify that frontmatter reference fields take bare IDs, not `[[wikilinks]]`.
- `.gitattributes` shipped in the kit to enforce LF line endings in scaffolded repositories.

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
