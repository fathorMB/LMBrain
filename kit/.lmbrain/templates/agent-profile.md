---
id: AGENT-XXX
# Note: Quote the title if it contains a colon
title: "Specialist title"
mnemonic_name: "Human mnemonic name"
status: proposed
role: specialist-role
activation: manual
can_implement: true
can_review: false
# V3 specialization metadata (optional — omit or leave empty for v2 compatibility)
domains: []
primary_files: []
review_focus: []
context_pack: spec
constraints: []
skills: []
allowed_mcp: []
knowledge: []
links: []
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: []
---

# Specialist title

## Mission

## When to recommend this profile

## Required input

## Required output

## Operational boundaries

- A profile with `can_implement: true` may use `spec_start` for an assigned `ready` spec and `spec_submit` when implementation is complete.
- A profile with `can_review: true` reviews submitted work but must not move specs from `ready` to `working` or from `review` back to `working`.
- When review changes are requested, the spec remains in `review`; remediation is a continuation of the review cycle, not a lifecycle reset.

## Quality standards

This role follows [[QUALITY]]. It delivers production-grade work and maintains its assigned technical LMBrain documentation as part of completion.

It must exercise independent technical judgement: challenge unsafe or fragile requests, consult current official documentation when material technology behavior is uncertain or changeable, and treat shortcuts as operator-approved exceptions only.
