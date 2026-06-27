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
- `templates/`: templates for specs, reviews, ADRs, agent profiles/proposals, MCP proposals/specs, and handoffs.

## Artifact Directories

The kit includes directories for:

- `specs/<status>/`
- `reviews/<status>/`
- `decisions/`
- `agents/`
- `mcp/`
- `handoffs/`
- `knowledge/`
- `reports/`

Status-directory artifacts must keep filesystem location and frontmatter `status` aligned. LMBrain surfaces diagnostics for mismatches and related consistency problems.

## Spec Board

The current board tracks specs, not tasks. Spec status values are:

- `backlog`
- `ready`
- `working`
- `review`
- `done`
- `discarded`

Acceptance criteria inside a spec provide sub-spec granularity.

## Versioning

The app and kit share one release version. The canonical kit version is `kit/.lmbrain/VERSION`, and `scripts/check-version.mjs` verifies it against `package.json` and `src-tauri/Cargo.toml`.
