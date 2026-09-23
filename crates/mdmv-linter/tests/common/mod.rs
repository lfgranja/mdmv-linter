//! Shared helpers for integration and contract tests.
//!
//! Test code is held to the same lint policy as the crate, so helpers avoid
//! `unwrap`/`expect`/`panic!` and return `Result` where failure is possible.
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// Error type used by test helpers.
pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

/// A clean, documented library source with zero violations.
pub const CLEAN_LIB: &str = "//! Sandbox crate.\n\
/// Adds one.\n\
#[must_use]\n\
pub fn bump(value: u8) -> u8 {\n\
    value.saturating_add(1)\n\
}\n";

/// The `mdmv-linter` binary under test.
pub fn binary() -> TestResult<Command> {
    Ok(Command::cargo_bin("mdmv-linter")?)
}

/// A sandbox cargo repository in a temporary directory.
pub struct Sandbox {
    dir: TempDir,
}

impl Sandbox {
    /// Create a single-crate sandbox whose `src/lib.rs` is `lib_rs`.
    pub fn single_crate(lib_rs: &str) -> TestResult<Self> {
        let sandbox = Self::empty()?;
        sandbox.write("Cargo.toml", PACKAGE_TOML)?;
        sandbox.write("src/lib.rs", lib_rs)?;
        Ok(sandbox)
    }

    /// Create an empty temporary directory (no manifest).
    pub fn empty() -> TestResult<Self> {
        Ok(Self {
            dir: tempfile::tempdir()?,
        })
    }

    /// Root path of the sandbox.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a file relative to the sandbox root, creating parents.
    pub fn write(&self, relative: &str, contents: &str) -> TestResult<()> {
        let target = self.path().join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, contents)?;
        Ok(())
    }

    /// Read a file relative to the sandbox root.
    pub fn read(&self, relative: &str) -> TestResult<String> {
        Ok(std::fs::read_to_string(self.path().join(relative))?)
    }

    /// Whether a path exists relative to the sandbox root.
    #[must_use]
    pub fn exists(&self, relative: &str) -> bool {
        self.path().join(relative).exists()
    }

    /// Run the binary with `args` in the sandbox and return the assertion.
    pub fn run(&self, args: &[&str]) -> TestResult<assert_cmd::assert::Assert> {
        let mut cmd = binary()?;
        cmd.current_dir(self.path());
        for arg in args {
            cmd.arg(arg);
        }
        Ok(cmd.assert())
    }

    /// A content digest over every non-`target/` file (path + bytes).
    pub fn tree_digest(&self) -> TestResult<String> {
        use sha2::{Digest, Sha256};

        let mut files: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        collect(self.path(), self.path(), &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();
        for (relative, bytes) in files {
            hasher.update(relative.to_string_lossy().as_bytes());
            hasher.update([0]);
            hasher.update(&bytes);
            hasher.update([0xff]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

const PACKAGE_TOML: &str =
    "[package]\nname = \"sandbox\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn collect(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> TestResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "target") {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, out)?;
        } else {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            out.push((relative, std::fs::read(&path)?));
        }
    }
    Ok(())
}
