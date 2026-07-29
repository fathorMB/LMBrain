# Architecture

LMBrain is split into a TypeScript frontend, a Tauri backend, and two Rust workspace crates.

## Workspace Crates

```text
src/           React frontend
src-tauri/     Tauri application backend
lmbrain-core/  Shared artifact mutation engine
lmbrain-mcp/   MCP stdio server for agents
```

The root `Cargo.toml` defines a workspace containing `src-tauri`, `lmbrain-core`, and `lmbrain-mcp`.

## Frontend

The frontend is React 19 with Vite and TypeScript. It uses inline component styling and a central `WorkspaceContext` for app state. Tauri commands are wrapped in `src/lib/commands.ts`.

Important areas:

- `src/components/Layout/`: shell, sidebar, top bar, modal surfaces.
- `src/components/Pulse/`: project pulse and recommended actions.
- `src/components/Wiki/`: Markdown tree and page viewer.
- `src/components/Design/`: design mockup browser and isolated HTML preview.
- `src/components/Insights/`: read-only project statistics and review-quality metrics.
- `src/components/Harnesses/`: local-machine harness probe/update status embedded in Settings.
- `src/components/Settings/`: addressable General, Harnesses, Project environment, and About tabs.
- `src/components/Skills/`: dedicated project-scoped skill browser.
- `src/components/Taskboard/`: spec board.
- `src/components/Sessions/`: tab-based session workspace with xterm terminal integration.
- `src/context/WorkspaceContext.tsx`: workspace state, navigation, session tab state (`SessionInfo[]`, `activeSessionId`), and data refresh.

## Tauri Backend

`src-tauri/src/lib.rs` wires Tauri commands, application state, and services:

- `WorkspaceService`: validates workspaces, tracks recent repositories, initializes the bundled kit, and compares project kit and bundled kit versions using semver logic to derive the workspace migration status (`KitMigrationStatus`).
- `PathGuard`: scopes file access to the selected workspace.
- `FileWatcherService`: watches the workspace and emits refresh events.
- `SessionManager`: owns PTY-backed interactive sessions.
- `HarnessApprovalStore`: machine-local digest-bound approval and applied-content state with corruption quarantine.

The backend reads `.lmbrain/` artifacts, parses Markdown/frontmatter, builds diagnostics, starts and stops watchers, registers MCP host files, and exposes session commands.

Harness governance uses strict Tauri-free manifest parsing and hashing in `lmbrain-core`, controlled read/validate/set verbs in `lmbrain-mcp`, and Tauri planner, approval, materializer, and drift services. Application requires an approved digest and commits affected native files as a locked staged batch with rollback.

## Artifact Model

The app treats `.lmbrain/` as the project source of truth. Key artifact families include specs, reviews, decisions, agent profiles and proposals, project-scoped skills, MCP records/proposals, handoffs, roadmap, status, and knowledge pages.

### Agent profiles (v3 specialization metadata)

Agent profiles now support optional specialization fields for granular role targeting:

- `mnemonic_name` — short human conversational label for the profile
- `domains` — list of domain tags (e.g. `[frontend, ui, react]`)
- `primary_files` — list of file globs the agent typically works with
- `review_focus` — list of review focus areas (e.g. `[accessibility, path-safety]`)
- `context_pack` — preferred context-pack tool (`spec` or `review`)
- `constraints` — list of operational constraints

All fields are optional. Existing v2 profiles without these fields continue to parse correctly.

### Agent proposals (v3 improvement loop)

Agent proposals now support a `proposal_type` field (`new-profile` or `improvement`), a `target_profile` field for improvement proposals targeting existing profiles, and an optional `proposed_mnemonic_name` for profiles that will be materialized later. Improvement proposals follow the same lifecycle as new-profile proposals but require operator approval before behavior-affecting changes become active.

### Project-scoped skills

Skills are `SKILL-*` Markdown artifacts under `.lmbrain/skills/<status>/`. They document reusable project procedures such as build, test, diagnostic, release, and review runbooks.

Skills are not executable capabilities. LMBrain parses and displays skill commands, falls back to fenced commands in the Procedure body when legacy frontmatter is empty, includes applicable active skills in context packs, and reports reference diagnostics, but it does not run skill commands automatically.

The app exposes skills through a dedicated `Skills` page rather than adding them to `Agents & MCP`. Specs and agent profiles may reference skills through optional `skills: []` frontmatter.

### Project insights

The Insights page is backed by `get_project_statistics`, while Agents & MCP also calls `get_agent_improvement_insights`. The latter deterministically aggregates categorized review evidence by profile and distinct spec, exposes fast-fail/cycle/first-pass/escalation metrics, and never mutates artifacts. Improvement proposal creation and application are separate governed MCP operations.

Local Harnesses is backed by `commands::harnesses`. Read-only probes resolve the same executables used by Sessions and execute only `--version`. Mutating updates are fixed per host, serialized by `HarnessManager`, rejected while matching sessions run, executed off the command thread with bounded time/output, and followed by an authoritative re-probe. No updater is run through an interpolated shell command.

OpenCode uses the same `AgentHost`/`ModelRoute` boundary as Pi and is launched
through Ollama. `commands::opencode_registration` owns the idempotent,
structure-preserving merge of the native local MCP entry in project
`opencode.json` plus a default-on LSP policy when no operator policy exists;
provider selection is supplied as session-scoped inline configuration for the
local Ollama OpenAI-compatible API. LMBrain starts OpenCode directly with the
selected workspace positional, avoiding nested-process cwd ambiguity on Windows.

Review-quality metrics are spec-centric where possible and cycle-aware. Valid structured lifecycle events are authoritative; explicit legacy cycle/count fields provide medium-confidence fallback, while status-only histories are excluded from first-pass claims unless separate dated review artifacts establish ordering. Metrics expose lifecycle and canonical-taxonomy coverage, confidence, remediation/escalation/takeover counts, and original-implementation-agent attribution. Taxonomy-v1 aliases normalize at read time with raw values retained; unknown categories remain diagnostics and never silently enter recurrence thresholds.

### Milestone intelligence (v3)

The milestone intelligence view is backed by `get_milestone_overview`, a Tauri command that joins data from `ROADMAP.md`, `build_specs`, `build_reviews`, and `build_adrs` to produce a derived `MilestoneOverview`. The overview includes per-milestone spec counts by status, linked specs with metadata, linked reviews, linked decisions, unresolved reference warnings, dependency status, and a recommended next action.

The derived data is produced by `contract::build_milestone_overview` and returned as a new `MilestoneOverview` model (not an extension of the existing `Roadmap` model, preserving backward compatibility). The frontend renders it as a sidebar/detail layout with click-through artifact navigation.

Design mockups under `.lmbrain/design/` are regular files rather than managed lifecycle artifacts. The backend scans that subtree with the same workspace path guard used for other local reads, and the frontend previews HTML with an isolated iframe surface.

Specs are the board unit. Current spec statuses are:

```text
backlog -> ready -> working -> review -> done
discarded
```

Specs may declare hard `depends_on: [SPEC-*]` prerequisites. The core builds the canonical acyclic graph, reports missing/self/duplicate/cycle diagnostics, and blocks normal readiness/start until every direct prerequisite is done. `spec_dependencies_set` is an audited backlog-only replacement; `spec_dependency_candidates` reports explicit legacy prose without promoting it.

`spec_park` is a dedicated `ready -> backlog` operation with required reason, optional revisit condition, append-only parking history, and recoverable atomic move behavior. It is deliberately absent from the app along with approval and all other status-changing actions; Board and spec detail only visualize dependencies and parking history.

`ready -> working` and `working -> review` are implementer-owned transitions. Submission mechanically requires a scoped non-empty Verification transcript and rejects stale kit-generated evidence. A spec stays in `review` while changes-requested findings are remediated; it is not moved back to `working`.

Tasks are not a first-class board artifact in the current product.

## Controlled Mutation Core

`lmbrain-core` is Tauri-free. It contains the controlled mutation logic shared by the app and the MCP server:

- artifact kind and path mapping;
- frontmatter parsing and surgical updates;
- atomic file writes and moves;
- state transitions;
- field setters;
- creation with allocated IDs;
- invariant checks.

Agents should use the MCP tools backed by this core instead of editing managed frontmatter by hand.

## MCP Server

`lmbrain-mcp` is a JSON-RPC stdio server. It supports initialization, `tools/list`, and `tools/call`. It accepts a workspace root from `--root`, `--root=<path>`, `LMBRAIN_ROOT`, or current working directory.

The server exposes specific tools such as:

- `spec_ready`, `spec_start`, `spec_submit`, `spec_done`, `spec_discard`, `spec_park`;
- `spec_dependencies_set`, `spec_dependency_context`, `spec_dependency_candidates`;
- `spec_attest_lead` plus the desktop-only operator evidence-attestation command; neither attestation operation changes spec status;
- `review_accept`, `review_changes_requested`, `review_block`, `review_supersede`, `review_remediation`, `review_escalate`, `review_takeover`, `review_migration_preview`;
- `verification_migration_preview`, which proposes unambiguous legacy Lead-owner corrections without mutating artifacts;
- `adr_accept`, `adr_reject`;
- `agent_activate`, `agent_deactivate`;
- `skill_activate`, `skill_retire`;
- `lmbrain_create`;
- `lmbrain_set_recommended_agent`;
- `lmbrain_set_agent_mnemonic_name`;
- `lmbrain_get_artifact`;
- `lmbrain_validate`;
- `lmbrain_feedback_record`, an autonomous Project Lead append to the portable kit-feedback report, and `lmbrain_feedback_report`, its read-only typed view;
- `lmbrain_list_ready_handoffs`.
- `harness_config_get`, `harness_config_validate`, `harness_config_set`.

### V3 context-pack tools (added in kit 2.2.7)

- `lmbrain_project_digest` — compact project overview: title/status, current milestone, ready/review specs, blockers, ready handoffs, active decisions, diagnostics summary, and version/health warnings. Returns JSON and Markdown summary. No required parameters.
- `lmbrain_spec_context` — spec handoff context: spec metadata, acceptance criteria checklist, linked decisions, recommended agent profile summary, applicable active skills, related reviews, referenced milestone, explicit files/areas, and diagnostics affecting the handoff. Returns JSON and Markdown summary. Requires `spec` parameter (ID or path).
- `lmbrain_review_context` — review context: acceptance criteria, implementation evidence, linked accepted/proposed reviews, relevant decisions, verification commands claimed by the specialist, and applicable verification/review skills. Returns JSON and Markdown summary. Requires `spec` parameter (ID or path).

All context-pack tools are read-only. They resolve references through existing ID/path logic and report missing links as structured warnings. Spec and review context include the lossless Required verification source, typed owner/phase/evidence requirements, profile guidance and digests, and applicable skill commands/digests. They are backed by `lmbrain-core/src/context.rs`.

Kit feedback is implemented separately from project findings in `lmbrain-core/src/kit_feedback.rs`. It uses a fixed report path and identity, typed append-only notes, the shared mutation lock and atomic writer, and minimal non-sensitive evidence. It never enters project lifecycle metrics or diagnostics.

Project orientation uses diagnostic schema v1 and project-digest schema v2. `lmbrain-core` is the single diagnostic rule engine for MCP validation, the bounded digest, Tauri statistics, and the Pulse drill-down. Each finding has a stable ID, code, severity, artifact/path, safe next action, and fixability. Digest counts and compatibility warning strings are derived from that same collection; bounded lists always report omitted counts. `STATUS.md` remains declared narrative state, while the digest separately derives lifecycle focus and reconciles it with `ROADMAP.md` and spec milestone frontmatter.

### Verification and governed improvement tools (2.9.0)

- `verification_manifest_get`, `verification_manifest_status`, `verification_manifest_init`, `verification_manifest_validate`, `verification_manifest_set`, and `verification_manifest_rollback` implement typed, deterministic, non-executing onboarding with atomic replacement, stale-write protection, and one-step recovery.
- `verification_manifest_approve` remains a separate MCP/operator action; the app intentionally exposes no approval or artifact-status action in its Verification settings. `spec_verify` implements digest-bound named-gate execution and attributable transcript generation only for the currently approved digest.
- `finding_create`, `finding_plan`, `finding_defer`, `finding_resolve`, `finding_accept_risk`, `finding_supersede`, and `finding_reopen` own the `FINDING-*` lifecycle. Core validates global identity, status directories, typed audit events, source-pair uniqueness, typed references, blocker cycles, authority, and resolution evidence under the shared mutation locks.
- `finding_context` derives bounded forward/reverse relations without rewriting review history; `finding_candidates` inventories stable-form legacy review tokens read-only and never infers disposition. Project/spec/review contexts and shared diagnostics consume the same core finding index.
- The desktop Findings route is read-only: filters, sorting, canonical relationship navigation, contextual counts, and governed prompt copying do not expose lifecycle transitions. Review verdicts and agent metrics remain review-based and are not recomputed from promoted findings.
- `agent_improvement_signals`, `agent_improvement_propose`, `agent_proposal_approve`/`reject`, and `agent_improvement_apply` implement a deterministic review-to-proposal-to-approved-additive-profile loop with stale-target protection.

It intentionally does not expose task tools. Operator-governed transitions such as ADR decisions and agent activation are exposed as explicit tools so the Project Lead can execute them only after a direct operator instruction.
