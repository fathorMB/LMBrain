use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::mutation_lock::WorkspaceLock;

pub const BRANCHING_STRATEGY_PATH: &str = ".lmbrain/BRANCHING.json";
pub const BRANCHING_STRATEGY_SCHEMA_VERSION: u32 = 1;
const MAX_STRATEGY_BYTES: u64 = 256 * 1024;
const BRANCHING_AUDIT_PATH: &str = ".lmbrain/BRANCHING.audit.jsonl";

#[derive(Debug, Error)]
pub enum BranchingStrategyError {
    #[allow(dead_code)]
    #[error("File system error at {path}: {error}")]
    Io { path: String, error: String },
    #[error("Branching strategy file size ({size} bytes) exceeds limit ({limit} bytes)")]
    FileTooLarge { size: u64, limit: u64 },
    #[error("Failed to parse branching strategy JSON: {0}")]
    Parse(String),
    #[error("Invalid schema version {version}; expected {expected}")]
    InvalidSchemaVersion { version: u32, expected: u32 },
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unauthorized actor '{actor}': mutating branching strategy requires operator authority ('operator')")]
    UnauthorizedActor { actor: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
pub enum BranchingTopology {
    MainOnly,
    GithubFlow,
    GitFlow,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct BranchNamingConfig {
    #[serde(default)]
    pub allowed_prefixes: Vec<String>,
    #[serde(default = "default_spec_branch_pattern")]
    pub spec_branch_pattern: String,
    #[serde(default = "default_true")]
    pub require_prefix: bool,
}

fn default_spec_branch_pattern() -> String {
    "{prefix}{spec_id_lowercase}-{slug}".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct BranchAuthorityConfig {
    #[serde(default)]
    pub lead_only_push_branches: Vec<String>,
    #[serde(default)]
    pub allow_specialist_push: bool,
    #[serde(default)]
    pub require_pr_for_merge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct CommitTriggersConfig {
    #[serde(default = "default_true")]
    pub commit_on_spec_completion: bool,
    #[serde(default = "default_true")]
    pub commit_on_doc_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
pub struct BranchingStrategy {
    pub schema_version: u32,
    pub topology: BranchingTopology,
    pub default_branch: String,
    #[serde(default)]
    pub protected_branches: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development_branch: Option<String>,
    pub branch_naming: BranchNamingConfig,
    pub authority: BranchAuthorityConfig,
    pub commit_triggers: CommitTriggersConfig,
}

impl Default for BranchingStrategy {
    fn default() -> Self {
        Self::default_scaffolded()
    }
}

impl BranchingStrategy {
    pub fn default_scaffolded() -> Self {
        Self {
            schema_version: BRANCHING_STRATEGY_SCHEMA_VERSION,
            topology: BranchingTopology::MainOnly,
            default_branch: "main".to_string(),
            protected_branches: vec!["main".to_string()],
            development_branch: None,
            branch_naming: BranchNamingConfig {
                allowed_prefixes: vec![
                    "feature/".to_string(),
                    "fix/".to_string(),
                    "codex/".to_string(),
                    "release/".to_string(),
                    "hotfix/".to_string(),
                ],
                spec_branch_pattern: "{prefix}{spec_id_lowercase}-{slug}".to_string(),
                require_prefix: true,
            },
            authority: BranchAuthorityConfig {
                lead_only_push_branches: vec!["main".to_string()],
                allow_specialist_push: false,
                require_pr_for_merge: false,
            },
            commit_triggers: CommitTriggersConfig {
                commit_on_spec_completion: true,
                commit_on_doc_change: true,
            },
        }
    }
}

pub fn validate_branching_strategy(
    strategy: &BranchingStrategy,
) -> Result<(), BranchingStrategyError> {
    if strategy.schema_version != BRANCHING_STRATEGY_SCHEMA_VERSION {
        return Err(BranchingStrategyError::InvalidSchemaVersion {
            version: strategy.schema_version,
            expected: BRANCHING_STRATEGY_SCHEMA_VERSION,
        });
    }

    if strategy.default_branch.trim().is_empty() {
        return Err(BranchingStrategyError::Validation(
            "default_branch cannot be empty".to_string(),
        ));
    }

    if strategy.branch_naming.require_prefix && strategy.branch_naming.allowed_prefixes.is_empty() {
        return Err(BranchingStrategyError::Validation(
            "allowed_prefixes cannot be empty when require_prefix is true".to_string(),
        ));
    }

    Ok(())
}

pub fn parse_branching_strategy(json: &str) -> Result<BranchingStrategy, BranchingStrategyError> {
    let strategy: BranchingStrategy =
        serde_json::from_str(json).map_err(|e| BranchingStrategyError::Parse(e.to_string()))?;
    validate_branching_strategy(&strategy)?;
    Ok(strategy)
}

pub fn load_branching_strategy(
    root: &Path,
) -> Result<Option<BranchingStrategy>, BranchingStrategyError> {
    let path = root.join(BRANCHING_STRATEGY_PATH);
    if !path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(&path).map_err(|e| BranchingStrategyError::Io {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;

    if metadata.len() > MAX_STRATEGY_BYTES {
        return Err(BranchingStrategyError::FileTooLarge {
            size: metadata.len(),
            limit: MAX_STRATEGY_BYTES,
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| BranchingStrategyError::Io {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;

    let strategy = parse_branching_strategy(&content)?;
    Ok(Some(strategy))
}

pub fn set_branching_strategy(
    root: &Path,
    strategy: &BranchingStrategy,
    actor: &str,
    reason: &str,
) -> Result<(), BranchingStrategyError> {
    if actor != "operator" {
        return Err(BranchingStrategyError::UnauthorizedActor {
            actor: actor.to_string(),
        });
    }

    validate_branching_strategy(strategy)?;

    let _lock = WorkspaceLock::acquire(root);

    let lmbrain_dir = root.join(".lmbrain");
    if !lmbrain_dir.exists() {
        fs::create_dir_all(&lmbrain_dir).map_err(|e| BranchingStrategyError::Io {
            path: lmbrain_dir.display().to_string(),
            error: e.to_string(),
        })?;
    }

    let formatted_json = serde_json::to_string_pretty(strategy)
        .map_err(|e| BranchingStrategyError::Parse(e.to_string()))?;

    let target_path = root.join(BRANCHING_STRATEGY_PATH);
    let tmp_path = root.join(".lmbrain/.BRANCHING.json.tmp");

    fs::write(&tmp_path, formatted_json.as_bytes()).map_err(|e| BranchingStrategyError::Io {
        path: tmp_path.display().to_string(),
        error: e.to_string(),
    })?;

    fs::rename(&tmp_path, &target_path).map_err(|e| BranchingStrategyError::Io {
        path: target_path.display().to_string(),
        error: e.to_string(),
    })?;

    // Record audit entry
    let audit_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "actor": actor,
        "reason": reason,
        "schema_version": strategy.schema_version,
        "topology": strategy.topology,
        "default_branch": strategy.default_branch
    });

    let audit_path = root.join(BRANCHING_AUDIT_PATH);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
    {
        let _ = writeln!(file, "{}", audit_entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_scaffolded_strategy_parses_and_validates() {
        let default_strat = BranchingStrategy::default_scaffolded();
        assert_eq!(default_strat.schema_version, 1);
        assert_eq!(default_strat.topology, BranchingTopology::MainOnly);
        assert_eq!(default_strat.default_branch, "main");
        assert!(validate_branching_strategy(&default_strat).is_ok());
    }

    #[test]
    fn set_branching_strategy_requires_operator_actor() {
        let dir = tempdir().unwrap();
        let strat = BranchingStrategy::default_scaffolded();

        let err =
            set_branching_strategy(dir.path(), &strat, "project-lead", "testing").unwrap_err();
        assert!(matches!(
            err,
            BranchingStrategyError::UnauthorizedActor { .. }
        ));

        let res = set_branching_strategy(dir.path(), &strat, "operator", "testing");
        assert!(res.is_ok());

        let loaded = load_branching_strategy(dir.path()).unwrap().unwrap();
        assert_eq!(loaded, strat);
    }
}
