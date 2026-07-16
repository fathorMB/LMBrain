use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document},
    harness_manifest::workspace_identity,
};

pub const VERIFICATION_MANIFEST_PATH: &str = ".lmbrain/verification.toml";
const SCHEMA_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_GATES: usize = 128;
const MAX_TIMEOUT_SECONDS: u64 = 3600;
const DEFAULT_TIMEOUT_SECONDS: u64 = 900;
const DEFAULT_OUTPUT_BYTES: usize = 128 * 1024;
const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub gates: Vec<VerificationGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationGate {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_cwd")]
    pub cwd: String,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub output_limit_bytes: Option<usize>,
    #[serde(default)]
    pub expected_exit_code: Option<i32>,
    #[serde(default)]
    pub result_matcher: Option<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

fn default_cwd() -> String {
    ".".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationApproval {
    pub workspace_fingerprint: String,
    pub manifest_digest: String,
    pub approved_at: String,
}

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
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationRunReport {
    pub spec_id: String,
    pub manifest_digest: String,
    pub workspace_fingerprint: String,
    pub transcript_hash: String,
    pub all_expectations_met: bool,
    pub results: Vec<VerificationGateResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranscriptState {
    Missing,
    Empty,
    HandAuthored,
    GeneratedFresh,
    GeneratedStale,
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("verification manifest does not exist: {0}")]
    MissingManifest(String),
    #[error("unsafe verification path: {0}")]
    UnsafePath(String),
    #[error("invalid verification manifest: {0}")]
    InvalidManifest(String),
    #[error("verification manifest is not approved for this workspace and digest")]
    ApprovalRequired,
    #[error("spec has no verification_gates references")]
    NoRequiredGates,
    #[error("unknown verification gate '{0}'")]
    UnknownGate(String),
    #[error("cannot read or write verification data: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot parse artifact: {0}")]
    Artifact(String),
    #[error("cannot launch verification gate '{gate}': {source}")]
    Launch {
        gate: String,
        #[source]
        source: std::io::Error,
    },
}

pub fn load_verification_manifest(root: &Path) -> Result<VerificationManifest, VerificationError> {
    let root = root.canonicalize()?;
    let path = root.join(VERIFICATION_MANIFEST_PATH);
    if !path.exists() {
        return Err(VerificationError::MissingManifest(
            path.display().to_string(),
        ));
    }
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_MANIFEST_BYTES
    {
        return Err(VerificationError::UnsafePath(path.display().to_string()));
    }
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(&root) {
        return Err(VerificationError::UnsafePath(path.display().to_string()));
    }
    parse_manifest(&fs::read_to_string(canonical)?)
}

pub fn parse_manifest(source: &str) -> Result<VerificationManifest, VerificationError> {
    if source.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(VerificationError::InvalidManifest(
            "manifest is too large".into(),
        ));
    }
    let manifest: VerificationManifest = toml::from_str(source)
        .map_err(|error| VerificationError::InvalidManifest(error.to_string()))?;
    let issues = validate_verification_manifest(&manifest);
    if issues.is_empty() {
        Ok(manifest)
    } else {
        Err(VerificationError::InvalidManifest(issues.join("; ")))
    }
}

pub fn validate_verification_manifest(manifest: &VerificationManifest) -> Vec<String> {
    let mut issues = Vec::new();
    if manifest.schema_version != SCHEMA_VERSION {
        issues.push(format!("schema_version must be {SCHEMA_VERSION}"));
    }
    if manifest.gates.is_empty() || manifest.gates.len() > MAX_GATES {
        issues.push(format!(
            "gates must contain between 1 and {MAX_GATES} entries"
        ));
    }
    let mut ids = BTreeSet::new();
    for gate in &manifest.gates {
        if !valid_id(&gate.id) {
            issues.push(format!("gate id '{}' is not portable", gate.id));
        }
        if !ids.insert(gate.id.clone()) {
            issues.push(format!("duplicate gate id '{}'", gate.id));
        }
        if gate.program.trim().is_empty()
            || Path::new(&gate.program).is_absolute()
            || gate.program.contains(['/', '\\'])
        {
            issues.push(format!(
                "gate '{}' program must be a PATH-resolved executable name",
                gate.id
            ));
        }
        if unsafe_relative(&gate.cwd) {
            issues.push(format!(
                "gate '{}' cwd must stay inside the workspace",
                gate.id
            ));
        }
        let timeout = gate.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS);
        if timeout == 0 || timeout > MAX_TIMEOUT_SECONDS {
            issues.push(format!(
                "gate '{}' timeout is outside 1..={MAX_TIMEOUT_SECONDS}",
                gate.id
            ));
        }
        let output = gate.output_limit_bytes.unwrap_or(DEFAULT_OUTPUT_BYTES);
        if output == 0 || output > MAX_OUTPUT_BYTES {
            issues.push(format!(
                "gate '{}' output limit is outside 1..={MAX_OUTPUT_BYTES}",
                gate.id
            ));
        }
        if let Some(matcher) = &gate.result_matcher {
            if Regex::new(matcher).is_err() {
                issues.push(format!("gate '{}' has an invalid result_matcher", gate.id));
            }
        }
        for (key, value) in &gate.environment {
            if !valid_env_key(key) || secret_like(key) || value.contains('\0') {
                issues.push(format!(
                    "gate '{}' has unsafe environment entry '{}'",
                    gate.id, key
                ));
            }
        }
    }
    issues
}

pub fn canonical_verification_manifest_digest(
    manifest: &VerificationManifest,
) -> Result<String, VerificationError> {
    let issues = validate_verification_manifest(manifest);
    if !issues.is_empty() {
        return Err(VerificationError::InvalidManifest(issues.join("; ")));
    }
    let bytes = serde_json::to_vec(manifest)
        .map_err(|error| VerificationError::InvalidManifest(error.to_string()))?;
    Ok(hex_digest(&bytes))
}

pub fn approve_verification_manifest(
    root: &Path,
    approval_store: &Path,
) -> Result<VerificationApproval, VerificationError> {
    let manifest = load_verification_manifest(root)?;
    let approval = VerificationApproval {
        workspace_fingerprint: workspace_identity(root)
            .map_err(|error| VerificationError::Artifact(error.to_string()))?
            .fingerprint,
        manifest_digest: canonical_verification_manifest_digest(&manifest)?,
        approved_at: Utc::now().to_rfc3339(),
    };
    if let Some(parent) = approval_store.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic_write(
        approval_store,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&approval)
                .map_err(|error| VerificationError::Artifact(error.to_string()))?
        ),
    )
    .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    Ok(approval)
}

pub fn execute_spec_verification(
    root: &Path,
    spec_path: &Path,
    approval_store: &Path,
) -> Result<VerificationRunReport, VerificationError> {
    let canonical_root = root.canonicalize()?;
    let canonical_spec = spec_path.canonicalize()?;
    if !canonical_spec.starts_with(&canonical_root) {
        return Err(VerificationError::UnsafePath(
            spec_path.display().to_string(),
        ));
    }
    let manifest = load_verification_manifest(&canonical_root)?;
    let manifest_digest = canonical_verification_manifest_digest(&manifest)?;
    let identity = workspace_identity(&canonical_root)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let approval_source = fs::read_to_string(approval_store).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            VerificationError::ApprovalRequired
        } else {
            VerificationError::Io(error)
        }
    })?;
    let approval: VerificationApproval = serde_json::from_str(&approval_source)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    if approval.workspace_fingerprint != identity.fingerprint
        || approval.manifest_digest != manifest_digest
    {
        return Err(VerificationError::ApprovalRequired);
    }

    let mut document = Document::parse(&fs::read_to_string(&canonical_spec)?)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let required = document.string_array("verification_gates");
    if required.is_empty() {
        return Err(VerificationError::NoRequiredGates);
    }
    let gate_contract_digest = gate_contract_digest(&required);
    let by_id: BTreeMap<_, _> = manifest.gates.iter().map(|gate| (&gate.id, gate)).collect();
    let mut results = Vec::new();
    for id in required {
        let gate = by_id
            .get(&id)
            .ok_or_else(|| VerificationError::UnknownGate(id.clone()))?;
        results.push(run_gate(&canonical_root, gate)?);
    }
    let source_fingerprint = workspace_content_fingerprint(&canonical_root)?;
    let transcript_without_hash = render_transcript(
        &manifest_digest,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        None,
    );
    let transcript_hash = hex_digest(transcript_without_hash.as_bytes());
    let transcript = render_transcript(
        &manifest_digest,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        Some(&transcript_hash),
    );
    document.body = replace_transcript(&document.body, &transcript)?;
    document.append_activity(&format!(
        "spec_verify generated transcript {} for workspace {}",
        transcript_hash, source_fingerprint
    ));
    atomic_write(&canonical_spec, &document.render())
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let all_expectations_met = results.iter().all(|result| result.expectation_met);
    Ok(VerificationRunReport {
        spec_id: document.value("id").unwrap_or_default(),
        manifest_digest,
        workspace_fingerprint: source_fingerprint,
        transcript_hash,
        all_expectations_met,
        results,
    })
}

pub fn transcript_state(root: &Path, body: &str) -> TranscriptState {
    let Some(implementation) = section_at_level(body, "Implementation evidence", 2) else {
        return TranscriptState::Missing;
    };
    let Some(section) = section_at_level(implementation, "Verification transcript", 3) else {
        return TranscriptState::Missing;
    };
    if !has_nonempty_fence(section) {
        return TranscriptState::Empty;
    }
    let Some(recorded) = metadata(section, "workspace-fingerprint") else {
        return TranscriptState::HandAuthored;
    };
    let Some(recorded_manifest) = metadata(section, "manifest-digest") else {
        return TranscriptState::GeneratedStale;
    };
    let Some(recorded_hash) = metadata(section, "transcript-hash") else {
        return TranscriptState::GeneratedStale;
    };
    let without_hash = section
        .lines()
        .filter(|line| !line.trim().starts_with("<!-- transcript-hash:"))
        .collect::<Vec<_>>()
        .join("\n");
    let canonical_transcript = format!("{}\n", without_hash.trim_matches('\n'));
    if hex_digest(canonical_transcript.as_bytes()) != recorded_hash {
        return TranscriptState::GeneratedStale;
    }
    let current_manifest = load_verification_manifest(root)
        .and_then(|manifest| canonical_verification_manifest_digest(&manifest));
    match (workspace_content_fingerprint(root), current_manifest) {
        (Ok(current), Ok(manifest)) if current == recorded && manifest == recorded_manifest => {
            TranscriptState::GeneratedFresh
        }
        _ => TranscriptState::GeneratedStale,
    }
}

pub fn transcript_state_for_document(root: &Path, document: &Document) -> TranscriptState {
    let state = transcript_state(root, &document.body);
    if state != TranscriptState::GeneratedFresh {
        return state;
    }
    let Some(implementation) = section_at_level(&document.body, "Implementation evidence", 2)
    else {
        return TranscriptState::GeneratedStale;
    };
    let Some(section) = section_at_level(implementation, "Verification transcript", 3) else {
        return TranscriptState::GeneratedStale;
    };
    let expected = gate_contract_digest(&document.string_array("verification_gates"));
    match metadata(section, "gate-contract-digest") {
        Some(recorded) if recorded == expected => TranscriptState::GeneratedFresh,
        _ => TranscriptState::GeneratedStale,
    }
}

pub fn workspace_content_fingerprint(root: &Path) -> Result<String, VerificationError> {
    let root = root.canonicalize()?;
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files)?;
    files.sort();
    let mut digest = Sha256::new();
    for path in files {
        let relative = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(fs::read(&path)?);
        digest.update([0]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn collect_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), VerificationError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let first = relative.components().next();
        if matches!(first, Some(Component::Normal(name)) if name == ".git" || name == "target" || name == "node_modules")
        {
            continue;
        }
        if relative.starts_with(".lmbrain/specs") || relative.starts_with(".lmbrain/reviews") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_files(root, &path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn run_gate(
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
    let mut command = Command::new(&gate.program);
    command
        .args(&gate.args)
        .current_dir(canonical_cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    for key in [
        "PATH",
        "PATHEXT",
        "SYSTEMROOT",
        "WINDIR",
        "HOME",
        "USERPROFILE",
        "TEMP",
        "TMP",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
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
        stdout,
        stderr,
    })
}

fn bounded_read(reader: impl Read, limit: usize) -> String {
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

fn terminate_process_tree(child: &mut std::process::Child) {
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

fn render_transcript(
    manifest_digest: &str,
    fingerprint: &str,
    gate_contract_digest: &str,
    results: &[VerificationGateResult],
    hash: Option<&str>,
) -> String {
    let mut text = format!(
        "<!-- generated-by: lmbrain-verify@{} -->\n<!-- manifest-digest: {manifest_digest} -->\n<!-- gate-contract-digest: {gate_contract_digest} -->\n<!-- workspace-fingerprint: {fingerprint} -->\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Some(hash) = hash {
        text.push_str(&format!("<!-- transcript-hash: {hash} -->\n"));
    }
    for result in results {
        text.push_str(&format!(
            "\n#### Gate `{}`\n\n```text\n$ {}\nstarted: {}\nfinished: {}\nduration_ms: {}\nexit_code: {}\ntimed_out: {}\nexpectation_met: {}\n--- stdout ---\n{}\n--- stderr ---\n{}\n```\n",
            result.id,
            result.command,
            result.started_at,
            result.finished_at,
            result.duration_ms,
            result.exit_code.map_or_else(|| "none".into(), |code| code.to_string()),
            result.timed_out,
            result.expectation_met,
            result.stdout,
            result.stderr
        ));
    }
    text
}

fn replace_transcript(body: &str, transcript: &str) -> Result<String, VerificationError> {
    let lines: Vec<&str> = body.lines().collect();
    let implementation = lines
        .iter()
        .position(|line| line.trim() == "## Implementation evidence")
        .ok_or_else(|| VerificationError::Artifact("missing ## Implementation evidence".into()))?;
    let existing = lines
        .iter()
        .enumerate()
        .skip(implementation + 1)
        .find(|(_, line)| line.trim() == "### Verification transcript")
        .map(|(index, _)| index);
    let mut output = Vec::new();
    if let Some(start) = existing {
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.trim_start().starts_with("### "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        output.extend_from_slice(&lines[..start]);
        output.push("### Verification transcript");
        output.push("");
        output.extend(transcript.lines());
        output.extend_from_slice(&lines[end..]);
    } else {
        let end = lines
            .iter()
            .enumerate()
            .skip(implementation + 1)
            .find(|(_, line)| line.trim_start().starts_with("## "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        output.extend_from_slice(&lines[..end]);
        output.push("");
        output.push("### Verification transcript");
        output.push("");
        output.extend(transcript.lines());
        output.extend_from_slice(&lines[end..]);
    }
    Ok(format!("{}\n", output.join("\n")))
}

fn section_at_level<'a>(body: &'a str, heading: &str, level: usize) -> Option<&'a str> {
    let marker = format!("{} {heading}", "#".repeat(level));
    let start = body.find(&marker)? + marker.len();
    let tail = &body[start..];
    let end = tail
        .match_indices('\n')
        .filter_map(|(offset, _)| {
            let line = &tail[offset + 1..];
            let count = line.bytes().take_while(|byte| *byte == b'#').count();
            (count > 0 && count <= level && line.as_bytes().get(count) == Some(&b' '))
                .then_some(offset)
        })
        .next()
        .unwrap_or(tail.len());
    Some(&tail[..end])
}

fn has_nonempty_fence(section: &str) -> bool {
    let mut in_fence = false;
    let mut content = false;
    for line in section.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence && content {
                return true;
            }
            in_fence = !in_fence;
            content = false;
        } else if in_fence && !line.trim().is_empty() {
            content = true;
        }
    }
    false
}

fn metadata(section: &str, key: &str) -> Option<String> {
    let prefix = format!("<!-- {key}:");
    section.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix("-->"))
            .map(|value| value.trim().to_string())
    })
}

fn render_command(gate: &VerificationGate) -> String {
    std::iter::once(gate.program.as_str())
        .chain(gate.args.iter().map(String::as_str))
        .map(shell_display)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_display(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-._/:=".contains(&byte))
    {
        value.into()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_env_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn secret_like(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    [
        "SECRET",
        "TOKEN",
        "PASSWORD",
        "CREDENTIAL",
        "PRIVATE_KEY",
        "API_KEY",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

fn unsafe_relative(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn gate_contract_digest(gates: &[String]) -> String {
    let encoded = serde_json::to_vec(gates).unwrap_or_default();
    hex_digest(&encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn manifest(program: &str) -> VerificationManifest {
        VerificationManifest {
            schema_version: 1,
            gates: vec![VerificationGate {
                id: "sample".into(),
                title: None,
                program: program.into(),
                args: Vec::new(),
                cwd: ".".into(),
                timeout_seconds: Some(5),
                output_limit_bytes: Some(4096),
                expected_exit_code: Some(0),
                result_matcher: None,
                environment: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn validates_strict_manifest_and_digest() {
        let valid = manifest("rustc");
        assert!(validate_verification_manifest(&valid).is_empty());
        assert_eq!(
            canonical_verification_manifest_digest(&valid)
                .unwrap()
                .len(),
            64
        );
        let mut invalid = valid.clone();
        invalid.gates[0].program = "../bad".into();
        assert!(!validate_verification_manifest(&invalid).is_empty());
    }

    #[test]
    fn transcript_presence_is_scoped_and_fenced() {
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Other\n### Verification transcript\n```\n```"
            ),
            TranscriptState::Missing
        );
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Implementation evidence\n### Verification transcript\n```\n```"
            ),
            TranscriptState::Empty
        );
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Implementation evidence\n### Verification transcript\n```text\n$ true\nok\n```"
            ),
            TranscriptState::HandAuthored
        );
    }

    #[test]
    fn approval_is_digest_and_workspace_bound() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&manifest("rustc")).unwrap(),
        )
        .unwrap();
        let approval_path = dir.path().join("local/approval.json");
        let approval = approve_verification_manifest(dir.path(), &approval_path).unwrap();
        assert_eq!(approval.manifest_digest.len(), 64);
        assert!(approval_path.exists());
    }

    #[test]
    fn workspace_fingerprint_ignores_managed_spec_evidence() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(dir.path().join("source.txt"), "one").unwrap();
        let first = workspace_content_fingerprint(dir.path()).unwrap();
        fs::write(
            dir.path().join(".lmbrain/specs/working/SPEC-001.md"),
            "evidence",
        )
        .unwrap();
        assert_eq!(first, workspace_content_fingerprint(dir.path()).unwrap());
        fs::write(dir.path().join("source.txt"), "two").unwrap();
        assert_ne!(first, workspace_content_fingerprint(dir.path()).unwrap());
    }

    #[test]
    fn executor_records_real_success_and_red_results_without_rewriting_them() {
        for (name, args, expected_green) in [
            ("green", vec!["--version".to_string()], true),
            (
                "red",
                vec!["--definitely-invalid-lmbrain-test-option".to_string()],
                false,
            ),
        ] {
            let dir = tempdir().unwrap();
            fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
            let mut configured = manifest("rustc");
            configured.gates[0].args = args;
            fs::write(
                dir.path().join(VERIFICATION_MANIFEST_PATH),
                toml::to_string(&configured).unwrap(),
            )
            .unwrap();
            let spec = dir
                .path()
                .join(format!(".lmbrain/specs/working/SPEC-{name}.md"));
            fs::write(&spec, format!(
                "---\nid: SPEC-{name}\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n"
            )).unwrap();
            let approval = dir.path().join("local/approval.json");
            approve_verification_manifest(dir.path(), &approval).unwrap();
            let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
            assert_eq!(report.all_expectations_met, expected_green);
            let source = fs::read_to_string(&spec).unwrap();
            assert!(source.contains("generated-by: lmbrain-verify"));
            assert!(source.contains(&format!("expectation_met: {expected_green}")));
            let document = Document::parse(&source).unwrap();
            assert_eq!(
                transcript_state(dir.path(), &document.body),
                TranscriptState::GeneratedFresh
            );
            assert_eq!(
                transcript_state_for_document(dir.path(), &document),
                TranscriptState::GeneratedFresh
            );
            let mut changed_contract = document.clone();
            changed_contract.set("verification_gates", "[other-gate]");
            assert_eq!(
                transcript_state_for_document(dir.path(), &changed_contract),
                TranscriptState::GeneratedStale
            );
            let tampered = document.body.replace(
                &format!("expectation_met: {expected_green}"),
                &format!("expectation_met: {}", !expected_green),
            );
            assert_eq!(transcript_state(dir.path(), &tampered), TranscriptState::GeneratedStale);
        }
    }

    #[test]
    fn missing_local_approval_fails_closed() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&manifest("rustc")).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-001.md");
        fs::write(&spec, "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        assert!(matches!(
            execute_spec_verification(dir.path(), &spec, &dir.path().join("missing.json")),
            Err(VerificationError::ApprovalRequired)
        ));
    }
}
