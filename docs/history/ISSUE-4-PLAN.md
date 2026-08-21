# LMBrain 2.9.2 — Implementation plan for issue #4

Issue: https://github.com/fathorMB/LMBrain/issues/4
Baseline: `main` @ `131cd34` (v2.9.1). Scope is strictly backward-compatible fixes; everything else stays in the 3.0.0 backlog (#3).

## Code-level findings (verified against the current tree)

| # | Workstream | Root cause located at |
|---|------------|----------------------|
| 1 | `lmbrain_get_artifact` path escape | `lmbrain-mcp/src/main.rs:473-483` — handler does `root.join(caller_path)` and reads, no `PathGuard`. A canonical guard already exists in `lmbrain-core/src/path.rs` (`PathGuard::resolve_existing`) but is unused here. Its errors also embed display paths, so it needs a sanitized error mapping for MCP. |
| 2 | `lmbrain_create` hardening | `lmbrain-core/src/transitions.rs:288-372` — (a) free-form `status` flows into `dir.join(&status)` → traversal + arbitrary directory creation; (b) `fs::create_dir_all` runs before any validation → residue on failure; (c) the `fields` loop runs **after** `document.set("id"/"status"/"created"/"updated")`, so callers can override all core fields; (d) no create-time invariants: a second `ready` handoff or a duplicate ID (via `fields` id override) is accepted; (e) unknown statuses silently create new status directories. |
| 3 | Verification snapshot consistency | `lmbrain-core/src/verification.rs:279-362` — `workspace_content_fingerprint` is computed only **after** all gates ran (line 328). Nothing detects workspace mutation *during* gate execution; the artifact lock in `write_verification_transcript` only protects the final write. |
| 4 | Review-context evidence extraction | `lmbrain-core/src/context.rs:1179-1206` (`extract_section`) — breaks at **any** line starting with `##`, so a `##` section is truncated at its first `###` subsection. With the spec template, the placeholder paragraph directly under `## Implementation evidence` is returned and every `### ...` evidence subsection is dropped (matches the AstraNexus SPEC-049 symptom). `extract_section_list` (1208-1235) has the same defect. A correct heading-level-aware helper already exists: `verification.rs:799` (`section_at_level`). |
| 5 | Version provenance | `src/components/Settings/SettingsView.tsx:55` — hardcoded `LMBrain 2.8.0 (development)`. Authoritative source chain already exists: `package.json` 2.9.1 → `tauri.conf.json` (`"version": "../package.json"`) → Tauri `getVersion()`. `lmbrain-mcp`/`lmbrain-core` are versioned independently (1.4.1) and must not be shown as the product version. |
| 6 | Terminal scroll/selection | `src/components/Sessions/SessionTerminal.tsx` + `src/lib/terminalWheel.ts` + `src/lib/terminalClipboard.ts` — wheel handler forces every non-codex host through synthetic PTY input (`tuiWheelInput`) regardless of the actual buffer; `scrollPage` assumes `"alternate"` for non-codex hosts; no runtime inspection of xterm mouse-tracking mode; no Select-text mode; Copy button only reads `getSelection()`; hint text doesn't mention Shift+drag; no "Copy visible output". |

## Ground rules for the whole patch

- Branch: `fix/issue-4-2.9.2` off `main`; one PR closing #4.
- Every phase lands with its tests; canonical gates stay green after each phase:
  `cargo test --workspace`, `cargo clippy`, `pnpm test`, `pnpm lint`, `pnpm build`, plus the MCP tool-discovery tests in `lmbrain-mcp`.
- Fail-closed, typed errors; never echo canonicalized host-absolute paths back to MCP callers (echoing the caller-supplied relative path is fine).
- No `.lmbrain` content migration; existing 2.9.1 workspaces must open unchanged.

## Phase 1 — Workspace path boundary for `lmbrain_get_artifact` (small)

1. Add `read_artifact(root, relative) -> Result<String, ArtifactReadError>` to `lmbrain-core` (new small module or `path.rs`):
   - Lexical pre-checks **before any FS access** (no existence oracle outside the root): reject absolute paths (incl. drive-letter and UNC forms), any `..` component, and empty paths. Accept both `/` and `\` separators for in-root paths.
   - Then `PathGuard::resolve_existing` for the canonicalize + prefix check (catches symlink/junction escapes).
   - Error enum with sanitized `Display`: `InvalidPath(relative)`, `NotFound(relative)`, `OutsideWorkspace(relative)` — never the resolved host path.
2. Route the `lmbrain_get_artifact` arm in `lmbrain-mcp/src/main.rs` through it.
3. Tests (core + an MCP dispatch test): valid nested read (`.lmbrain/specs/backlog/x.md`), `../escape`, `..\escape`, absolute `C:\...` and `/etc/...`, mixed separators, `.lmbrain/../..`, symlink escape (`#[cfg(unix)]` symlink; `#[cfg(windows)]` junction via `std::os::windows::fs::symlink_dir` fallback to junction, skip if unprivileged), and error-message assertions proving no host-absolute path leaks.

## Phase 2 — Harden `lmbrain_create` (medium)

All in `lmbrain-core/src/transitions.rs` (+ MCP dispatch tests):

1. **Status allowlist per kind** — new `allowed_creation_statuses(kind) -> &[&str]`. Baseline (initial states only, matching `default_status` and the transition graph):
   - spec: `backlog`; review: `pending`; adr/agent/agent-proposal/mcp-proposal/skill: `proposed`; mcp: `specified`; handoff: `ready`.
   - Anything else → typed `InvalidCreationStatus { kind, status }`. This inherently kills traversal via status, but still add an explicit defense: reject any status containing path separators, `..`, or non-`[a-z0-9-]` characters before dir derivation.
   - Note in CHANGELOG: values previously accepted by accident now fail closed (issue explicitly allows this).
2. **Reserved fields** — reject `fields` keys (case-insensitive, trimmed): `id`, `status`, `created`, `updated`, `title`, plus lifecycle-managed keys (`activity`-related and override-reason keys handled by `Document::append_*`). Typed `ReservedField(key)` error. Alternative of silently ignoring is rejected: fail closed per issue.
3. **Create-time invariants before any write**:
   - unique-ID check under the existing allocation lock (post-allocation, pre-write);
   - `invariants::single_ready_handoff` when creating a handoff (status `ready`).
4. **Atomicity / no residue** — reorder `create`: validate request (status, fields, invariants that don't need the lock) → acquire allocation lock → validate lock-dependent invariants → `fs::create_dir_all` → `atomic_write`. Ensure the allocation lock file is removed on every error path (wrap in a drop guard like `ArtifactMutationLock` instead of the manual `remove_file`).
5. **Schema** — keep `lmbrain_create` inputSchema shape unchanged (`status` stays a free string in the schema; validation is server-side) to preserve backward compatibility of the public contract.
6. Tests: full kind×status matrix (accept/reject), reserved-field rejection per key, traversal statuses (`../x`, `a/b`, `a\b`, absolute), regression: second ready handoff fails, regression: failed create leaves the tempdir byte-identical (snapshot dir listing before/after), duplicate-ID rejection, MCP dispatch tests for the new typed errors.

## Phase 3 — Snapshot-consistent verification evidence (medium)

In `lmbrain-core/src/verification.rs`:

1. In `execute_spec_verification`: compute `pre_fingerprint` **before the first gate**, `post_fingerprint` after the last gate.
2. If `pre != post`: do **not** publish a success transcript. Write invalidated evidence instead — transcript records both fingerprints and reason `workspace changed during gate execution`; `VerificationRunReport` gains `invalidated: bool` + `invalidation_reason: Option<String>` (additive, serde-defaulted) and `all_expectations_met` forced `false`.
3. Extend `render_transcript` metadata with both fingerprints (keep the existing single-fingerprint field name for the post fingerprint so `transcript_state` freshness checks — `GeneratedStale` — keep working unchanged; add the pre fingerprint as a new line).
4. Do not touch the 2.9.1 merge-safe write path (`write_verification_transcript`) beyond passing the extra metadata; all existing concurrency tests (`verification_merge_uses_the_latest_spec_body`, moved-spec, gate-contract-change) must stay green untouched.
5. Doc note (docs/kit.md or CONTRACT notes): the final artifact lock protects only the transcript write; full isolated-worktree/per-gate input scoping is deferred to 3.0.0 (#3).
6. Tests: gate whose command mutates a workspace file → publication refused, both fingerprints + reason recorded; unchanged workspace → behavior identical to 2.9.1; `transcript_state` still reports `GeneratedFresh/GeneratedStale` correctly with the new metadata.

## Phase 4 — Heading-level-aware review-context extraction (small/medium)

In `lmbrain-core/src/context.rs`:

1. Replace `extract_section`/`extract_section_list` scanning with level-aware logic (same contract as `verification.rs::section_at_level`): a match at level N includes everything until the next heading of level ≤ N. Handle `#`/`##`/`###` match levels as today (prefer the shallowest match).
2. Skip fenced code blocks when scanning for headings (a `## ...` inside a ``` fence must not terminate a section) — cheap state flag while iterating lines.
3. Placeholder behavior: with level-aware extraction the nested `### ...` evidence is now included, so the template placeholder no longer masks it. Keep the "empty section ⇒ keep searching" behavior only for truly empty sections.
4. Warnings instead of silence: when the review-context evidence section is missing, malformed (e.g. heading with no content and no subsections), or truncated by the existing output bound, push an explicit message into the context `warnings` vec (already rendered under `## Warnings`). Response shape stays additive.
5. Tests: `##` section containing `###`/`####` subsections; template placeholder followed by `###` evidence (the SPEC-049 reproduction); malformed/empty headings; heading-like lines inside code fences; bounded truncation emits a warning; existing digest/spec-context tests stay green.

## Phase 5 — Version provenance in Settings/About (small)

1. `SettingsView.tsx` AboutPanel: replace the hardcoded string with Tauri `getVersion()` from `@tauri-apps/api/app` (async → local state, fallback label `Unknown` on failure). Single authoritative chain: `package.json` → `tauri.conf.json` → build.
2. Keep the existing distinguishable rows: Application (product), Project kit, Bundled kit. Do not surface the `lmbrain-mcp` crate version as the product version (optionally add an explicit "MCP component" row using a value provided by the backend, not a hardcode — only if trivial).
3. Update `src/__tests__/SettingsView.test.tsx`: mock `@tauri-apps/api/app`, assert the displayed version follows the mock (proves "follows package/build metadata"), assert the hardcode is gone.

## Phase 6 — Embedded terminal: scroll policy + Select text mode (large)

This is the biggest phase; split into three sub-steps, each with tests.

### 6a. Runtime-informed scroll policy (`src/lib/terminalWheel.ts` + `SessionTerminal.tsx`)

1. Build a pure policy function `resolveScrollPolicy({ host, bufferType, mouseTrackingActive }) -> { wheel: "local" | "tui-input" | "delegate-to-app", page: ..., bottom: ..., supported: boolean }`:
   - Inspect `term.buffer.active.type` for **every** host (today only codex) and xterm's modes API (`term.modes.mouseTrackingMode`) at event time.
   - Normal buffer → always local xterm scrollback (all hosts).
   - Alternate buffer → per-host mapping for claude / codex / pi / opencode (preserve the working Pi PageUp/PageDown mapping and the OpenCode Ctrl+Alt sequences exactly as today); when the app enabled mouse tracking, let xterm deliver real mouse reports instead of synthesizing PageUp/Down.
   - Unknown host or unmapped combination → `supported: false`: do **not** swallow the wheel; show a visible hint ("This TUI controls scrolling; use its own keys") via the existing feedback strip.
2. Rework `attachCustomWheelEventHandler` and `scrollPage`/`scrollToBottom` in `SessionTerminal.tsx` to consult the policy instead of the current host special-cases.
3. Tests: extend `terminalWheel.test.ts` into a full host×buffer×mouse-tracking matrix; assert Pi mapping unchanged; assert unknown → visible degradation, never silent drop.

### 6b. Select text mode + mouse-tracking suspension

1. New toggle button **Select text** in the terminal toolbar. On enter:
   - snapshot the current mouse-tracking state from `term.modes` (tracking mode + SGR encoding);
   - locally reset tracking by writing DECRST to xterm only (`\x1b[?1000;1002;1003;1006l` as applicable) — nothing is sent to the PTY, the harness still believes tracking is on;
   - ordinary drag selection now works.
2. On exit: restore exactly the snapshotted modes via DECSET writes (again local-only).
3. Robustness: while the mode is active, watch `session-output` writes — if the harness re-enables tracking (its DECSET arrives through the normal write path), re-assert the local reset (idempotent, debounced); if the session exits or the buffer switches, leave selection mode cleanly and restore nothing stale.
4. Hint line update: "Mouse is captured by the TUI — Shift+drag to select, or use Select text mode" shown when tracking is active on Windows/Linux; remove any implication that xterm scrollback holds full chat history.
5. Keyboard: keep `terminalClipboard.ts` semantics — Ctrl+C without selection still reaches the PTY as SIGINT (already the case, add an explicit test); Ctrl+Shift+C and the Copy button stay consistent.
6. Tests: new `terminalSelection.ts` pure module (mode snapshot/restore sequence computation) with unit tests; component-level tests with a mocked xterm asserting enter/exit writes, harness re-enable handling, and safe teardown.

### 6c. Copy visible output + actionable copy failures

1. New **Copy visible** action: serialize the visible viewport via `term.buffer.active` (`viewportY..viewportY+rows`, `line.translateToString(true)`), joined with newlines; label/feedback states it copies the *current viewport, not the full conversation*.
2. Failure taxonomy in `copySelection`/`copyVisible` feedback: (a) no selection, (b) clipboard API unavailable, (c) empty visible output, (d) WebView clipboard write rejection — four distinct messages.
3. Tests: extend `terminalClipboard.test.ts` + component tests for each failure branch and the viewport serialization.

### 6d. Manual harness verification checklist

Automated integration against real TUIs isn't feasible in CI; add a checklist to `BUG-REPORT.md` workflow / `docs/sessions.md`: for each of Claude Code, Codex, Pi, OpenCode — wheel up/down, Page up/down, Bottom, Shift+drag, Select text mode, Copy, Copy visible, Ctrl+C SIGINT. Run it on Windows before release and record results in the PR.

## Phase 7 — Docs, versioning, release

1. Bump product version 2.9.1 → 2.9.2: `package.json`, `src-tauri/Cargo.toml` (+ `Cargo.lock`). Bump `lmbrain-core`/`lmbrain-mcp` 1.4.1 → 1.4.2. Follow `docs/release.md` for the kit `VERSION`/`CHANGELOG.md`/`MIGRATIONS.md` procedure (verify exact steps there during implementation).
2. Changelog entry: security/correctness summary (path boundary, create hardening fail-closed behavior change, verification invalidation semantics, review-context fix, version provenance, terminal fixes) — no exploit walkthrough.
3. Document the two behavior changes explicitly: invalid create requests now fail closed; verification runs where the workspace changes mid-run are invalidated.
4. PR referencing #4 with the manual checklist results and gate evidence; link release + PR from the issue.

## Suggested execution order and sizing

| Order | Phase | Size | Rationale |
|-------|-------|------|-----------|
| 1 | Phase 1 (get_artifact guard) | S | Highest severity security fix, smallest diff |
| 2 | Phase 2 (create hardening) | M | Second security fix, same files/test harness as 1 |
| 3 | Phase 4 (context extraction) | S/M | Isolated, pure-function fix; unblocks dogfooding fastest |
| 4 | Phase 3 (verification fingerprints) | M | Builds on stable verification tests |
| 5 | Phase 5 (version) | XS | Trivial |
| 6 | Phase 6 (terminal) | L | Largest and UI-risky; land last with manual checklist |
| 7 | Phase 7 (release) | S | Final |

Phases 1–4 are pure Rust (lmbrain-core + lmbrain-mcp, no app restart needed — satisfies the "no app process restart" requirement); 5–6 are frontend-only; nothing requires kit content migration.

## Open decisions taken (revisit only if the owner disagrees)

- Creation-status allowlist = initial statuses only (no fast-path creation into `ready`/`accepted` etc.). Rationale: matches the transition graph; anything else should go through governed transitions.
- Reserved-field violations fail closed (error) rather than being silently ignored.
- The transcript's existing fingerprint field keeps the post-run fingerprint so 2.9.1 freshness checks remain valid; the pre-run fingerprint is an additive metadata line.
- Select-text mode implemented via local-only DECRST/DECSET writes to xterm (no PTY traffic), with re-assertion if the harness re-enables tracking.
