---
id: REPORT-2026-07-03-ui-regression-takeover
title: UI regression corrective takeover
status: active
created: 2026-07-03
updated: 2026-07-03
tags: [ui, roadmap, sessions, design]
links: [SPEC-021, SPEC-025, SPEC-026]
---
# UI regression corrective takeover

## Scope

Operator explicitly requested immediate corrective implementation for three app regressions found against the Nucleus workspace:

- Roadmap milestones are not displayed for `E:\Git\Nucleus`.
- Session start button/tab alignment is visually off and terminal tab history is difficult or impossible to scroll.
- Design mockup previews show the Vite/React fallback instead of the packaged Nucleus mockups.

## Rationale

This is a bounded corrective pass over already implemented UI/data surfaces. The mockup issue has already had a failed tentative remediation, and the roadmap/session defects block verification of completed V3 features.

## Verification plan

- Add regression coverage for Nucleus-style roadmap milestone IDs and inline roadmap references.
- Add frontend regression coverage for protocol-backed design preview URLs.
- Run targeted frontend/Rust tests, then broad tests/build where feasible.
