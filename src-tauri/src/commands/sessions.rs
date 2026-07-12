use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::commands::mcp_registration::resolve_mcp_command_for_root;
use crate::commands::pi_registration::{has_pinned_pi_mcp_extension, PI_MCP_EXTENSION_SOURCE};
use crate::errors::AppError;
use crate::models::session::{
    AgentHost, ModelRoute, OllamaModel, SessionExitPayload, SessionInfo, SessionOutputPayload,
    SessionStartRequest, SessionStatus,
};

const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 32;
const SESSION_OUTPUT_EVENT: &str = "session-output";
const SESSION_EXIT_EVENT: &str = "session-exit";

pub struct SessionManager {
    inner: Arc<Mutex<SessionManagerInner>>,
}

struct SessionManagerInner {
    sessions: HashMap<String, ManagedSession>,
}

struct ManagedSession {
    info: SessionInfo,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    /// Output produced before the frontend terminal attached. The PTY emits its
    /// first frame (e.g. a TUI entering the alternate screen) before xterm has
    /// registered its `session-output` listener; without this buffer that frame is
    /// lost and the terminal stays blank. Replayed verbatim on attach.
    pre_attach: String,
    attached: bool,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionManagerInner {
                sessions: HashMap::new(),
            })),
        }
    }

    pub fn start(
        &self,
        cwd: &Path,
        app: AppHandle,
        request: SessionStartRequest,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        let label = default_label(&request);
        let cmd = build_command(&request, cwd)?;

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: DEFAULT_ROWS,
                cols: DEFAULT_COLS,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| AppError::Session(err.to_string()))?;

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|err| AppError::Session(err.to_string()))?;

        let killer = child.clone_killer();
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|err| AppError::Session(err.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|err| AppError::Session(err.to_string()))?;

        {
            let mut inner = self.lock_inner();
            inner.sessions.insert(
                id.clone(),
                ManagedSession {
                    info: SessionInfo {
                        id: id.clone(),
                        label,
                        host: request.host.clone(),
                        route: request.route.clone(),
                        model: request.model.clone(),
                        status: SessionStatus::Running,
                        exit_code: None,
                    },
                    master: pair.master,
                    writer,
                    killer,
                    pre_attach: String::new(),
                    attached: false,
                },
            );
        }

        spawn_output_reader(self.inner.clone(), app.clone(), id.clone(), reader);
        spawn_exit_watcher(self.inner.clone(), app, id.clone(), child);

        Ok(id)
    }

    pub fn write(&self, id: &str, data: &str) -> Result<(), AppError> {
        let mut inner = self.lock_inner();
        let session = inner
            .sessions
            .get_mut(id)
            .ok_or_else(|| AppError::Session(format!("Unknown session: {id}")))?;

        session
            .writer
            .write_all(data.as_bytes())
            .and_then(|_| session.writer.flush())
            .map_err(|err| AppError::Session(err.to_string()))
    }

    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), AppError> {
        let mut inner = self.lock_inner();
        let session = inner
            .sessions
            .get_mut(id)
            .ok_or_else(|| AppError::Session(format!("Unknown session: {id}")))?;

        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| AppError::Session(err.to_string()))
    }

    pub fn kill(&self, id: &str) -> Result<(), AppError> {
        let mut inner = self.lock_inner();
        let mut session = inner
            .sessions
            .remove(id)
            .ok_or_else(|| AppError::Session(format!("Unknown session: {id}")))?;
        session
            .killer
            .kill()
            .map_err(|err| AppError::Session(err.to_string()))
    }

    /// Mark a session attached and return the output buffered before attach. From
    /// this point the reader emits live `session-output` events. Atomic under the
    /// lock so no chunk is lost or duplicated across the handoff.
    pub fn attach(&self, id: &str) -> Result<String, AppError> {
        let mut inner = self.lock_inner();
        let session = inner
            .sessions
            .get_mut(id)
            .ok_or_else(|| AppError::Session(format!("Unknown session: {id}")))?;
        session.attached = true;
        Ok(std::mem::take(&mut session.pre_attach))
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let inner = self.lock_inner();
        let mut sessions: Vec<_> = inner
            .sessions
            .values()
            .map(|session| session.info.clone())
            .collect();
        sessions.sort_by(|left, right| {
            left.label
                .cmp(&right.label)
                .then_with(|| left.id.cmp(&right.id))
        });
        sessions
    }

    pub fn kill_all(&self) {
        let mut inner = self.lock_inner();
        for session in inner.sessions.values_mut() {
            let _ = session.killer.kill();
        }
        inner.sessions.clear();
    }

    pub fn running_labels_for_host(&self, host: &AgentHost) -> Vec<String> {
        let inner = self.lock_inner();
        inner
            .sessions
            .values()
            .filter(|session| {
                session.info.host == *host && session.info.status == SessionStatus::Running
            })
            .map(|session| session.info.label.clone())
            .collect()
    }

    fn lock_inner(&self) -> MutexGuard<'_, SessionManagerInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.kill_all();
    }
}

pub fn list_ollama_models() -> Result<Vec<OllamaModel>, AppError> {
    fetch_ollama_models_from_api().or_else(|_| list_ollama_models_from_cli())
}

#[derive(Debug, PartialEq, Eq)]
struct LaunchSpec {
    program: String,
    args: Vec<String>,
}

pub fn preflight_session(cwd: &Path, request: &SessionStartRequest) -> Result<(), AppError> {
    validate_route(request)?;

    if matches!(request.route, ModelRoute::Ollama) {
        require_command_on_path("ollama")?;
        let model = required_ollama_model(request)?;
        let available = fetch_ollama_models_from_api().map_err(|_| {
            AppError::Session(
                "Ollama is not reachable at http://localhost:11434. Start it outside LMBrain and retry."
                    .into(),
            )
        })?;
        if !available.iter().any(|entry| entry.name == model) {
            return Err(AppError::Session(format!(
                "Ollama model `{model}` is unavailable or does not advertise tool support"
            )));
        }
    }

    if matches!(request.host, AgentHost::Pi) {
        let pi = require_command_on_path("pi")?;
        let output = Command::new(&pi)
            .arg("list")
            .arg("--approve")
            .current_dir(cwd)
            .env("PI_OFFLINE", "1")
            .env("PI_SKIP_VERSION_CHECK", "1")
            .env("PI_TELEMETRY", "0")
            .output()
            .map_err(|err| {
                AppError::Session(format!(
                    "Unable to inspect Pi packages with `{}`: {err}",
                    pi.to_string_lossy()
                ))
            })?;
        let package_list = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if !output.status.success() || !has_pinned_pi_mcp_extension(&package_list) {
            return Err(AppError::Session(format!(
                "Pi MCP integration is not ready. Reopen the workspace to retry automatic preparation, or run `pi install {PI_MCP_EXTENSION_SOURCE} -l --approve` in this workspace, then retry."
            )));
        }
    }

    if matches!(request.host, AgentHost::Opencode) {
        resolve_opencode_command().ok_or_else(|| {
            AppError::Session(
                "Required executable `opencode` was not found on PATH. Install it outside LMBrain and retry."
                    .into(),
            )
        })?;
    }

    Ok(())
}

fn build_command(request: &SessionStartRequest, cwd: &Path) -> Result<CommandBuilder, AppError> {
    let spec = launch_spec(request, cwd)?;
    let mut builder = CommandBuilder::new(spec.program);
    for arg in spec.args {
        builder.arg(arg);
    }
    if matches!(
        (&request.host, &request.route),
        (AgentHost::Claude, ModelRoute::Native)
    ) {
        configure_lmbrain_mcp_environment(&mut builder, cwd);
    }
    if matches!(
        (&request.host, &request.route),
        (AgentHost::Pi, ModelRoute::Ollama)
    ) {
        builder.env("PI_OFFLINE", "1");
        builder.env("PI_SKIP_VERSION_CHECK", "1");
        builder.env("PI_TELEMETRY", "0");
    }
    if matches!(
        (&request.host, &request.route),
        (AgentHost::Opencode, ModelRoute::Ollama)
    ) {
        // LMBrain owns wheel/selection behavior in its embedded xterm. OpenCode's
        // mouse capture otherwise makes scrolling dependent on nested terminal
        // mouse protocols, which is unreliable in packaged Windows WebView2.
        builder.env("OPENCODE_DISABLE_MOUSE", "true");
        builder.env(
            "OPENCODE_CONFIG_CONTENT",
            opencode_ollama_config(required_ollama_model(request)?)?,
        );
    }
    builder.cwd(cwd);
    Ok(builder)
}

fn launch_spec(request: &SessionStartRequest, cwd: &Path) -> Result<LaunchSpec, AppError> {
    validate_route(request)?;
    let spec = match (&request.host, &request.route) {
        (AgentHost::Claude, ModelRoute::Native) => LaunchSpec {
            program: "claude".into(),
            args: Vec::new(),
        },
        (AgentHost::Claude, ModelRoute::Ollama) => LaunchSpec {
            program: "ollama".into(),
            args: vec![
                "launch".into(),
                "claude".into(),
                "--model".into(),
                required_ollama_model(request)?.to_string(),
            ],
        },
        (AgentHost::Pi, ModelRoute::Ollama) => LaunchSpec {
            program: "ollama".into(),
            args: vec![
                "launch".into(),
                "pi".into(),
                "--model".into(),
                required_ollama_model(request)?.to_string(),
            ],
        },
        (AgentHost::Opencode, ModelRoute::Ollama) => LaunchSpec {
            program: resolve_opencode_command()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "opencode".into()),
            args: vec![
                cwd.to_string_lossy().into_owned(),
                "--model".into(),
                format!("ollama/{}", required_ollama_model(request)?),
            ],
        },
        (AgentHost::Codex, ModelRoute::Native) => LaunchSpec {
            program: resolve_codex_command(request.codex_bin.as_deref()),
            args: vec!["--no-alt-screen".into()],
        },
        _ => unreachable!("validated session route"),
    };
    Ok(spec)
}

fn opencode_ollama_config(model: &str) -> Result<String, AppError> {
    serde_json::to_string(&json!({
        "provider": {
            "ollama": {
                "npm": "@ai-sdk/openai-compatible",
                "name": "Ollama",
                "options": { "baseURL": "http://localhost:11434/v1" },
                "models": { (model): { "name": model } }
            }
        }
    }))
    .map_err(AppError::from)
}

fn required_ollama_model(request: &SessionStartRequest) -> Result<&str, AppError> {
    request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Session("An Ollama model is required".into()))
}

fn resolve_opencode_command() -> Option<PathBuf> {
    let resolved = command_on_path("opencode")?;
    if cfg!(windows)
        && resolved
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "cmd" | "bat"))
    {
        let package_root = resolved
            .parent()?
            .join("node_modules")
            .join("opencode-ai");
        let platform_package = match std::env::consts::ARCH {
            "aarch64" => "opencode-windows-arm64",
            _ => "opencode-windows-x64",
        };
        for package in [platform_package, "opencode-windows-x64-baseline"] {
            let native = package_root
                .join("node_modules")
                .join(package)
                .join("bin")
                .join("opencode.exe");
            if native.is_file() {
                return Some(native);
            }
        }
    }
    Some(resolved)
}

fn require_command_on_path(command: &str) -> Result<PathBuf, AppError> {
    command_on_path(command).ok_or_else(|| {
        AppError::Session(format!(
            "Required executable `{command}` was not found on PATH. Install it outside LMBrain and retry."
        ))
    })
}

pub(crate) fn command_on_path(command: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) {
        vec![
            format!("{command}.exe"),
            format!("{command}.cmd"),
            format!("{command}.bat"),
            command.to_string(),
        ]
    } else {
        vec![command.to_string()]
    };
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

fn validate_route(request: &SessionStartRequest) -> Result<(), AppError> {
    let valid = matches!(
        (&request.host, &request.route),
        (AgentHost::Claude, ModelRoute::Native)
            | (AgentHost::Claude, ModelRoute::Ollama)
            | (AgentHost::Codex, ModelRoute::Native)
            | (AgentHost::Pi, ModelRoute::Ollama)
            | (AgentHost::Opencode, ModelRoute::Ollama)
    );
    if valid {
        Ok(())
    } else {
        Err(AppError::Session(format!(
            "Unsupported session route: {:?} via {:?}",
            request.host, request.route
        )))
    }
}

fn configure_lmbrain_mcp_environment(command: &mut CommandBuilder, cwd: &Path) {
    let mcp_command = resolve_mcp_command_for_root(cwd);
    let mcp_path = Path::new(&mcp_command);
    if !mcp_path.is_file() {
        return;
    }

    command.env("LMBRAIN_MCP_BIN", &mcp_command);
    if let Some(parent) = mcp_path.parent().and_then(prepend_to_path) {
        command.env("PATH", parent);
    }
}

fn prepend_to_path(dir: &Path) -> Option<OsString> {
    let mut paths = vec![dir.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).ok()
}

fn default_label(request: &SessionStartRequest) -> String {
    request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| match (&request.host, &request.route) {
            (AgentHost::Claude, ModelRoute::Native) => "Claude".to_string(),
            (AgentHost::Claude, ModelRoute::Ollama) => request
                .model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|model| format!("Claude via {model}"))
                .unwrap_or_else(|| "Claude via Ollama".to_string()),
            (AgentHost::Codex, ModelRoute::Native) => "Codex".to_string(),
            (AgentHost::Pi, ModelRoute::Ollama) => request
                .model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|model| format!("Pi via {model}"))
                .unwrap_or_else(|| "Pi via Ollama".to_string()),
            (AgentHost::Opencode, ModelRoute::Ollama) => request
                .model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|model| format!("OpenCode via {model}"))
                .unwrap_or_else(|| "OpenCode via Ollama".to_string()),
            _ => "Agent session".to_string(),
        })
}

pub fn resolve_codex_command(configured: Option<&str>) -> String {
    if let Ok(value) = std::env::var("LMBRAIN_CODEX_BIN") {
        if !value.trim().is_empty() {
            return value;
        }
    }

    if let Some(value) = configured.map(str::trim).filter(|value| !value.is_empty()) {
        return value.to_string();
    }

    if let Some(installed) = newest_desktop_codex_command() {
        return installed.to_string_lossy().into_owned();
    }

    codex_on_path().unwrap_or_else(|| "codex".to_string())
}

fn newest_desktop_codex_command() -> Option<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    newest_desktop_codex_command_in(Path::new(&local_app_data))
}

fn newest_desktop_codex_command_in(local_app_data: &Path) -> Option<PathBuf> {
    let bin_dir = local_app_data.join("OpenAI").join("Codex").join("bin");
    let entries = std::fs::read_dir(bin_dir).ok()?;
    entries
        .flatten()
        .filter_map(|entry| {
            let candidate = entry
                .path()
                .join(if cfg!(windows) { "codex.exe" } else { "codex" });
            let metadata = candidate.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let modified = metadata.modified().ok()?;
            Some((modified, candidate))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn codex_on_path() -> Option<String> {
    let path = std::env::var_os("PATH")?;
    let candidates: &[&str] = if cfg!(windows) {
        &["codex.exe", "codex"]
    } else {
        &["codex"]
    };

    std::env::split_paths(&path)
        .flat_map(|dir| candidates.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
        .map(|candidate| candidate.to_string_lossy().into_owned())
}

fn spawn_output_reader(
    sessions: Arc<Mutex<SessionManagerInner>>,
    app: AppHandle,
    id: String,
    mut reader: Box<dyn Read + Send>,
) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        // Carries any trailing bytes of a multi-byte UTF-8 sequence that was split
        // across reads, so wide/box-drawing TUI glyphs are not corrupted at chunk
        // boundaries.
        let mut pending: Vec<u8> = Vec::new();
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(len) => {
                    pending.extend_from_slice(&buffer[..len]);
                    let valid_len = match std::str::from_utf8(&pending) {
                        Ok(_) => pending.len(),
                        Err(err) if err.valid_up_to() > 0 => err.valid_up_to(),
                        // No valid prefix: if we already hold >= 4 bytes the leading
                        // byte cannot be the start of a longer incomplete sequence, so
                        // flush lossily rather than stalling; otherwise wait for more.
                        Err(_) if pending.len() >= 4 => pending.len(),
                        Err(_) => 0,
                    };
                    if valid_len == 0 {
                        continue;
                    }

                    let data = String::from_utf8_lossy(&pending[..valid_len]).to_string();
                    pending.drain(..valid_len);

                    // Under the lock: if the terminal has attached, emit live;
                    // otherwise buffer until attach replays it. If the session is
                    // gone, stop reading.
                    let emit = {
                        let mut inner = sessions
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        match inner.sessions.get_mut(&id) {
                            Some(session) if session.attached => true,
                            Some(session) => {
                                session.pre_attach.push_str(&data);
                                false
                            }
                            None => break,
                        }
                    };
                    if emit {
                        let _ = app.emit(
                            SESSION_OUTPUT_EVENT,
                            SessionOutputPayload {
                                id: id.clone(),
                                data,
                            },
                        );
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn spawn_exit_watcher(
    sessions: Arc<Mutex<SessionManagerInner>>,
    app: AppHandle,
    id: String,
    mut child: Box<dyn Child + Send + Sync>,
) {
    thread::spawn(move || {
        let code = child
            .wait()
            .ok()
            .and_then(|status| i32::try_from(status.exit_code()).ok());

        {
            let mut inner = sessions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(session) = inner.sessions.get_mut(&id) {
                session.info.status = SessionStatus::Exited;
                session.info.exit_code = code;
            }
        }

        let _ = app.emit(
            SESSION_EXIT_EVENT,
            SessionExitPayload {
                id: id.clone(),
                code,
            },
        );
    });
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaApiModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaApiModel {
    name: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    remote_host: Option<String>,
}

fn fetch_ollama_models_from_api() -> Result<Vec<OllamaModel>, AppError> {
    let response = ureq::get("http://localhost:11434/api/tags")
        .call()
        .map_err(|err| AppError::Session(err.to_string()))?;
    let body = response
        .into_string()
        .map_err(|err| AppError::Session(err.to_string()))?;
    let parsed: OllamaTagsResponse =
        serde_json::from_str(&body).map_err(|err| AppError::Session(err.to_string()))?;

    let mut models: Vec<_> = parsed
        .models
        .into_iter()
        .filter_map(|model| {
            if !model
                .capabilities
                .iter()
                .any(|capability| capability == "tools")
            {
                return None;
            }

            Some(OllamaModel {
                cloud: is_cloud_model(&model.name, model.remote_host.as_deref()),
                name: model.name,
                capabilities: model.capabilities,
            })
        })
        .collect();

    models.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(models)
}

fn list_ollama_models_from_cli() -> Result<Vec<OllamaModel>, AppError> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .map_err(|err| AppError::Session(err.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::Session(if stderr.is_empty() {
            "Failed to run `ollama list`".into()
        } else {
            stderr
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut models = parse_ollama_list_output(&stdout)
        .into_iter()
        .map(|name| OllamaModel {
            cloud: is_cloud_model(&name, None),
            name,
            capabilities: Vec::new(),
        })
        .collect::<Vec<_>>();
    models.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(models)
}

fn parse_ollama_list_output(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            trimmed.split_whitespace().next().map(ToOwned::to_owned)
        })
        .collect()
}

fn is_cloud_model(name: &str, host: Option<&str>) -> bool {
    name.ends_with(":cloud")
        || host
            .map(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                !normalized.is_empty()
                    && !normalized.contains("localhost")
                    && !normalized.contains("127.0.0.1")
            })
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        default_label, is_cloud_model, launch_spec, newest_desktop_codex_command_in,
        opencode_ollama_config, parse_ollama_list_output, resolve_codex_command,
        resolve_opencode_command, validate_route, LaunchSpec,
    };
    use crate::models::session::{AgentHost, ModelRoute, SessionStartRequest};

    #[test]
    fn parses_ollama_list_rows() {
        let output = "NAME                ID              SIZE      MODIFIED\nglm-5.1:cloud       abcdef          4.2 GB    2 hours ago\nqwen3:32b           987654          19 GB     4 days ago\n";
        let parsed = parse_ollama_list_output(output);
        assert_eq!(
            parsed,
            vec!["glm-5.1:cloud".to_string(), "qwen3:32b".to_string()]
        );
    }

    #[test]
    fn derives_default_labels_from_mode() {
        let native = SessionStartRequest {
            host: AgentHost::Claude,
            route: ModelRoute::Native,
            model: None,
            label: None,
            codex_bin: None,
        };
        let ollama = SessionStartRequest {
            host: AgentHost::Claude,
            route: ModelRoute::Ollama,
            model: Some("glm-5.1:cloud".into()),
            label: None,
            codex_bin: None,
        };
        let codex = SessionStartRequest {
            host: AgentHost::Codex,
            route: ModelRoute::Native,
            model: None,
            label: None,
            codex_bin: None,
        };
        let pi = SessionStartRequest {
            host: AgentHost::Pi,
            route: ModelRoute::Ollama,
            model: Some("qwen3.5:cloud".into()),
            label: None,
            codex_bin: None,
        };
        let opencode = SessionStartRequest {
            host: AgentHost::Opencode,
            route: ModelRoute::Ollama,
            model: Some("qwen3-coder:cloud".into()),
            label: None,
            codex_bin: None,
        };

        assert_eq!(default_label(&native), "Claude");
        assert_eq!(default_label(&ollama), "Claude via glm-5.1:cloud");
        assert_eq!(default_label(&codex), "Codex");
        assert_eq!(default_label(&pi), "Pi via qwen3.5:cloud");
        assert_eq!(default_label(&opencode), "OpenCode via qwen3-coder:cloud");
    }

    #[test]
    fn builds_pi_ollama_launch_spec_with_discrete_arguments() {
        let request = SessionStartRequest {
            host: AgentHost::Pi,
            route: ModelRoute::Ollama,
            model: Some(" qwen3.5:cloud ".into()),
            label: None,
            codex_bin: None,
        };
        assert_eq!(
            launch_spec(&request, Path::new("/workspace")).unwrap(),
            LaunchSpec {
                program: "ollama".into(),
                args: vec![
                    "launch".into(),
                    "pi".into(),
                    "--model".into(),
                    "qwen3.5:cloud".into(),
                ],
            }
        );
    }

    #[test]
    fn builds_opencode_ollama_launch_spec_with_discrete_arguments() {
        let request = SessionStartRequest {
            host: AgentHost::Opencode,
            route: ModelRoute::Ollama,
            model: Some(" qwen3-coder:cloud ".into()),
            label: None,
            codex_bin: None,
        };
        assert_eq!(
            launch_spec(&request, Path::new("/workspace")).unwrap(),
            LaunchSpec {
                program: resolve_opencode_command()
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "opencode".into()),
                args: vec![
                    "/workspace".into(),
                    "--model".into(),
                    "ollama/qwen3-coder:cloud".into(),
                ],
            }
        );
    }

    #[test]
    fn forwards_windows_workspace_as_direct_opencode_project_argument() {
        let request = SessionStartRequest {
            host: AgentHost::Opencode,
            route: ModelRoute::Ollama,
            model: Some("qwen3-coder:cloud".into()),
            label: None,
            codex_bin: None,
        };
        let workspace = r"C:\Work Space\Prøject";
        let spec = launch_spec(&request, Path::new(workspace)).unwrap();
        assert_eq!(spec.args[0], workspace);
        assert_eq!(spec.args[1], "--model");
        assert_eq!(spec.args[2], "ollama/qwen3-coder:cloud");
        assert_eq!(spec.args.len(), 3);
    }

    #[test]
    fn builds_scoped_inline_ollama_provider_for_opencode() {
        let config = opencode_ollama_config("deepseek-v4-flash:cloud").unwrap();
        let value: serde_json::Value = serde_json::from_str(&config).unwrap();
        assert_eq!(
            value["provider"]["ollama"]["options"]["baseURL"],
            "http://localhost:11434/v1"
        );
        assert_eq!(
            value["provider"]["ollama"]["models"]["deepseek-v4-flash:cloud"]["name"],
            "deepseek-v4-flash:cloud"
        );
    }

    #[test]
    fn resolves_native_opencode_binary_behind_windows_npm_shim() {
        if !cfg!(windows) {
            return;
        }
        let resolved = resolve_opencode_command().expect("OpenCode is not installed");
        assert_eq!(
            resolved.extension().and_then(|extension| extension.to_str()),
            Some("exe")
        );
        assert!(resolved.is_file());
    }


    #[test]
    fn launches_codex_inline_to_preserve_xterm_scrollback() {
        let request = SessionStartRequest {
            host: AgentHost::Codex,
            route: ModelRoute::Native,
            model: None,
            label: None,
            codex_bin: Some("codex-custom".into()),
        };
        assert_eq!(
            launch_spec(&request, Path::new("/workspace")).unwrap(),
            LaunchSpec {
                program: "codex-custom".into(),
                args: vec!["--no-alt-screen".into()],
            }
        );
    }

    #[test]
    fn rejects_unsupported_host_route_combinations_and_missing_models() {
        let pi_native = SessionStartRequest {
            host: AgentHost::Pi,
            route: ModelRoute::Native,
            model: None,
            label: None,
            codex_bin: None,
        };
        let codex_ollama = SessionStartRequest {
            host: AgentHost::Codex,
            route: ModelRoute::Ollama,
            model: Some("qwen".into()),
            label: None,
            codex_bin: None,
        };
        let pi_without_model = SessionStartRequest {
            host: AgentHost::Pi,
            route: ModelRoute::Ollama,
            model: Some("  ".into()),
            label: None,
            codex_bin: None,
        };
        let opencode_native = SessionStartRequest {
            host: AgentHost::Opencode,
            route: ModelRoute::Native,
            model: None,
            label: None,
            codex_bin: None,
        };

        assert!(validate_route(&pi_native).is_err());
        assert!(validate_route(&codex_ollama).is_err());
        assert!(launch_spec(&pi_without_model, Path::new("/workspace")).is_err());
        assert!(validate_route(&opencode_native).is_err());
    }

    #[test]
    fn detects_cloud_models() {
        assert!(is_cloud_model("glm-5.1:cloud", None));
        assert!(is_cloud_model("qwen3:32b", Some("api.ollama.ai")));
        assert!(!is_cloud_model("qwen3:32b", Some("localhost")));
    }

    #[test]
    fn detects_newest_desktop_codex_binary() {
        let dir = tempfile::tempdir().unwrap();
        let old_dir = dir.path().join("OpenAI/Codex/bin/old");
        let new_dir = dir.path().join("OpenAI/Codex/bin/new");
        std::fs::create_dir_all(&old_dir).unwrap();
        std::fs::create_dir_all(&new_dir).unwrap();
        let binary = if cfg!(windows) { "codex.exe" } else { "codex" };
        std::fs::write(old_dir.join(binary), "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(new_dir.join(binary), "").unwrap();

        let detected = newest_desktop_codex_command_in(dir.path()).unwrap();
        assert_eq!(detected, new_dir.join(binary));
    }

    #[test]
    fn codex_resolver_honors_configured_path() {
        let configured = resolve_codex_command(Some(r"E:\Tools\codex.exe"));
        assert_eq!(configured, r"E:\Tools\codex.exe");
    }
}
