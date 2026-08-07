//! Read-side access to the machine-local harness approval store. Since #87
//! the store and every mutating operation (approve, revoke, apply) live in
//! `lmbrain_core::harness_environment` and are exercised by the Project Lead
//! through the MCP server; the app only consults status and drift.

use std::{collections::BTreeMap, fs, path::Path, path::PathBuf};

pub use lmbrain_core::harness_environment::{HarnessApprovalState, HarnessApprovalStatus};

pub struct HarnessApprovalStore {
    path: PathBuf,
}

impl HarnessApprovalStore {
    /// `app_data_dir` is Tauri's per-identifier data directory, which matches
    /// `lmbrain_core::harness_environment::default_harness_approval_store_path`
    /// (`<data dir>/com.lmbrain.app/lmbrain/harness-approvals.json`).
    pub fn initialize(app_data_dir: &Path) -> Result<Self, String> {
        let directory = app_data_dir.join("lmbrain");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        Ok(Self {
            path: directory.join("harness-approvals.json"),
        })
    }

    pub fn status(&self, root: &Path) -> Result<HarnessApprovalStatus, String> {
        lmbrain_core::harness_environment::harness_approval_status(root, &self.path)
    }

    pub fn applied_files(&self, root: &Path) -> Result<BTreeMap<String, String>, String> {
        lmbrain_core::harness_environment::applied_files(root, &self.path)
    }
}
