use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const HARNESS_MANIFEST_PATH: &str = ".lmbrain/HARNESSES.json";
pub const HARNESS_MANIFEST_SCHEMA_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_ENVIRONMENT_ENTRIES: usize = 64;
const MAX_VALUE_BYTES: usize = 4096;
const HARNESS_AUDIT_PATH: &str = ".lmbrain/HARNESSES.audit.jsonl";
const HARNESS_LOCK_PATH: &str = ".lmbrain/.harness-config.lock";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub hosts: BTreeMap<HarnessHost, HostConfiguration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessHost {
    ClaudeCode,
    Codex,
    Pi,
    OpenCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HostConfiguration {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub required_tools: BTreeSet<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lsp: Option<LspRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspRequirement {
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityState {
    Configured,
    PrerequisiteReady,
    Active,
    InactiveLazy,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceIdentity {
    pub canonical_root: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessManifestMutation {
    pub path: String,
    pub digest: String,
    pub workspace_fingerprint: String,
}

#[derive(Debug, Error)]
pub enum HarnessManifestError {
    #[error("harness manifest does not exist: {0}")]
    Missing(String),
    #[error("harness manifest path escapes the workspace: {0}")]
    UnsafePath(String),
    #[error("cannot read harness manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid harness manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid harness manifest: {0:?}")]
    Validation(Vec<HarnessValidationIssue>),
}

pub fn load_harness_manifest(root: &Path) -> Result<HarnessManifest, HarnessManifestError> {
    let canonical_root = root.canonicalize()?;
    let path = root.join(HARNESS_MANIFEST_PATH);
    let canonical_path = path
        .canonicalize()
        .map_err(|_| HarnessManifestError::Missing(path.display().to_string()))?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(HarnessManifestError::UnsafePath(path.display().to_string()));
    }
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(HarnessManifestError::UnsafePath(path.display().to_string()));
    }
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(HarnessManifestError::Validation(vec![
            HarnessValidationIssue {
                path: "$".into(),
                message: format!("manifest exceeds {MAX_MANIFEST_BYTES} bytes"),
            },
        ]));
    }
    let source = fs::read_to_string(canonical_path)?;
    let manifest: HarnessManifest = serde_json::from_str(&source)?;
    let issues = validate_harness_manifest(&manifest);
    if issues.is_empty() {
        Ok(manifest)
    } else {
        Err(HarnessManifestError::Validation(issues))
    }
}

pub fn parse_harness_manifest(source: &str) -> Result<HarnessManifest, HarnessManifestError> {
    if source.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(HarnessManifestError::Validation(vec![
            HarnessValidationIssue {
                path: "$".into(),
                message: format!("manifest exceeds {MAX_MANIFEST_BYTES} bytes"),
            },
        ]));
    }
    let manifest: HarnessManifest = serde_json::from_str(source)?;
    let issues = validate_harness_manifest(&manifest);
    if issues.is_empty() {
        Ok(manifest)
    } else {
        Err(HarnessManifestError::Validation(issues))
    }
}

pub fn set_harness_manifest(
    root: &Path,
    manifest: &HarnessManifest,
) -> Result<HarnessManifestMutation, HarnessManifestError> {
    let issues = validate_harness_manifest(manifest);
    if !issues.is_empty() {
        return Err(HarnessManifestError::Validation(issues));
    }
    let canonical_root = root.canonicalize()?;
    let brain = canonical_root.join(".lmbrain");
    let metadata = fs::symlink_metadata(&brain)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(HarnessManifestError::UnsafePath(
            brain.display().to_string(),
        ));
    }
    let _lock = ManifestLock::acquire(&canonical_root.join(HARNESS_LOCK_PATH))?;
    let path = brain.join("HARNESSES.json");
    if path.exists() && fs::symlink_metadata(&path)?.file_type().is_symlink() {
        return Err(HarnessManifestError::UnsafePath(path.display().to_string()));
    }
    let rendered = format!("{}\n", serde_json::to_string_pretty(manifest)?);
    crate::frontmatter::atomic_write(&path, &rendered)
        .map_err(|error| HarnessManifestError::Io(std::io::Error::other(error.to_string())))?;
    let identity = workspace_identity(&canonical_root)?;
    let digest = canonical_manifest_digest(manifest)?;
    append_audit(&brain, &digest)?;
    Ok(HarnessManifestMutation {
        path: HARNESS_MANIFEST_PATH.into(),
        digest,
        workspace_fingerprint: identity.fingerprint,
    })
}

pub fn validate_harness_manifest(manifest: &HarnessManifest) -> Vec<HarnessValidationIssue> {
    let mut issues = Vec::new();
    if manifest.schema_version != HARNESS_MANIFEST_SCHEMA_VERSION {
        issue(
            &mut issues,
            "schema_version",
            format!("expected schema version {HARNESS_MANIFEST_SCHEMA_VERSION}"),
        );
    }
    for (host, config) in &manifest.hosts {
        let base = format!("hosts.{}", host_name(*host));
        if config.lsp.is_some() && !matches!(host, HarnessHost::ClaudeCode | HarnessHost::OpenCode)
        {
            issue(
                &mut issues,
                format!("{base}.lsp"),
                "LSP policy is not supported by this host",
            );
        }
        for tool in &config.required_tools {
            if !valid_identifier(tool) {
                issue(
                    &mut issues,
                    format!("{base}.required_tools"),
                    format!("'{tool}' must be a portable tool identifier, not a path or command"),
                );
            }
        }
        for (key, value) in &config.environment {
            if !valid_environment_key(key) {
                issue(
                    &mut issues,
                    format!("{base}.environment.{key}"),
                    "invalid environment variable name",
                );
            }
            if secret_like(key) {
                issue(
                    &mut issues,
                    format!("{base}.environment.{key}"),
                    "secret-like environment keys are forbidden",
                );
            }
            if value.contains('\0') || Path::new(value).is_absolute() || has_parent_component(value)
            {
                issue(
                    &mut issues,
                    format!("{base}.environment.{key}"),
                    "values cannot contain NUL, absolute machine paths, or parent traversal",
                );
            }
            if value.len() > MAX_VALUE_BYTES {
                issue(
                    &mut issues,
                    format!("{base}.environment.{key}"),
                    format!("value exceeds {MAX_VALUE_BYTES} bytes"),
                );
            }
        }
        if config.environment.len() > MAX_ENVIRONMENT_ENTRIES {
            issue(
                &mut issues,
                format!("{base}.environment"),
                format!("at most {MAX_ENVIRONMENT_ENTRIES} entries are allowed"),
            );
        }
    }
    issues
}

pub fn canonical_manifest_digest(
    manifest: &HarnessManifest,
) -> Result<String, HarnessManifestError> {
    let issues = validate_harness_manifest(manifest);
    if !issues.is_empty() {
        return Err(HarnessManifestError::Validation(issues));
    }
    let canonical = serde_json::to_vec(manifest)?;
    Ok(hex_digest(&canonical))
}

pub fn workspace_identity(root: &Path) -> Result<WorkspaceIdentity, HarnessManifestError> {
    let canonical = root.canonicalize()?;
    let canonical_root = canonical.to_string_lossy().replace('\\', "/");
    Ok(WorkspaceIdentity {
        fingerprint: hex_digest(canonical_root.as_bytes()),
        canonical_root,
    })
}

fn issue(
    issues: &mut Vec<HarnessValidationIssue>,
    path: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(HarnessValidationIssue {
        path: path.into(),
        message: message.into(),
    });
}

fn host_name(host: HarnessHost) -> &'static str {
    match host {
        HarnessHost::ClaudeCode => "claude-code",
        HarnessHost::Codex => "codex",
        HarnessHost::Pi => "pi",
        HarnessHost::OpenCode => "open-code",
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn valid_environment_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn secret_like(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    [
        "SECRET",
        "TOKEN",
        "PASSWORD",
        "PASSWD",
        "CREDENTIAL",
        "PRIVATE_KEY",
        "API_KEY",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

fn has_parent_component(value: &str) -> bool {
    Path::new(value)
        .components()
        .any(|component| component == Component::ParentDir)
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn content_digest(bytes: &[u8]) -> String {
    hex_digest(bytes)
}

fn append_audit(brain: &Path, digest: &str) -> Result<(), HarnessManifestError> {
    let path = brain
        .parent()
        .expect(".lmbrain has a workspace parent")
        .join(HARNESS_AUDIT_PATH);
    if path.exists() && fs::symlink_metadata(&path)?.file_type().is_symlink() {
        return Err(HarnessManifestError::UnsafePath(path.display().to_string()));
    }
    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "action": "harness_config_set",
        "manifest_digest": digest,
    });
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{entry}")?;
    file.sync_data()?;
    Ok(())
}

struct ManifestLock(std::path::PathBuf);

impl ManifestLock {
    fn acquire(path: &Path) -> Result<Self, HarnessManifestError> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    HarnessManifestError::Io(std::io::Error::new(
                        error.kind(),
                        "harness manifest mutation already in progress",
                    ))
                } else {
                    HarnessManifestError::Io(error)
                }
            })?;
        Ok(Self(path.to_path_buf()))
    }
}

impl Drop for ManifestLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn manifest() -> HarnessManifest {
        HarnessManifest {
            schema_version: 1,
            hosts: BTreeMap::from([(
                HarnessHost::OpenCode,
                HostConfiguration {
                    enabled: true,
                    required_tools: BTreeSet::from(["rust-analyzer".into()]),
                    environment: BTreeMap::from([("RUST_LOG".into(), "info".into())]),
                    lsp: Some(LspRequirement { required: true }),
                },
            )]),
        }
    }

    #[test]
    fn canonical_digest_is_stable_across_json_formatting() {
        let expected = canonical_manifest_digest(&manifest()).unwrap();
        let compact = serde_json::to_string(&manifest()).unwrap();
        let parsed: HarnessManifest = serde_json::from_str(&compact).unwrap();
        assert_eq!(expected, canonical_manifest_digest(&parsed).unwrap());
        assert_eq!(expected.len(), 64);
    }

    #[test]
    fn rejects_unknown_fields_and_unsupported_capabilities() {
        let unknown = r#"{"schema_version":1,"hosts":{},"command":"pwsh"}"#;
        assert!(serde_json::from_str::<HarnessManifest>(unknown).is_err());
        let mut candidate = manifest();
        let lsp = candidate.hosts.remove(&HarnessHost::OpenCode).unwrap();
        candidate.hosts.insert(HarnessHost::Pi, lsp);
        assert!(validate_harness_manifest(&candidate)
            .iter()
            .any(|issue| issue.path.ends_with(".lsp")));
    }

    #[test]
    fn rejects_secrets_paths_commands_and_traversal() {
        let mut candidate = manifest();
        let host = candidate.hosts.get_mut(&HarnessHost::OpenCode).unwrap();
        host.required_tools.insert("cargo test".into());
        host.environment.insert("API_TOKEN".into(), "hidden".into());
        host.environment
            .insert("CONFIG".into(), "../outside".into());
        let issues = validate_harness_manifest(&candidate);
        assert!(issues.len() >= 3, "{issues:?}");
    }

    #[test]
    fn loader_rejects_missing_and_accepts_confined_regular_file() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        assert!(matches!(
            load_harness_manifest(dir.path()),
            Err(HarnessManifestError::Missing(_))
        ));
        fs::write(
            dir.path().join(HARNESS_MANIFEST_PATH),
            serde_json::to_vec_pretty(&manifest()).unwrap(),
        )
        .unwrap();
        assert_eq!(load_harness_manifest(dir.path()).unwrap(), manifest());
    }

    #[test]
    fn workspace_identity_is_normalized_and_stable() {
        let dir = tempdir().unwrap();
        let first = workspace_identity(dir.path()).unwrap();
        let second = workspace_identity(&dir.path().join(".")).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.fingerprint.len(), 64);
    }

    #[test]
    fn setter_writes_canonical_content_and_secret_free_audit() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        let result = set_harness_manifest(dir.path(), &manifest()).unwrap();
        assert_eq!(result.digest.len(), 64);
        assert_eq!(load_harness_manifest(dir.path()).unwrap(), manifest());
        let audit = fs::read_to_string(dir.path().join(HARNESS_AUDIT_PATH)).unwrap();
        assert!(audit.contains("harness_config_set"));
        assert!(!audit.contains("RUST_LOG"));
        assert!(!dir.path().join(HARNESS_LOCK_PATH).exists());
    }

    #[test]
    fn rejects_oversized_manifest_and_environment_values() {
        let oversized = "x".repeat(MAX_MANIFEST_BYTES as usize + 1);
        assert!(matches!(
            parse_harness_manifest(&oversized),
            Err(HarnessManifestError::Validation(_))
        ));
        let mut candidate = manifest();
        candidate
            .hosts
            .get_mut(&HarnessHost::OpenCode)
            .unwrap()
            .environment
            .insert("VALUE".into(), "x".repeat(MAX_VALUE_BYTES + 1));
        assert!(validate_harness_manifest(&candidate)
            .iter()
            .any(|issue| issue.message.contains("exceeds")));
    }
}
