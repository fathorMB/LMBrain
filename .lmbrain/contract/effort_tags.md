# Effort Tiers & Spec Tags Capability Module

This module defines effort estimation heuristics and tag taxonomy for specs in LMBrain workspaces.

## Scope & Application

Applied when estimating change scope or categorizing specs with descriptive tags.

## Implementation Effort Tiers

Required before a spec moves to `ready`:
- `capability_tier`:
  - `luna`: Small footprint (roughly ~2 files, known change).
  - `terra`: Medium footprint (several files in one subsystem).
  - `sol`: Large footprint (cross-subsystem, frontend + backend + contract).
- `thinking_level`: `minimal`, `standard`, `extended`, `maximum` (defaults from tier).
- `effort_observations`: Specialist-recorded observations after implementation (`spec_record_effort_observation`).

## Spec Tag Vocabulary

- Assigned via `spec_set_tags`.
- Normalized lowercase `^[a-z0-9][a-z0-9-]*$`.
- Cannot restate structured fields (`milestone`, `area`, `priority`).

## Effort & Tag Verbs

- `spec_set_effort`: Set capability tier and thinking level for a backlog or ready spec.
- `spec_record_effort_observation`: Record post-implementation actual effort observation.
- `spec_set_tags`: Assign normalized tags to a spec.
