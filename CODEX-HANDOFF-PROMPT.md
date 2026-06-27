# Task: add OpenAI Codex CLI support to LMBrain (parity with Claude Code)

You are working in the LMBrain repository (E:\Git\LMBrain): a desktop app built with
Tauri 2 (Rust backend) + React 19 + TypeScript + Vite, with a Cargo workspace
(`src-tauri`, `lmbrain-core`, `lmbrain-mcp`). Windows is the primary platform.

Read `CODEX-SUPPORT-PLAN.md` in the repo root FIRST — it is the source of truth for
this work (analysis, locked decisions, verified facts, risks). This prompt is the
implementation brief; where they overlap, the plan document wins.

## Goal
Make LMBrain agent-agnostic by giving Codex CLI the same support it already gives
Claude Code: MCP registration, interactive sessions, and agent instructions. Four
pieces, all in scope for v1.

## Locked decisions (do NOT redesign)
- MCP for Codex → a PROJECT-SCOPED `.codex/config.toml` written at the workspace root
  with `[mcp_servers.lmbrain]`, PLUS ensuring the workspace is trusted in the user
  config. Do NOT use `codex mcp add` and do NOT register globally.
- Codex sessions → NATIVE Codex only (no `ollama launch codex` in v1).
- The same `lmbrain-mcp` binary serves Codex (stdio JSON-RPC) — no second server.

## Verified facts about the target machine (re-verify, don't hardcode)
- Codex is installed but NOT on PATH. The real binary is
  `%LOCALAPPDATA%\OpenAI\Codex\bin\<hash>\codex.exe`. Resolution is mandatory.
- The user's `~/.codex/config.toml` (honor `CODEX_HOME`) is rich and personal:
  `model`, plugins, existing `[mcp_servers.*]` (node_repl/loomle/unreal), and many
  `[projects.<path>]` trust entries (mixed casing). PRESERVE it entirely — only ever
  ADD a trust entry if missing.
- The current repo path is already trusted, so test the trust-merge against a fresh
  path too (not just an already-trusted one).

## What to build

### A. Codex MCP registration — new `src-tauri/src/commands/codex_registration.rs`
Mirror `src-tauri/src/commands/mcp_registration.rs` (study `build_mcp_config`,
`register_mcp_server`, `resolve_mcp_command`). Wire it into `open_workspace` in
`src-tauri/src/lib.rs`, right next to the existing `.mcp.json` registration
(best-effort, never block opening the workspace).
- Write/merge `<workspace>/.codex/config.toml`:
  ```toml
  [mcp_servers.lmbrain]
  command = "<resolve_mcp_command() result>"
  args = ["--root", "<absolute workspace path>"]
  ```
  Use the `toml_edit` crate for a format-preserving, idempotent merge (rewrite only when
  the content changes — like `register_mcp_server` does). Reuse `resolve_mcp_command()`.
- Ensure trust in `$CODEX_HOME/config.toml` (default `~/.codex/config.toml`): if absent,
  add `[projects.'<workspace>'] trust_level = "trusted"`. Never modify other entries;
  preserve the whole file with `toml_edit`; write atomically. Match Codex's exact
  path-key convention for the trust lookup (see risks).
- Add unit tests: idempotent merge, preservation of existing `[mcp_servers.*]` and other
  tables, trust entry added only when missing.

### B. AGENTS.md scaffolding
Generate/update a root `AGENTS.md` (Codex's native instruction file — note it lives
OUTSIDE `.lmbrain/`). Use a delimited managed block (`<!-- lmbrain:begin -->` …
`<!-- lmbrain:end -->`) so an existing user `AGENTS.md` is merged, not clobbered; rewrite
only the managed block; idempotent. Keep the block a concise POINTER to `.lmbrain/AGENT.md`
+ `CONTRACT.md`/`QUALITY.md` (stay well under `project_doc_max_bytes` = 32 KiB). Do this
where the kit is initialized / the workspace is opened, consistent with the other
registration writes.

### C. Native Codex sessions
- Backend `src-tauri/src/commands/sessions.rs` + `src-tauri/src/models/session.rs`: extend
  `SessionMode` with a `Codex` variant. In `build_command`, for `Codex` use a new
  `resolve_codex_command()`, `cwd` = workspace root, env inherited, NO `--model`.
- `resolve_codex_command()` order: `LMBRAIN_CODEX_BIN` env → a path configured in Settings
  → newest `%LOCALAPPDATA%\OpenAI\Codex\bin\*\codex.exe` → `codex`/`codex.exe` on PATH.
- Frontend `src/components/Sessions/SessionsView.tsx`: add a third `ModeButton` "Codex"
  (no model dropdown in Codex mode). Update `SessionMode` in `src/types/index.ts` and the
  default-label logic. Default label "Codex".

### D. Kit & docs
- `kit/.lmbrain/mcp/`: add a `codex-config.toml` example next to `lmbrain-mcp.json`; update
  `kit/.lmbrain/mcp/README.md` to document Codex registration (project `.codex/config.toml`
  + trust).
- Update `kit/.lmbrain/OPERATOR.md` and `kit/.lmbrain/README.md` for multi-agent support.
- Update `src/components/Agents/AgentsMCPView.tsx` to show the MCP is registered for both
  Claude (`.mcp.json`) and Codex (`.codex/config.toml`).

## Validate early — these are the real risks
1. **Project-scoped `mcp_servers` actually loaded.** There's history of Codex ignoring
   `config.toml` MCP servers (openai/codex#3441). After Phase A, launch real `codex` in the
   workspace and confirm the `lmbrain-*` tools appear. If not, diagnose before continuing.
2. **`lmbrain-mcp` and JSON-RPC notifications.** `lmbrain-mcp/src/main.rs` currently replies
   to every method; JSON-RPC notifications (`notifications/*`, no `id`) must NOT get a reply.
   Verify Codex's MCP handshake works; if it doesn't, fix `lmbrain-mcp` to skip replying to
   id-less/notification messages and to tolerate a newer `protocolVersion`. Keep the existing
   `lmbrain-mcp/tests/protocol.rs` green and add coverage.
3. **Trust path-key normalization** — match how Codex canonicalizes the project path (casing,
   separators) so you neither duplicate nor miss an existing trust entry.
4. **`codex.exe` resolution** robust to the changing `<hash>` dir (pick newest); honor the
   env/Settings overrides.

## New dependency
- Rust: `toml_edit` (in `src-tauri/Cargo.toml`).

## Build it in phases (each compiles and passes before the next)
1. `codex_registration.rs` headless: project `.codex/config.toml` write/merge + ensure-trust
   in the user config, with tests. Wire into `open_workspace`.
2. Validate MCP ↔ Codex (risks 1 and 2); fix `lmbrain-mcp` if needed.
3. Native Codex sessions (`resolve_codex_command` + `SessionMode::Codex` + UI).
4. `AGENTS.md` scaffolding (managed block).
5. Kit & docs + an ADR (extends ADR-001/ADR-006: writing config OUTSIDE the repo —
   `~/.codex/config.toml` — is a new surface) + CHANGELOG.

## Conventions & guardrails
- Match the surrounding code style: backend uses small service structs in `AppState` and thin
  `#[tauri::command]` wrappers in `lib.rs`; the frontend uses inline styles and a reducer-based
  `WorkspaceContext`. The poison-recovery `Mutex` lock pattern and idempotent "rewrite only on
  change" file writes are already established — follow them.
- PRESERVE the user's global `~/.codex/config.toml`: only ADD a missing trust entry, never edit
  or reorder anything else; write atomically (temp + rename); surface what was written.
- Do NOT alter the controlled-mutation engine (`lmbrain-core`) except the `lmbrain-mcp`
  notification fix if risk 2 requires it.
- Any new ADR must be created with `status: proposed` (accepting ADRs is operator-only).
- Do NOT commit or push unless explicitly asked; do NOT bump versions or edit VERSION files
  (the operator handles releases).
- Before handing back, run and report: `cargo test --workspace`,
  `cargo clippy --workspace --all-targets` (keep it warning-clean), `pnpm test`, `pnpm lint`,
  and `pnpm build`. Also report the result of the live Codex validation (risk 1): whether
  `lmbrain-*` tools actually appear in a real `codex` session opened on this workspace. State
  clearly which phases are done and any deviations from `CODEX-SUPPORT-PLAN.md`.
