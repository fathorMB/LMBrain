use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File, OpenOptions},
    io,
    path::Path,
    hash::{Hash, Hasher},
    thread,
    time::{Duration, Instant},
};

use fs2::FileExt;

const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

/// Cross-process advisory lock for all mutations in a workspace.
///
/// Lock files live under the system temporary directory rather than the
/// workspace. This keeps them out of source control and lets migrations swap
/// `.lmbrain` while a lock is held (notably on Windows).
/// The operating system releases advisory locks when a process exits,
/// so crashes cannot leave a permanently held lock.
pub struct WorkspaceLock {
    file: File,
}

impl WorkspaceLock {
    pub fn acquire(root: &Path) -> io::Result<Self> {
        let canonical_root = root.canonicalize()?;
        let mut hasher = DefaultHasher::new();
        canonical_root.hash(&mut hasher);
        let lock_dir = std::env::temp_dir().join("lmbrain-locks");
        fs::create_dir_all(&lock_dir)?;
        let lock_path = lock_dir.join(format!("{:016x}.lock", hasher.finish()));
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

fn is_contention(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
        || error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn workspace_lock_does_not_modify_the_workspace() {
        let dir = tempdir().unwrap();
        {
            let _lock = WorkspaceLock::acquire(dir.path()).unwrap();
            assert!(!dir.path().join(".lmbrain").exists());
        }

        // Lock is released on drop and can be reacquired
        let _second_lock = WorkspaceLock::acquire(dir.path()).unwrap();
    }
}
