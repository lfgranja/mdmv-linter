//! Crash-safe atomic file writes.
//!
//! Protocol (T014): create the parent directory if missing, write to a temp
//! file **in the same directory** (same filesystem so the rename cannot
//! degrade to a copy), flush with `sync_all`, `persist` (rename) into place,
//! then fsync the parent directory so the rename itself survives a crash.

use std::io::Write as _;
use std::path::Path;

use crate::error::{Error, Result};

/// Write `contents` to `path` atomically and durably.
///
/// The temp file is created in the same parent directory as `path`; missing
/// parent directories are created first. On any failure, no partially written
/// state is visible at `path`.
pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = parent_dir(path);
    std::fs::create_dir_all(&parent)
        .map_err(|e| Error::config(format!("cannot create directory {}: {e}", parent.display())))?;

    let mut tmp = tempfile::NamedTempFile::new_in(&parent).map_err(|e| {
        Error::config(format!(
            "cannot create temp file in {}: {e}",
            parent.display()
        ))
    })?;
    let tmp_path = tmp.path().to_path_buf();

    tmp.write_all(contents).map_err(|e| {
        Error::config(format!(
            "cannot write temp file {}: {e}",
            tmp_path.display()
        ))
    })?;
    tmp.as_file_mut().sync_all().map_err(|e| {
        Error::config(format!(
            "cannot flush temp file {}: {e}",
            tmp_path.display()
        ))
    })?;

    tmp.persist(path).map_err(|e| {
        Error::config(format!(
            "cannot move temp file {} into place at {}: {}",
            tmp_path.display(),
            path.display(),
            e.error
        ))
    })?;

    fsync_dir(&parent)
}

/// Write a `&str`, delegating to [`write_atomic`].
pub fn write_atomic_str(path: &Path, contents: &str) -> Result<()> {
    write_atomic(path, contents.as_bytes())
}

/// The parent directory of `path`, with bare file names resolved to `.`.
fn parent_dir(path: &Path) -> std::borrow::Cow<'_, Path> {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => std::borrow::Cow::Borrowed(p),
        _ => std::borrow::Cow::Borrowed(Path::new(".")),
    }
}

/// Best-effort durability fence: fsync the directory itself.
fn fsync_dir(dir: &Path) -> Result<()> {
    let fd = rustix::fs::open(dir, rustix::fs::OFlags::RDONLY, rustix::fs::Mode::empty()).map_err(
        |e| {
            Error::config(format!(
                "cannot open directory {} for fsync: {e}",
                dir.display()
            ))
        },
    )?;
    rustix::fs::fsync(&fd)
        .map_err(|e| Error::config(format!("cannot fsync directory {}: {e}", dir.display())))?;
    Ok(())
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
    fn write_atomic_round_trip() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let target = dir.path().join("nested").join("out.txt");

        write_atomic_str(&target, "hello\n")?;
        assert_eq!(std::fs::read_to_string(&target)?, "hello\n");

        write_atomic(&target, &[1, 2, 3])?;
        assert_eq!(std::fs::read(&target)?, vec![1, 2, 3]);

        let siblings: Vec<String> = std::fs::read_dir(dir.path().join("nested"))?
            .filter_map(std::result::Result::ok)
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        assert!(
            !siblings.iter().any(|n| n.contains(".tmp")),
            "stray temp files: {siblings:?}"
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn pure_text_round_trip_streams_bytes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let target = dir.path().join("data.bin");
        let payload: Vec<u8> = (0..10_000u32).map(|i| (i % 251) as u8).collect();
        write_atomic(&target, &payload)?;
        assert_eq!(std::fs::read(&target)?, payload);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn write_atomic_fails_cleanly_when_target_is_dir() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let target = dir.path().join("subdir");
        std::fs::create_dir(&target)?;

        assert!(write_atomic(&target, b"x").is_err());
        Ok(())
    }
}
