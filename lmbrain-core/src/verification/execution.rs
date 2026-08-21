use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use regex::Regex;
use serde::Serialize;

use super::{
    manifest::{
        VerificationGate, DEFAULT_OUTPUT_BYTES, DEFAULT_TIMEOUT_SECONDS,
    },
    VerificationError,
};

#[derive(Debug, Clone, Serialize)]
pub struct VerificationGateResult {
    pub id: String,
    pub command: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub expectation_met: bool,
    /// Names only; inherited values are deliberately never retained or
    /// serialized. The list is deterministic for diagnostic comparison.
    pub removed_environment_variables: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

pub struct MinimalGateEnvironment {
    pub preserved: BTreeMap<OsString, OsString>,
    pub removed: Vec<String>,
}

pub fn run_gate(
    root: &Path,
    gate: &VerificationGate,
) -> Result<VerificationGateResult, VerificationError> {
    let cwd = root.join(&gate.cwd);
    let canonical_cwd = cwd.canonicalize()?;
    if !canonical_cwd.starts_with(root) {
        return Err(VerificationError::UnsafePath(cwd.display().to_string()));
    }
    let started_at = Utc::now().to_rfc3339();
    let started = Instant::now();
    let minimal_environment = minimal_gate_environment();
    let mut command = Command::new(&gate.program);
    command
        .args(&gate.args)
        .current_dir(canonical_cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    command.envs(&minimal_environment.preserved);
    command.envs(&gate.environment);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return Ok(VerificationGateResult {
                id: gate.id.clone(),
                command: render_command(gate),
                started_at,
                finished_at: Utc::now().to_rfc3339(),
                duration_ms: started.elapsed().as_millis(),
                exit_code: None,
                timed_out: false,
                expectation_met: false,
                removed_environment_variables: minimal_environment.removed,
                stdout: String::new(),
                stderr: format!("LMBrain could not launch the gate: {error}"),
            });
        }
    };
    let limit = gate.output_limit_bytes.unwrap_or(DEFAULT_OUTPUT_BYTES);
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let out_reader = thread::spawn(move || bounded_read(stdout, limit));
    let err_reader = thread::spawn(move || bounded_read(stderr, limit));
    let timeout = Duration::from_secs(gate.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS));
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (Some(status), false);
        }
        if started.elapsed() >= timeout {
            terminate_process_tree(&mut child);
            break (child.wait().ok(), true);
        }
        thread::sleep(Duration::from_millis(25));
    };
    let stdout = out_reader.join().unwrap_or_default();
    let stderr = err_reader.join().unwrap_or_default();
    let exit_code = status.and_then(|status| status.code());
    let expected = gate.expected_exit_code.unwrap_or(0);
    let matcher_ok = gate.result_matcher.as_ref().map_or(true, |pattern| {
        Regex::new(pattern)
            .map(|regex| regex.is_match(&stdout) || regex.is_match(&stderr))
            .unwrap_or(false)
    });
    Ok(VerificationGateResult {
        id: gate.id.clone(),
        command: render_command(gate),
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis(),
        exit_code,
        timed_out,
        expectation_met: !timed_out && exit_code == Some(expected) && matcher_ok,
        removed_environment_variables: minimal_environment.removed,
        stdout,
        stderr,
    })
}

pub fn minimal_gate_environment() -> MinimalGateEnvironment {
    minimal_gate_environment_from(std::env::vars_os(), cfg!(windows))
}

pub fn minimal_gate_environment_from(
    inherited: impl IntoIterator<Item = (OsString, OsString)>,
    windows: bool,
) -> MinimalGateEnvironment {
    let mut allowed = vec![
        "PATH",
        "PATHEXT",
        "SYSTEMROOT",
        "WINDIR",
        "HOME",
        "USERPROFILE",
        "TEMP",
        "TMP",
    ];
    if windows {
        // Rust's MSVC target consults this system root to discover installed
        // Visual Studio instances and their linker. It is machine-scoped, not
        // user/session-scoped, and is intentionally preserved only on Windows.
        allowed.push("ProgramData");
    }

    let mut preserved = BTreeMap::new();
    let mut removed = BTreeSet::new();
    for (key, value) in inherited {
        let name = key.to_string_lossy();
        if allowed
            .iter()
            .any(|candidate| name.eq_ignore_ascii_case(candidate))
        {
            preserved.insert(key, value);
        } else {
            removed.insert(if windows {
                name.to_ascii_uppercase()
            } else {
                name.into_owned()
            });
        }
    }
    MinimalGateEnvironment {
        preserved,
        removed: removed.into_iter().collect(),
    }
}

pub fn bounded_read(reader: impl Read, limit: usize) -> String {
    let mut bytes = Vec::new();
    let _ = reader.take(limit as u64 + 1).read_to_end(&mut bytes);
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    if truncated {
        text.push_str("\n...[output truncated by LMBrain]...");
    }
    text
}

pub fn terminate_process_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(unix)]
    unsafe {
        // The gate is spawned as its own process group, so a timeout also
        // terminates descendants instead of leaving background work behind.
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
}

pub fn render_command(gate: &VerificationGate) -> String {
    std::iter::once(gate.program.as_str())
        .chain(gate.args.iter().map(String::as_str))
        .map(shell_display)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn shell_display(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-._/:=".contains(&byte))
    {
        value.into()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}
