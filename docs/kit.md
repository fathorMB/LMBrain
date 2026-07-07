# Kit

`kit/.lmbrain/` is the clean reusable LMBrain kit bundled with the app and copied into target repositories.

The root `.lmbrain/` directory is not versioned in this repository. If present locally, it is dogfooding state for this checkout only.

## Bundling And Initialization

The Tauri config bundles `../kit/.lmbrain/` as an app resource. In development, the app uses the repository path directly. In production, `initialize_workspace_kit` copies the bundled kit into the selected repository as `.lmbrain/`.

Initialization refuses to overwrite an existing `.lmbrain/`.

## Important Kit Files

- `AGENT.md`: Project Lead operating contract.
- `CONTRACT.md`: Markdown artifact contract and status rules.
- `QUALITY.md`: production quality policy for handoffs.
- `OPERATOR.md`: human operator guide.
- `README.md`: kit entry point.
- `CHANGELOG.md`, `MIGRATIONS.md`, `VERSION`: kit versioning and upgrade information.
- `templates/`: templates for specs, reviews, ADRs, agent profiles/proposals, skills, MCP proposals/specs, and handoffs.
- `design/`: operator-loaded design mockups used as support material for specs and implementation handoffs.

## Artifact Directories

The kit includes directories for:

- `specs/<status>/`
- `reviews/<status>/`
- `decisions/`
- `agents/`
- `skills/`
- `design/`
- `mcp/`
- `handoffs/`
- `knowledge/`
- `reports/`

Status-directory artifacts must keep filesystem location and frontmatter `status` aligned. LMBrain surfaces diagnostics for mismatches and related consistency problems.

`design/` is intentionally not a managed artifact directory. It stores self-contained HTML/CSS/JS mockups and optional README/manifest metadata that the Project Lead may reference from specs.

`skills/` stores `SKILL-*` project-scoped procedures in `active/`, `proposed/`, and `retired/`. Skills are Markdown runbooks for manually started agents; LMBrain displays their commands and includes applicable active skills in context packs, but does not execute them.

## Spec Board

The current board tracks specs, not tasks. Spec status values are:

- `backlog`
- `ready`
- `working`
- `review`
- `done`
- `discarded`

Acceptance criteria inside a spec provide sub-spec granularity.

## V3 context economy

The kit includes context-pack MCP tools for token-efficient agent workflow:

- **Project Lead bootstrap:** Use `lmbrain_project_digest` instead of reading the entire `.lmbrain/` directory.
- **Specialist handoff:** Use `lmbrain_spec_context` for a compact spec context before expanding to full artifacts.
- **Review:** Use `lmbrain_review_context` for acceptance criteria, evidence, linked reviews, and verification/review skills.

Context packs are derived views only. Source artifacts remain the system of record. See `CONTRACT.md` for the full context-pack contract.

## V3 agent taxonomy

The kit ships granular specialist profiles for recurring bounded work:

| Profile | Name | ID | Domains |
| --- | --- | --- | --- |
| Frontend UI Specialist | Marta Pixelperfetta | AGENT-FRONTEND-UI | frontend, ui, react, typescript, css |
| Tauri/Rust Backend Specialist | Bruno Fileguard | AGENT-TAURI-BACKEND | tauri, rust, backend |
| MCP/Contract Specialist | Vera Protocollo | AGENT-MCP-CONTRACT | mcp, contract, core |
| Kit/Docs/Release Specialist | Nina Changelog | AGENT-KIT-DOCS | kit, docs, release |
| Product Reviewer/QA | Clara Redpen | AGENT-REVIEWER | review, qa, testing |
| Design Specialist | Lia Wireframe | AGENT-DESIGN | design, ui-ux |

All profiles use `activation: manual`. The Project Lead recommends the most specific profile for each spec. `mnemonic_name` is a human conversational label only; authority still comes from the profile's `id`, `status`, and capability fields. See `agents/registry.md` for the full registry.

### Controlled improvement loop

Improvement proposals use the existing `agents/proposals/` mechanism with `proposal_type: improvement` and a `target_profile` field. The Project Lead may create improvement proposals from accepted reviews, repeated remediation findings, implementation evidence, diagnostics, or operator feedback. Operator approval is required before any behavior-affecting profile change becomes active.

## Project-scoped skills

The kit supports reusable agent procedures as `SKILL-*` artifacts:

- `skills/proposed/` for drafts;
- `skills/active/` for procedures available to handoffs and context packs;
- `skills/retired/` for obsolete procedures.

Specs and agent profiles may reference skills with `skills: []`. A skill may also declare `applies_to`, `domains`, `commands`, `risk`, and `requires_operator_approval`. Commands are documented instructions for agents to run manually in their assigned environment.

## Versioning

The app and kit share one release version. The canonical kit version is `kit/.lmbrain/VERSION`, and `scripts/check-version.mjs` verifies it against `package.json` and `src-tauri/Cargo.toml`.

## Kit Migration

The application automatically compares the opened workspace `.lmbrain` kit version (from `.lmbrain/VERSION`) against the app's bundled kit version. If the project version is older, a "Migration available" status is surfaced in the project metadata panel on the Project Pulse view.

Instead of performing automated updates that could overwrite custom files or project-specific configurations, the application provides a copyable **Project Lead migration prompt**. When copied and executed by the operator's Project Lead agent:
1. The Project Lead reads `.lmbrain/MIGRATIONS.md` to review the migration steps and history.
2. The Project Lead compares templates and prepares a migration plan.
3. The Project Lead requests operator confirmation before writing any changes.
4. The Project Lead updates `.lmbrain/VERSION` only after validation tests succeed.
