# Sessions

The Sessions view launches and monitors interactive agent terminals inside LMBrain.

Supported host/connection combinations:

- native Claude Code: `claude`;
- Claude through Ollama: `ollama launch claude --model <model>`;
- native Codex: `codex`.
- Pi through Ollama: `ollama launch pi --model <model>`.

The session contract separates the agent host (`claude`, `codex`, or `pi`)
from the connection route (`native` or `ollama`). Claude supports both,
Codex supports native only, and Pi supports Ollama only. Unsupported
combinations are rejected before a PTY is opened.

Pi requires the audited project-local package pin
`npm:pi-mcp-extension@1.5.0`. During workspace preparation LMBrain installs the
exact pin only when missing, while showing the current loading stage. A failure
does not block Pulse and is shown as a persistent warning. Before allocating a Pi PTY, the backend performs
offline/non-mutating readiness checks for the `ollama` and `pi` executables,
the Ollama API and selected tool-capable model, and the exact Pi MCP extension
version. The modal stays open and displays an actionable error when a check
fails.

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

Terminals are rendered by `SessionTerminal`, using `@xterm/xterm` and
`@xterm/addon-fit`. Every open tab keeps its xterm instance mounted; inactive
panes are hidden and the selected pane is refit/focused when activated. This
preserves in-memory scrollback, selection, and output accumulated while another
tab or app view is active. History is still process-memory state and does not
survive closing the tab or restarting LMBrain.

Each terminal shows Copy and Paste controls plus shortcut guidance:

- select text and press `Ctrl+C`, or use `Ctrl+Shift+C` / macOS `Cmd+C`, to copy;
- bare `Ctrl+C` with no selection remains the terminal interrupt signal;
- use `Ctrl+Shift+V` / macOS `Cmd+V`, or the Paste button, to paste;
- paste goes through xterm's paste path so applications receive bracketed-paste
  sequences when they enable that terminal mode.

Clipboard success, empty-selection/clipboard, and permission failures are shown
in the terminal toolbar. Clipboard access remains user initiated; LMBrain does
not read or write the clipboard in the background.

Session state lives in `WorkspaceContext` as `SessionInfo[]` with an `activeSessionId`. The Sessions view remains mounted while the app is open so terminals survive navigation between views.

## Ollama Models

Ollama model discovery runs in Rust, not in the webview. The backend first queries the local Ollama API at `http://localhost:11434/api/tags`, then falls back to parsing `ollama list` output. Models are filtered to those with tool capability where available.

Session start is stricter than display discovery: every Ollama-routed launch
must revalidate the selected model against the live API before PTY allocation.
This prevents stale, arbitrary, or non-tool-capable model identifiers from
reaching an agent command.

## Codex Binary Resolution

Native Codex sessions resolve the executable from:

1. `LMBRAIN_CODEX_BIN`;
2. a configured path passed from settings/local storage;
3. the newest Desktop Codex binary under `%LOCALAPPDATA%\OpenAI\Codex\bin\*\codex.exe`;
4. `codex.exe` or `codex` on `PATH`.

## Scope

Sessions are ephemeral. They live while the app is open and are not restored after restart.
