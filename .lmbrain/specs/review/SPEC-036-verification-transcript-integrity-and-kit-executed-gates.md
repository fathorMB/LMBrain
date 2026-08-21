---
id: SPEC-036
title: "Verification transcript integrity and kit-executed gates"
status: review
kind: feature
priority: high
area: mcp-contract-and-quality
milestone: M-04
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: [ADR-012]
links: []
created: 2026-07-12
updated: 2026-07-16
tags: [verification, transcripts, mcp, spec-submit, security, 2.8.0]
activity:
  - date: 2026-07-12
    action: "approved by operator"
  - date: 2026-07-12
    action: "created"
  - date: 2026-07-16
    action: "transitioned ready -> working"
  - date: 2026-07-16
    action: "transitioned working -> review"
---
# Verification transcript integrity and kit-executed gates

## Objective

Prevent evidence-empty spec submissions mechanically and provide an operator-approved `spec_verify` workflow whose transcripts are produced by LMBrain from actual process results rather than written by the implementation agent.

## Context

The AstraNexus Project Lead reported repeated `spec_submit` calls where required checks were omitted or claimed without execution. Its `MCP-PROP-001` records thirteen occurrences across four specialist profiles, including two transcripts whose stated passing counts were impossible for the submitted code. Procedural wording and reviewer fast-fail rules reduced wasted review work but did not prevent submission or transcript synthesis.

Inspection of LMBrain 2.7.2 confirms the structural gap: `spec_submit` currently maps directly to the generic `working -> review` transition. The transition validates legal state and existing generic invariants but does not inspect `## Implementation evidence` for a verification transcript. `force: true` plus a mandatory reason already exists and records an override.

Presence validation fixes omission but cannot establish authenticity. Authenticity requires LMBrain to execute pre-approved gates, capture their real result, and bind that evidence to the workspace state that was verified.

## Scope

### Included

- Require a `### Verification transcript` subsection inside `## Implementation evidence` for `working -> review`.
- Reject a missing section, an empty section, or a section without a fenced block containing a non-empty command/result line.
- Return an actionable error showing the required transcript shape.
- Preserve `force: true` with mandatory reason and audit the bypass explicitly.
- Update the spec template, quality policy, Lead review procedure, contract, changelog, and migration guidance.
- Add an optional, versioned `.lmbrain/verification.toml` manifest containing named verification gates.
- Reference required named gates from a spec through typed frontmatter.
- Prefer direct `program` plus `args`; permit an explicitly declared shell/script only as a distinct high-risk gate type.
- Require machine-local operator approval bound to the canonical verification-manifest digest before executing any gate.
- Invalidate approval when executable gate content, shell, arguments, cwd, allowed environment, or other material policy changes.
- Add an MCP `spec_verify` verb that executes only required, approved manifest gates for one spec.
- Apply per-gate cwd confinement, timeout, output bounds, process-tree termination, environment policy, and serialization.
- Capture command identity, start/end time, exit code, timeout state, bounded stdout/stderr, manifest digest, LMBrain version, and workspace fingerprint.
- Atomically replace only the managed Verification transcript subsection; preserve every other body section and frontmatter field except the audit entry.
- Write an honest transcript for both passing and failing gates; a red result is evidence and is not rewritten as success.
- Mark transcripts as kit-generated and record a content hash plus workspace fingerprint in the artifact audit trail.
- Recompute the workspace fingerprint at `spec_submit` and flag/reject stale generated evidence unless force is explicitly authorized.
- Distinguish generated/fresh, generated/stale, hand-authored, missing, and force-bypassed evidence in review context and UI.
- Keep reviewer independent verification mandatory; generated execution evidence does not prove test adequacy or acceptance-criterion satisfaction.

### Excluded

- Claiming that a non-empty hand-authored transcript is truthful.
- Inferring required commands from prose, accepting commands supplied in an MCP call, or letting an agent mutate the approved manifest and execute it without renewed approval.
- Automatic execution during workspace open, `spec_submit`, review, file watching, or background refresh.
- Treating a failing gate as passing or preventing its honest transcript from being recorded.
- Capturing secrets, the unrestricted parent environment, unlimited logs, or interactive terminal input.
- Replacing CI attestation, code review, or the Project Lead's independent verification.

## Existing-project analysis

- `lmbrain-core/src/transitions.rs` centralizes transition invariants and already supports audited force overrides.
- `lmbrain-mcp/src/main.rs` maps `spec_submit` to the generic transition and exposes no verification executor.
- `Document` plus `atomic_write` provide the established safe artifact mutation path.
- Context packs already extract implementation evidence and can expose verification provenance without adding a parallel review model.
- Active project skills contain procedural commands but are runbooks, not executable authority; they must not silently become executable.
- [[SPEC-035-settings-and-project-harness-governance]] introduces the same necessary pattern for repository-controlled execution intent: preview plus machine-local digest-bound operator approval.

## Technical proposal

### Mechanical submit invariant

Add a spec-specific invariant for `working -> review`. Parse Markdown headings structurally and constrain the search to `## Implementation evidence`. The nested `### Verification transcript` body is valid only when it contains at least one fenced code block with a non-whitespace line. Do not accept the same heading elsewhere in the document. A force override must append an explicit audit action identifying the submit invariant and reason.

### Verification manifest

Define a schema-versioned `.lmbrain/verification.toml`. A gate has a stable ID, display name, execution form, optional confined cwd, timeout, output limit, expected exit code and optional result matcher. Direct execution is the default:

```toml
schema_version = 1

[[gates]]
id = "rust-workspace-tests"
program = "cargo"
args = ["test", "--workspace"]
cwd = "."
timeout_seconds = 900
expected_exit_code = 0
```

Shell gates require an explicit shell identifier and script field, are labeled high-risk in preview, and receive separate approval. Environment inheritance uses a documented minimal baseline; extra non-secret names/values must be declared. Secret values are never stored in the manifest or transcript.

Specs declare `verification_gates: [rust-workspace-tests]`. The manifest remains operator-authored policy; the Project Lead may propose changes through controlled mutation, but execution approval is a separate local operator action.

### Freshness and provenance

Before execution, compute a Git-aware workspace fingerprint from HEAD, index diff, working-tree diff, and content hashes of relevant untracked files, excluding `.git`, generated build/cache directories, and the managed transcript mutation itself under documented deterministic rules. Store the fingerprint in the generated transcript and audit entry. Recompute it at submission; a mismatch means stale evidence. Non-Git repositories may use a bounded filesystem fingerprint only if its semantics are specified and tested; otherwise generated attestation is unavailable rather than misleading.

`spec_verify` writes results even when a gate exits non-zero or times out. Infrastructure failure before a process result is also recorded honestly. The tool result separately reports whether all declared expectations matched, but evidence creation is not conditional on green status.

## Files and areas involved

- `lmbrain-core/src/transitions.rs`, Markdown section parsing, fingerprinting, verification models/executor
- `lmbrain-core/tests/` transition, parser, security, executor, fingerprint, and atomic-write tests
- `lmbrain-mcp/src/main.rs` tool schema, approval boundary, and `spec_verify`
- Tauri models/commands if Settings exposes approval, preview, and execution state
- Settings Project environment or a dedicated Verification section
- kit templates, `CONTRACT.md`, `QUALITY.md`, `AGENT.md`, `CHANGELOG.md`, `MIGRATIONS.md`
- context packs, product/architecture documentation, version alignment

## Acceptance criteria

- [ ] `spec_submit` rejects a missing, misplaced, empty, or fence-empty Verification transcript with an actionable expected-shape error.
- [ ] A non-empty fenced transcript inside Implementation evidence passes the mechanical presence gate without being labeled authenticated.
- [ ] `force: true` requires a reason and records the exact invariant bypass; force without reason fails.
- [ ] Existing `spec_done`, criteria, review, status-directory, and preservation invariants remain unchanged.
- [ ] The bundled spec template and migration add the transcript section and explain that predictions or summaries without actual output are not execution evidence.
- [ ] `.lmbrain/verification.toml` has a versioned, strict schema and rejects unknown fields, duplicate IDs, invalid cwd, unsafe limits, malformed matchers, and secret-like fields.
- [ ] Specs can reference only existing named gates; missing or duplicate references are diagnostic errors.
- [ ] No process runs until the operator approves the canonical manifest digest locally; any material gate change invalidates approval.
- [ ] `spec_verify` accepts a spec identity, never an ad-hoc command, and executes only its approved declared gates.
- [ ] Direct argv execution is the default; shell gates are explicit, separately identified as high risk, and use a declared supported shell.
- [ ] Execution confines cwd to the workspace, uses controlled environment inheritance, enforces time/output bounds, and terminates descendant processes reliably on Windows and POSIX.
- [ ] Passing, failing, timed-out, and launch-failed gates all produce honest bounded transcript entries with command, timestamps, exit/result state, and stdout/stderr.
- [ ] Only the managed transcript subsection and audit metadata are changed, atomically; all unrelated body and frontmatter content is byte-semantically preserved.
- [ ] Generated transcripts include LMBrain version, manifest digest, content hash, and workspace fingerprint.
- [ ] `spec_submit` rejects stale generated transcripts after tracked, staged, unstaged, or relevant untracked source changes unless explicitly forced.
- [ ] Hand-authored transcripts remain structurally admissible but are clearly marked unverified in audit/review context.
- [ ] Reviewer guidance continues to require independent reruns and assessment of whether declared gates are sufficient.
- [ ] Tests reproduce the AstraNexus failure classes: omission is blocked, synthetic hand text is untrusted, real red output is preserved, and workspace changes invalidate generated evidence.
- [ ] Full Rust/frontend gates, contract/migration tests, and Windows packaged verification pass before 2.8.0 release.

## Implementation plan

1. Approve [[ADR-012-kit-executed-verification-with-local-digest-approval]] and finalize transcript parsing/provenance semantics.
2. Implement the mechanical `spec_submit` invariant and force-audit tests first.
3. Update templates, kit contract, quality/review instructions, and migration fixtures.
4. Implement strict manifest parsing, controlled mutation/read APIs, preview, and local digest approval.
5. Implement direct and explicitly shelled runners with bounds, process-tree ownership, transcript rendering, and atomic section replacement.
6. Implement Git-aware freshness fingerprinting and stale-submit enforcement.
7. Surface provenance, approval, execution results, and stale state in context packs and Settings.
8. Run isolated security/behavior tests, full gates, and an operator-coordinated Windows sample-project demonstration.

## Required verification

- Unit fixtures for heading scope, fence parsing, Unicode/line-ending variants, misleading duplicate headings, missing/empty/valid content, and force audit.
- Manifest parser and canonical-digest property/fixture tests.
- Fake executable tests for success, red exit, timeout, large mixed output, spawn failure, descendant termination, cwd, and environment isolation.
- Atomicity and preservation tests with failure injection.
- Git fixtures covering clean, staged, unstaged, untracked, renamed, deleted, ignored, and transcript-only changes.
- MCP tests proving ad-hoc commands and unapproved/stale manifests cannot execute.
- Sample repository demonstrations of rejected omission, audited force bypass, genuine passing transcript, and genuine failing transcript.
- Full workspace tests/checks plus explicit packaged Windows testing coordinated with the operator; do not start or stop an existing LMBrain instance.

## Production quality and documentation

- Follow [[QUALITY]]; verification integrity is a security and trust boundary, not a convenience feature.
- Document exactly what kit-generated evidence proves and what it does not prove.
- Never imply shell sandboxing where the operating system provides none; operator approval is informed consent, not isolation.
- Report any platform limitation or process-tree leak as blocking rather than shipping a misleading executor.

## Risks and open decisions

- Verification executes repository code with user privileges. Digest approval reduces surprise but is not a sandbox.
- Workspace fingerprint exclusions must be narrow and deterministic or stale evidence can be replayed after meaningful changes.
- Tests that depend on external databases, credentials, containers, or GUI interaction need explicit preconditions and may remain honestly red/unrun.
- Shell syntax is platform-specific; direct argv gates should cover the common path and shell gates must declare portability expectations.
- Output redaction cannot reliably discover arbitrary secrets after emission; the runner must minimize inherited environment and warn operators that commands control their own output.

## Instructions for the assigned specialist

- Start only after this spec and ADR are explicitly approved; call `spec_start` first and `spec_submit` only with the required real transcript.
- Implement the mechanical gate as a reviewable first slice, but do not declare the synthesis problem solved until kit-generated provenance and freshness are complete.
- Do not execute real project verification commands or start/stop LMBrain without separate operator coordination.
- Record exact changed files, tests, security cases, migrations, and limitations.

## Implementation evidence

> Filled in by the specialist after completion.

### Changes made

- Added the strict verification manifest, canonical digest, machine-local approval, confined direct executor, bounded output, Windows/POSIX process-tree timeout cleanup, attributable transcript rendering, integrity hash, and workspace/manifest freshness checks.
- Added the mechanical `working -> review` transcript invariant with audited force override and MCP `verification_manifest_get`, `verification_manifest_approve`, and `spec_verify` verbs.
- Added success, red-result, missing-approval, tampering/freshness, scoped-heading, and submit-transition regression coverage.

### Files changed

- `lmbrain-core/src/verification.rs`, `lmbrain-core/src/transitions.rs`, `lmbrain-core/tests/transitions.rs`
- `lmbrain-mcp/src/main.rs`, verification exports/dependencies, bundled kit contract/templates/docs

### Verification performed

- `cargo test --workspace`: green (all non-ignored Rust tests; 3 existing manual harness tests ignored).
- `pnpm lint`, `pnpm test`, `pnpm build`: green (121 frontend tests).
- `git diff --check` and `node scripts/check-version.mjs`: green.
- `pnpm tauri build`: reached Tauri release compilation, then failed with Windows `Access is denied` while the already-running instance held the sidecar path. No process was stopped or restarted.

### Verification transcript

```text
$ cargo test -p lmbrain-core -p lmbrain-mcp
42 core tests passed; 12 transition tests passed; 13 MCP tests passed; 3 protocol tests passed; 0 failed.

$ cargo test --workspace
All non-ignored workspace tests passed; 3 pre-existing manual harness tests ignored; 0 failed.

$ pnpm lint && pnpm test && pnpm build
eslint: passed
vitest: 21 files, 121 tests passed
production frontend build: passed

$ pnpm tauri build
FAILED honestly: tauri-build returned Windows error 5 (Access is denied) while the existing LMBrain/sidecar instance remained in use. No app process was started or stopped.
```

### Deviations from the specification

- 2.9.0 intentionally ships only direct `program` plus `args` gates. Arbitrary shell/script gates were not added because they weaken the approval and portability boundary; a future schema can add them only with a separate risk model.
- Approval/execution are available through MCP and core; a dedicated Settings approval surface is deferred.
- Packaged runtime smoke remains pending because the operator required the existing instance to remain untouched.

### Handoff status

- [x] Ready for Project Lead review
