# Product

LMBrain is a local Markdown project brain paired with a desktop visualizer and agent tooling.

The main product idea is simple: a repository owns its project state as Markdown files under `.lmbrain/`, and LMBrain provides a local desktop app plus controlled mutation tools so humans and coding agents can work against that state without introducing a database or hosted service.

## What Ships

- `kit/.lmbrain/`: the reusable project-brain template copied into target repositories.
- Desktop app: a Tauri 2 application with a React 19 frontend.
- `lmbrain-core`: a Rust crate for controlled artifact creation, transitions, path safety, frontmatter editing, and invariants.
- `lmbrain-mcp`: an MCP stdio server that exposes safe mutation verbs to agent hosts.
- CI release pipeline: version alignment, tests, installer builds, MCP binary artifacts, and GitHub Release publishing.

## Core Workflow

1. A user copies or initializes the kit into a target repository.
2. The user opens that repository in LMBrain.
3. LMBrain reads `.lmbrain/` and shows project pulse, wiki, board, design mockups, roadmap, reviews, durable findings, decisions, agents, and MCP state.
4. LMBrain registers local agent tooling for supported hosts.
5. The user manually starts agents or sessions when needed.

LMBrain does not automatically start agents and does not require a remote service.

The application header provides an explicit current-view refresh. It reloads
shared workspace artifacts, diagnostics, Git metadata, and view-local queries
without restarting the watcher, agent processes, or persistent session
terminals. Refresh success and failure are shown directly in the header.

## Unread Badges

Sidebar entries carry a numeric badge counting the items on that page the
operator has not seen yet. One shared policy applies everywhere:

- Badged pages are the ones that own a collection of items: Board, Reviews,
  Findings, Kit Feedback, Decisions, Agents, MCP, and Skills. Agents and MCP
  count profiles and proposals together.
- Wiki, Design, and Repository are never badged: they render documents and Git
  state rather than governed items. Pulse, Insights, Roadmap, and Sessions are
  not badged either, because they derive from artifacts already counted on the
  pages that own them.
- An item is unread when it is new to the workspace, or when its status or its
  `updated` date changed since the page was last displayed. Malformed records
  still count, so parsing problems stay visible.
- Displaying a page marks everything currently on it as read, including items
  that arrive from a watcher update while the page is open. Opening a single
  spec marks only that spec, leaving the rest of the Board unread.
- A workspace opened for the first time is baselined as fully read, so badges
  report what happened since, instead of the entire existing backlog.
- Read state is stored per workspace path in local browser storage. It survives
  refreshes and application restarts, and is never shared between workspaces.
  Unreadable or malformed stored state is discarded and rebaselined rather than
  breaking navigation.
- The count is part of the navigation entry's accessible name (for example
  "Reviews, 3 unread items"), so it is never conveyed by the badge alone.

## Main Views

- Pulse: current project state, diagnostics, and recommended actions.
- Insights: read-only artifact inventory, spec flow, review-quality statistics, and full-width reliability checks that disclose missing review metadata and expandable contract diagnostics. Each diagnostic can copy the same corrective agent prompt used by Pulse; diagnostic remediation remains operator-controlled.
- Wiki: file tree and Markdown rendering for the `.lmbrain/` workspace.
- Board: specifications grouped by status.
- Design: operator-loaded self-contained HTML/CSS/JS mockups from `.lmbrain/design/`.
- Roadmap (milestone intelligence): milestones with derived spec status, reviews, decisions, risks, dependencies, and next actions.
- Reviews and Decisions: project governance artifacts.
- Findings: a read-only active/history workspace for durable cross-spec observations, canonical relationships, typed lifecycle evidence, and governed-action prompts. It never duplicates lifecycle authority in the app.
- Board/spec detail: read-only hard-prerequisite blockers, prerequisite-complete filtering, and preserved parking history. Approval, dependency mutation, parking, and status changes stay outside the app.
- Project Lead experience: operator-facing responses use concise plain language in the operator's language, while technical density remains available in artifacts and agent handoffs. Evidence-backed feedback about LMBrain itself accumulates in one portable report for the product team.
- Agents & MCP: agent profiles, proposals, MCP records, and built-in MCP tools.
- Sessions: floating interactive terminals for supported agent CLIs.
- Session terminals expose consistent wheel and page navigation across normal and full-screen buffers, including packaged Windows builds.
- Local Harnesses: user-level Claude Code, Codex, Pi, and OpenCode installation status, exact paths/versions, and explicitly confirmed self-updates with logs and post-update verification.
- Settings: local preferences and agent binary paths.

## Local-First Boundaries

LMBrain reads and writes local files selected by the user. Repository state remains versionable Markdown. Generated host configuration such as `.mcp.json`, `.codex/`, `opencode.json`, and `AGENTS.md` is workspace-local and machine-specific, so it is ignored in this repository.
