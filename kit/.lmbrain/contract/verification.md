# Verification Gates Capability Module

This module defines the optional executable and attestation verification capability for LMBrain workspaces.

## Scope & Application

When `.lmbrain/verification.toml` is present or a spec declares `verification_gates`, this module governs gate declarations, execution, transcripts, and attestations.

## Verification Manifest

`.lmbrain/verification.toml` is the strict, versioned registry of named verification gates.
- Direct program/argv execution, confined `cwd`, minimal sanitized environment, timeout/output limits, and process-tree termination.
- Execution requires machine-local approval bound to the canonical manifest digest and workspace identity via `verification_manifest_approve`.
- May declare `fingerprint_exclude` for workspace-relative build output paths.

## Verification Lifecycle & Verbs

- `verification_manifest_status`: Report manifest status (`absent`, `invalid`, `unsafe`, `unapproved`, `approved`, `stale`, `approval-invalid`).
- `verification_manifest_init`: Preview deterministic manifest discovery from repository metadata.
- `verification_manifest_validate`: Validate complete TOML schema.
- `verification_manifest_set`: Atomically replace manifest with optimistic digest checking.
- `verification_manifest_rollback`: Restore prior version.
- `verification_manifest_approve`: Digest-bound operator approval.
- `spec_set_verification_gates`: Replace a spec's declared verification gates in `backlog`, `ready`, or `working`.
- `spec_verify`: Execute approved gates for a spec and splice the managed transcript.

## Verification Transcript Region

Inside `### Verification transcript`, `spec_verify` manages only the region delimited by:
```text
<!-- lmbrain-generated-verification:start -->
...
<!-- lmbrain-generated-verification:end -->
```
Hand-authored evidence outside this region is preserved.

## Attestation Authority

- `agent` / `kit` requirements: `phase=before-submit`.
- `lead` / `operator` requirements: `phase=before-done`.
- Lead attestations use `spec_attest_lead`.
- Operator attestations are performed by the operator via the desktop Operations page, or delegated via `spec_attest_operator_delegated`.
- Attestation records evidence only; it never changes artifact lifecycle status.
