# Sessions

The Sessions view launches and monitors interactive agent terminals inside LMBrain.

Supported launch modes:

- native Claude Code: `claude`;
- Claude through Ollama: `ollama launch claude --model <model>`;
- native Codex: `codex`.

## Backend

Sessions are owned by `SessionManager` in `src-tauri/src/commands/sessions.rs`.

The backend:

- starts each process in a PTY with `portable-pty`;
- uses the active workspace root as `cwd`;
- streams PTY output to the frontend as `session-output`;
- accepts terminal input through `session_write`;
- propagates terminal size through `session_resize`;
- buffers output before frontend attach so initial TUI frames are not lost;
- reports exits through `session-exit`;
- kills sessions when requested or when the app exits.

Native Claude sessions load the workspace `.mcp.json` just like an external
Claude Code launch. LMBrain refreshes that file when the workspace is opened and
again immediately before starting a native Claude session. It prefers a
resolvable absolute bundled `lmbrain-mcp` sidecar path when it can discover one,
and appends that binary's directory plus `LMBRAIN_MCP_BIN` to the child environment, so
app-launched Claude sessions are not dependent on the user's interactive shell
`PATH`.

## Frontend

`src/components/Sessions/SessionsView.tsx` renders a tab-based session workspace. A tab strip shows all active sessions; clicking a tab switches the active terminal. Each tab displays the session label, mode/exit status, and a close button.

The active terminal is rendered by `SessionTerminal`, which uses `@xterm/xterm` and `@xterm/addon-fit`. Only the active session's terminal is mounted; switching tabs re-attaches via the backend's pre-attach buffering, preserving output history.

Session state lives in `WorkspaceContext` as `SessionInfo[]` with an `activeSessionId`. The Sessions view remains mounted while the app is open so terminals survive navigation between views.

## Ollama Models

Ollama model discovery runs in Rust, not in the webview. The backend first queries the local Ollama API at `http://localhost:11434/api/tags`, then falls back to parsing `ollama list` output. Models are filtered to those with tool capability where available.

## Codex Binary Resolution

Native Codex sessions resolve the executable from:

1. `LMBRAIN_CODEX_BIN`;
2. a configured path passed from settings/local storage;
3. the newest Desktop Codex binary under `%LOCALAPPDATA%\OpenAI\Codex\bin\*\codex.exe`;
4. `codex.exe` or `codex` on `PATH`.

## Scope

Sessions are ephemeral. They live while the app is open and are not restored after restart.
