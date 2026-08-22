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

## Harness-agnostic dispatch model selection

`capability_tier` is also the canonical dispatch model class. When the operator explicitly authorizes the Project Lead to dispatch a spec, the Lead must select the specialist model from this field and pass it explicitly to the harness:

| `capability_tier` | Required model class | Anthropic / Claude | Tier-aware model aliases (including Codex when offered) |
| --- | --- | --- | --- |
| `sol` | highest-capability | `opus` | model name ending in `-sol` |
| `terra` | balanced | `sonnet` | model name ending in `-terra` |
| `luna` | fast, economical | `haiku` | model name ending in `-luna` |

For another harness, resolve its documented native model that matches the required model class. Do not change the spec estimate to fit the available models. If the harness does not expose an explicit model selector, no matching model is available, or the mapping cannot be determined unambiguously, dispatch fails closed: do not spawn and report the limitation to the operator.

The Lead must resolve this mapping for each spec in a batch. It must never omit the model selector, inherit the Lead's model, or use one model for all dispatched specs. `thinking_level` controls reasoning effort separately when supported and never changes this tier-to-model mapping.

## Spec Tag Vocabulary

- Assigned via `spec_set_tags`.
- Normalized lowercase `^[a-z0-9][a-z0-9-]*$`.
- Cannot restate structured fields (`milestone`, `area`, `priority`).

## Effort & Tag Verbs

- `spec_set_effort`: Set capability tier and thinking level for a backlog or ready spec.
- `spec_record_effort_observation`: Record post-implementation actual effort observation.
- `spec_set_tags`: Assign normalized tags to a spec.
