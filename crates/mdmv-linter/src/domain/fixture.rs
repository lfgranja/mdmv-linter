//! Test fixtures validating the clippy lint rules through intentional
//! violations captured with `#[expect(...)]` canary annotations.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// One recorded canary violation inside a fixture crate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationRecord {
    /// Lint group the canary is expected to fire into (e.g. `clippy::pedantic`).
    pub lint_group: String,
    /// Expected severity when the lint fires.
    pub severity: String,
    /// Source reference for the violation site, e.g. `src/lib.rs:12`.
    pub line_ref: String,
    /// Exact compiler diagnostic text expected for the canary.
    pub expected_text: String,
}

/// Whether the fixture crate must fail or pass compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedCompile {
    /// Compilation must fail (canaries fire as diagnostics).
    Fail,
    /// Compilation must pass (no unfulfilled canaries remain).
    Pass,
}

/// A self-contained fixture crate exercising canary lints.
#[derive(Debug, Clone)]
pub struct Fixture {
    /// Directory containing the fixture `Cargo.toml`, sources, and `clippy.toml`.
    pub fixture_dir: PathBuf,
    /// Canary violations the fixture is expected to produce.
    pub violations: Vec<ViolationRecord>,
    /// Whether the fixture crate must compile or fail.
    pub expected_compile_result: ExpectedCompile,
    /// Canonical target name of the fixture crate.
    pub config_target: String,
}

impl Fixture {
    /// Build a fixture that must fail compilation, with the default target.
    pub fn new(fixture_dir: PathBuf, violations: Vec<ViolationRecord>) -> Self {
        Self {
            fixture_dir,
            violations,
            expected_compile_result: ExpectedCompile::Fail,
            config_target: "self".to_owned(),
        }
    }

    /// Structural validation: the directory must contain a `clippy.toml` and
    /// at least one canary record.
    pub fn validate(&self) -> Result<()> {
        let clippy_toml = Path::new(&self.fixture_dir).join("clippy.toml");
        if !clippy_toml.is_file() {
            return Err(Error::config(format!(
                "fixture at {} is invalid: missing required clippy.toml",
                clippy_toml.display()
            )));
        }
        if self.violations.is_empty() {
            return Err(Error::config(format!(
                "fixture at {} is invalid: must declare at least one canary violation",
                self.fixture_dir.display()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> ViolationRecord {
        ViolationRecord {
            lint_group: "clippy::pedantic".to_owned(),
            severity: "warn".to_owned(),
            line_ref: "src/lib.rs:12".to_owned(),
            expected_text: "unused crate dependency".to_owned(),
        }
    }

    /// Create a temp directory containing a `clippy.toml`.
    fn dir_with_clippy_toml() -> std::io::Result<PathBuf> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("clippy.toml"), b"")?;
        let path = dir.path().to_path_buf();
        let leaked = PathBuf::from(path.to_string_lossy().into_owned());
        std::mem::forget(dir);
        Ok(leaked)
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic on failure"
    )]
    fn validate_accepts_well_formed_fixture() -> Result<()> {
        let dir = dir_with_clippy_toml().map_err(|err| Error::config(err.to_string()))?;
        let fixture = Fixture::new(dir.clone(), vec![record()]);
        assert!(fixture.validate().is_ok());
        drop(std::fs::remove_dir_all(dir));
        Ok(())
    }

    #[test]
    fn validate_rejects_missing_clippy_toml() {
        let fixture = Fixture::new(PathBuf::from("/nonexistent-mdmv-fixture"), vec![record()]);
        assert!(fixture.validate().is_err());
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic on failure"
    )]
    fn validate_rejects_empty_canary_list() -> Result<()> {
        let dir = dir_with_clippy_toml().map_err(|err| Error::config(err.to_string()))?;
        let fixture = Fixture::new(dir.clone(), Vec::new());
        assert!(fixture.validate().is_err());
        drop(std::fs::remove_dir_all(dir));
        Ok(())
    }

    #[test]
    fn new_defaults_to_fail_compile() {
        let fixture = Fixture::new(PathBuf::from("fixtures/x"), vec![record()]);
        assert_eq!(fixture.expected_compile_result, ExpectedCompile::Fail);
        assert_eq!(fixture.config_target, "self");
    }
}
