# LMBrain 3.0.2 — Implementation plan for issue #8

Issue: https://github.com/fathorMB/LMBrain/issues/8
Baseline: `main` @ `6dce627` (v3.0.1). Scope: the two items in the issue body plus the
`spec_verify` invalidation finding reported in the issue comments and explicitly targeted
at 3.0.2. The second comment (governed browser-tool capability) is a preliminary analysis
of a larger feature with its own acceptance criteria and safety model; it is **not** part
of this patch and should be split into its own issue.

## Code-level findings (verified against the current tree)

| # | Workstream | Findings located at |
|---|------------|---------------------|
| 1 | Antigravity MCP support | No Antigravity code exists anywhere in the tree. Registration adapters live in `src-tauri/src/commands/` (`mcp_registration.rs`, `codex_registration.rs`, `pi_registration.rs`, `opencode_registration.rs`) and are invoked best-effort from `open_workspace` (`src-tauri/src/lib.rs:65-71`) and `initialize_workspace_kit` (`lib.rs:99-105`). External research (see "Antigravity conventions" below): Antigravity has **no project-level MCP configuration** — only a user-global `mcp_config.json` (`~/.gemini/antigravity/mcp_config.json` for the 1.x IDE; `~/.gemini/config/mcp_config.json` for the 2.0 unified CLI+IDE layout), JSON `mcpServers` schema with `command`/`args`/`env` for stdio servers. Antigravity reads root `AGENTS.md` (already scaffolded by `codex_registration::scaffold_agents_md`). |
| 2 | Actions runs panel + details modal | Backend `src-tauri/src/commands/github_integration.rs:150-202` already fetches `/actions/runs?per_page=10` **without a status filter**, but the payload model (`GitHubWorkflowRun`, lines 19-29) carries only `id/name/head_branch/head_sha/status/conclusion/html_url/created_at` — not enough for a details modal. Frontend `src/components/Repository/RepositoryView.tsx:132-144` (`getRunStatusStyles`) maps `skipped`, `neutral`, `action_required`, `stale`, `startup_failure` to the generic indigo "pending" style, so non-failure completed runs are visually indistinct (the likely source of the "failures only" perception, compounded by `per_page=10`). Each run row is an `<a>` straight to GitHub (lines 552-613) — no modal. The established modal pattern to follow is `src/components/Repository/GitDiffModal.tsx` (overlay + `role="dialog"` + Escape + focus restore). |
| 3 | `spec_verify` invalidation false positive | `lmbrain-core/src/verification.rs:512-564` — `workspace_content_fingerprint` hashes every file under the root except hard-coded `.git`/`target`/`node_modules` and `.lmbrain/specs|reviews`. A gate that emits build artifacts anywhere else (e.g. `apps/client/dist/**`) always changes the post-gate fingerprint (`verification.rs:333-343`), so `invalidated` is structurally true for artifact-producing gate sets. `VerificationGate` (`verification.rs:42-63`) has no exclusion field. Freshness recheck at `verification.rs:457-486` recomputes the current fingerprint with the same fixed exclusions. |

## Ground rules for the whole patch

- Branch: `fix/issue-8-3.0.2` off `main`; one PR closing #8.
- Canonical gates stay green after each phase: `cargo test --workspace`, `cargo clippy`,
  `pnpm test`, `pnpm lint`, `pnpm build`.
- All host-config writes remain best-effort, idempotent, and merge-preserving — never
  block workspace open, never touch unrelated keys/servers.
- No `.lmbrain` schema version bump; all manifest changes must be additive and must not
  change the canonical digest of existing manifests (details in Phase 3).
- Version bump to 3.0.2 (`package.json`, kit `VERSION`/`CHANGELOG`/`MIGRATIONS` note) in
  the final phase, per `docs/release.md`.

## Phase 1 — Antigravity MCP registration (medium)

### Antigravity conventions (researched, to re-verify against the installed version during implementation)

- MCP config is **user-global only**: `~/.gemini/antigravity/mcp_config.json`
  (IDE 1.x, Windows: `%USERPROFILE%\.gemini\antigravity\mcp_config.json`) and/or
  `~/.gemini/config/mcp_config.json` (2.0 unified CLI+IDE "single source of truth").
- Schema: top-level `mcpServers` object; stdio servers use `command`, `args`, `env`,
  optional `disabled`; remote servers use `serverUrl`. Same shape LMBrain already writes
  for Claude Code's `.mcp.json`.
- Project instructions: Antigravity reads root `AGENTS.md` (cross-tool standard), so the
  existing scaffolded pointer block already gives Antigravity sessions the
  `.lmbrain/AGENT.md` / `CONTRACT.md` / `QUALITY.md` orientation path.

### Design decisions

1. **User-global entry, last-opened-workspace wins.** Because there is no project-scoped
   config, LMBrain maintains a single `mcpServers.lmbrain` entry whose `args` carry
   `--root <workspace>`; opening a different workspace rewrites the entry. This is the
   documented limitation (one Antigravity-registered workspace at a time). Rejected
   alternative: per-workspace entry names (`lmbrain-<hash>`) — pollutes the user's global
   config with stale entries and multiplies tool names in the Antigravity UI.
2. **Install-detection before writing.** Unlike the project-local adapters, this write
   lands in the user's home. Only write when Antigravity presence is detectable:
   update whichever of the two config locations exists (**both** if both exist, so IDE 1.x
   and the 2.0 CLI stay consistent); if neither file exists but a parent directory
   (`~/.gemini/antigravity/` or `~/.gemini/config/`) exists, create the file there; if no
   trace of Antigravity exists, skip silently. Never create `~/.gemini` itself.
3. **Test seam:** `LMBRAIN_ANTIGRAVITY_HOME` env override for the base directory,
   mirroring the `CODEX_HOME` pattern in `codex_registration.rs:158-167`.

### Steps

1. New module `src-tauri/src/commands/antigravity_registration.rs`:
   - `build_antigravity_config(existing, command, root) -> Result<String, AppError>` —
     JSON merge identical in spirit to `mcp_registration::build_mcp_config`: parse or
     start empty, replace a non-object root, upsert only `mcpServers.lmbrain`
     (`{"command": ..., "args": ["--root", root]}`), preserve every other key and server
     (including `serverUrl`-based entries and `disabled` flags on other servers).
   - `antigravity_config_targets() -> Vec<PathBuf>` — resolves the candidate paths with
     the detection rules above (env override → `USERPROFILE`/`HOME`).
   - `register_antigravity_mcp_server(root, command) -> Result<Vec<PathBuf>, AppError>` —
     for each target: read existing, build, `write_if_changed` (reuse the temp-file +
     rename atomic pattern already used by `mcp_registration.rs:46-50`).
2. Wire into both call sites in `src-tauri/src/lib.rs` (`open_workspace` and
   `initialize_workspace_kit`) as `let _ = ...` alongside the other adapters, and into the
   re-registration path around `lib.rs:574` if it enumerates hosts.
3. Add the same registration to `mod.rs`.
4. **Explicitly out of scope** (matches the issue's out-of-scope + acceptance criteria):
   no Antigravity session launching, no Settings → Harnesses probe/update entry (the IDE
   self-updates), no `HarnessHost::Antigravity` variant in `.lmbrain/HARNESSES.json`
   (the governed-environment materializer is a separate opt-in feature; adding the host
   there is follow-up work — note it in the docs as a limitation).
5. Docs — `docs/agent-hosts.md`, new "Antigravity" section: config locations for both
   layouts, the user-global/last-opened-workspace limitation, install-detection behavior,
   AGENTS.md discovery, statement that sessions are launched only from the Antigravity
   IDE, and that no project-local generated file is added (nothing new to gitignore).
6. Tests (unit, in-module, using `LMBRAIN_ANTIGRAVITY_HOME` + tempdirs):
   - creates config when the 1.x dir exists but the file doesn't;
   - updates both files when both layouts exist;
   - skips entirely when no Antigravity trace exists (no `~/.gemini` creation);
   - preserves unrelated servers/keys (`serverUrl` server, `disabled`, custom top-level
     keys) byte-for-byte apart from the `lmbrain` entry;
   - idempotent (second run writes nothing);
   - replaces a non-object root;
   - re-registration on a second workspace rewrites `--root` (last-opened wins).

## Phase 2 — GitHub Actions runs: complete listing + details modal (medium)

### Backend (`src-tauri/src/commands/github_integration.rs`)

1. Extend `GitHubWorkflowRun` (additive) with the metadata the modal needs, all already
   present in the `/actions/runs` list payload — no extra API call:
   `display_title`, `event`, `run_number`, `run_attempt`, `updated_at`,
   `run_started_at: Option<String>`, `actor: Option<String>` (from `actor.login`),
   `workflow_id`. Keep every field tolerant of absence (`unwrap_or` / `Option`), matching
   the existing parsing style.
2. Raise `per_page` from 10 to 30 (parity with the PR list) and keep the request
   explicitly unfiltered by status/conclusion.
3. Mirror the new fields in `src/types/index.ts` (`GitHubWorkflowRun`).

### Frontend

1. New `src/components/Repository/WorkflowRunModal.tsx` modeled on `GitDiffModal.tsx`
   (same overlay/dialog/Escape/focus-restore skeleton and CSS classes from
   `RepositoryView.css`, adding a small `repository-run-modal` variant if needed).
   Content, rendered from the already-fetched run object:
   - header: workflow name + `#run_number` (+ `attempt N` when > 1), status/conclusion
     badge with icon;
   - detail grid: branch (mono), triggering event, commit `head_sha` (short, mono),
     actor, created/started/updated timestamps (locale string);
   - footer: "Open on GitHub" anchor (`target="_blank" rel="noopener noreferrer"`) to
     `html_url`;
   - every missing/empty field renders as "—" — partial metadata must never break the
     dashboard (issue acceptance criterion).
2. `RepositoryView.tsx`:
   - replace the per-run `<a>` with a `<button type="button">` (keeping the row visuals)
     that sets `selectedRun`, exactly like the Changed Files rows drive `selectedFile`;
     render `<WorkflowRunModal run={selectedRun} onClose={...}>` next to `GitDiffModal`;
   - rewrite `getRunStatusStyles` into an exhaustive, shared map covering:
     - `completed` + `success` (green/check_circle), `failure`/`timed_out`/
       `startup_failure` (red/error), `cancelled` (grey/cancel), `skipped`
       (grey/skip_next), `neutral` (grey/remove), `stale` (grey/history),
       `action_required` (amber/warning);
     - `in_progress` (indigo/progress_activity, spinning), `queued`/`waiting`/
       `pending`/`requested` (indigo/pending);
     - unknown values fall back to the neutral style with the raw label — never hidden;
   - failures stay instantly identifiable (red is used only for failing conclusions);
   - accessibility: each row gets an `aria-label` naming the workflow, status/conclusion,
     and branch (state is conveyed by text + icon, not color alone).
3. Move the status map into a small pure helper (e.g. `src/lib/workflowRunStatus.ts`) so
   it is unit-testable and shared by row and modal.
4. Tests:
   - `workflowRunStatus.test.ts` — exhaustive status/conclusion → label/style mapping,
     unknown-value fallback;
   - extend `src/__tests__/RepositoryView.test.tsx` — runs with each
     status/conclusion render distinct badge text; clicking a run opens the modal;
     Escape/backdrop closes it; GitHub link present with correct href; run with missing
     `run_started_at`/`actor` renders "—" without crashing; empty/error/no-dashboard
     states unchanged;
   - new `src/__tests__/WorkflowRunModal.test.tsx` following `GitDiffModal.test.tsx`.
5. Docs: short update to `docs/repository.md` (all outcomes shown; click-through modal).

## Phase 3 — Per-gate fingerprint exclusions for `spec_verify` (medium, fail-closed)

Adopt option 1 from the issue comment (declared, digest-bound, auditable), plus the
option-3 messaging improvement. Option 2 (globally excluding gitignored paths) is
rejected: a gate could hide foreign mutations in ignored paths.

1. `lmbrain-core/src/verification.rs` — extend `VerificationGate` with
   `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub fingerprint_exclude: Vec<String>`.
   `skip_serializing_if` is **mandatory**: the canonical manifest digest is
   `serde_json::to_vec(manifest)` (`verification.rs:248-258`), so the field must not
   change the digest of existing manifests — existing operator approvals stay valid.
   Adding exclusions to a gate *does* change the digest and forces re-approval — that is
   the intended audit property.
2. Validation (in `validate_verification_manifest`): each entry must be non-empty, not
   `.`, workspace-relative (reject absolute paths and drive letters), no `..` component
   (reuse `unsafe_relative`), no NUL, ≤ 256 bytes; ≤ 32 entries per gate; forbid entries
   equal to or inside `.lmbrain` (managed state must always stay fingerprinted).
3. Fingerprinting: change `workspace_content_fingerprint(root)` to an internal
   `workspace_content_fingerprint_with(root, exclusions: &BTreeSet<PathBuf>)`; keep the
   existing zero-exclusion public signature as a wrapper for untouched callers.
   `collect_files` skips any path whose workspace-relative form starts with an excluded
   prefix (normalize separators the same way the digest loop does at line 523).
4. `execute_spec_verification`: compute the union of `fingerprint_exclude` across the
   **executed gate set only**, and apply the same set to both the pre- and post-gate
   snapshots (`verification.rs:333` and `:341`). A mutation inside an excluded path no
   longer invalidates; any change outside the union still does.
5. Invalidation message (option 3, cheap version): when `pre != post`, keep the existing
   reason but append a hint — "if a gate intentionally writes build artifacts, declare
   the output directory in that gate's `fingerprint_exclude`". No per-file attribution
   (would require per-file digests; out of scope for a patch release).
6. Freshness recheck (`transcript_state`, `verification.rs:457-486`): recompute the
   current fingerprint with the exclusion union derived from the **current approved
   manifest** for the spec's `verification_gates`. The existing `manifest-digest` and
   `gate-contract-digest` bindings already mark the transcript stale if the manifest or
   gate set changed, so deriving exclusions from the current manifest is sound. Thread
   the manifest (already loaded by callers of `transcript_state_for_document`) down to
   the fingerprint recompute.
7. Docs: `docs/kit.md` verification section + kit operator docs
   (`kit/.lmbrain/OPERATOR.md`) — field syntax, the security model (exclusions are part
   of the approved digest; agents cannot self-exclude without operator re-approval), and
   the AstraNexus-style example (`fingerprint_exclude = ["apps/client/dist"]`).
   `kit/.lmbrain/CHANGELOG.md` + `MIGRATIONS.md`: additive, schema stays v1, existing
   manifests and approvals unaffected.
8. Tests (`lmbrain-core`):
   - reproduction: gate whose command writes into `some/dist` → without exclusion:
     invalidated (current behavior, must keep failing); with
     `fingerprint_exclude = ["some/dist"]`: all green, `invalidated == false`,
     `workspace_fingerprint_before == workspace_fingerprint`, transcript publishable and
     `GeneratedFresh`;
   - gate writes outside the excluded prefix → still invalidated, reason includes the
     new hint;
   - canonical digest of a manifest **without** the field is byte-identical to before
     the change (regression pin against approval invalidation);
   - adding an exclusion changes the digest → `ApprovalRequired` until re-approved;
   - validation rejections: absolute path, `..`, `.`, `.lmbrain`, `.lmbrain/specs`,
     empty string, > 32 entries;
   - freshness: transcript generated with exclusions stays `GeneratedFresh` after a
     post-run rebuild that only touches the excluded dir; becomes `GeneratedStale` when
     a non-excluded file changes;
   - existing merge/concurrency tests stay green untouched.

## Phase 4 — Release chores (small)

1. Bump `package.json` to 3.0.2 (Tauri reads it via `tauri.conf.json`).
2. Kit `VERSION`, `CHANGELOG.md`, `MIGRATIONS.md` entries (no migration steps required —
   all changes additive).
3. `docs/README.md` index if new doc sections were added.
4. Full gate run: `cargo test --workspace`, `cargo clippy`, `pnpm test`, `pnpm lint`,
   `pnpm build`.
5. PR closing #8; note in the PR body that the browser-MCP capability comment was split
   out (link the follow-up issue once opened).

## Open questions / risks

- **Antigravity config-path drift**: the 1.x vs 2.0 locations come from external docs
  (Google Codelabs, github-mcp-server install guide, community guides) and Antigravity
  ships frequent updates; before implementation, confirm the paths against the actual
  installed Antigravity on the target machine (the "Manage MCP Servers → View raw
  config" UI shows the real file). The dual-target write keeps us correct across both.
- **Single-workspace limitation**: acceptable for 3.0.2 and documented; if operators run
  multiple LMBrain workspaces concurrently under Antigravity, revisit with per-workspace
  entries or a root-inferring `lmbrain-mcp` mode as follow-up.
- **`deny_unknown_fields` on `VerificationManifest`**: a 3.0.2 kit workspace using
  `fingerprint_exclude` will fail manifest parsing on an older LMBrain (< 3.0.2). This is
  fail-closed and acceptable for a patch, but call it out in `MIGRATIONS.md`.
