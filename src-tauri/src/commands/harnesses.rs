use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::commands::sessions::{command_on_path, resolve_codex_command};
use crate::models::harness::{HarnessProbeState, HarnessStatus, HarnessUpdateResult};
use crate::models::session::AgentHost;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MAX_CAPTURE_BYTES: usize = 64 * 1024;

#[derive(Clone, Default)]
pub struct HarnessManager {
    updating: Arc<Mutex<Option<AgentHost>>>,
}

impl HarnessManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_begin(&self, host: &AgentHost) -> Result<HarnessUpdateLease, String> {
        let mut updating = self
            .updating
            .lock()
            .map_err(|_| "Harness update state is unavailable".to_string())?;
        if let Some(active) = updating.as_ref() {
            return Err(format!(
                "Another harness update is already running for {}",
                harness_label(active)
            ));
        }
        *updating = Some(host.clone());
        Ok(HarnessUpdateLease {
            host: host.clone(),
            updating: self.updating.clone(),
        })
    }
}

pub struct HarnessUpdateLease {
    host: AgentHost,
    updating: Arc<Mutex<Option<AgentHost>>>,
}

impl Drop for HarnessUpdateLease {
    fn drop(&mut self) {
        if let Ok(mut updating) = self.updating.lock() {
            if updating.as_ref() == Some(&self.host) {
                *updating = None;
            }
        }
    }
}

pub fn probe_all(codex_bin: Option<&str>) -> Vec<HarnessStatus> {
    [
        AgentHost::Claude,
        AgentHost::Codex,
        AgentHost::Pi,
        AgentHost::Opencode,
    ]
    .into_iter()
    .map(|host| probe_harness(&host, codex_bin))
    .collect()
}

pub fn probe_harness(host: &AgentHost, codex_bin: Option<&str>) -> HarnessStatus {
    let probed_at = chrono::Local::now().to_rfc3339();
    let (install_url, install_command) = install_guidance(host);
    let Some(executable) = resolve_harness(host, codex_bin) else {
        return HarnessStatus {
            host: host.clone(),
            label: harness_label(host).into(),
            state: HarnessProbeState::Missing,
            executable: None,
            version: None,
            detail: Some("Executable not found".into()),
            probed_at,
            install_url: install_url.into(),
            install_command: install_command.into(),
        };
    };

    match run_process(&executable, &["--version"], PROBE_TIMEOUT) {
        Ok(output) if output.exit_code == Some(0) && !output.timed_out => {
            let raw = if output.stdout.trim().is_empty() {
                output.stderr.trim()
            } else {
                output.stdout.trim()
            };
            match parse_version(raw) {
                Some(version) => HarnessStatus {
                    host: host.clone(),
                    label: harness_label(host).into(),
                    state: HarnessProbeState::Installed,
                    executable: Some(executable.to_string_lossy().into_owned()),
                    version: Some(version),
                    detail: None,
                    probed_at,
                    install_url: install_url.into(),
                    install_command: install_command.into(),
                },
                None => error_status(
                    host,
                    executable,
                    probed_at,
                    install_url,
                    install_command,
                    format!(
                        "Version output could not be parsed: {}",
                        truncate_text(raw, 512)
                    ),
                ),
            }
        }
        Ok(output) => error_status(
            host,
            executable,
            probed_at,
            install_url,
            install_command,
            if output.timed_out {
                "Version probe timed out".into()
            } else {
                format!(
                    "Version probe exited with {:?}: {}",
                    output.exit_code,
                    truncate_text(&output.stderr, 512)
                )
            },
        ),
        Err(error) => error_status(
            host,
            executable,
            probed_at,
            install_url,
            install_command,
            error,
        ),
    }
}

pub fn update_harness(
    host: &AgentHost,
    codex_bin: Option<&str>,
) -> Result<HarnessUpdateResult, String> {
    let before = probe_harness(host, codex_bin);
    if before.state != HarnessProbeState::Installed {
        return Err(format!(
            "{} cannot be updated because its installation is not ready",
            before.label
        ));
    }
    let executable = PathBuf::from(
        before
            .executable
            .as_ref()
            .ok_or_else(|| "Installed harness has no executable path".to_string())?,
    );
    let args = update_args(host);
    let output = run_process(&executable, args, UPDATE_TIMEOUT)?;
    let after = probe_harness(host, codex_bin);
    let (success, already_current) =
        update_outcome(output.exit_code, output.timed_out, &before, &after);
    Ok(HarnessUpdateResult {
        host: host.clone(),
        success,
        already_current,
        before,
        after,
        exit_code: output.exit_code,
        timed_out: output.timed_out,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn update_outcome(
    exit_code: Option<i32>,
    timed_out: bool,
    before: &HarnessStatus,
    after: &HarnessStatus,
) -> (bool, bool) {
    let success = exit_code == Some(0) && !timed_out && after.state == HarnessProbeState::Installed;
    (success, success && before.version == after.version)
}

pub fn active_session_error(host: &AgentHost, running: &[String]) -> Option<String> {
    if running.is_empty() {
        return None;
    }
    Some(format!(
        "Close the running {} session(s) before updating: {}",
        harness_label(host),
        running.join(", ")
    ))
}

#[derive(Debug)]
struct ProcessOutput {
    exit_code: Option<i32>,
    timed_out: bool,
    stdout: String,
    stderr: String,
}

fn run_process(program: &Path, args: &[&str], timeout: Duration) -> Result<ProcessOutput, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(home) = user_home() {
        command.current_dir(home);
    }
    configure_process_group(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("Unable to start {}: {error}", program.display()))?;
    let stdout = child.stdout.take().map(spawn_bounded_reader);
    let stderr = child.stderr.take().map(spawn_bounded_reader);
    let started = Instant::now();
    let (exit_code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if started.elapsed() < timeout => thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                terminate_process_tree(&mut child);
                let status = child.wait().ok().and_then(|status| status.code());
                break (status, true);
            }
            Err(error) => return Err(format!("Unable to wait for updater: {error}")),
        }
    };
    Ok(ProcessOutput {
        exit_code,
        timed_out,
        stdout: join_reader(stdout),
        stderr: join_reader(stderr),
    })
}

fn spawn_bounded_reader<R: Read + Send + 'static>(mut reader: R) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    let remaining = MAX_CAPTURE_BYTES.saturating_sub(captured.len());
                    captured.extend_from_slice(&buffer[..read.min(remaining)]);
                }
            }
        }
        let _ = sender.send(captured);
    });
    receiver
}

fn join_reader(reader: Option<mpsc::Receiver<Vec<u8>>>) -> String {
    let bytes = reader
        .and_then(|receiver| receiver.recv_timeout(Duration::from_secs(2)).ok())
        .unwrap_or_default();
    let mut output = String::from_utf8_lossy(&bytes).into_owned();
    if bytes.len() == MAX_CAPTURE_BYTES {
        output.push_str("\n[output truncated by LMBrain]");
    }
    output
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(windows)]
fn terminate_process_tree(child: &mut std::process::Child) {
    let _ = Command::new("taskkill.exe")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut std::process::Child) {
    let pgid = child.id() as libc::pid_t;
    unsafe {
        libc::kill(-pgid, libc::SIGTERM);
    }
    thread::sleep(Duration::from_millis(250));
    if child.try_wait().ok().flatten().is_none() {
        unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_tree(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn resolve_harness(host: &AgentHost, codex_bin: Option<&str>) -> Option<PathBuf> {
    match host {
        AgentHost::Claude => command_on_path("claude"),
        AgentHost::Pi => command_on_path("pi"),
        AgentHost::Opencode => command_on_path("opencode"),
        AgentHost::Codex => {
            let resolved = resolve_codex_command(codex_bin);
            let path = PathBuf::from(&resolved);
            if path.is_file() {
                Some(path)
            } else if Path::new(&resolved).components().count() == 1 {
                command_on_path("codex")
            } else {
                None
            }
        }
    }
}

fn update_args(host: &AgentHost) -> &'static [&'static str] {
    match host {
        AgentHost::Claude | AgentHost::Codex => &["update"],
        AgentHost::Pi => &["update", "--self", "--no-approve"],
        AgentHost::Opencode => &["upgrade"],
    }
}

fn harness_label(host: &AgentHost) -> &'static str {
    match host {
        AgentHost::Claude => "Claude Code",
        AgentHost::Codex => "Codex",
        AgentHost::Pi => "Pi",
        AgentHost::Opencode => "OpenCode",
    }
}

fn install_guidance(host: &AgentHost) -> (&'static str, &'static str) {
    match host {
        AgentHost::Claude => (
            "https://docs.anthropic.com/en/docs/claude-code/getting-started",
            "npm install -g @anthropic-ai/claude-code",
        ),
        AgentHost::Codex => (
            "https://github.com/openai/codex#installing-and-running-codex-cli",
            "npm install -g @openai/codex",
        ),
        AgentHost::Pi => (
            "https://github.com/badlogic/pi-mono",
            "npm install -g @earendil-works/pi-coding-agent",
        ),
        AgentHost::Opencode => ("https://opencode.ai/docs/", "npm install -g opencode-ai"),
    }
}

fn error_status(
    host: &AgentHost,
    executable: PathBuf,
    probed_at: String,
    install_url: &str,
    install_command: &str,
    detail: String,
) -> HarnessStatus {
    HarnessStatus {
        host: host.clone(),
        label: harness_label(host).into(),
        state: HarnessProbeState::Error,
        executable: Some(executable.to_string_lossy().into_owned()),
        version: None,
        detail: Some(detail),
        probed_at,
        install_url: install_url.into(),
        install_command: install_command.into(),
    }
}

fn parse_version(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric()
                    && character != '.'
                    && character != '-'
                    && character != '+'
            })
        })
        .find(|token| {
            !token.is_empty()
                && token
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit())
                && token.contains('.')
        })
        .map(ToOwned::to_owned)
}

fn truncate_text(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{
        active_session_error, install_guidance, parse_version, run_process, truncate_text,
        update_args, update_outcome, HarnessManager, HarnessProbeState, HarnessStatus,
        MAX_CAPTURE_BYTES,
    };
    use crate::models::session::AgentHost;
    use std::time::Duration;

    #[test]
    fn parses_supported_harness_version_output() {
        assert_eq!(
            parse_version("2.1.206 (Claude Code)"),
            Some("2.1.206".into())
        );
        assert_eq!(parse_version("codex-cli 0.144.1"), Some("0.144.1".into()));
        assert_eq!(parse_version("0.79.10"), Some("0.79.10".into()));
        assert_eq!(parse_version("not a version"), None);
    }

    #[test]
    fn updater_arguments_are_fixed_and_host_specific() {
        assert_eq!(update_args(&AgentHost::Claude), &["update"]);
        assert_eq!(update_args(&AgentHost::Codex), &["update"]);
        assert_eq!(
            update_args(&AgentHost::Pi),
            &["update", "--self", "--no-approve"]
        );
        assert_eq!(update_args(&AgentHost::Opencode), &["upgrade"]);
    }

    #[test]
    fn update_manager_serializes_mutations_and_releases_lease() {
        let manager = HarnessManager::new();
        let lease = manager.try_begin(&AgentHost::Claude).unwrap();
        assert!(manager.try_begin(&AgentHost::Pi).is_err());
        drop(lease);
        assert!(manager.try_begin(&AgentHost::Pi).is_ok());
    }

    #[test]
    fn install_guidance_never_uses_elevation() {
        for host in [
            AgentHost::Claude,
            AgentHost::Codex,
            AgentHost::Pi,
            AgentHost::Opencode,
        ] {
            let (_, command) = install_guidance(&host);
            assert!(!command.contains("sudo"));
        }
    }

    #[test]
    fn diagnostic_text_is_bounded() {
        assert_eq!(truncate_text("abcdef", 3), "abc");
    }

    #[test]
    fn post_probe_is_authoritative_for_update_outcomes() {
        let before = installed_status("1.0.0");
        let updated = installed_status("1.1.0");
        let unchanged = installed_status("1.0.0");
        let mut failed_probe = installed_status("1.1.0");
        failed_probe.state = HarnessProbeState::Error;

        assert_eq!(
            update_outcome(Some(0), false, &before, &updated),
            (true, false)
        );
        assert_eq!(
            update_outcome(Some(0), false, &before, &unchanged),
            (true, true)
        );
        assert_eq!(
            update_outcome(Some(1), false, &before, &updated),
            (false, false)
        );
        assert_eq!(
            update_outcome(Some(0), true, &before, &updated),
            (false, false)
        );
        assert_eq!(
            update_outcome(Some(0), false, &before, &failed_probe),
            (false, false)
        );
    }

    #[test]
    fn active_session_gate_is_host_specific_and_actionable() {
        assert!(active_session_error(&AgentHost::Codex, &[]).is_none());
        let error =
            active_session_error(&AgentHost::Codex, &["Review work".into(), "Debug".into()])
                .unwrap();
        assert!(error.contains("Codex"));
        assert!(error.contains("Review work, Debug"));
    }

    #[test]
    fn process_runner_terminates_on_timeout() {
        let (program, args) = if cfg!(unix) {
            (std::path::PathBuf::from("sleep"), vec!["2".to_string()])
        } else {
            (
                std::env::current_exe().unwrap(),
                vec![
                    "--exact".to_string(),
                    "commands::harnesses::tests::long_running_child".to_string(),
                    "--ignored".to_string(),
                ],
            )
        };
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = run_process(&program, &args_ref, Duration::from_millis(100)).unwrap();
        assert!(output.timed_out);
    }

    #[test]
    fn process_runner_bounds_captured_output() {
        let (program, args) = if cfg!(unix) {
            (std::path::PathBuf::from("sh"), vec!["-c".to_string(), format!("yes x | head -c {}", MAX_CAPTURE_BYTES + 1024)])
        } else {
            (
                std::env::current_exe().unwrap(),
                vec![
                    "--exact".to_string(),
                    "commands::harnesses::tests::large_output_child".to_string(),
                    "--ignored".to_string(),
                    "--nocapture".to_string(),
                ],
            )
        };
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = run_process(&program, &args_ref, Duration::from_secs(5)).unwrap();
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.len() <= MAX_CAPTURE_BYTES + 40);
        assert!(output.stdout.contains("[output truncated by LMBrain]"));
    }

    #[test]
    #[ignore]
    fn long_running_child() {
        std::thread::sleep(Duration::from_secs(2));
    }

    #[test]
    #[ignore]
    fn large_output_child() {
        print!("{}", "x".repeat(MAX_CAPTURE_BYTES + 1024));
    }

    #[test]
    #[ignore = "manual read-only smoke test for operator-installed harnesses"]
    fn probes_operator_installed_harnesses() {
        let statuses = super::probe_all(None);
        assert_eq!(statuses.len(), 4);
        for status in statuses {
            eprintln!(
                "{}: {:?} {:?} {:?}",
                status.label, status.state, status.version, status.executable
            );
            assert_ne!(status.state, HarnessProbeState::Error);
        }
    }

    fn installed_status(version: &str) -> HarnessStatus {
        HarnessStatus {
            host: AgentHost::Claude,
            label: "Claude Code".into(),
            state: HarnessProbeState::Installed,
            executable: Some("claude".into()),
            version: Some(version.into()),
            detail: None,
            probed_at: "now".into(),
            install_url: "https://example.com".into(),
            install_command: "install".into(),
        }
    }
}
