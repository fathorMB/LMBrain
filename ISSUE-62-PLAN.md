# Issue #62 — Design governed branching strategies for the LMBrain kit

Milestone 4.0.0. Technical discovery and implementation plan for Phase 1 (Lead-mediated strategy design and management).

---

## 1. Problem statement

LMBrain kit currently lacks an explicit, machine-readable declaration of a project's Git branching strategy. While LMBrain observes local and remote Git branch state for visual display (commit graph, current branch, uncommitted files, GitHub branch PRs), it has no model for the project's *intended* branch topology, naming conventions, promotion rules, or Lead push authority.

Without a declared strategy:
1. The Project Lead cannot factor branching rules into spec planning, assignment creation, or release coordination.
2. Agents have no governed guidance on expected branch naming (`feature/SPEC-xxx`, `fix/xxx`) or commit/push permissions.
3. The kit cannot report drift when observed repository state diverges from project intent.

### 1.1 What the audit measured

| Observation | Evidence |
| --- | --- |
| Zero branching strategy artifacts or schemas in the kit | `grep -i "branching" E:\Git\LMBrain\kit\.lmbrain\` returns zero schema definitions or governed config files |
| Existing workspace branches are diverse and ungoverned | `git branch -a` shows 14 local/remote branches with prefixes `codex/`, `feature/`, `fix/`, `main` |
| Observed repository state is decoupled from policy | `src-tauri/src/commands/git_details.rs` (local) and `github_integration.rs` (GitHub API) inspect real branches/commits but have no declared policy to validate against |
| `lmbrain-core` has no Git dependency or branch model | `lmbrain-core` is pure Rust filesystem-backed logic; it contains zero Git execution or branch topology primitives |
| LMBrain invariant: no direct Git execution or agent spawning by kit | `CONTRACT.md` and issue #62 mandate that a declared strategy must remain a governed policy and Lead/agent guidance, never automated `git` execution |

---

## 2. Operator workflows

1. **"What is the intended branching topology and commit policy for this project?"** — Read by Project Lead during workspace orientation and context pack creation.
2. **"Does this spec or assignment comply with branch naming and authority rules?"** — Read by agents during spec context generation (`lmbrain_spec_context`).
3. **"Is the repository operating in alignment with our declared strategy?"** — Read by `build_diagnostics` to report drift (e.g. branch naming mismatch, unauthorized direct pushes) without mutating Git or project intent.
4. **"How does a new or existing repository initialize its branching strategy?"** — Governed initialization via `branching_strategy_set` (explicit or scaffolded default), ensuring unconfigured repositories remain clean (`absent` state) rather than receiving silent mutations.

---

## 3. Recorded decisions

Answered by the operator on 2026-08-01.

### 3.1 Canonical Artifact Format and Location
- **Decision**: File `.lmbrain/BRANCHING.json` with `schema_version: 1`.
- **Rationale**: Formato JSON con validazione di schema strict, analogo a `.lmbrain/HARNESSES.json`. Garantisce determinismo, parsing diretto in Rust (`lmbrain-core`) e gestione atomica tramite verbi MCP (`branching_strategy_get` / `branching_strategy_set`).

### 3.2 Lead vs. Operator Authority
- **Decision**: Le modifiche alla Branching Strategy richiedono l'approvazione diretta o l'intervento dell'operatore (`actor: operator`).
- **Rationale**: Il Project Lead può ispezionare o proporre, ma la scrittura/mutazione della politica di branching richiede `actor: operator` per proteggere l'integrità del repository e la governance.

### 3.3 Handling Unconfigured & Legacy Repositories
- **Decision**: Trattare lo stato come `unconfigured` (`absent`), segnalandolo nei context pack e nelle diagnostiche (Severity Info) senza creare o modificare alcun file in silenzio. Il default scaffolded (`main-only` con push riservato al Lead) si applica solo all'inizializzazione esplicita (`branching_strategy_init` / kit scaffolding).
- **Rationale**: Garantisce la retrocompatibilità e rispetta l'autonomia dei repository esistenti.

### 3.4 Repository Observation & Drift Diagnostics
- **Decision**: Emettere diagnostiche non distruttive (`branching-strategy-absent`, `invalid-branch-name`, `unauthorized-push-target`) tramite `build_diagnostics` senza eseguire comandi Git o mutare file in silenzio.
- **Rationale**: Avvisa l'operatore e il Lead in presenza di divergenze senza bloccare il flusso o tentare automazioni pericolose su Git.

---

## 4. Technical Architecture

### 4.1 Canonical Schema (`.lmbrain/BRANCHING.json`)

```json
{
  "schema_version": 1,
  "topology": "main-only",
  "default_branch": "main",
  "protected_branches": ["main"],
  "development_branch": null,
  "branch_naming": {
    "allowed_prefixes": ["feature/", "fix/", "codex/", "release/", "hotfix/"],
    "spec_branch_pattern": "{prefix}{spec_id_lowercase}-{slug}",
    "require_prefix": true
  },
  "authority": {
    "lead_only_push_branches": ["main"],
    "allow_specialist_push": false,
    "require_pr_for_merge": false
  },
  "commit_triggers": {
    "commit_on_spec_completion": true,
    "commit_on_doc_change": true
  }
}
```

### 4.2 Kit Scaffolding Default Strategy

When scaffolded, the project receives:
- `topology`: `"main-only"`
- Work performed only on `main`
- Project Lead is sole actor authorized to commit and push
- Lead commits on spec completion and doc changes
- Implementation specialists do not commit or push directly

### 4.3 Diagnostics (`diagnostics.rs`)

- `branching-strategy-absent` (Info): Repository has no declared branching strategy.
- `invalid-branch-name` (Warning): Current branch does not match declared allowed prefixes.
- `unprotected-branch-divergence` (Warning): Observed default branch differs from declared `default_branch`.
- `unauthorized-push-target` (Warning): Action attempts push authorization on Lead-only branch.

### 4.4 Context Packs (`context.rs`)

- `ProjectDigest`: includes `branching_strategy` summary (topology, default branch, Lead push policy).
- `SpecContext`: includes applicable branch naming conventions and push permissions for assigning agents.

---

## 5. Technical Impact

- `kit/.lmbrain/CONTRACT.md`: document `BRANCHING.json` schema, rules, and default scaffolding policy.
- `kit/.lmbrain/MIGRATIONS.md`: 4.0.0 migration entry for branching strategy capability.
- `lmbrain-core`:
  - `branching_strategy.rs` (new module): schema structs, load, validate, set, and default generator.
  - `diagnostics.rs`: drift diagnostics for branching strategy.
  - `context.rs`: surface strategy in `ProjectDigest` and `SpecContext`.
- `lmbrain-mcp`:
  - MCP verbs: `branching_strategy_get`, `branching_strategy_set`.
- `src-tauri` & `frontend`:
  - Expose branching strategy status in RepositoryView / Project overview.

---

## 6. Implementation Breakdown

| # | Item | Size | Acceptance Criteria |
|---|---|---|---|
| 1 | `branching_strategy.rs` in `lmbrain-core` | M | Canonical struct, JSON parser, schema validation, atomic file writer over mutation lock |
| 2 | Diagnostics for branching strategy | S | `branching-strategy-absent`, `invalid-branch-name`, `unprotected-branch-divergence` in `build_diagnostics` |
| 3 | MCP Verbs (`branching_strategy_get`, `branching_strategy_set`) | M | Registered in `lmbrain-mcp`, validated schema, audited mutations |
| 4 | Context Pack Integration | S | `ProjectDigest` and `SpecContext` carry branching policy summary |
| 5 | Documentation and Kit updates | S | `CONTRACT.md`, `MIGRATIONS.md`, kit scaffolding template, `docs/kit.md` |

---

## 7. Status

| # | Item | State |
|---|---|---|
| 1 | Core model and parser | done — `branching_strategy.rs` module, schema validation, atomic file writer over mutation lock |
| 2 | Drift diagnostics | done — `branching-strategy-absent`, `branching-strategy-invalid` in `diagnostics.rs` |
| 3 | MCP verbs | done — `branching_strategy_get`, `branching_strategy_set` in `lmbrain-mcp` with `actor: operator` authority check |
| 4 | Context packs integration | done — `BranchingStrategyDigest` in `ProjectDigest` and `SpecContext` |
| 5 | Docs & Kit scaffolding | done — `CONTRACT.md`, `MIGRATIONS.md`, `docs/kit.md` |
