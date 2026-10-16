//! Advisory process lock over `.mdmv/lock` (T015).
//!
//! Multiple concurrent `mdmv-linter` invocations must serialize on a single
//! repository. The lock file itself carries no state; it exists solely so the
//! OS (`flock`) can arbitrate. The lock is intentionally released when the
//! [`RepoLock`] value is dropped (the open file handle is closed).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fs4::FileExt as _;

use crate::error::{Error, Result};

/// How long to wait for the lock before giving up.
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Interval between `try_lock` attempts while waiting for contention to clear.
const RETRY_INTERVAL: Duration = Duration::from_millis(100);

/// An held advisory lock on a repository lock file.
///
/// Releasing the lock happens on drop, by closing the open handle.
#[derive(Debug)]
pub struct RepoLock {
    _file: std::fs::File,
    #[expect(dead_code, reason = "kept for diagnostics when the lock lifetime ends")]
    path: PathBuf,
}

impl RepoLock {
    /// Try to acquire the exclusive lock at `path`, retrying every
    /// [`RETRY_INTERVAL`] until `timeout` elapses.
    ///
    /// On timeout, returns a configuration error telling the operator to wait
    /// for the other invocation to finish.
    pub fn acquire(path: &Path, timeout: Duration) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;

        let deadline = Instant::now() + timeout;
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => {
                    return Ok(Self {
                        _file: file,
                        path: path.to_path_buf(),
                    });
                }
                Err(err) => {
                    if Instant::now() >= deadline {
                        return Err(Error::config(format!(
                            "another mdmv-linter invocation holds {} (or the lock \
                             cannot be taken: {err}); retry after it completes",
                            path.display()
                        )));
                    }
                    std::thread::sleep(RETRY_INTERVAL);
                }
            }
        }
    }

    /// Acquire using the default [`LOCK_TIMEOUT`].
    pub fn acquire_default(path: &Path) -> Result<Self> {
        Self::acquire(path, LOCK_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn acquire_succeeds_and_file_persists() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let lock_path = dir.path().join(".mdmv").join("lock");
        std::fs::create_dir_all(dir.path().join(".mdmv"))?;

        let guard = RepoLock::acquire(&lock_path, Duration::from_secs(1))?;
        assert!(lock_path.exists());
        drop(guard);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn acquire_times_out_when_contention_persists() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let lock_path = dir.path().join("lock");

        // Hold the lock in a separate wrapper so the second acquisition is
        // refused while this handle is still alive.
        let held = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)?;
        if held.try_lock_exclusive().is_ok() {
            let attempt = RepoLock::acquire(&lock_path, Duration::from_millis(200));
            assert!(attempt.is_err(), "expected contention timeout");
            let msg = attempt.err().map_or_else(String::new, |e| e.to_string());
            assert!(msg.contains("holds"), "message mentions the holder: {msg}");
        }
        Ok(())
    }
}
