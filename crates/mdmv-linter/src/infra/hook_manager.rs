//! Local git hook installation (Wave 1: lefthook only).

use std::io::Write;
use std::path::Path;

use crate::error::{Error, Result};

/// Manages the local pre-commit hook that runs the canonical verify script.
pub trait HookManager {
    /// Stable manager identifier.
    fn name(&self) -> &'static str;

    /// Install the local pre-commit hook that runs `verify_script`
    /// (the canonical `.mdmv/verify.sh`), without requiring a binary
    /// in the target repository.
    ///
    /// # Errors
    ///
    /// Returns an error when the hook manifest cannot be written.
    fn install(&self, repo_root: &Path, verify_script: &Path) -> Result<()>;
}

/// A pre-commit hook entry rendered into the hook manifest.
struct HookEntry {
    /// Yaml block to ensure is present in the file, so the install is
    /// idempotent.
    block: String,
    /// Marker searched for in the existing file to detect an entry we
    /// already installed.
    marker: String,
}

/// [`HookManager`] backed by `lefthook.yml` in the repository root.
///
/// The verification runs the canonical script directly, so the target
/// repository needs no `mdmv-linter` binary installed.
#[derive(Debug, Default, Clone, Copy)]
pub struct LefthookManager;

impl HookManager for LefthookManager {
    fn name(&self) -> &'static str {
        "lefthook"
    }

    fn install(&self, repo_root: &Path, verify_script: &Path) -> Result<()> {
        let manifest_path = repo_root.join("lefthook.yml");
        let entry = Self::entry(repo_root, verify_script);

        let raw = std::fs::read_to_string(&manifest_path).unwrap_or_default();

        if !raw.contains(&entry.marker) {
            let merged = merge_manifest(&raw, &entry.block);
            if let Some(parent) = manifest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            write_atomically(&manifest_path, &merged)?;
        }

        Ok(())
    }
}

impl LefthookManager {
    /// Build the pre-commit hook entry for this marker set.
    fn entry(repo_root: &Path, verify_script: &Path) -> HookEntry {
        let command = display_command(repo_root, verify_script);
        let block = format!("pre-commit:\n  hooks:\n    mdmv-linter:\n      run: {command}\n");
        HookEntry {
            marker: format!("      run: {command}"),
            block,
        }
    }
}

/// Render the verify-script invocation for the hook entry.
///
/// The path is made relative to `repo_root` when possible, because
/// lefthook executes commands with the repository root as working
/// directory; absolute invocations stay absolute for worktrees living
/// outside `repo_root`.
fn display_command(repo_root: &Path, verify_script: &Path) -> String {
    match verify_script.strip_prefix(repo_root) {
        Ok(relative) => relative.to_string_lossy().into_owned(),
        Err(_) => verify_script.to_string_lossy().into_owned(),
    }
}

/// Append `block` to `raw`, or return `block` alone when `raw` is empty.
fn merge_manifest(raw: &str, block: &str) -> String {
    if raw.is_empty() {
        block.to_owned()
    } else if raw.ends_with('\n') {
        format!("{raw}{block}")
    } else {
        format!("{raw}\n{block}")
    }
}

/// Write file contents through a sibling temporary file, then rename it
/// into place, so an interrupted write never leaves a truncated file.
fn write_atomically(path: &Path, contents: &str) -> Result<()> {
    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let file_name = path
        .file_name()
        .map(std::ffi::OsStr::to_os_string)
        .unwrap_or_default();

    let temp = tempfile::NamedTempFile::new_in(&parent)?;
    let mut handle = temp.as_file();
    handle.write_all(contents.as_bytes())?;
    handle.flush()?;

    // Give the temporary file the same access rights as a plain created file.
    let target = parent.join(file_name);
    temp.persist(&target)
        .map_err(|err| err.error)
        .map_err(Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{display_command, merge_manifest, LefthookManager};

    #[test]
    fn entry_marks_persistent_hook() {
        let entry = LefthookManager::entry(Path::new("/repo"), Path::new("/repo/.mdmv/verify.sh"));
        assert!(entry.block.contains("mdmv-linter:"));
        assert!(entry.marker.contains(".mdmv/verify.sh"));
    }

    #[test]
    fn merge_appends_to_existing_manifest() {
        let merged = merge_manifest("pre-existing: true\n", "pre-commit:\n");
        assert_eq!(merged, "pre-existing: true\npre-commit:\n");
    }

    #[test]
    fn merge_writes_fresh_manifest_when_empty() {
        let merged = merge_manifest("", "pre-commit:\n");
        assert_eq!(merged, "pre-commit:\n");
    }

    #[test]
    fn command_is_relative_to_repo_root() {
        let command = display_command(Path::new("/repo"), Path::new("/repo/.mdmv/verify.sh"));
        assert_eq!(command, ".mdmv/verify.sh");
    }
}
