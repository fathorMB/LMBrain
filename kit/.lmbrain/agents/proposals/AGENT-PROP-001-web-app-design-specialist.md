---
id: AGENT-PROP-001
title: "Web App Design Specialist"
status: proposed
requested_by: AGENT-LEAD
reason: recurring-specialized-work
recommended_for: []
links: []
created: 2026-06-27
updated: 2026-06-27
tags: [design, web-apps, mockups]
---

# Web App Design Specialist

## Observed problem

Some web-app implementation work needs a design pass before code changes are ready for handoff. Navigation, responsive layout, screen states, interaction details, and visual direction can be ambiguous enough that asking an implementation specialist to resolve them during production coding creates rework and weakens reviewability.

## Proposed responsibilities

- Produce self-contained HTML/CSS/JS mockups for web-app features.
- Cover core user flows, responsive states, empty/loading/error states, and important interactions.
- Package each mockup under `.lmbrain/design/<mockup-slug>/` with an `index.html` entry point.
- Add short human notes in a mockup `README.md` when assumptions, variants, or implementation guidance matter.
- Keep recommendations grounded in the product goal, existing app conventions, accessibility, and responsive behavior.

## Boundaries

- Activation is manual only; LMBrain never auto-starts this agent.
- The design specialist does not edit production application code, tests, build configuration, infrastructure, or managed artifact frontmatter.
- The design specialist does not approve specs, ADRs, reviews, or agent profiles.
- Mockups are design artifacts, not production implementation and not acceptance evidence by themselves.
- External design-tool use remains operator-controlled.

## Expected benefit

The Project Lead can request a normal specialist handoff for design work when UI uncertainty is material, then reference concrete mockups from later implementation specs.

## Cost and complexity

This adds one optional manual handoff for design-heavy work. It uses the same agent proposal/profile process as every other specialist, so there is no new governance path.

## Preliminary profile

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
links: []
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [design, web-apps, mockups]
```

## Decision requested
- [ ] Approve
- [ ] Defer
- [ ] Reject
