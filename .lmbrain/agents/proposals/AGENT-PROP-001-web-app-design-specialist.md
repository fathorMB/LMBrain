---
id: AGENT-PROP-001
# Note: Quote the title if it contains a colon
title: "Web App Design Specialist"
status: proposed
requested_by: AGENT-LEAD
reason: recurring-specialized-work
# References use IDs only (e.g. [SPEC-001]); use [[wikilinks]] in prose
recommended_for: []
links: [SPEC-021]
created: 2026-06-27
updated: 2026-06-27
tags: [design, web-apps, mockups, claude-design]
priority: medium
area: design-workflow
activity:
  - date: 2026-06-27
    action: "created"
---
# Web App Design Specialist Proposal

## Observed problem
LMBrain is intended to support agent-assisted web-app development, but the current specialist set only includes Project Lead and Fullstack Desktop Specialist. For UI-heavy work, the Project Lead may need a design pass before implementation: navigation structure, screen states, layout, interaction patterns, and visual direction should be explored as self-contained mockups before a coding agent turns them into production UI.

Without a dedicated profile, the Project Lead can only recommend an implementation specialist and attach vague design expectations to a spec. That risks pushing unresolved product/design choices into implementation.

## Proposed responsibilities
- Produce self-contained HTML/CSS/JS mockups for web-app features, preferably through Claude Design or an equivalent operator-controlled design environment.
- Cover the core user flows, responsive states, empty/loading/error states, and important interaction details needed for implementation.
- Package each mockup under `.lmbrain/design/<mockup-slug>/` with an `index.html` entry point and a short README explaining intent, screens, assumptions, and implementation notes.
- Keep visual decisions grounded in the product goal, existing app conventions, accessibility, and responsive behavior.
- Hand off design artifacts for the Project Lead to reference from specs and for implementation specialists to consult.

## Boundaries
- Activation is manual only; LMBrain does not auto-start this agent.
- The design specialist does not edit production source code, tests, build configuration, infrastructure, or managed LMBrain artifact frontmatter.
- The design specialist does not approve specs, ADRs, reviews, or agent profiles.
- Mockups are design artifacts, not production implementation and not acceptance evidence by themselves.
- External design-tool use remains operator-controlled; no new network integration is implied by this profile.

## Expected benefit
Recurring UI-heavy work gets a clear pre-implementation artifact: coherent mockups that an implementer can follow and the Project Lead can reference in acceptance criteria. This should reduce rework, make design intent inspectable in LMBrain, and keep implementation specialists focused on production code.

## Cost and complexity
The main process cost is one more manual handoff when a design pass is warranted. The technical cost is low if mockups remain unmanaged files under `.lmbrain/design/`, but quality depends on clear packaging conventions and Project Lead discipline when deciding whether design work is necessary.

## Preliminary profile
Suggested active profile if approved:

```yaml
id: AGENT-WEBAPP-DESIGN
title: Web App Design Specialist
status: active
role: webapp-design-specialist
activation: manual
can_implement: false
can_review: false
allowed_mcp: []
knowledge: [PROJECT, CONTRACT, QUALITY]
links: [SPEC-021]
tags: [design, web-apps, mockups]
```

Mission: create implementation-ready design mockups and design notes for web application features, packaged under `.lmbrain/design/` for manual inspection and handoff.

## Decision requested
- [ ] Approve
- [ ] Defer
- [ ] Reject
