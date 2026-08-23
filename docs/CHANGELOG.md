# Changelog

All notable changes to the LMBrain kit are recorded here.

The `VERSION` file is the canonical, machine-readable kit version.

## 5.0.1 - 2026-08-23

### Fixed

- **Debt migration source discovery.** `debt_migration_preview` now shares the artifact-discovery scaffolding exclusions, so kit-shipped `findings/**/README.md` files and templates are not parsed or listed as durable sources. Their path operations are reported separately as `scaffolding_items`; artifact-shaped files remain strictly validated, and identical kit-installed debt scaffolding destinations are reconciled only when their bytes match.
- **Explicit review-reference classification.** Qualified `REVIEW-NNN-FINDING-MMM` tokens are consumed atomically as review-local references. Bare `FINDING-NNN` tokens resolve against the durable artifact index first and against the containing review's own findings section independently; genuine collisions and unresolved references still fail closed.
- **Complete, auditable preflight.** Malformed sources and ambiguous or unresolved review references are reported together in deterministic order. The preview exposes every durable/review-local token mapping and binds that inventory into its digest before the confirmed atomic migration.

## 5.0.0 - 2026-08-21

### Kit migration conflict policy

- **Baseline-aware kit realignment.** `kit_migrate` records the digests the kit shipped in `.lmbrain/.kit-baseline.json`. The preview classifies every kit-owned file as `kit-owned`, `kit-owned-modified`, or `kit-owned-unverified`, lists confirmed local edits under `locally_modified`, and binds the classification into the preview digest so an edit made after previewing invalidates the plan instead of being silently overwritten.

### Added

- **Layered kit contract and controlled migration.** The core governance contract is separated from optional capability modules. `kit_migration_preview` and `kit_migrate` provide a digest-bound, staged upgrade path for kit-owned files while retaining a recoverable backup.
- **Single workspace index and Operations view.** Workspace discovery has a typed index for context consumers, and operator-owned verification gates are collected in a dedicated desktop view.

### Changed

- **Typed lifecycle vocabulary.** The 5.0 lifecycle contract uses only canonical status strings; task artifacts and retired task-board terminology are removed from templates and documentation.

### Fixed

- **Release verification.** CI aligns all release versions, prepares the MCP sidecar before Tauri checks, and fails when generated TypeScript bindings differ from the committed file.

## 4.2.2 - 2026-08-20

### Added

- **Governed verification gate binding (#119).** `spec_set_verification_gates` replaces the executable gate contract a spec declares. It validates every ID against the current verification manifest, requires actor, reason, and the exact source digest, and appends a `verification_gate_events` record. Replacement is confined to `backlog`, `ready`, and `working`, so a contract a spec has already been verified against is never swapped underneath its transcript. The path from an approved manifest to an executed gate is now walkable entirely through verbs.
- **Visible acceptance criteria (#120).** Validation reports each unsatisfied acceptance criterion on a spec in `review` or `done` as `acceptance-criterion-unsatisfied`: informative in `review`, a warning in `done`. Unrecognized markers, waivers without a debt reference, and waivers naming a missing debt are each named for what they are, instead of collapsing into silence.

### Documentation

- **Acceptance criteria markers (#121).** The contract and the agent instructions now define the three recognized markers, including `- [~] <criterion> | waived=DEBT-xxx` for a consciously waived criterion, and state that any other marker counts as not satisfied. The marker existed and was enforced since 4.0.x but was documented only in release notes, so agents invented conventions the tools could not read.

## 4.2.1 - 2026-08-13

### Fixed

- **Real-time refresh after migration.** The desktop watcher now observes the workspace root non-recursively and reattaches its recursive `.lmbrain` watch after a controlled atomic directory replacement. Sidebar unread badges update from subsequent artifact changes without requiring a manual refresh or page navigation, while unrelated repository events remain ignored.

## 4.2.0 - 2026-08-12

### Added

- **First-class Debts system (#110).** Replaced legacy durable findings with a typed Debts subsystem (`DEBT-*` artifacts, `debt_*` verbs, state machine, and dedicated UI view). Includes tool-driven migration (`debt_migrate`) with preview and digest verification.
- **Duplicate canonical section diagnostics (#112).** Validation detects and flags duplicate canonical Markdown sections in specs.
- **Verification environment reporting (#109).** Minimal environments now report stripped environment variables as diagnostics.

### Fixed

- **Windows minimal environment (#108).** Preserved `ProgramData` in minimal verification environments on Windows.
- **Roadmap milestone unification (#114).** Unified milestone parsing and status diagnostics across the roadmap subsystem.

## 4.1.0 - 2026-08-10

### Added

- **Operator-invited dreaming.** The Project Lead now recognizes only explicit operator invitations to a bounded dreaming session. It can capture grounded, tentative `DREAM-*` technical- or design-debt observations through `dream_capture`, retaining a context digest and artifact provenance. Dreams are distinct from findings, specs, ADRs, and roadmap entries; they are never promoted automatically.
- **Dream Journal.** The desktop sidebar includes a read-only Dream Journal with state, classification, area, and confidence filters, provenance details, refresh, empty/error/malformed states, and copyable governed follow-up prompts.
- **Review remediation ranking.** Insights replaces change-request dimensions with a confidence-aware ranking of specs by observed remediation cycles. Status-only histories are excluded rather than represented as zero cycles.

### Fixed

- **Finding supersession shape.** `superseded_by` is now written as the contract's list-valued YAML field.
- **Multiline acceptance criteria.** Compact spec context retains wrapped checklist text instead of silently truncating it.

## 4.0.3 - 2026-08-04

### Fixed

- **Linux desktop and worktree fixes (#92–#99).** Improved Linux worktree visibility and platform path handling.
- **PathGuard normalization (#91).** Strip created spec paths against PathGuard-normalized roots in concurrency tests.

## 4.0.2 - 2026-08-03

### Added

- **Frontmatter repair verb (#82).** Added `lmbrain_repair_frontmatter` to repair corrupted frontmatter blocks cleanly.
- **Lead-managed environment (#82–#88).** Improved environment diagnostics and governed browser capabilities.

### Fixed

- **Frontmatter scalar preservation (#82).** Fixed empty-valued scalar keys corrupting subsequent frontmatter blocks.

## 4.0.1 - 2026-08-01

### Fixed

- **Kit feedback fixes (#66–#80).** Fixed duplicate activity blocks, inline arrays, and review status transition handling.

## 4.0.0 - 2026-08-01

### Added

- **Branching strategies.** Added declarative branching strategy metadata and validation.
- **Governance lifecycle improvements.** Expanded MCP governance tools and status audit trails.

## 3.1.4 - 2026-07-31

### Fixed

- **Responsive workspace snapshots (#40).** Replaced the twelve-command workspace refresh with one coherent snapshot that parses each artifact family once and reuses the same statistics and diagnostics in Pulse and Insights.
- **Off-main-thread desktop operations.** Tauri commands that perform filesystem, Git, verification, harness, and aggregation work now execute asynchronously instead of blocking the desktop main thread.
- **Coalesced refresh pipeline.** Watcher and manual refresh bursts are serialized into one active request plus at most one trailing refresh, and every caller commits only the newest completed snapshot.
- **Pulse statistics remount regression.** Pulse and Insights consume snapshot statistics from workspace state, so navigation, explicit view remounts, and React StrictMode no longer start independent whole-project scans.

The 3.1.3 changelog entry for asynchronous background loading was premature: the corresponding source change was absent from that release. Version 3.1.4 is the corrective implementation and verification release for issue #40.

## 3.1.3 - 2026-07-31

### Fixed

- **Kit-Owned File Realignment Procedure (#34).** Added explicit migration guidance and audit steps in `MIGRATIONS.md` to realign kit-owned governance files (`CHANGELOG.md`, `README.md`, `MIGRATIONS.md`, `reports/README.md`, `templates/`) with bundled defaults while preserving project customizations.
- **Windows Node REPL Kernel Path Initialization (#35).** Ensured recursive parent directory creation for Node REPL user configuration and kernel asset paths on Windows, preventing `os error 3` path failures.
- **Browser Skill & URL Policy Alignment (#36, #37).** Reconciled Browser skill specifications and integrated browser URL policy rules for trusted workspace local files and claimed user-opened tabs.
- **Atomic Attestation and Checklist Auto-Check for `owner=lead` Gates (#38).** Updated `spec_attest_lead` to automatically check the `- [ ]` -> `- [x]` item in the spec body for `owner=lead` verification gates upon recording a passing attestation.
- **Waived Acceptance Criteria with Active Findings (#39).** Added support for waived acceptance criteria syntax (`- [~] text | waived=FINDING-xxx`) backed by active findings during `spec_done` closeouts and validations.
- **Asynchronous 3.1.x Background Loading (#40).** Ensured background data loading and state reconciliation introduced in 3.1.x run asynchronously without blocking the desktop UI thread.
- **Lead Remediation Verification Event (#41).** Added the `review_remediation_verified` MCP verb (`actor_role: project-lead`) to record Lead verification of remediation cycles without changing review status.
- **Window Close Confirmation (#42).** Intercepted application window close events to prompt for confirmation when active agent sessions are open.

## 3.1.2 - 2026-07-29

### Added

- **Kit Feedback Desktop View.** Added a first-class read-only view in the desktop application sidebar for reviewing typed feedback notes recorded in `reports/lmbrain-kit-feedback.md`. Includes summary metrics, category and severity filters, text search, and expandable detail cards for observed/expected behavior, impact, evidence, workarounds, and suggested improvements.

### Fixed

- **Template Exclusion in Artifact Discovery.** `lmbrain_validate` and `markdown_paths` now exclude the `.lmbrain/templates/` directory from live artifact discovery, preventing bundled templates with placeholder IDs (such as `templates/finding.md`) from triggering false status-directory mismatch diagnostics.
- **Operator Verification Auto-Check.** Attesting evidence for an `owner=operator` verification gate via the application or API now automatically checks the checklist item (`- [x]`) in the spec body. Lead-owned gates continue to require explicit Lead verification editing.
- **Roadmap Spec Extraction Scope.** Milestone spec membership parsing in `ROADMAP.md` now strictly extracts `SPEC-*` IDs from bracket-delimited lists (`[SPEC-001, ...]`), preventing parenthetical notes or cross-references in prose from creating unintended milestone membership.
- **Migration Guidance for Pre-Existing Gates.** Updated the `3.1.0` migration guide in `MIGRATIONS.md` to document the use of `verification_migration_preview` for reclassifying pre-existing `owner=operator` gates to `owner=lead`.
- **Terminology Disambiguation.** Standardized kit documentation to reserve "handoff" (`HANDOFF-*`) exclusively for Project Lead session-continuity artifacts, using "spec assignment" for delegating work to specialist agents.

## 3.1.1 - 2026-07-29

### Fixed

- **Template and governed artifact discovery (#24).** Excluded `.lmbrain/templates/` from governed artifact discovery, live invariant checks (`unique_ids`), and contract diagnostics. Official templates copied during kit migration require no manual placeholder edits.
- **Finding scaffolding and human-friendly details (#23).** Candidate finding listing (`list_findings`) considers only `FINDING-*.md` files under `.lmbrain/findings`, ignoring scaffolding `README.md` files while keeping structurally malformed findings visible with explicit diagnostics. Summary counts include `superseded`. The Finding Detail modal presents statement, body, state disposition explanation, metadata, and readable relation titles (`${id} · ${title} (${status})`).
- **Wiki kit feedback and findings tree filtering (#22).** Exposed `.lmbrain/reports/lmbrain-kit-feedback.md` in the Wiki when present without auto-exposing future files in `reports/`. Excluded scaffolding `README.md` files from the Findings section of the Wiki tree, wikilink index, and counts.
- **Project Pulse Backlog metric and shared Insight Reliability component (#25).** Included a fifth metric card for `Backlog` (`SpecStatus::Backlog`) in `build_pulse_data`. Replaced the legacy raw diagnostics list in Project Pulse with the shared `InsightReliability` component, maintaining neutral footer text and preserving the top Findings warning banner.

## 3.1.0 - 2026-07-29

### Added

- **First-class cross-spec findings.** Globally unique `FINDING-*` artifacts have governed lifecycle, typed audit events, validated canonical relations, bounded context packs, actionable diagnostics, read-only legacy candidate inventory, and a dedicated read-only desktop workspace with contextual links.
- **Typed lifecycle and review evidence.** Semantic review verdict history, cycle-aware metrics, explicit authority, and lead/operator verification attestations preserve historical truth without coupling evidence to status changes.
- **Shared diagnostics and project digest v2.** Core, MCP, and app consume stable diagnostic IDs, declared/derived state reconciliation, bounded lifecycle lists, exact omission counts, and actionable next steps.
- **Verification onboarding.** Deterministic repository discovery, validation, status, atomic manifest replacement, and guarded rollback remain non-executing; approval remains a separate MCP/operator action and is not available in the app.
- **Hard spec dependencies.** Typed acyclic `depends_on` graphs, governed dependency replacement, lifecycle enforcement, bounded contexts, diagnostics, conservative legacy candidates, and read-only Board/detail visualization keep downstream work from becoming ready too early.
- **Governed ready-spec parking.** `spec_park` returns only ready work to backlog with preserved typed history, reason/revisit evidence, atomic move recovery, and normal re-approval. The app shows parked state but exposes no status-changing action.
- **Human-friendly Project Lead communication.** Operator-facing conversation follows the operator's language, expands shorthand, and explains practical impact concisely while technical artifacts and specialist handoffs retain dense exact terminology.
- **Portable LMBrain field feedback.** The Project Lead can autonomously append typed, non-sensitive observations about LMBrain itself to `reports/lmbrain-kit-feedback.md`; a read-only MCP report makes the accumulated notes directly deliverable to the LMBrain team without affecting project lifecycle.

### Fixed

- Spec detail navigation returns to the board through an accessible action, malformed/stale lifecycle evidence fails visibly, and canonical review/finding taxonomy remains separate from agent-effectiveness scoring.

## 3.0.2 - 2026-07-19

### Added

- **Antigravity MCP support.** Opening or contextualizing a project now registers the `lmbrain-mcp` server with the Antigravity IDE through its user-global `mcp_config.json`, updating both the 1.x IDE and 2.0 unified CLI+IDE layouts when present while preserving every unrelated server and key. Registration only writes where an Antigravity installation is already detectable, and the single entry targets the most recently opened workspace. Sessions remain launched from the Antigravity IDE; no in-app session flow is added. Project orientation flows through the existing root `AGENTS.md` pointer block, which Antigravity reads natively.
- **Declared verification build outputs.** `verification.toml` gates may declare `fingerprint_exclude` workspace-relative output paths. The pre/post snapshot fingerprints and later freshness checks skip the declared union for the executed gate set, so a bundler gate that rewrites its own `dist/**` is no longer structurally invalidated. Exclusions reject absolute paths, traversal, and `.lmbrain`; they are part of the canonical manifest digest, so declaring one requires fresh operator approval while exclusion-free manifests keep their existing digest and approvals. Foreign mutations still invalidate, with a hint pointing at `fingerprint_exclude`.

### Fixed

- **Complete GitHub Actions runs panel with details modal.** The repository dashboard now surfaces workflow runs of every outcome — success, failure, timed-out, startup-failure, cancelled, skipped, neutral, stale, action-required, queued, waiting, and in-progress — each with a distinct icon and text label so state is never conveyed by color alone, and unknown states fall back to a readable neutral style instead of disappearing. Selecting a run opens a read-only details modal (same interaction as the diff viewer) with run number and attempt, branch, triggering event, commit, actor, timestamps, and a direct GitHub link; missing metadata renders as placeholders.

## 3.0.1 - 2026-07-18

### Fixed

- **Deterministic installer quality gates.** Frontend lint and tests are independent GitHub Actions steps, so a failing native command cannot be masked by a later passing command on PowerShell. The repository dashboard and transcript search now satisfy the shared TypeScript, React Hooks, Fast Refresh, and intentional ANSI-control-sequence lint contracts on both installer platforms.
- **Responsive repository dashboard and safe diff inspection.** Dense worktrees no longer force page-level horizontal scrolling: long paths are contained and discoverable, the dashboard uses wide displays more effectively and collapses for narrower windows, and changed files open a read-only unified-diff modal. Diff retrieval distinguishes index/worktree/untracked targets, rejects paths outside the selected repository, disables external diff/text-conversion tools and color, and bounds oversized previews by bytes and rendered line count.
- **Persistent GitHub PAT storage.** Repository authentication now enables the native Windows Credential Manager, Apple Keychain, and Linux Secret Service backends required by keyring 3 instead of silently using its in-memory mock store. Save operations verify the credential through a fresh entry before reporting success, empty tokens are rejected, missing-token deletion is idempotent, and credential-store read errors remain actionable.
- **Branded application icon.** The default Tauri logo is replaced across the desktop executable, installers, platform icon sets, favicon, and project picker by LMBrain's purple stylized-brain mark, with a shared scalable source and legible small-size output.

## 3.0.0 - 2026-07-17

### Added

- **Git locale & GitHub Dashboard.** Integrazione nativa read-only per mostrare lo stato Git locale, file modificati per stato e delta ahead/behind, integrando l'API GitHub per elencare le PR e i workflow runs con memorizzazione sicura del PAT nel keyring di sistema.
- **Trascrizioni e Ricerca Log.** Storico delle sessioni terminali salvato in modo asincrono nel backend Rust (`full_transcript`), visualizzatore virtualizzato affiancato (`HistorySearchPanel.tsx`) con pulizia sequenze ANSI, ricerca case-insensitive, selezione multipla delle righe e copia bulk in clipboard.

## 2.9.2 - 2026-07-17

### Security

- **Workspace path boundaries for artifact reads.** `lmbrain_get_artifact` resolves caller paths through the canonical workspace guard. Absolute and rooted paths, parent traversal with either separator, and symlink/junction escapes are rejected — traversal lexically before any filesystem access — and typed errors never disclose host filesystem paths.
- **Hardened artifact creation.** `lmbrain_create` accepts only initial lifecycle statuses per artifact kind and validates the derived status directory; traversal, separators, and unknown statuses fail closed. Core-owned fields (`id`, `title`, `status`, `created`, `updated`, `activity`) are reserved, field keys and values that could inject frontmatter lines are rejected, and unique-ID plus single-ready-handoff invariants run under the allocation lock before any write. Invalid requests leave no directories, files, activity entries, or lock residue. The public tool schema is unchanged; values previously accepted accidentally now fail with typed errors.

### Fixed

- **Snapshot-consistent verification evidence.** `spec_verify` fingerprints the workspace before the first gate and after the final gate and records both. When they differ the transcript is explicitly invalidated with the reason, the run reports failure, and the evidence can never satisfy submission freshness checks — even when the workspace later matches the post-gate fingerprint. The artifact lock protects only the transcript write; isolated verification worktrees remain deferred to 3.0.0.
- **Review context keeps nested evidence.** Markdown section extraction is heading-level aware: a `##` section retains its `###` and deeper subsections, heading-like lines inside code fences are content, and an empty duplicate heading no longer hides a later populated section. Missing, empty, or truncated implementation evidence now produces explicit warnings instead of silently incomplete context.
- **Displayed version provenance.** Settings/About resolves the application version from build metadata instead of a hardcoded string; app, kit, and MCP component versions stay distinguishable.
- **Embedded terminal scrolling, selection, and copy.** Scroll gestures are resolved at event time from the active buffer and mouse-tracking state per harness; unmapped combinations degrade visibly instead of swallowing input. A Select text mode suspends mouse reporting locally for ordinary drag selection and restores the exact prior mode; Copy visible copies the current viewport without a selection; copy failures are individually actionable.

## 2.9.1 - 2026-07-16

### Fixed

- **Verification preserves agent evidence.** `spec_verify` now owns only an explicitly delimited generated region inside `### Verification transcript`; hand-authored commands, output, and surrounding implementation evidence are preserved across first and repeated runs. Existing 2.9.0 generated transcripts migrate in place on the next run without deleting adjacent manual evidence.
- **Verification no longer restores stale specs.** Gate execution re-reads and merges into the latest spec under a short per-artifact mutation lock. A moved/deleted spec or changed gate contract now fails with an explicit concurrency error instead of recreating an obsolete working copy or overwriting newer edits.
- **Lifecycle mutations share artifact locks.** Status transitions and controlled field setters serialize their final read/write with verification transcript commits, closing the `spec_verify`/`spec_submit` lost-update race.

## 2.9.0 - 2026-07-16

### Added

- **Attributable verification gates.** Optional strict `.lmbrain/verification.toml`, local digest approval, bounded direct execution, honest green/red transcripts, and stale-evidence detection are exposed through dedicated MCP verbs.
- **Context-complete handoffs.** Spec and review packs include typed Required verification contracts, complete profile guidance/digests, and command-bearing skill summaries without conflating verification checklists with acceptance criteria.
- **Governed profile learning.** Categorized review evidence produces read-only distinct-spec signals and effectiveness metrics; explicit proposals apply only after operator approval and stale-target validation.

### Changed

- **Submission integrity.** `spec_submit` now requires a non-empty fenced Verification transcript under Implementation evidence. Force bypasses remain reasoned and audited.

## 2.8.0 - 2026-07-16

### Added

- **Governed project harness intent.** The kit defines the strict, versioned `.lmbrain/HARNESSES.json` contract. It excludes secrets, machine paths, commands, scripts, hooks, and unsupported host capabilities; repository intent requires separate digest-bound local approval before materialization.
- **Controlled harness-manifest mutations.** MCP read, validate, and set verbs expose project intent without accepting ad-hoc commands; writes are validated, atomic, serialized, and accompanied by digest-only audit evidence.
- **Machine-local harness approval.** Repository harness intent remains inert until the operator approves the current manifest digest for the canonical workspace identity. Changed manifests become stale, moved workspaces do not reuse approval, and corrupt local approval state is quarantined.
- **Read-only harness planning.** LMBrain derives deterministic per-host capability, tool-readiness, native-file ownership, preservation, change, and conflict previews without modifying host configuration.
- **Approved atomic harness application.** Native project configuration is applied only for the approved manifest digest under a shared mutation lock, with staged batch replacement, rollback, idempotence, preservation, and machine-local applied-content hashes for drift detection.
- **Functional Settings workspace.** Settings now provides addressable General, Harnesses, Project environment, and About tabs. Local Harnesses and the machine-local Codex override move out of primary workspace navigation, while project preview, approval, apply, drift, and Lead guidance are presented together.

### Fixed

- **Silent Windows harness probes.** Harness version probes, update subprocesses, and timeout cleanup run without flashing transient console windows.
- **Silent Windows workspace preparation.** Read-only Git metadata, Pi readiness/install verification, and Ollama fallback discovery no longer flash helper console windows while opening a project or preparing sessions.
- **Harness prompt copy feedback.** Project environment now confirms successful clipboard copies and reports clipboard failures instead of leaving the action visually silent.

## 2.7.3 - 2026-07-12

### Fixed

- **Windows installer release gate.** OpenCode executable-resolution tests now use isolated npm-shim fixtures instead of requiring OpenCode to be installed on the GitHub-hosted Windows runner.

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
