use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use fs2::FileExt;

const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

/// Cross-process advisory lock for all mutations in a workspace.
///
/// Lock file is located at `.lmbrain/.mutation.lock`.
/// The operating system releases advisory locks when a process exits,
/// so crashes cannot leave a permanently held lock.
pub struct WorkspaceLock {
    file: File,
}

impl WorkspaceLock {
    pub fn acquire(root: &Path) -> io::Result<Self> {
        let lmbrain_dir = root.join(".lmbrain");
        fs::create_dir_all(&lmbrain_dir)?;
        let lock_path = lmbrain_dir.join(".mutation.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;

        let started = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Self { file }),
                Err(error) if is_contention(&error) => {
                    if started.elapsed() >= LOCK_TIMEOUT {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "timed out acquiring workspace mutation lock",
                        ));
                    }
                    thread::sleep(LOCK_RETRY);
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for WorkspaceLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

pub(crate) struct ArtifactMutationLock;

impl ArtifactMutationLock {
    #[inline]
    pub(crate) fn acquire(root: &Path, _artifact_id: &str) -> io::Result<WorkspaceLock> {
        WorkspaceLock::acquire(root)
    }
}

fn is_contention(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
        || error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn workspace_lock_creates_single_mutation_lockfile() {
        let dir = tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        fs::create_dir_all(&lmbrain).unwrap();

        {
            let _lock = WorkspaceLock::acquire(dir.path()).unwrap();
            let lock_file = lmbrain.join(".mutation.lock");
            assert!(lock_file.exists());
        }

        // Lock is released on drop and can be reacquired
        let _second_lock = WorkspaceLock::acquire(dir.path()).unwrap();
    }
}
