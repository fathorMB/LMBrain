# Kit Migrations

This document describes how to update an existing LMBrain kit between released versions.

## Current policy

The current kit is `4.2.2`.

### 4.2.2 (governed verification gates and visible acceptance criteria)

Supported source version is `4.2.1`. This maintenance release changes no artifact schema and requires no content rewrite.

1. Update the desktop application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. Preserve project-specific content while realigning kit-owned release documentation and update `.lmbrain/VERSION` to `4.2.2` after validation.
3. Run `lmbrain_validate`.

Specs may now declare their executable gates through `spec_set_verification_gates`. Existing specs are unaffected: an empty `verification_gates` set remains valid and simply executes no gate. Specs that already reference gates keep their references, and the new optional `verification_gate_events` field only appears once a governed replacement is recorded.

Validation reports `acceptance-criterion-unsatisfied` for each criterion that is not satisfied or validly waived on a spec in `review` or `done`. Existing workspaces may therefore surface new informational diagnostics on specs already in review, and warnings on legacy `done` specs closed before the invariant was enforced. This is reporting only: no transition behaviour changes, and `spec_done` remains the single hard gate.

Rollback to 4.2.1 leaves workspace data intact. `verification_gate_events` written by 4.2.2 is preserved as unread frontmatter, and the new diagnostics simply stop being reported.

### 4.2.1 (real-time refresh after controlled migrations)

Supported source version is `4.2.0`. This maintenance release changes no artifact schema and requires no content rewrite.

1. Update the desktop application and bundled kit together.
2. Preserve project-specific content while realigning kit-owned release documentation and update `.lmbrain/VERSION` to `4.2.1` after validation.
3. Run `lmbrain_validate`. Existing `DEBT-*` artifacts and review-local `RF-*` identifiers remain unchanged.

The desktop file watcher now survives an atomic replacement of `.lmbrain`, such as the controlled 4.2.0 debt migration swap. It reattaches to the replacement directory and refreshes sidebar unread badges without a manual refresh or page change.

Rollback to 4.2.0 does not change workspace data, but the older desktop watcher can again become detached if `.lmbrain` is replaced while the application is open.

### 4.2.0 (durable Debts and review-local RF identifiers)

Supported source version is `4.1.0`. This is a breaking, controlled migration: legacy aliases and fallback readers are intentionally not retained.

1. Commit or back up the complete workspace and require a clean worktree. Update the desktop application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together, but do not edit `.lmbrain/VERSION` yet.
2. Call `debt_migration_preview`. Review its deterministic path/reference inventory, the per-review `FINDING-*`/`F-*` to `RF-*` mappings, and the returned preview digest. Preview is read-only and fails on malformed artifacts, unsafe paths, ambiguous review wikilinks, or unresolved migration input.
3. After explicit operator confirmation, call `debt_migrate` with `confirmed: true` and the exact `expected_preview_digest`. The operation stages the entire brain, renames `.lmbrain/findings/` to `.lmbrain/debts/`, converts durable IDs to `DEBT-*`, converts `finding_events` and `finding_*` lifecycle references, converts review-local identifiers to `RF-*`, and changes the heading to `## Review findings`.
4. The migration validates the staged workspace before swapping it into place. It updates `.lmbrain/VERSION` to `4.2.0` only after validation succeeds. Any preflight or staging failure leaves the original `.lmbrain/` unchanged; a failed final swap restores the pre-migration directory.
5. Run `lmbrain_validate`; verify unique IDs, resolvable references, matching status directories, preserved lifecycle chronology/evidence, and successful waived criteria using `waived=DEBT-*`.
6. Run a zero-residue search over operational content: `rg "FINDING-|finding_events|finding_(create|plan|defer|resolve|accept_risk|supersede|reopen|context|candidates)" .lmbrain --glob '!MIGRATIONS.md' --glob '!CHANGELOG.md'`. Investigate every result; do not add compatibility aliases.
7. Review the complete Git diff before committing. Confirm the desktop **Debts** route, MCP catalog, review-local findings, and relevant project/spec/review context packs.

Rollback is restoration of the pre-migration Git commit or backup. Do not attempt a reverse in-place rename and do not run a 4.1.x binary against migrated `DEBT-*` artifacts.

### 4.1.0 (Dream Journal, grounded dreaming, and review-cycle insights)

Supported source version is `4.0.3`. The migration is additive and no existing artifact is rewritten automatically.

1. Update the desktop application, `lmbrain-core`, `lmbrain-mcp`, and the bundled kit together. Review the diff before copying kit-owned files; preserve project-specific profiles, skills, plans, and historical artifacts.
2. Replace the kit-owned `AGENT.md` and `templates/project-lead-bootstrap-prompt.md` guidance. The Project Lead must enter dreaming only after an explicit operator invitation (for example, “fatti un pisolino” or “vatti a riposare”), confirm the bounded scope, and never activate dreaming from ordinary exploratory conversation.
3. Tell the Lead to use `lmbrain_project_digest` and concrete artifact references as its grounding set, then use `dream_capture` only for tentative observations. Every new `DREAM-*` needs `technical-debt` or `design-debt`, confidence, a context digest, related artifact IDs, rationale, and a suggested disposition. Do not import terminal transcripts or infer unverified facts.
4. Do not convert existing findings, roadmap notes, review bullets, or specs into dreams. Existing project content remains unchanged. The first `dream_capture` creates `.lmbrain/dreams/captured/` as needed; no manual status-directory move or frontmatter edit is allowed.
5. Dream promotion remains manual and governed. The Lead must explicitly discuss/triage a dream before creating or linking a Finding, Spec, ADR, or backlog entry. The desktop Dream Journal is read-only by design.
6. In Insights, interpret the remediation ranking only where lifecycle history is available. Status-only legacy reviews are intentionally excluded and must not be treated as first-pass or zero-cycle evidence.
7. Run `lmbrain_validate`, inspect the migration diff and Dream Journal, then update `.lmbrain/VERSION` to `4.1.0` only after validation succeeds.

Suggested migration prompt for the Project Lead:

> Read `CONTRACT.md`, `QUALITY.md`, `AGENT.md`, `MIGRATIONS.md`, and `VERSION`. We are upgrading LMBrain from 4.0.3 to 4.1.0. Preserve project-specific content and do not rewrite existing artifacts. Explain the new explicitly-invited dreaming boundary, inspect the current project digest, and report whether any migration action is needed. Do not create a dream, finding, spec, or roadmap entry unless I explicitly request that follow-up.

Rollback to 4.0.3 is data-safe for existing artifacts. Keep any `DREAM-*` files: 4.0.3 ignores the new family, but it must not be used to edit or relocate it. Do not run `dream_capture` with an older MCP binary.

### 4.0.3 (maintenance release — accessibility taxonomy, attribution integrity, delegated operator attestation, feedback lifecycle, Linux fixes, worktree visibility)

Supported source version is `4.0.2`. Existing workspaces require no manual content migration.

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. Review finding taxonomy v1 gains the canonical `accessibility` category with aliases `a11y`, `wcag`, and `accessibility-fix`; previously recorded raw `accessibility` values normalize at read time and now count toward recurrence signals and category coverage (#92).
3. Governed review writes validate agent attribution: an `implementation_agent` or `remediation_agent` that is an unreplaced template placeholder (`AGENT-XXX`) or does not resolve to an existing `AGENT-*` profile is rejected; existing artifacts surface `review-attribution-unresolved` diagnostics; the new `review_set_implementation_agent` verb corrects a wrong attribution by fixing the field and appending an `attribution-correction` event (#93).
4. An operator gate approved out of band can be closed without forcing: `spec_attest_operator_delegated` records the operator attestation with the operator's name, the channel, and the quoted authorization, auditable as delegated via new optional attestation fields (#94).
5. Kit feedback notes gain an append-only lifecycle: `lmbrain_feedback_resolve` appends `resolved`/`reconfirmed` events, the report derives per-note status, and the desktop Feedback view and JSON export surface it (#95).
6. `harness_plan_preview` browser readiness probes the Playwright-managed Chromium executable the provider actually launches (honoring the revision pinned by `playwright-core`) instead of any `chromium-*` directory name, and the not-ready detail names the install command (#96).
7. Desktop fixes for Linux (WebKitGTK): design mockup previews use the platform-correct custom-protocol URL instead of rendering silently blank, and native `<select>` controls are normalized so filter dropdowns keep the app styling (#97, #98).
8. The Repository view lists linked git worktrees (agent workspaces) with per-worktree changed files and diffs, resolved exclusively from git's own worktree registry (#99).
9. Update `.lmbrain/VERSION` to `4.0.3` after validating the release.

Rollback to 4.0.2 is data-safe: 4.0.3 introduces no breaking frontmatter schema changes. Attestations carrying the new delegation fields and feedback reports carrying `resolutions` remain parseable by 4.0.2, which ignores the extra fields.

### 4.0.2 (maintenance release — frontmatter integrity, Lead-managed environment, governed browser capability)

Supported source version is `4.0.1`. Existing workspaces require no manual content migration.

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. The frontmatter parser no longer nests every key after an empty-valued key (e.g. the template's `area: `), which was the root cause of duplicate `activity:` blocks written by every governed setter (#82). Contended artifact locks now retry correctly on Windows instead of failing instantly (#83).
3. Artifacts already corrupted by duplicate top-level keys can be repaired with the new operator-authorized `lmbrain_repair_frontmatter` verb; the repair is audited in the activity log (#83). `spec_dependency_context` now reports malformed specs explicitly instead of silently shrinking the graph (#85).
4. Creation normalizes list-valued fields (e.g. `related_decisions`) and validation reports `scalar-in-list-field` diagnostics (#84).
5. The harness manifest supports the typed `browser_mcp` capability for Claude Code (operator-provisioned Playwright MCP, isolated headed profile) (#86).
6. Environment authority moves to the Project Lead: approval, materialization, and revocation are MCP verbs (`harness_manifest_approve`, `harness_config_apply`, `harness_approval_revoke`); the desktop app's Environment page is read-only consultation, and the Settings tabs Project environment and Verification are removed (#87). Existing machine-local approvals are preserved.
7. Verification transcript parsing is fence-aware: Markdown heading lines pasted inside the ```` ``` ```` fence (e.g. generated reports) no longer truncate the section or make `spec_submit` report an empty transcript, and `spec_verify` splices its managed region correctly around them (#90).
8. Update `.lmbrain/VERSION` to `4.0.2` after validating the release.

Rollback to 4.0.1 is data-safe: 4.0.2 introduces no breaking frontmatter schema changes; the optional `browser_mcp` capability must be removed from `HARNESSES.json` before rolling back, since 4.0.1 rejects unknown fields.

### 4.0.1 (maintenance release — kit feedback fixes KIT-NOTE-001 through KIT-NOTE-015)

Supported source version is `4.0.0`. Existing workspaces require no manual content migration.

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. `activity:` YAML list handling now supports inline `activity: []` arrays and prevents duplicate mapping keys across spec metadata mutations.
3. Diagnostic `next_action` text for project status and milestone explicitly names frontmatter keys (`status:`, `milestone:`).
4. Validation rules ignore discarded specs for roadmap membership checks, reserve prose-referenced `FINDING-\d+` IDs during allocation, and validate declared executable gates against approved verification manifests.
5. Update `.lmbrain/VERSION` to `4.0.1` after validating the release.

Rollback to 4.0.0 is data-safe: 4.0.1 introduces no breaking frontmatter schema changes.

### 4.0.0 (governed spec tags, implementation estimates, decision supersession, and declared branching strategy)

Supported source version is `3.1.4`. Existing workspaces keep parsing unchanged: the new fields and configurations are optional in the parser.

1. Update the desktop application, `lmbrain-core`, `lmbrain-mcp`, and the bundled kit together.
2. **No automatic rewrite happens.** Existing `tags` values are preserved exactly as written.
3. Review the new `field-restating-tag` diagnostics. Tags that duplicate `milestone`, `area`, or `priority` (for example `3.1.0` or `milestone-m02`) remain readable but are rejected by the next `spec_set_tags` mutation. Clean them deliberately.
4. Assign `capability_tier` and `thinking_level` through `spec_set_effort` to specs you intend to move to `ready`. The `ready` transition now fails closed without them.
5. Specs already in `ready`, `working`, or `review` without an estimate are not blocked; they report `missing-effort-estimate` until a Lead sets one.
6. `supersedes` and `superseded_by` on decisions are now read, validated, and displayed. They were previously declared by the template and used by nothing, so one-sided relationships are expected in existing workspaces.
7. Review the new `dangling-supersession` and `supersession-not-mutual` diagnostics. Each names a decision that a successor claims to have retired but which is still presented as authoritative. Running `adr_supersede` on the pair repairs both sides; the verb is idempotent, so it is safe to re-run.
8. **Declared Branching Strategy.** Repositories without `.lmbrain/BRANCHING.json` report state `absent` (`unconfigured`). Use operator-governed `branching_strategy_set` to define or initialize your project's strategy.
9. Update `.lmbrain/VERSION` to `4.0.0` after validating the release.

Rollback to 3.1.4 is data-safe: the new frontmatter fields and `.lmbrain/BRANCHING.json` are ignored by the older parser, and no existing field changed shape.

### 3.1.4 (responsive workspace snapshot loading)

Supported source version is `3.1.3`. Existing workspaces require no artifact or frontmatter migration.

1. Update the desktop application and bundled kit together.
2. No project content rewrite is required; the change is limited to desktop loading, refresh coordination, and statistics reuse.
3. Update `.lmbrain/VERSION` to `3.1.4` after validating the release.

Rollback to 3.1.3 is data-safe but restores the main-thread loading regression fixed by 3.1.4.

### 3.1.3 (kit-owned file realignment procedure, Node REPL kernel path fix, Browser URL policy alignment, lead gate auto-check, waived criteria, async background loading)

Supported source version is `3.1.2`. Existing workspaces require no manual content migration.

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. **Kit-owned file realignment audit**: Compare project kit-owned files inside `.lmbrain/` (`CHANGELOG.md`, `README.md`, `MIGRATIONS.md`, `reports/README.md`, and `templates/`) against the bundled kit defaults. Realign kit-owned files that contain only additive release lines, while strictly preserving project-specific customizations (such as agent profiles and skill registers).
3. Attesting evidence for an `owner=lead` verification gate via `spec_attest_lead` now automatically marks the checklist item `- [x]` in the spec body, aligned with `owner=operator` behavior.
4. Spec closeout via `spec_done` now supports waived acceptance criteria syntax (`- [~] text | waived=FINDING-xxx`) when referenced active findings exist.
5. Project Lead remediation verification checks can now be recorded without status changes using `review_remediation_verified` (`actor_role: project-lead`).
6. Update `.lmbrain/VERSION` to `3.1.3` after validating the release.

Rollback to 3.1.2 is data-safe: 3.1.3 introduces no breaking frontmatter schema changes.

### 3.1.2 (bundled template fix, operator verification auto-check, kit feedback UI, spec assignment disambiguation)

Supported source version is `3.1.0` or `3.1.1`. Existing workspaces require no manual content migration.

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. `lmbrain_validate` and artifact discovery now automatically exclude `.lmbrain/templates/` from live artifact checks, so bundled templates (e.g. `templates/finding.md`) can be used as-is without raising status-directory mismatch diagnostics.
3. In the desktop application, attesting evidence for an `owner=operator` verification gate now automatically marks the checklist item `- [x]` in the spec body, eliminating the need to manually check the item outside the app.
4. Operator feedback notes recorded in `reports/lmbrain-kit-feedback.md` can now be reviewed directly in the application via the new **Kit Feedback** view in the sidebar.
5. Update `.lmbrain/VERSION` to `3.1.2` after validating the release.

Rollback to 3.1.0 is data-safe: 3.1.2 introduces no new artifact shapes or frontmatter schemas.

### 3.1.1 (templates isolation, scaffolding exclusion, Wiki report, Pulse backlog & shared reliability)

Supported source version is `3.1.0`. Upgrade is explicit, additive, and backward compatible:

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. Template `.lmbrain/templates/finding.md` containing `id: FINDING-XXX` can be copied without manual edits: templates are isolated from live artifact discovery and validation.
3. No manual project content changes are required for existing migrated projects.
4. Review diagnostics with `lmbrain_validate` and change `.lmbrain/VERSION` to `3.1.1`.

### 3.1.0 (governed findings, lifecycle integrity, diagnostics, and verification onboarding)

Supported source version is `3.0.2`. Upgrade is explicit and additive:

1. Update the application, `lmbrain-core`, `lmbrain-mcp`, and bundled kit together.
2. Preview the Git diff, then add `findings/` status directories, their README files, and `templates/finding.md`. Opening or refreshing a workspace never creates them.
3. Run `finding_candidates` to inventory stable-form legacy review entries. The report is read-only and treats `origin_artifact + origin_ref` as the candidate identity; repeated local tokens across reviews are not duplicates. Select promotions manually and create only observations that remain durable.
4. Validate all new references and statuses with `lmbrain_validate`. Do not promote unresolved verification gates automatically and do not reopen limitations already resolved by documentation/evidence.
5. If adopting repository verification gates, use `verification_manifest_init` for a non-executing preview, then validate/set the complete manifest. Approval remains a separate operator action and is intentionally absent from the app.
6. Run `verification_migration_preview` to inventory `owner=operator` verification requirements whose text describes a Project Lead action. Review each candidate and reclassify to `owner=lead` where appropriate. Requirements that describe operator actions stay `owner=operator`. This step is essential for projects that had before-done verification requirements before `owner=lead` existed.
7. Add `depends_on: []`, `dependency_events: []`, and `parking_events: []` to the spec template. Existing specs may omit them and behave as dependency-free, never previously parked.
8. Run `spec_dependency_candidates` to inventory only explicit legacy hard-dependency prose. Review candidates manually, then use `spec_dependencies_set` while the spec is in backlog. Never infer or promote prose automatically.
9. A ready spec whose contract must change first uses `spec_park`; the desktop app deliberately offers no approval or status-change action. Re-entry still requires normal `spec_ready`.
10. Add `reports/lmbrain-kit-feedback.md` and the updated Project Lead/bootstrap instructions. Existing project feedback is not inferred. The Lead may begin appending typed notes autonomously after migration.
11. Review the final Git diff and diagnostics before changing `.lmbrain/VERSION` to `3.1.0`.

Rollback to 3.0.2 preserves Markdown evidence: older LMBrain versions ignore the `findings/` family, `depends_on`, typed event fields, and the kit feedback report but must not delete them. They do not enforce dependency prerequisites or understand semantic parking/feedback writes, so do not perform those mutations with an older binary. Do not move finding or feedback content into STATUS/BACKLOG as a competing lifecycle source. A 3.1 verification manifest using only the prior schema remains parse-compatible, but machine-local approval should be revoked/reviewed when changing app versions.

### 3.0.2 (Antigravity MCP support, complete Actions panel, declared build outputs)

Supported source version is `3.0.1`; existing workspaces require no content migration.

1. Update the application, `lmbrain-core`, and `lmbrain-mcp` together.
2. Antigravity users: reopen the workspace in LMBrain so the `lmbrain` entry is merged into the user-global Antigravity `mcp_config.json`, then reload MCP servers in the Antigravity IDE. The entry targets the most recently opened workspace.
3. Optionally declare `fingerprint_exclude` on `verification.toml` gates that write build artifacts, then re-approve the manifest digest — declaring an exclusion always invalidates the previous approval. Manifests without exclusions keep their digest and approval.
4. Update `.lmbrain/VERSION` to `3.0.2` after validating the release.

Rollback to 3.0.1 is data-safe with one caveat: a manifest that declares `fingerprint_exclude` fails strict parsing on older versions (fail-closed). Remove the field and re-approve if rolling back.

### 3.0.1 (installer publication gate correction)

Supported source version is `3.0.0`; existing workspaces require no content migration.

1. Update the application and bundled kit together.
2. No `.lmbrain/` artifacts, frontmatter, configuration, or GitHub credentials need to be rewritten.
3. Update `.lmbrain/VERSION` to `3.0.1` after validating the release.

Rollback to 3.0.0 is data-safe because 3.0.1 changes no artifact shapes or runtime persistence contracts.

### 3.0.0 (Git & GitHub Dashboard and Session Transcript Search)

Supported source version is `2.9.2`; existing workspaces require no content migration.

1. Update the application, `lmbrain-core`, and `lmbrain-mcp` together.
2. Setup GitHub Personal Access Token (PAT) in the new Repository dashboard to securely view remote Pull Requests and workflow runs.
3. Use the new "Search logs" button inside terminal sessions to search log histories, select lines, and copy code blocks.
4. Update `.lmbrain/VERSION` to `3.0.0` after validation.

Rollback to 2.9.2 is safe: 3.0.0 writes no new artifact shapes.

### 2.9.2 (security and workflow correctness patch)

Supported source version is `2.9.1`; existing workspaces require no content migration.

1. Update the application, `lmbrain-core`, and `lmbrain-mcp` together. No app process needs to be running to apply the library/MCP fixes.
2. Behavior change — creation requests: `lmbrain_create` now fails closed on non-initial statuses, reserved core-owned fields, and malformed field pairs that earlier versions accepted accidentally. Adjust any automation that relied on those values; the tool schema itself is unchanged.
3. Behavior change — verification: a `spec_verify` run whose workspace content changes between the first and last gate is recorded as invalidated and cannot back a submission. Rerun verification on a quiescent workspace.
4. Update `.lmbrain/VERSION` to `2.9.2` after validation.

Rollback to 2.9.1 is safe: 2.9.2 writes no new artifact shapes. Transcripts generated by 2.9.2 add two metadata comment lines that 2.9.1 ignores.

### 2.9.1 (verification transcript data-loss fix)

Supported source version is `2.9.0`; no manual artifact rewrite is required.

1. Update the application, `lmbrain-core`, and `lmbrain-mcp` together.
2. Keep agent-authored evidence in `### Verification transcript`. On the next `spec_verify` run, LMBrain migrates the legacy generated transcript into its own delimited region and preserves manual evidence before or after it.
3. If verification reports that the spec moved or its `verification_gates` changed while gates were running, inspect the current artifact and rerun. LMBrain intentionally leaves the current file untouched.
4. Update `.lmbrain/VERSION` to `2.9.1` after validation.

Rollback restores 2.9.0 code but must retain all transcript evidence. A transcript containing the 2.9.1 managed-region comments remains readable Markdown, although 2.9.0 must not be used to regenerate it because that version replaces the full section.

### 2.9.0 (verification integrity and governed agent improvement)

Supported source versions are `2.8.x`; changes are additive except for the new submit invariant.

1. Add `### Verification transcript` beneath `## Implementation evidence` in active spec templates and working specs before submission. Paste actual command/output in a fenced block; summaries are not execution evidence.
2. Convert Required verification entries gradually to `ID | kind=... | owner=... | phase=... | evidence=... | text`. Legacy prose remains visible with warnings and is never silently rewritten.
3. Optionally add `.lmbrain/verification.toml` with strict named direct-program gates and add `verification_gates` references to specs. Review the manifest, then approve its exact digest locally; repository content alone does not authorize execution.
4. Optionally add `finding_categories` and `implementation_agent` to review frontmatter. Historical freeform reviews remain uncategorized rather than guessed.
5. Improvement proposals created by 2.9 store target digests and additive patch fields. Approve explicitly and apply only while the target digest is current.
6. Validate the project, run canonical Rust/frontend gates, then update `.lmbrain/VERSION` to `2.9.0`.

Rollback restores 2.8.x and the prior template/version. Remove only unused verification policy after revoking its local approval; retain generated transcripts and applied proposal audit history.

### 2.8.0 (project harness governance — planned additive migration)

Supported source versions are `2.7.x`. Add `.lmbrain/HARNESSES.json` only when the operator chooses to adopt project harness governance; opening an existing project must not create or apply it. The initial safe manifest is `{ "schema_version": 1, "hosts": {} }`. Review all host requirements and the native-file preview before granting the machine-local digest-bound approval. Validate schema diagnostics and confirm that no native harness configuration changes before explicit approval. Roll back by revoking local approval and removing the manifest only after confirming it contains no project intent that must be preserved.

### 2.7.3 (Windows installer test reliability)

`2.7.3` removes an environment dependency from the Windows installer test gate. Runtime behavior, project configuration, and the Markdown artifact contract are unchanged.

When upgrading from `2.7.2`:

1. No `.lmbrain/` artifacts, frontmatter, or configuration need migration.
2. Update `.lmbrain/VERSION` to `2.7.3` after validating the release.
3. Roll back by restoring LMBrain `2.7.2` and the prior `.lmbrain/VERSION`; no artifact rollback is required.

### 2.7.2 (OpenCode environment and packaged terminal fixes)

`2.7.2` explicitly anchors OpenCode to the selected workspace while routing its
model through a session-scoped local Ollama provider,
enables OpenCode built-in LSP integration when the project has no existing LSP
policy, and updates the embedded terminal scrolling contract. The Markdown
artifact contract is unchanged.

When upgrading from `2.7.0` or `2.7.1`:

1. No `.lmbrain/` artifacts or frontmatter need migration.
2. Reopen the workspace so LMBrain can merge `lsp: true` into generated `opencode.json`; an existing `lsp: false` or custom `lsp` object is preserved.
3. OpenCode may download supported built-in language servers into its user cache. Set `OPENCODE_DISABLE_LSP_DOWNLOAD=true` outside LMBrain if automatic LSP downloads are not allowed.
4. Start an OpenCode session and verify `@` file completion is rooted at the selected workspace.
5. Verify mouse-wheel and explicit Page up/Page down controls in the packaged Windows app.
6. Update `.lmbrain/VERSION` to `2.7.2` after validation.
7. Roll back by restoring LMBrain `2.7.1` and the prior `.lmbrain/VERSION`; remove the generated `lsp` key only if it was introduced by LMBrain and the project does not want LSP integration.

### 2.7.0 (local harness lifecycle - app-only feature)

`2.7.0` adds the Local Harnesses page for probing and explicitly updating user-level Claude Code, Codex, Pi, and OpenCode installations. It also adds OpenCode sessions through Ollama and generated project-local `opencode.json` MCP registration. Harness binaries, authentication, and update state remain outside the project; the Markdown artifact contract is unchanged.

When upgrading an existing `2.6.x` brain to `2.7.0`:

1. No `.lmbrain/` files, directories, frontmatter, or project dependencies need to change.
2. Open Local Harnesses and confirm that each installed executable path/version matches the binary the operator expects LMBrain to launch.
3. If a custom Codex executable is configured in Settings, verify the Codex card reports that exact path.
4. Do not run a harness update while matching sessions are active. LMBrain enforces this gate, but operators should still review the confirmation and updater output.
5. Opening a workspace may create or merge ignored `opencode.json` with only the `mcp.lmbrain` entry; review existing OpenCode project configuration before removing the ignore rule or committing it.
6. Update `.lmbrain/VERSION` to `2.7.0` after validation.
7. Roll back by restoring LMBrain `2.6.x`, deleting only LMBrain's `mcp.lmbrain` OpenCode entry if desired, and restoring the prior `.lmbrain/VERSION`. Harness updates already completed are user-level operations and must be rolled back through that harness's supported installation process if necessary.

### 2.6.1 (Codex alternate-buffer scrolling - app-only fix)

`2.6.1` launches Codex in its supported inline mode to preserve xterm scrollback, keeps buffer-aware wheel routing for other full-screen terminal applications, and adds an explicit current-view data refresh in the app header. The Markdown artifact contract and project configuration are unchanged.

When upgrading an existing `2.6.0` brain to `2.6.1`:

1. No file moves, frontmatter edits, configuration changes, or generated-state cleanup are required.
2. Open a Codex session and verify that mouse-wheel input scrolls the TUI conversation while ordinary terminal output still scrolls local xterm history.
3. Use the header refresh on a view with a resolved diagnostic and verify that stale warnings disappear without restarting sessions or the application.
4. Update `.lmbrain/VERSION` to `2.6.1` after validation.
5. Roll back by restoring LMBrain `2.6.0` and the prior `.lmbrain/VERSION`; no artifact rollback is required.

### 2.6.0 (Pi sessions and workspace/session UX - app-derived)

`2.6.0` adds Pi as an operator-started agent host through Ollama, visible workspace preparation with exact project-local Pi MCP dependency bootstrap, persistent session scrollback/clipboard controls, and actionable Insights reliability details. The Markdown artifact contract and existing project frontmatter are unchanged.

Supported source versions: `2.5.1` and earlier released 2.x kits.

When upgrading an existing brain to `2.6.0`:

1. No `.lmbrain/` file moves, directory additions, or frontmatter edits are required.
2. Open the project with LMBrain `2.6.0` and allow workspace validation to complete. Pi preparation is optional to core workspace access and may create or update project-local `.pi/settings.json`, `.pi/mcp.json`, and `.pi/npm/` state without changing `.lmbrain/` artifacts.
3. Review project-owned `.pi/settings.json` before committing it. Generated `.pi/mcp.json` and `.pi/npm/` content should remain ignored according to the repository policy.
4. Validate that existing artifacts parse, status directories match frontmatter, and controlled MCP tools remain available for the agent hosts the project uses.
5. Update `.lmbrain/VERSION` to `2.6.0` only after validation succeeds.
6. Roll back by restoring the prior application and `.lmbrain/VERSION`. Preserve project-owned Pi settings; remove generated Pi integration state only after reviewing it separately.

### 2.5.1 (project insights statistics - app-derived)

`2.5.1` adds an app Insights page with read-only statistics derived from existing LMBrain artifacts. It does not require artifact rewrites.

When upgrading an existing brain to `2.5.1`:

1. No file moves or frontmatter changes are required.
2. Existing review-quality statistics depend on review `spec` links and `created` dates. Missing links or dates are surfaced as denominator/exclusion counts rather than silently inferred.
3. Update `.lmbrain/VERSION` to `2.5.1` after opening the project with the bundled app and validating that existing artifacts still parse.

### 2.5.0 (project-scoped agent skills - additive)

`2.5.0` adds project-scoped `SKILL-*` procedure artifacts, a dedicated Skills app page, context-pack skill summaries, and controlled skill lifecycle tools. Skills are Markdown runbooks for manually started agents; LMBrain does not execute skill commands automatically.

When upgrading an existing brain to `2.5.0`:

1. Review project-specific customizations before copying any bundled kit file. Do not blindly overwrite existing `AGENT.md`, `CONTRACT.md`, templates, registries, or profile files.
2. Add the `skills/` directory structure with `active/`, `proposed/`, and `retired/` if absent.
3. Add `skills/README.md`, `skills/registry.md`, and `templates/skill.md` if absent.
4. Merge `CONTRACT.md`, `AGENT.md`, `templates/spec.md`, and `templates/agent-profile.md` additions for `SKILL-*` artifacts and optional `skills: []` references.
5. Do not create active project skills automatically. The Project Lead may propose skills after validating concrete project procedures, and the operator approves activation.
6. Validate that existing specs and agent profiles still parse. If custom specs/profiles already have a `skills` field, ensure referenced `SKILL-*` artifacts exist or intentionally leave diagnostics visible until they are created.
7. Update `.lmbrain/VERSION` to `2.5.0` only after the additive merges and validation checks succeed.
8. Roll back by restoring the project `.lmbrain/` diff from version control; this migration does not require destructive file moves.

### 2.4.1 (agent mnemonic names and lifecycle invariant alignment - additive)

`2.4.1` adds human mnemonic names for agent profiles, aligns existing project brains with the corrected spec lifecycle and `spec_done` invariant behavior, and normalizes bundled kit paths in migration prompts on Windows. The artifact contract remains backward-compatible: existing profiles without `mnemonic_name` and proposals without `proposed_mnemonic_name` remain valid.

When upgrading an existing brain to `2.4.1`:

1. Review project-specific customizations before copying any bundled kit file. Do not blindly overwrite existing `AGENT.md`, `CONTRACT.md`, `OPERATOR.md`, templates, profiles, or registries.
2. Add `mnemonic_name` to existing agent profiles where absent. Prefer short human labels that are memorable, lightly ironic, and role-aligned. Bundled defaults:
   - `AGENT-LEAD`: `Ada Checklist`
   - `AGENT-FRONTEND-UI`: `Marta Pixelperfetta`
   - `AGENT-TAURI-BACKEND`: `Bruno Fileguard`
   - `AGENT-MCP-CONTRACT`: `Vera Protocollo`
   - `AGENT-KIT-DOCS`: `Nina Changelog`
   - `AGENT-REVIEWER`: `Clara Redpen`
   - `AGENT-DESIGN`: `Lia Wireframe`
3. Add `proposed_mnemonic_name` to agent proposals where a future profile name is already known. Leave it absent for historical proposals when no suitable name is clear.
4. Merge the bundled `templates/agent-profile.md` and `templates/agent-proposal.md` additions so new profiles/proposals include mnemonic-name fields.
5. Merge `AGENT.md`, `CONTRACT.md`, `OPERATOR.md`, `agents/README.md`, `agents/registry.md`, and `specs/README.md` guidance for:
   - `mnemonic_name` / `proposed_mnemonic_name`;
   - `ready -> working` and `working -> review` being implementer-owned;
   - specs staying in `review` through changes-requested remediation;
   - `spec_done` depending on checked acceptance criteria, implementation evidence, and accepted review.
6. If the project has custom active profiles, keep their status and authority metadata unchanged. Add only the new mnemonic-name metadata unless the operator explicitly approves broader profile changes.
7. Validate with the bundled app and MCP tools. For a project that previously required forced `spec_done` due to the known evidence/criteria false-negative, verify a representative done-ready spec has checked criteria under `## Acceptance criteria`, content under `## Implementation evidence` or `## Evidence`, and an accepted linked review.
8. Update `.lmbrain/VERSION` to `2.4.1` only after the additive merges and validation checks succeed.
9. Roll back by restoring the project `.lmbrain/` diff from version control; this migration does not require destructive file moves.

### 2.3.3 (design preview and Nucleus roadmap fix - additive)

`2.3.3` fixes desktop-app rendering and parsing behavior without changing the Markdown artifact contract. It does not require project artifact rewrites. When upgrading an existing brain to `2.3.3`:

1. No file moves or frontmatter changes are required.
2. Existing `.lmbrain/design/<package>/index.html` mockups can continue to use relative CSS and JavaScript assets; the app inlines those local assets for preview rendering.
3. Existing roadmap milestone IDs such as `M0`, `M4`, and `M-01` remain valid.
4. Update `.lmbrain/VERSION` to `2.3.3` only after validating the project with the bundled app and MCP tools.

### 2.3.2 (design package preview fix - additive)

`2.3.2` improves the desktop app's Design view preview loading for multi-file mockup packages. It does not require project artifact rewrites. When upgrading an existing brain to `2.3.2`:

1. No file moves or frontmatter changes are required.
2. Existing `.lmbrain/design/<package>/index.html` mockups continue to use relative package assets such as `assets/app.js` and `assets/design-system.css`.
3. Update `.lmbrain/VERSION` to `2.3.2` only after validating the project with the bundled app and MCP tools.

### 2.3.1 (approval governance alignment - additive)

`2.3.1` aligns approval governance across the app, kit prompts, and MCP tools. It does not require project artifact rewrites. When upgrading an existing brain to `2.3.1`:

1. No file moves or frontmatter changes are required.
2. Use Project Lead prompts and controlled MCP tools for ADR decisions and agent profile activation/deactivation on explicit operator instruction.
3. Update `.lmbrain/VERSION` to `2.3.1` only after validating the project with the bundled app and MCP tools.

### 2.3.0 (v3 package release - additive)

`2.3.0` is the formal package release for the v3 app/kit workflow. It does not introduce additional project artifact contract changes beyond the `2.2.7` v3 context-economy migration. When upgrading an existing brain to `2.3.0`:

1. If the project is already on `2.2.7`, no file moves or frontmatter changes are required.
2. If the project is older than `2.2.7`, apply the `2.2.7` migration steps below first, preserving project-specific content and custom agent profiles.
3. Validate the migrated project with the bundled app and MCP tools.
4. Update `.lmbrain/VERSION` to `2.3.0` only after validation succeeds.
5. Roll back by restoring the project `.lmbrain/` diff from version control; no destructive migration step is required.

### 2.2.7 (v3 context economy — additive)

`2.2.7` adds context-pack MCP tools (`lmbrain_project_digest`, `lmbrain_spec_context`, `lmbrain_review_context`), granular specialist-profile guidance, and v3 context-economy workflow docs. It is **additive and backward-compatible**: no existing project artifact changes meaning, and no existing artifact frontmatter must be rewritten. When upgrading an existing brain to `2.2.7`:

1. No file moves or frontmatter changes are required.
2. Existing artifacts remain valid.
3. The new MCP tools become available automatically when the app registers `lmbrain-mcp`.
4. The updated handoff prompt includes context-economy guidance; existing prompts still work.
5. Review `AGENT.md`, `CONTRACT.md`, `OPERATOR.md`, and `templates/project-lead-bootstrap-prompt.md` for the updated context-tier guidance.
6. Add missing bundled granular specialist profiles from `agents/profiles/` only when their IDs do not already exist in the project:
   - `AGENT-FRONTEND-UI`
   - `AGENT-TAURI-BACKEND`
   - `AGENT-MCP-CONTRACT`
   - `AGENT-KIT-DOCS`
   - `AGENT-REVIEWER`
   - `AGENT-DESIGN`
7. Add bundled v3 agent proposal examples from `agents/proposals/` only when their IDs or filenames do not already exist. Do not overwrite project-specific proposals.
8. Merge the v3 registry rows and "V3 controlled improvement loop" guidance from `agents/registry.md` additively. Preserve all project-specific active profiles and proposals.
9. Keep existing project-customized agent profiles active/inactive exactly as they are unless the operator explicitly approves a profile status change.
10. Update `.lmbrain/VERSION` to `2.2.7` only after the additive file/registry updates and validation checks succeed.

### 1.1.0 (Contract v0.2 — additive)

`1.1.0` adds the `rejected` status across proposable artifacts and defines Agent-proposal statuses (see [[ADR-003-reject-as-first-class-status]]). It is **additive and backward-compatible**: no existing artifact changes meaning, and no frontmatter must be rewritten. When upgrading an existing brain to `1.1.0`:

1. Add the `specs/rejected/` directory (with a `.gitkeep`).
2. No other file moves or frontmatter changes are required.
3. Existing artifacts remain valid; `rejected` simply becomes an available status.

When any kit-changing version is released, the author MUST document migration guidance for that version in this file. The guidance section (headed by `### <version>`) must include:
1. the supported source version(s);
2. required file additions, moves, renames, or frontmatter edits;
3. any manual review required from the human operator;
4. validation steps to run after upgrading;
5. rollback guidance where applicable.

## Migration principles

- Never silently destroy or overwrite project knowledge.
- Preserve custom project content and unknown Markdown files.
- Prefer additive, backward-compatible changes where possible.
- Use explicit, versioned instructions for breaking contract changes.
- Require human confirmation before a future application performs repository writes for migration.
- Update `VERSION` only after every required migration step and validation check succeed.

## Planned validation after a future migration

- `VERSION` contains the expected released version.
- Required root documents and directories exist.
- Artifact IDs remain unique.
- Status-directory paths and frontmatter status values agree.
- References to specs, reviews, ADRs, agents, MCPs, and handoffs resolve.
- Git diff is reviewed before committing the update.
