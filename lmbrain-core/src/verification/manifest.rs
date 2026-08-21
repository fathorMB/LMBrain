use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::VerificationError;
use crate::{
    frontmatter::atomic_write,
    harness_manifest::workspace_identity,
};

pub const VERIFICATION_MANIFEST_PATH: &str = ".lmbrain/verification.toml";
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
pub const MAX_GATES: usize = 128;
pub const MAX_FINGERPRINT_EXCLUDES: usize = 32;
pub const MAX_FINGERPRINT_EXCLUDE_BYTES: usize = 256;
pub const MAX_TIMEOUT_SECONDS: u64 = 3600;
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 900;
pub const DEFAULT_OUTPUT_BYTES: usize = 128 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct VerificationManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub gates: Vec<VerificationGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingerprint_exclude: Vec<String>,
}

pub fn default_cwd() -> String {
    ".".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationApproval {
    pub workspace_fingerprint: String,
    pub manifest_digest: String,
    pub approved_at: String,
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
        for argument in &gate.args {
            if unsafe_command_value(argument) || looks_machine_local(argument) {
                issues.push(format!(
                    "gate '{}' contains an unsafe or shell-interpolated argument",
                    gate.id
                ));
            }
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
            if !valid_env_key(key)
                || secret_like(key)
                || unsafe_command_value(value)
                || looks_machine_local(value)
            {
                issues.push(format!(
                    "gate '{}' has unsafe environment entry '{}'",
                    gate.id, key
                ));
            }
        }
        if gate.fingerprint_exclude.len() > MAX_FINGERPRINT_EXCLUDES {
            issues.push(format!(
                "gate '{}' declares more than {MAX_FINGERPRINT_EXCLUDES} fingerprint_exclude entries",
                gate.id
            ));
        }
        for entry in &gate.fingerprint_exclude {
            if entry.trim().is_empty()
                || entry.len() > MAX_FINGERPRINT_EXCLUDE_BYTES
                || entry.contains('\0')
                || unsafe_relative(entry)
            {
                issues.push(format!(
                    "gate '{}' fingerprint_exclude entry '{}' must be a workspace-relative path",
                    gate.id, entry
                ));
                continue;
            }
            let normalized = normalized_exclusion(entry);
            if normalized.as_os_str().is_empty() {
                issues.push(format!(
                    "gate '{}' fingerprint_exclude entry '{}' does not name a path",
                    gate.id, entry
                ));
            } else if normalized.starts_with(".lmbrain") {
                issues.push(format!(
                    "gate '{}' fingerprint_exclude entry '{}' cannot exclude managed .lmbrain state",
                    gate.id, entry
                ));
            }
        }
    }
    issues
}

pub fn unsafe_command_value(value: &str) -> bool {
    value.contains('\0')
        || value.contains(['\r', '\n'])
        || value.contains("$(")
        || value.contains("${")
        || value.contains('`')
        || value.to_ascii_lowercase().contains("token=")
        || value.to_ascii_lowercase().contains("password=")
        || value.to_ascii_lowercase().contains("secret=")
}

pub fn looks_machine_local(value: &str) -> bool {
    Path::new(value).is_absolute()
        || value.starts_with("~/")
        || value.starts_with(r"~\")
        || value.to_ascii_lowercase().contains(r"\users\")
        || value.to_ascii_lowercase().contains("/home/")
}

pub fn normalized_exclusion(value: &str) -> PathBuf {
    value
        .split(['/', '\\'])
        .filter(|part| !part.is_empty() && *part != ".")
        .collect()
}

pub fn manifest_exclusions(manifest: &VerificationManifest, gates: &[String]) -> BTreeSet<PathBuf> {
    manifest
        .gates
        .iter()
        .filter(|gate| gates.contains(&gate.id))
        .flat_map(|gate| gate.fingerprint_exclude.iter())
        .map(|entry| normalized_exclusion(entry))
        .filter(|path| !path.as_os_str().is_empty())
        .collect()
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

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub fn valid_env_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub fn secret_like(value: &str) -> bool {
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

pub fn unsafe_relative(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
}

pub fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn gate_contract_digest(gates: &[String]) -> String {
    let encoded = serde_json::to_vec(gates).unwrap_or_default();
    hex_digest(&encoded)
}
