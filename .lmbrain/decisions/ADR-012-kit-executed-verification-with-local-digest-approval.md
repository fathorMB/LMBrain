---
id: ADR-012
title: "Kit-executed verification with local digest approval"
status: accepted
decision_date: 2026-07-12
decider: operator
supersedes: []
superseded_by: []
links: [SPEC-036]
tags: [architecture, verification, execution, security, mcp]
created: 2026-07-12
updated: 2026-07-12
activity:
  - date: 2026-07-12
    action: "accepted by operator"
  - date: 2026-07-12
    action: "created"
---

# Kit-executed verification with local digest approval

## Context

Procedural requirements cannot reliably distinguish real command output from omitted or synthesized verification claims. A structural transcript gate prevents empty submissions but still accepts fabricated fenced text. LMBrain can establish execution provenance only by running predeclared gates itself and recording the observed process result.

Verification commands execute repository code with the user's privileges. A committed manifest alone is not consent: an untrusted branch or pull could change it. The execution boundary therefore needs the same repository-intent/local-approval split proposed for project harness governance.

## Proposed decision

- Make a scoped Verification transcript mandatory for `spec_submit`, with an audited `force + reason` escape hatch.
- Add a versioned `.lmbrain/verification.toml` manifest of named gates and typed spec references.
- Add an explicit `spec_verify` operation; never execute verification automatically during open, refresh, submit, or review.
- Require machine-local operator approval bound to the canonical manifest digest before any gate execution.
- Prefer direct executable/argv gates. Treat declared shell scripts as a separate high-risk form requiring explicit disclosure and approval.
- Capture real success, failure, timeout, and launch errors into a bounded kit-generated transcript rather than writing only green results.
- Bind generated evidence to a deterministic workspace fingerprint and reject it as stale after meaningful workspace changes.
- Limit automatic writes to the managed transcript subsection plus audit metadata, using atomic preservation-aware mutation.
- Keep hand-authored transcripts possible for projects without an executable manifest, but mark them unauthenticated and preserve independent Lead verification.

## Alternatives considered

### Presence gate only

Necessary but insufficient. It fixes omission while leaving synthesized output indistinguishable from real output.

### Trust any committed verification manifest automatically

Rejected because checking out a repository revision could authorize arbitrary code execution without informed local consent.

### Infer commands from specifications or skills

Rejected because prose and runbooks are not an executable security policy, and an MCP caller must never supply an ad-hoc command to the runner.

### Require only CI results

Rejected as the sole mechanism because LMBrain is local-first and many project gates require local services or pre-review checks. CI attestation may be added later as another provenance source.

## Consequences

- LMBrain gains an explicit code-execution subsystem requiring strict lifecycle, process-tree, output, environment, approval, and audit controls.
- Projects can obtain trustworthy evidence that a command ran, but not proof that the command was sufficient or the code is correct.
- Generated evidence becomes stale when the verified workspace changes and must be regenerated or explicitly overridden.
- Existing projects may continue with hand-authored evidence, now clearly identified as unauthenticated.
- The kit template, contract, quality policy, review context, migration guidance, and Settings UI must evolve together.

## Review conditions

Revisit when LMBrain has a real sandbox, supports remote CI attestations, needs secret-backed verification environments, or the workspace fingerprint cannot cover a supported version-control workflow without false freshness claims.
