# Kit Migrations

This document describes how to update an existing LMBrain kit between released versions.

## Current policy

The current released kit is `2.7.0`.

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
