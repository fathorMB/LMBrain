---
title: Skills
updated: 2026-07-07
---

# Skills

Skills are project-scoped operational procedures that manually started agents can read and follow during implementation or review.

They are Markdown knowledge artifacts, not executable tools. LMBrain does not run skill commands automatically.

## Layout

- `active/` - approved skills available for handoffs and context packs.
- `proposed/` - drafted skills awaiting operator approval or refinement.
- `retired/` - skills that should no longer be recommended.

Use `templates/skill.md` for new skills.

## Governance

The Project Lead may propose skills for recurring project work such as build, test, diagnostics, release checks, or review workflows. Skills that introduce destructive, credentialed, release-affecting, expensive, or otherwise risky procedures should remain proposed until the operator approves them.

Specialists may follow active skills that apply to their handoff and must record the commands/procedures actually used in implementation evidence.
