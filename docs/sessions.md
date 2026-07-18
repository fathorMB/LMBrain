# Sessions

The Sessions view launches and monitors interactive agent terminals inside LMBrain.

Supported host/connection combinations:

- native Claude Code: `claude`;
- Claude through Ollama: `ollama launch claude --model <model>`;
- native Codex: `codex`.
- Pi through Ollama: `ollama launch pi --model <model>`.
- OpenCode through Ollama: `opencode <workspace> --model ollama/<model>` with a session-scoped provider for the local Ollama API.

The session contract separates the agent host (`claude`, `codex`, `pi`, or `opencode`)
from the connection route (`native` or `ollama`). Claude supports both,
Codex supports native only, while Pi and OpenCode support Ollama only. Unsupported
combinations are rejected before a PTY is opened.

Pi requires the audited project-local package pin
`npm:pi-mcp-extension@1.5.0`. During workspace preparation LMBrain installs the
exact pin only when missing, while showing the current loading stage. A failure
does not block Pulse and is shown as a persistent warning. Before allocating a Pi PTY, the backend performs
offline/non-mutating readiness checks for the `ollama` and `pi` executables,
the Ollama API and selected tool-capable model, and the exact Pi MCP extension
version. The modal stays open and displays an actionable error when a check
fails.

OpenCode needs no extension. It consumes LMBrain's native local MCP entry from
project `opencode.json`; preflight requires both OpenCode and the local Ollama API.
LMBrain starts OpenCode directly with the absolute workspace positional and
`ollama/<model>`, while `OPENCODE_CONFIG_CONTENT` provides the official
OpenAI-compatible provider at `http://localhost:11434/v1`. Project file search
and LSP roots therefore no longer depend on a nested Windows launcher.
Generated OpenCode configuration includes `@workspace/` as an explicit local
reference, providing deterministic project-file completion independently of the
ordering used by OpenCode's bare `@` suggestion popup.

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

Wheel handling is buffer-aware. Normal terminal output follows xterm's native
wheel path, including Codex sessions launched with `--no-alt-screen`. When a
full-screen application activates the alternate buffer, LMBrain delegates mouse
tracking to xterm or sends the harness's documented wheel binding. Modifier-
assisted wheel gestures are left to xterm/the host.

LMBrain uses xterm 6 for corrected alternate-buffer wheel and viewport handling.
Embedded OpenCode sessions disable OpenCode mouse capture so xterm remains the
single owner of wheel and selection behavior. OpenCode uses its documented
alternate message bindings (`Ctrl+Alt+Y/E`) because wheel events are not handled
reliably through Windows ConPTY.

Each terminal shows Copy and Paste controls plus shortcut guidance:

- select text and press `Ctrl+C`, or use `Ctrl+Shift+C` / macOS `Cmd+C`, to copy;
- bare `Ctrl+C` with no selection remains the terminal interrupt signal;
- use `Ctrl+Shift+V` / macOS `Cmd+V` to paste;
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

LMBrain launches the interactive Codex CLI with its supported
`--no-alt-screen` option. Inline mode preserves conversation output in xterm's
normal scrollback buffer, so the embedded terminal can scroll and select the
full transcript consistently. This option affects only LMBrain-launched Codex
sessions and does not modify the user's Codex configuration.

## Terminal scroll and selection policy

Scroll gestures are resolved at event time from the active xterm buffer and
the mouse-tracking mode the TUI actually enabled, never from a per-host
assumption:

- Normal buffer: native xterm wheel scrolling for every host.
- Alternate buffer with mouse tracking enabled: wheel events delegate to
  xterm's native mouse reports so the TUI scrolls itself.
- Alternate buffer without tracking: Pi uses Page Up/Down; OpenCode uses its
  documented Ctrl+Alt line bindings; Codex delegates to xterm's
  alternate-screen arrow emulation; Claude Code and unknown hosts degrade
  visibly with a hint instead of swallowing the gesture.

The terminal toolbar intentionally does not duplicate native copy, paste, or
scroll actions. Drag to select in normal buffers; use Shift+drag while a TUI
captures the mouse. Standard terminal shortcuts handle clipboard interaction.
Durable transcript search remains available from the session's **Search logs**
panel.

### Manual harness verification checklist

Run before release on Windows for each supported harness (Claude Code, Codex,
Pi, OpenCode):

- [ ] Mouse wheel up/down scrolls (or visibly explains why not).
- [ ] Shift+drag selects while the TUI tracks the mouse.
- [ ] Copy and paste work through the documented keyboard shortcuts.
- [ ] Ctrl+C without a selection interrupts the harness (SIGINT).
- [ ] Clipboard failures show a specific, actionable message.

## Scope

Sessions are ephemeral. They live while the app is open and are not restored after restart.
