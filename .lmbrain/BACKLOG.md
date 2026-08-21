---
title: Product and technical backlog
updated: 2026-08-10
---

# Backlog

This is a concise, prioritized index of opportunities and work areas. Implementation handoffs (specs) live under `specs/`.

## Now

- [[SPEC-049-fix-specification-detail-navigation-and-breadcrumb-semantics]] - isolated 3.1.0 navigation/accessibility regression; suitable as the first low-risk fix after approval.
- [[SPEC-050-unify-actionable-diagnostics-and-reconcile-the-project-digest]] - shared diagnostic and project-state foundation for the rest of 3.1.0.
- [[SPEC-051-add-governed-review-verdict-transitions-and-structured-lifecycle-events]] - restore truthful negative review verdicts and create authoritative cycle history.
- [[SPEC-001-tauri-read-only-desktop-mvp]]
- [[SPEC-023-v3-context-economy]] - define token-efficient context packs and MCP read tools before the rest of v3 work.
- [[SPEC-036-verification-transcript-integrity-and-kit-executed-gates]] - 2.9.0 foundation: reject evidence-empty submissions and establish operator-approved execution provenance.

## Next

- [[SPEC-071-rank-specs-by-reliably-observed-review-remediation-cycles-in-insights]] - issue #105: replace Insights agent/area change-request rates with a source-aware spec remediation-cycle ranking.
- [[SPEC-070-add-operator-invoked-project-lead-dreaming-sessions-and-dream-journal]] - issue #104: operator-invoked, provenance-preserving reflective planning and a read-only Dream Journal; does not promote hypotheses automatically.
- [[SPEC-052-make-agent-effectiveness-metrics-cycle-aware-and-taxonomy-stable]] - consume structured review events after SPEC-051.
- [[SPEC-053-enforce-before-done-verification-gates-with-distinct-lead-and-operator-attestation]] - enforce phase/owner authority after the diagnostic foundation.
- [[SPEC-054-bootstrap-and-diagnose-the-verification-manifest-safely]] - add preview-first manifest onboarding without weakening digest approval.
- [[SPEC-055-add-governed-ready-spec-parking-with-preserved-approval-history]] - add reasoned `ready -> backlog` parking.
- [[SPEC-056-add-first-class-spec-dependencies-with-lifecycle-enforcement]] - add a validated hard-prerequisite graph and transition enforcement.
- [[SPEC-057-define-first-class-cross-spec-finding-architecture-and-core-lifecycle]] - issue #12 core/MCP tranche; blocked on operator acceptance of [[ADR-014-promoted-findings-have-independent-lifecycle-while-reviews-preserve-historical-outcome]].
- [[SPEC-058-add-findings-desktop-experience-and-explicit-legacy-migration]] - follows SPEC-057.
- [[SPEC-059-integrate-qualify-and-release-lmbrain-3-1-0]] - release integration after accepted leaf reviews; issue #12 is mandatory, not a deferrable follow-up.
- [[SPEC-024-v3-agent-taxonomy-and-improvement-loop]] - introduce granular specialists and a controlled self-improvement proposal loop.
- [[SPEC-025-v3-session-tabs]] - replace floating session windows with tabbed sessions.
- [[SPEC-026-v3-milestone-intelligence]] - redesign milestones as actionable project intelligence.
- [[SPEC-035-settings-and-project-harness-governance]] - make Settings functional, move Local Harnesses into it, and add Lead-governed project harness environments for 2.8.0.
- [[SPEC-036-verification-transcript-integrity-and-kit-executed-gates]] - mechanically reject evidence-empty submissions and add operator-approved kit-generated verification provenance.
- [[SPEC-037-close-related-reviews-when-a-spec-is-done]] - atomically address resolved corrective reviews at spec closeout without losing their historical verdict.
- [[SPEC-038-context-complete-handoffs-and-structured-verification-gate-contracts]] - expose every executable, manual, and operator gate plus approved profile/skill guidance to implementers before submission.
- [[SPEC-039-governed-agent-improvement-recommendations-proposal-application-and-effectiveness-metrics]] - derive evidence-backed improvement signals, apply approved bounded profile changes, and measure recurrence by profile.
- [[SPEC-040-lmbrain-2-9-0-release-integration-migration-and-astranexus-regression-qualification]] - integrate the release train, preserve customized projects during migration, and qualify 2.9.0 against AstraNexus failure classes.
- [[SPEC-060-kit-file-realignment-and-drift-diagnostics]] - issue #34: kit-owned file realignment procedure and drift detection during migration.
- [[SPEC-061-fix-windows-node-repl-kernel-path-initialization]] - issue #35: Windows Node REPL kernel asset directory initialization fix.
- [[SPEC-062-align-browser-skill-and-url-policy-for-local-files]] - issue #36: align Browser skill specification and integrated browser URL policy for file:// targets.
- [[SPEC-063-unblock-read-only-ops-on-claimed-local-tabs]] - issue #37: unblock DOM snapshot/screenshot operations on claimed user-opened file:// tabs.
- [[SPEC-064-atomic-attestation-and-checklist-completion-for-lead-gates]] - issue #38: auto-mark checklist item in spec_attest_lead for owner=lead gates.
- [[SPEC-065-support-waived-acceptance-criteria-linked-to-findings]] - issue #39: support waived acceptance criteria syntax (waived=FINDING-xxx) at spec_done.
- [[SPEC-066-make-3-1-x-background-loading-asynchronous]] - issue #40: make 3.1.x background data operations asynchronous to prevent UI stalls.
- [[SPEC-067-add-lead-remediation-verification-review-event]] - issue #41: add Lead remediation verification verb and actor support for review events.
- [[SPEC-068-add-active-sessions-close-confirmation-dialog]] - issue #42: show close confirmation dialog when active sessions are open on app window exit.


## Later

## Parking lot
