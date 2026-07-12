use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use lmbrain_core::{
    canonical_manifest_digest, load_harness_manifest, workspace_identity, HarnessManifestError,
};

const STORE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ApprovalStoreData {
    schema_version: u32,
    #[serde(default)]
    approvals: BTreeMap<String, ApprovalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalRecord {
    manifest_digest: String,
    approved_at: String,
    #[serde(default)]
    applied_files: BTreeMap<String, String>,
    #[serde(default)]
    applied_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessApprovalState {
    Unconfigured,
    ApprovalRequired,
    Approved,
    Stale,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessApprovalStatus {
    pub state: HarnessApprovalState,
    pub manifest_digest: Option<String>,
    pub approved_digest: Option<String>,
    pub approved_at: Option<String>,
    pub workspace_fingerprint: String,
}

pub struct HarnessApprovalStore {
    path: PathBuf,
    data: Mutex<ApprovalStoreData>,
}

impl HarnessApprovalStore {
    pub fn initialize(app_data_dir: &Path) -> Result<Self, String> {
        let directory = app_data_dir.join("lmbrain");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let path = directory.join("harness-approvals.json");
        let data = if path.exists() {
            match fs::read_to_string(&path)
                .map_err(|error| error.to_string())
                .and_then(|source| {
                    serde_json::from_str::<ApprovalStoreData>(&source)
                        .map_err(|error| error.to_string())
                }) {
                Ok(data) if data.schema_version == STORE_SCHEMA_VERSION => data,
                Ok(_) | Err(_) => {
                    let backup = directory.join(format!(
                        "harness-approvals.corrupt-{}.json",
                        Utc::now().format("%Y%m%dT%H%M%S")
                    ));
                    fs::rename(&path, backup).map_err(|error| error.to_string())?;
                    empty_store()
                }
            }
        } else {
            empty_store()
        };
        let store = Self {
            path,
            data: Mutex::new(data),
        };
        store.save()?;
        Ok(store)
    }

    pub fn status(&self, root: &Path) -> Result<HarnessApprovalStatus, String> {
        let identity = workspace_identity(root).map_err(|error| error.to_string())?;
        let manifest_digest = match load_harness_manifest(root) {
            Ok(manifest) => {
                Some(canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?)
            }
            Err(HarnessManifestError::Missing(_)) => None,
            Err(error) => return Err(error.to_string()),
        };
        let data = self
            .data
            .lock()
            .map_err(|_| "approval store lock poisoned")?;
        let record = data.approvals.get(&identity.fingerprint);
        let state = match (&manifest_digest, record) {
            (None, _) => HarnessApprovalState::Unconfigured,
            (Some(_), None) => HarnessApprovalState::ApprovalRequired,
            (Some(current), Some(approved)) if current == &approved.manifest_digest => {
                HarnessApprovalState::Approved
            }
            (Some(_), Some(_)) => HarnessApprovalState::Stale,
        };
        Ok(HarnessApprovalStatus {
            state,
            manifest_digest,
            approved_digest: record.map(|record| record.manifest_digest.clone()),
            approved_at: record.map(|record| record.approved_at.clone()),
            workspace_fingerprint: identity.fingerprint,
        })
    }

    pub fn approve(
        &self,
        root: &Path,
        expected_digest: &str,
    ) -> Result<HarnessApprovalStatus, String> {
        let identity = workspace_identity(root).map_err(|error| error.to_string())?;
        let manifest = load_harness_manifest(root).map_err(|error| error.to_string())?;
        let current = canonical_manifest_digest(&manifest).map_err(|error| error.to_string())?;
        if current != expected_digest {
            return Err(
                "manifest changed since preview; refresh and approve the current digest".into(),
            );
        }
        {
            let mut data = self
                .data
                .lock()
                .map_err(|_| "approval store lock poisoned")?;
            data.approvals.insert(
                identity.fingerprint,
                ApprovalRecord {
                    manifest_digest: current,
                    approved_at: Utc::now().to_rfc3339(),
                    applied_files: BTreeMap::new(),
                    applied_at: None,
                },
            );
        }
        self.save()?;
        self.status(root)
    }

    pub fn revoke(&self, root: &Path) -> Result<HarnessApprovalStatus, String> {
        let identity = workspace_identity(root).map_err(|error| error.to_string())?;
        self.data
            .lock()
            .map_err(|_| "approval store lock poisoned")?
            .approvals
            .remove(&identity.fingerprint);
        self.save()?;
        self.status(root)
    }

    pub fn record_application(
        &self,
        root: &Path,
        manifest_digest: &str,
        files: &[(String, String)],
    ) -> Result<(), String> {
        let identity = workspace_identity(root).map_err(|error| error.to_string())?;
        {
            let mut data = self
                .data
                .lock()
                .map_err(|_| "approval store lock poisoned")?;
            let record = data
                .approvals
                .get_mut(&identity.fingerprint)
                .ok_or("manifest is not approved")?;
            if record.manifest_digest != manifest_digest {
                return Err("approval digest no longer matches apply result".into());
            }
            record.applied_files = files.iter().cloned().collect();
            record.applied_at = Some(Utc::now().to_rfc3339());
        }
        self.save()
    }

    pub fn applied_files(&self, root: &Path) -> Result<BTreeMap<String, String>, String> {
        let identity = workspace_identity(root).map_err(|error| error.to_string())?;
        Ok(self
            .data
            .lock()
            .map_err(|_| "approval store lock poisoned")?
            .approvals
            .get(&identity.fingerprint)
            .map(|record| record.applied_files.clone())
            .unwrap_or_default())
    }

    fn save(&self) -> Result<(), String> {
        let data = self
            .data
            .lock()
            .map_err(|_| "approval store lock poisoned")?;
        let content = format!(
            "{}\n",
            serde_json::to_string_pretty(&*data).map_err(|error| error.to_string())?
        );
        lmbrain_core::frontmatter::atomic_write(&self.path, &content)
            .map_err(|error| error.to_string())?;
        restrict_permissions(&self.path)?;
        Ok(())
    }
}

fn empty_store() -> ApprovalStoreData {
    ApprovalStoreData {
        schema_version: STORE_SCHEMA_VERSION,
        approvals: BTreeMap::new(),
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lmbrain_core::{set_harness_manifest, HarnessManifest};

    fn setup() -> (tempfile::TempDir, tempfile::TempDir, HarnessApprovalStore) {
        let workspace = tempfile::tempdir().unwrap();
        fs::create_dir(workspace.path().join(".lmbrain")).unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let store = HarnessApprovalStore::initialize(app_data.path()).unwrap();
        (workspace, app_data, store)
    }

    fn manifest(enabled: bool) -> HarnessManifest {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "hosts": {"codex": {"enabled": enabled}}
        }))
        .unwrap()
    }

    #[test]
    fn approval_is_digest_bound_and_manifest_changes_make_it_stale() {
        let (workspace, _app_data, store) = setup();
        assert_eq!(
            store.status(workspace.path()).unwrap().state,
            HarnessApprovalState::Unconfigured
        );
        set_harness_manifest(workspace.path(), &manifest(false)).unwrap();
        let required = store.status(workspace.path()).unwrap();
        assert_eq!(required.state, HarnessApprovalState::ApprovalRequired);
        let approved = store
            .approve(
                workspace.path(),
                required.manifest_digest.as_deref().unwrap(),
            )
            .unwrap();
        assert_eq!(approved.state, HarnessApprovalState::Approved);
        set_harness_manifest(workspace.path(), &manifest(true)).unwrap();
        assert_eq!(
            store.status(workspace.path()).unwrap().state,
            HarnessApprovalState::Stale
        );
    }

    #[test]
    fn stale_preview_digest_cannot_be_approved_and_revoke_is_idempotent() {
        let (workspace, _app_data, store) = setup();
        set_harness_manifest(workspace.path(), &manifest(false)).unwrap();
        assert!(store.approve(workspace.path(), "0").is_err());
        assert_eq!(
            store.revoke(workspace.path()).unwrap().state,
            HarnessApprovalState::ApprovalRequired
        );
        assert_eq!(
            store.revoke(workspace.path()).unwrap().state,
            HarnessApprovalState::ApprovalRequired
        );
    }

    #[test]
    fn corrupt_store_is_quarantined_without_reusing_approval() {
        let app_data = tempfile::tempdir().unwrap();
        let directory = app_data.path().join("lmbrain");
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("harness-approvals.json"), "not json").unwrap();
        let store = HarnessApprovalStore::initialize(app_data.path()).unwrap();
        assert!(store.data.lock().unwrap().approvals.is_empty());
        assert!(fs::read_dir(directory).unwrap().flatten().any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with("harness-approvals.corrupt-")));
    }
}
