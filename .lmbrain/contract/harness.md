# Harness Manifest Capability Module

This module defines the optional project harness declaration and configuration capability for LMBrain workspaces.

## Scope & Application

When `.lmbrain/HARNESSES.json` is present or host configuration is managed, this module governs harness intent, validation, approval, and application.

## Harness Manifest

`.lmbrain/HARNESSES.json` is the strict, versioned source of project harness intent for supported hosts (`claude-code`, `codex`, `pi`, `open-code`).
- Each host declares `enabled`, portable `required_tools`, non-secret `environment` values, and optional `lsp.required`.
- Schema is strict: unknown fields, commands, scripts, hooks, paths, secrets, or unsupported capabilities are rejected.

## Harness Verbs

- `harness_config_get`: Read current manifest intent and status.
- `harness_config_validate`: Validate schema without modifying files.
- `harness_config_set`: Atomically update harness manifest.
- `harness_manifest_approve`: Digest-bound operator approval.
- `harness_approval_revoke`: Revoke approval.
- `harness_approval_status`: Check approval state against canonical digest.
- `harness_plan_preview`: Preview file modifications without writing.
- `harness_config_apply`: Apply approved manifest to native host files.
- `harness_drift_status`: Compare applied files against expected digest.
