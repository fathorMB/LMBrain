use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::VerificationError;

pub fn workspace_content_fingerprint(root: &Path) -> Result<String, VerificationError> {
    workspace_content_fingerprint_with(root, &BTreeSet::new())
}

/// Fingerprint the workspace while skipping the operator-approved
/// gate-declared exclusion paths. Pre- and post-gate snapshots must use the
/// same exclusion set for the comparison to be meaningful.
pub fn workspace_content_fingerprint_with(
    root: &Path,
    exclusions: &BTreeSet<PathBuf>,
) -> Result<String, VerificationError> {
    let root = root.canonicalize()?;
    let mut files = Vec::new();
    collect_files(&root, &root, exclusions, &mut files)?;
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
    exclusions: &BTreeSet<PathBuf>,
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
        if relative.starts_with(".lmbrain/specs")
            || relative.starts_with(".lmbrain/reviews")
            || relative == Path::new(".lmbrain/.mutation.lock")
            || path.file_name().and_then(|n| n.to_str()) == Some(".mutation.lock")
        {
            continue;
        }
        if exclusions
            .iter()
            .any(|exclusion| relative.starts_with(exclusion))
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_files(root, &path, exclusions, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}
