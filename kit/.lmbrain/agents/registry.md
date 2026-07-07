---
title: Agent registry
updated: 2026-06-22
---

# Agent Registry

| ID | Name | Role | Status | Activation | Definition | Domains |
| --- | --- | --- | --- | --- | --- | --- |
| AGENT-LEAD | Ada Checklist | Project Lead | active | manual | [[project-lead]] | project-management |
| AGENT-FRONTEND-UI | Marta Pixelperfetta | Frontend UI Specialist | proposed | manual | [[AGENT-FRONTEND-UI]] | frontend, ui, react |
| AGENT-TAURI-BACKEND | Bruno Fileguard | Tauri/Rust Backend Specialist | proposed | manual | [[AGENT-TAURI-BACKEND]] | tauri, rust, backend |
| AGENT-MCP-CONTRACT | Vera Protocollo | MCP/Contract Specialist | proposed | manual | [[AGENT-MCP-CONTRACT]] | mcp, contract, core |
| AGENT-KIT-DOCS | Nina Changelog | Kit/Docs/Release Specialist | proposed | manual | [[AGENT-KIT-DOCS]] | kit, docs, release |
| AGENT-REVIEWER | Clara Redpen | Product Reviewer/QA | proposed | manual | [[AGENT-REVIEWER]] | review, qa, testing |
| AGENT-DESIGN | Lia Wireframe | Design Specialist | proposed | manual | [[AGENT-DESIGN]] | design, ui-ux |

Add specialist profiles only when a real project need justifies them. Keep profiles in `profiles/` and proposals in `proposals/`.

**Activation guard:** Profiles with `status: proposed` are not ready for implementation handoff. The Project Lead must ask the operator to approve and activate a proposed profile (set `status: active`) before recommending it for a spec. The operator activates profiles by updating the frontmatter `status` field.

## V3 controlled improvement loop

Improvement proposals follow the same lifecycle as new-profile proposals but use `proposal_type: improvement` and specify a `target_profile`. The Project Lead may create improvement proposals from accepted reviews, repeated remediation findings, implementation evidence, diagnostics, or operator feedback. Operator approval is required before any behavior-affecting profile change becomes active.
