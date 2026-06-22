---
title: Project pulse
updated: 2026-06-22
---

# Project Pulse

## Current focus

Test round 1 remediation ([[SPEC-011-remediate-test-round-1-findings]]) accepted; cutting the **1.0.6** GitHub release.

## Current milestone

M-01 — Read-only desktop workspace.

## Awaiting remediation

None. WI-1 (phantom `UNKNOWN` artifacts), WI-3 (collapsible wiki tree), WI-2 (LF line endings), and WI-4 (template hardening) are accepted via [[REVIEW-011-spec-011-escalation-verification]]; the earlier WI-4 scope/version regressions from [[REVIEW-010-spec-011-remediate-test-round-1]] are resolved.

## In progress

1.0.6 release: versions aligned across `package.json`, `Cargo.toml`, `Cargo.lock`, kit and live `VERSION`; `check-version.mjs` green. Pending the `build-installers.yml` CI run (native Rust build/tests + installer artifacts).

## Blockers and risks

- Native Rust verification and the clean-build LF scaffold check (SPEC-010 F-1 end-to-end) cannot run in the Project Lead environment; they are gated by the release CI. Confirm the green CI run and an LF scaffold from the published 1.0.6 installer before closing F-1.

## Next recommended actions

1. Push to `main`; confirm `build-installers.yml` passes and publishes the 1.0.6 release.
2. After release, scaffold from the 1.0.6 installer and byte-check the brain is LF to close SPEC-010 F-1.
3. Plan test round 2.

## Recent decisions

- [[ADR-001-desktop-first-tauri]]
