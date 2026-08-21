---
id: SPEC-043
title: "Sidebar simplification and exit guard"
status: backlog
kind: feature
priority: medium
area: frontend
milestone: M-06
recommended_agent: AGENT-FULLSTACK-DESKTOP
related_tasks: []
related_decisions: []
links: []
created: 2026-07-17
updated: 2026-07-17
tags: [frontend, tauri, react]
---

# Sidebar simplification and exit guard

## Objective
Simplify workspace navigation by removing the top workspace switcher from the sidebar, splitting the "Agents & MCP" screen into two dedicated tabs, and implementing an explicit confirmation dialog before leaving the workspace.

## Context
Accidentally clicking the workspace header in the sidebar returns the operator to the repository picker, terminating running sessions and watchers without warning. In addition, combining "Agents" and "MCP" features under a single screen increases coupling and memory footprint.

## Scope
### Included
- Remove workspace switcher button and its subsequent divider from the top of the sidebar.
- Split "Agents & MCP" into two independent views: `AgentsView` and `McpView`.
- Add a dedicated **Leave workspace** exit action at the bottom of the sidebar.
- Implement an accessible confirmation modal when leaving the workspace that traps focus and is dismissible via Escape.
- Trigger standard cleanup (watcher shutdown, session termination) only after user confirms leaving.

### Excluded
- Modifying the styling of the main menu picker.

## Existing-project analysis
`Sidebar.tsx` currently renders a `<button onClick={goToPicker}>` at the top. The navigation item maps `agents` to `<AgentsMCPView />`. The backend implements watcher teardown and PTY shutdown inside `goToPicker`'s cleanup sequence in the frontend hooks (`useWorkspace`).

## Technical proposal
1. **Sidebar Changes:**
   - Remove workspace switcher from the top.
   - Separate `agents` nav key from `mcp` in `NAV_ITEMS`.
   - Add a bottom menu item "Leave workspace" with a material icon (e.g. `logout` or `exit_to_app`).
2. **Splitting Views:**
   - Split `AgentsMCPView.tsx` into `AgentsView.tsx` and `McpView.tsx`.
   - Update `AppShell.tsx` switch statement to handle both case `agents` and case `mcp` independently.
3. **Exit Dialog Modal:**
   - Add a state `showExitConfirm` in `useWorkspace` context or `Sidebar.tsx`.
   - Implement `LeaveWorkspaceModal.tsx` rendering a React portal modal.
   - Modal should trap focus using a standard hook/ref and restore focus on dismiss.
   - Pressing Escape, clicking backdrop, or clicking "Stay in workspace" should close the modal.
   - Clicking "Leave workspace" triggers the destructive exit cleanup flow.

## Files and areas involved
- `src/components/Layout/Sidebar.tsx`
- `src/components/Layout/AppShell.tsx`
- `src/components/Layout/LeaveWorkspaceModal.tsx` [NEW]
- `src/components/Agents/AgentsView.tsx` [NEW]
- `src/components/Agents/McpView.tsx` [NEW]
- `src/components/Agents/AgentsMCPView.tsx` [DELETE]
- `src/types.ts` or `src/context/WorkspaceContext.tsx` for state changes.

## Acceptance criteria
- [ ] The workspace selector button at the top of the sidebar is gone.
- [ ] "Agents" and "MCP" are separate navigation links on the sidebar.
- [ ] Clicking "Leave workspace" opens a confirmation dialog with text: "Leave this workspace? You'll return to the main menu. Any running agent sessions for this workspace will be stopped."
- [ ] Escape key, backdrop clicks, and clicking "Stay in workspace" close the dialog without changing workspace state.
- [ ] Confirming the exit action calls `goToPicker` and cleans up watchers and active sessions.

## Required verification
### Automated Tests
- `pnpm test` (verify all components mount and run successfully)

### Manual Verification
- Click "Leave workspace", press Escape, verify modal closes and focus returns.
- Click "Leave workspace", click "Leave", verify app returns to picker and all running sessions/watchers are terminated.

## Production quality and documentation
- Follow [[QUALITY]].

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete.

## Implementation evidence
> Filled in by the specialist after completion.
