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
    mutation_lock::ArtifactMutationLock,
};

pub const VERIFICATION_MANIFEST_PATH: &str = ".lmbrain/verification.toml";
const SCHEMA_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_GATES: usize = 128;
const MAX_TIMEOUT_SECONDS: u64 = 3600;
const DEFAULT_TIMEOUT_SECONDS: u64 = 900;
const DEFAULT_OUTPUT_BYTES: usize = 128 * 1024;
const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const GENERATED_TRANSCRIPT_START: &str = "<!-- lmbrain-generated-verification:start -->";
const GENERATED_TRANSCRIPT_END: &str = "<!-- lmbrain-generated-verification:end -->";

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
    /// Fingerprint captured after the final gate; kept under the historical
    /// name so 2.9.1 freshness checks keep working.
    pub workspace_fingerprint: String,
    /// Fingerprint captured before the first gate ran.
    pub workspace_fingerprint_before: String,
    pub transcript_hash: String,
    pub all_expectations_met: bool,
    /// True when the workspace changed between the pre- and post-gate
    /// fingerprints; such evidence is never publishable as fresh.
    pub invalidated: bool,
    pub invalidation_reason: Option<String>,
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
    #[error("spec changed while verification was running: {0}")]
    ConcurrentModification(String),
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

    let document = Document::parse(&fs::read_to_string(&canonical_spec)?)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    let spec_id = document
        .value("id")
        .ok_or_else(|| VerificationError::Artifact("missing spec id".into()))?;
    let required = document.string_array("verification_gates");
    if required.is_empty() {
        return Err(VerificationError::NoRequiredGates);
    }
    let gate_contract_digest = gate_contract_digest(&required);
    let by_id: BTreeMap<_, _> = manifest.gates.iter().map(|gate| (&gate.id, gate)).collect();
    // The final artifact lock only protects the transcript write below; the
    // gate-execution interval itself is snapshot-checked by comparing a
    // fingerprint taken before the first gate with one taken after the last.
    // Full isolated-worktree/per-gate input scoping is deferred to 3.0.0.
    let pre_fingerprint = workspace_content_fingerprint(&canonical_root)?;
    let mut results = Vec::new();
    for id in required {
        let gate = by_id
            .get(&id)
            .ok_or_else(|| VerificationError::UnknownGate(id.clone()))?;
        results.push(run_gate(&canonical_root, gate)?);
    }
    let source_fingerprint = workspace_content_fingerprint(&canonical_root)?;
    let invalidation_reason = (pre_fingerprint != source_fingerprint).then(|| {
        "workspace content changed during gate execution; evidence is not snapshot-consistent"
            .to_string()
    });
    let transcript_without_hash = render_transcript(
        &manifest_digest,
        &pre_fingerprint,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        invalidation_reason.as_deref(),
        None,
    );
    let transcript_hash = hex_digest(transcript_without_hash.as_bytes());
    let transcript = render_transcript(
        &manifest_digest,
        &pre_fingerprint,
        &source_fingerprint,
        &gate_contract_digest,
        &results,
        invalidation_reason.as_deref(),
        Some(&transcript_hash),
    );
    write_verification_transcript(
        &canonical_root,
        &canonical_spec,
        &spec_id,
        &document.string_array("verification_gates"),
        &transcript,
        &transcript_hash,
        &source_fingerprint,
    )?;
    let all_expectations_met =
        invalidation_reason.is_none() && results.iter().all(|result| result.expectation_met);
    Ok(VerificationRunReport {
        spec_id,
        manifest_digest,
        workspace_fingerprint: source_fingerprint,
        workspace_fingerprint_before: pre_fingerprint,
        transcript_hash,
        all_expectations_met,
        invalidated: invalidation_reason.is_some(),
        invalidation_reason,
        results,
    })
}

fn write_verification_transcript(
    root: &Path,
    canonical_spec: &Path,
    spec_id: &str,
    required_gates: &[String],
    transcript: &str,
    transcript_hash: &str,
    source_fingerprint: &str,
) -> Result<(), VerificationError> {
    let _lock = ArtifactMutationLock::acquire(root, spec_id)?;
    if !canonical_spec.exists()
        || canonical_spec
            .canonicalize()
            .map(|path| path != canonical_spec)
            .unwrap_or(true)
    {
        return Err(VerificationError::ConcurrentModification(
            "the spec was moved, replaced, or deleted; verification evidence was not written"
                .into(),
        ));
    }

    let current_source = fs::read_to_string(canonical_spec)?;
    let mut current = Document::parse(&current_source)
        .map_err(|error| VerificationError::Artifact(error.to_string()))?;
    if current.value("id").as_deref() != Some(spec_id) {
        return Err(VerificationError::ConcurrentModification(
            "the artifact at the original path has a different id".into(),
        ));
    }
    if current.string_array("verification_gates") != required_gates {
        return Err(VerificationError::ConcurrentModification(
            "verification_gates changed; rerun verification against the new gate contract".into(),
        ));
    }

    current.body = replace_transcript(&current.body, transcript)?;
    current.append_activity(&format!(
        "spec_verify generated transcript {transcript_hash} for workspace {source_fingerprint}"
    ));
    if fs::read_to_string(canonical_spec)? != current_source {
        return Err(VerificationError::ConcurrentModification(
            "the spec changed again while verification evidence was being merged".into(),
        ));
    }
    atomic_write(canonical_spec, &current.render())
        .map_err(|error| VerificationError::Artifact(error.to_string()))
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
    let Some(generated) = generated_transcript(section) else {
        if section.contains("generated-by: lmbrain-verify")
            || section.contains(GENERATED_TRANSCRIPT_START)
            || section.contains(GENERATED_TRANSCRIPT_END)
        {
            return TranscriptState::GeneratedStale;
        }
        return TranscriptState::HandAuthored;
    };
    let Some(recorded) = metadata(generated, "workspace-fingerprint") else {
        return TranscriptState::HandAuthored;
    };
    // Evidence explicitly invalidated at generation time (the workspace
    // changed between the pre- and post-gate fingerprints) is never fresh,
    // even when the current workspace matches the recorded post fingerprint.
    if metadata(generated, "invalidated").is_some() {
        return TranscriptState::GeneratedStale;
    }
    if metadata(generated, "workspace-fingerprint-before")
        .is_some_and(|before| before != recorded)
    {
        return TranscriptState::GeneratedStale;
    }
    let Some(recorded_manifest) = metadata(generated, "manifest-digest") else {
        return TranscriptState::GeneratedStale;
    };
    let Some(recorded_hash) = metadata(generated, "transcript-hash") else {
        return TranscriptState::GeneratedStale;
    };
    if !transcript_hash_matches(generated, &recorded_hash) {
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
    let Some(generated) = generated_transcript(section) else {
        return TranscriptState::GeneratedStale;
    };
    let expected = gate_contract_digest(&document.string_array("verification_gates"));
    match metadata(generated, "gate-contract-digest") {
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
    pre_fingerprint: &str,
    fingerprint: &str,
    gate_contract_digest: &str,
    results: &[VerificationGateResult],
    invalidation_reason: Option<&str>,
    hash: Option<&str>,
) -> String {
    // `workspace-fingerprint` keeps carrying the post-gate value so 2.9.1
    // freshness checks stay valid; the pre-gate value is additive metadata.
    let mut text = format!(
        "<!-- generated-by: lmbrain-verify@{} -->\n<!-- manifest-digest: {manifest_digest} -->\n<!-- gate-contract-digest: {gate_contract_digest} -->\n<!-- workspace-fingerprint-before: {pre_fingerprint} -->\n<!-- workspace-fingerprint: {fingerprint} -->\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Some(reason) = invalidation_reason {
        text.push_str(&format!("<!-- invalidated: {reason} -->\n"));
    }
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
    let mut output: Vec<String> = Vec::new();
    if let Some(start) = existing {
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.trim_start().starts_with("### "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        let section = lines[start + 1..end].join("\n");
        let start_count = section.matches(GENERATED_TRANSCRIPT_START).count();
        let end_count = section.matches(GENERATED_TRANSCRIPT_END).count();
        if start_count != end_count || start_count > 1 {
            return Err(VerificationError::Artifact(
                "verification transcript has an incomplete or duplicate LMBrain-managed region"
                    .into(),
            ));
        }
        let managed = format!(
            "{GENERATED_TRANSCRIPT_START}\n{}\n{GENERATED_TRANSCRIPT_END}",
            transcript.trim_matches('\n')
        );
        let updated = if let Some((range_start, range_end)) = generated_transcript_range(&section) {
            format!(
                "{}{}{}",
                &section[..range_start],
                managed,
                &section[range_end..]
            )
        } else if section.trim().is_empty() {
            format!("\n{managed}\n")
        } else {
            format!("{}\n\n{managed}\n", section.trim_end_matches('\n'))
        };
        output.extend(lines[..=start].iter().map(|line| (*line).to_string()));
        output.extend(updated.lines().map(str::to_string));
        output.extend(lines[end..].iter().map(|line| (*line).to_string()));
    } else {
        let end = lines
            .iter()
            .enumerate()
            .skip(implementation + 1)
            .find(|(_, line)| line.trim_start().starts_with("## "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        output.extend(lines[..end].iter().map(|line| (*line).to_string()));
        output.push(String::new());
        output.push("### Verification transcript".into());
        output.push(String::new());
        output.push(GENERATED_TRANSCRIPT_START.into());
        output.extend(transcript.trim_matches('\n').lines().map(str::to_string));
        output.push(GENERATED_TRANSCRIPT_END.into());
        output.extend(lines[end..].iter().map(|line| (*line).to_string()));
    }
    Ok(format!("{}\n", output.join("\n")))
}

fn generated_transcript(section: &str) -> Option<&str> {
    if let Some(start) = section.find(GENERATED_TRANSCRIPT_START) {
        let content_start = start + GENERATED_TRANSCRIPT_START.len();
        let end = section[content_start..].find(GENERATED_TRANSCRIPT_END)? + content_start;
        return Some(section[content_start..end].trim_matches('\n'));
    }
    generated_transcript_range(section).map(|(start, end)| section[start..end].trim_matches('\n'))
}

fn generated_transcript_range(section: &str) -> Option<(usize, usize)> {
    if let Some(start) = section.find(GENERATED_TRANSCRIPT_START) {
        let after_start = start + GENERATED_TRANSCRIPT_START.len();
        let end = section[after_start..].find(GENERATED_TRANSCRIPT_END)?
            + after_start
            + GENERATED_TRANSCRIPT_END.len();
        return Some((start, end));
    }

    let start = section.find("<!-- generated-by: lmbrain-verify@")?;
    let legacy = &section[start..];
    let recorded_hash = metadata(legacy, "transcript-hash")?;
    let mut candidate_ends = legacy
        .match_indices('\n')
        .map(|(offset, _)| offset + 1)
        .collect::<Vec<_>>();
    if candidate_ends.last().copied() != Some(legacy.len()) {
        candidate_ends.push(legacy.len());
    }
    candidate_ends.into_iter().find_map(|end| {
        transcript_hash_matches(&legacy[..end], &recorded_hash).then_some((start, start + end))
    })
}

fn transcript_hash_matches(transcript: &str, recorded_hash: &str) -> bool {
    let without_hash = transcript
        .lines()
        .filter(|line| !line.trim().starts_with("<!-- transcript-hash:"))
        .collect::<Vec<_>>()
        .join("\n");
    let canonical = format!("{}\n", without_hash.trim_matches('\n'));
    hex_digest(canonical.as_bytes()) == recorded_hash
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
                "---\nid: SPEC-{name}\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n"
            )).unwrap();
            let approval = dir.path().join("local/approval.json");
            approve_verification_manifest(dir.path(), &approval).unwrap();
            let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
            assert_eq!(report.all_expectations_met, expected_green);
            let source = fs::read_to_string(&spec).unwrap();
            assert!(source.contains("generated-by: lmbrain-verify"));
            assert!(source.contains("$ manual-check\nmanual result"));
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
            assert_eq!(
                transcript_state(dir.path(), &tampered),
                TranscriptState::GeneratedStale
            );
        }
    }

    #[test]
    fn workspace_mutation_during_gates_invalidates_the_transcript() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(dir.path().join("source.txt"), "original").unwrap();
        let mut configured = manifest(if cfg!(windows) { "cmd" } else { "sh" });
        configured.gates[0].args = if cfg!(windows) {
            vec!["/C".into(), "echo mutated>> source.txt".into()]
        } else {
            vec!["-c".into(), "echo mutated >> source.txt".into()]
        };
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-mut.md");
        fs::write(&spec, "---\nid: SPEC-mut\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(report.invalidated);
        assert!(
            !report.all_expectations_met,
            "a run that mutated the workspace must not publish success"
        );
        assert_ne!(report.workspace_fingerprint_before, report.workspace_fingerprint);
        assert!(report
            .invalidation_reason
            .as_deref()
            .unwrap()
            .contains("changed during gate execution"));

        let source = fs::read_to_string(&spec).unwrap();
        assert!(source.contains("workspace-fingerprint-before:"));
        assert!(source.contains("<!-- invalidated: workspace content changed"));
        // The decisive regression: the current workspace now matches the
        // recorded post-gate fingerprint, yet the evidence must stay stale.
        assert_eq!(
            workspace_content_fingerprint(dir.path()).unwrap(),
            report.workspace_fingerprint
        );
        let document = Document::parse(&source).unwrap();
        assert_eq!(
            transcript_state(dir.path(), &document.body),
            TranscriptState::GeneratedStale
        );
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedStale
        );
    }

    #[test]
    fn quiescent_gates_record_matching_fingerprints_and_stay_fresh() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        let mut configured = manifest("rustc");
        configured.gates[0].args = vec!["--version".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-quiet.md");
        fs::write(&spec, "---\nid: SPEC-quiet\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(!report.invalidated);
        assert!(report.invalidation_reason.is_none());
        assert_eq!(report.workspace_fingerprint_before, report.workspace_fingerprint);
        assert!(report.all_expectations_met);
        let document = Document::parse(&fs::read_to_string(&spec).unwrap()).unwrap();
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedFresh
        );
    }

    fn sample_transcript(fingerprint: &str) -> (String, String) {
        let without_hash =
            render_transcript("manifest", fingerprint, fingerprint, "contract", &[], None, None);
        let hash = hex_digest(without_hash.as_bytes());
        let transcript = render_transcript(
            "manifest",
            fingerprint,
            fingerprint,
            "contract",
            &[],
            None,
            Some(&hash),
        );
        (transcript, hash)
    }

    #[test]
    fn generated_transcript_preserves_hand_authored_evidence() {
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n\n### Verification performed\nManual verification summary.\n";
        let (transcript, _) = sample_transcript("first");
        let updated = replace_transcript(body, &transcript).unwrap();

        assert!(updated.contains("$ manual-check\nmanual result"));
        assert!(updated.contains(GENERATED_TRANSCRIPT_START));
        assert!(updated.contains(GENERATED_TRANSCRIPT_END));
        assert!(updated.contains("### Verification performed\nManual verification summary."));
    }

    #[test]
    fn generated_transcript_updates_only_its_managed_region() {
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n";
        let (first, _) = sample_transcript("first");
        let once = replace_transcript(body, &first).unwrap();
        let (second, _) = sample_transcript("second");
        let twice = replace_transcript(&once, &second).unwrap();

        assert!(twice.contains("$ manual-check\nmanual result"));
        assert!(!twice.contains("workspace-fingerprint: first"));
        assert!(twice.contains("workspace-fingerprint: second"));
        assert_eq!(twice.matches(GENERATED_TRANSCRIPT_START).count(), 1);
        assert_eq!(twice.matches(GENERATED_TRANSCRIPT_END).count(), 1);
    }

    #[test]
    fn legacy_generated_transcript_is_migrated_without_losing_manual_evidence() {
        let (legacy, _) = sample_transcript("legacy");
        let body = format!(
            "## Implementation evidence\n\n### Verification transcript\n\n{legacy}\n```text\n$ manual-after-legacy\nmanual result\n```\n"
        );
        let (replacement, _) = sample_transcript("replacement");
        let updated = replace_transcript(&body, &replacement).unwrap();

        assert!(updated.contains("$ manual-after-legacy\nmanual result"));
        assert!(!updated.contains("workspace-fingerprint: legacy"));
        assert!(updated.contains("workspace-fingerprint: replacement"));
    }

    #[test]
    fn verification_merge_uses_the_latest_spec_body() {
        let dir = tempdir().unwrap();
        let spec_dir = dir.path().join(".lmbrain/specs/working");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = spec_dir.join("SPEC-001.md");
        fs::write(
            &spec,
            "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Changes made\n\ninitial\n\n### Verification transcript\n",
        )
        .unwrap();
        let canonical = spec.canonicalize().unwrap();
        let latest = fs::read_to_string(&spec)
            .unwrap()
            .replace("initial", "initial\nlate agent edit");
        fs::write(&spec, latest).unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["sample".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap();

        let updated = fs::read_to_string(spec).unwrap();
        assert!(updated.contains("late agent edit"));
        assert!(updated.contains(GENERATED_TRANSCRIPT_START));
    }

    #[test]
    fn verification_does_not_recreate_a_spec_moved_during_gate_execution() {
        let dir = tempdir().unwrap();
        let working = dir.path().join(".lmbrain/specs/working");
        let review = dir.path().join(".lmbrain/specs/review");
        fs::create_dir_all(&working).unwrap();
        fs::create_dir_all(&review).unwrap();
        let spec = working.join("SPEC-001.md");
        fs::write(
            &spec,
            "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n",
        )
        .unwrap();
        let canonical = spec.canonicalize().unwrap();
        let moved = review.join("SPEC-001.md");
        fs::rename(&spec, &moved).unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        let error = write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["sample".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            VerificationError::ConcurrentModification(_)
        ));
        assert!(!spec.exists());
        assert!(moved.exists());
    }

    #[test]
    fn verification_does_not_write_when_the_gate_contract_changes() {
        let dir = tempdir().unwrap();
        let spec_dir = dir.path().join(".lmbrain/specs/working");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = spec_dir.join("SPEC-001.md");
        let changed = "---\nid: SPEC-001\nstatus: working\nverification_gates: [replacement]\n---\n\n## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n";
        fs::write(&spec, changed).unwrap();
        let canonical = spec.canonicalize().unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        let error = write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["original".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            VerificationError::ConcurrentModification(_)
        ));
        assert_eq!(fs::read_to_string(spec).unwrap(), changed);
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
