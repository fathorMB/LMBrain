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

## Frontend

`src/components/Sessions/SessionsView.tsx` renders a canvas of floating session windows with `react-rnd`. Each window embeds `SessionTerminal`, which uses `@xterm/xterm` and `@xterm/addon-fit`.

Session window geometry and z-order live in `WorkspaceContext`. The Sessions view remains mounted while the app is open so terminals survive navigation between views.

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
