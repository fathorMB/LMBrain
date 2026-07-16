use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use fs2::FileExt;
use sha2::{Digest, Sha256};

const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

/// Cross-process advisory lock for mutations of one managed artifact.
///
/// The lock file is retained intentionally. The operating system releases its
/// lock when a process exits, so a crash cannot leave a permanently held lock.
pub(crate) struct ArtifactMutationLock {
    file: File,
}

impl ArtifactMutationLock {
    pub(crate) fn acquire(root: &Path, artifact_id: &str) -> io::Result<Self> {
        let workspace_key = root.to_string_lossy();
        #[cfg(windows)]
        let workspace_key = workspace_key.to_ascii_lowercase();
        let workspace_digest = Sha256::digest(workspace_key.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let lock_dir = std::env::temp_dir()
            .join("lmbrain-locks")
            .join(workspace_digest);
        fs::create_dir_all(&lock_dir)?;
        let safe_id: String = artifact_id
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let path = lock_dir.join(format!("artifact-{safe_id}.lock"));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let started = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Self { file }),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    if started.elapsed() >= LOCK_TIMEOUT {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            format!("timed out acquiring mutation lock for {artifact_id}"),
                        ));
                    }
                    thread::sleep(LOCK_RETRY);
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for ArtifactMutationLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}
