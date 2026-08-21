# LMBrain Kit Feedback Capability Module

This module defines the upstream field feedback reporting channel for LMBrain workspaces.

## Scope & Application

When direct evidence uncovers usability bugs, workflow friction, or contract gaps in LMBrain itself, notes are recorded in `reports/lmbrain-kit-feedback.md`.

## Feedback Structure

- Identity: `LMBRAIN-KIT-FEEDBACK`, with typed `notes` array.
- Each note has a stable `KIT-NOTE-*` ID, timestamp, LMBrain version, category, severity, summary, impact, and evidence.
- Categories: `bug`, `usability`, `workflow`, `documentation`, `compatibility`, `performance`, `improvement`.
- Severities: `blocking`, `high`, `medium`, `low`, `info`.

## Feedback Verbs

- `lmbrain_feedback_record`: Autonomously append an evidence-backed feedback note.
- `lmbrain_feedback_report`: Read-only view of recorded feedback notes.
- `lmbrain_feedback_resolve`: Mark a note resolved or reconfirmed against an LMBrain release.
