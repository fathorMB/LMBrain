---
id: ADR-006
title: In-app PTY session execution for Claude Code
status: proposed
decision_date:
decider:
supersedes: []
superseded_by: []
links: [ADR-001]
tags: [architecture, sessions, terminal, process-execution]
---

# In-app PTY session execution for Claude Code

## Context

LMBrain is desktop-first and already operates directly on local repositories per [[ADR-001-desktop-first-tauri]]. The new Sessions view needs to launch and monitor interactive Claude Code terminals inside the app, preserve them while the app remains open, and support both native `claude` and `ollama launch claude --model <model>` entry points.

This extends the product from read-oriented repository services into operator-triggered local process execution. The chosen implementation also needs to render full-screen TUI behavior inside the webview without introducing native OS windows or a browser-side localhost networking exception.

## Decision

LMBrain will execute Claude Code sessions through a backend-managed PTY service:

- The Tauri backend owns PTY lifecycle, process spawning, output streaming, resize propagation, and shutdown cleanup.
- Interactive terminal rendering lives in the webview via `xterm.js`.
- Session windows are in-app floating panels, not native windows.
- Ollama model discovery runs in Rust against `http://localhost:11434/api/tags`, with CLI fallback if the local API is unavailable.
- All session processes inherit the desktop app environment and use the currently opened workspace root as `cwd`.

## Consequences

### Positive

- Claude Code keeps its normal interactive TUI behavior inside LMBrain.
- The existing webview CSP remains locked down; localhost discovery happens in Rust instead of the browser.
- Multiple concurrent sessions can be orchestrated without leaving the app shell.
- PTY ownership in one backend service gives a single place to guarantee cleanup on app shutdown.

### Constraints

- LMBrain now has an explicit process-execution surface and must keep it operator-triggered and tightly scoped to the active workspace context.
- Windows ConPTY behavior becomes part of the supported runtime surface, especially for resize and terminal rendering.
- Sessions are ephemeral: they are not restored after app restart in v1.

## Alternatives considered

### Native OS terminal windows

Rejected because the feature requires in-app floating windows and persistent session UI state while switching LMBrain views.

### Browser-side terminal plus direct localhost fetches

Rejected because the webview CSP intentionally blocks localhost requests and because process lifecycle belongs in the desktop backend.

### Tauri shell plugin

Rejected because direct PTY spawning via Rust is smaller, keeps full terminal control in one service, and avoids expanding the plugin surface.
