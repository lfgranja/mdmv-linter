//! Commit manifest for `.mdmv/state.toml` and install-state classification (T016).
//!
//! The manifest records exactly which files the harness owns and their digests
//! at install time. It is written **last** in the commit protocol so that its
//! mere existence means "the install performed all preceding writes"; its
//! absence means the run either never happened or was interrupted. Drift is a
//! digest mismatch between the recorded digests and the current disk state.

use std::path::Path;

use sha2::Digest as _;

use crate::error::{Error, Result};
use crate::infra::atomic_fs;

/// An artifact the harness owns: repository-relative path plus its SHA-256
/// digest at install time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactEntry {
    /// Repository-relative path of the artifact (forward slashes).
    pub path: String,
    /// Hex-encoded SHA-256 of the file as installed.
    pub sha256: String,
}

/// The pinned commit manifest persisted at [`Manifest::FILE`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    /// Manifest schema version.
    pub version: u32,
    /// Files installed by the harness, digest-pinned.
    pub artifacts: Vec<ArtifactEntry>,
    /// Gate mode recorded at install time.
    pub gate_mode: String,
    /// Tier recorded at install time.
    pub tier: String,
    /// Policy version recorded at install time.
    pub policy_version: String,
}

impl Manifest {
    /// Repository-relative location of the manifest.
    pub const FILE: &'static str = ".mdmv/state.toml";

    /// Current manifest schema version.
    pub const VERSION: u32 = 1;

    /// Build a manifest at the current schema version.
    pub fn new(
        gate_mode: String,
        tier: String,
        policy_version: String,
        artifacts: Vec<ArtifactEntry>,
    ) -> Self {
        Self {
            version: Self::VERSION,
            artifacts,
            gate_mode,
            tier,
            policy_version,
        }
    }

    /// Load the manifest from `repo_root`, or `Ok(None)` if it is absent.
    pub fn read(repo_root: &Path) -> Result<Option<Self>> {
        let path = repo_root.join(Self::FILE);
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(Error::config(format!(
                    "cannot read {}: {e}",
                    path.display()
                )))
            }
        };
        toml::from_str(&raw)
            .map(Some)
            .map_err(|e| Error::config(format!("cannot parse {}: {e}", path.display())))
    }

    /// Render the manifest as canonical TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| Error::config(format!("cannot serialize manifest: {e}")))
    }

    /// Commit the manifest: the **last** write of an install, atomically.
    pub fn write_last(repo_root: &Path, manifest: &Self) -> Result<()> {
        atomic_fs::write_atomic_str(&repo_root.join(Self::FILE), &manifest.to_toml_string()?)
    }
}

/// Hex-encoded SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha2::Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        std::fmt::Write::write_fmt(&mut hex, format_args!("{byte:02x}")).ok();
    }
    hex
}

/// Hex-encoded SHA-256 digest of the on-disk bytes at `path`.
pub fn digest_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::config(format!("cannot read {}: {e}", path.display())))?;
    Ok(sha256_hex(&bytes))
}

/// Install-state classification, the verdict of the reconciliation check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// No manifest and no harness artifacts: the repo was never managed.
    Unmanaged,
    /// No manifest, but harness artifacts exist: a run was interrupted.
    Interrupted,
    /// Manifest present and every recorded digest matches the disk.
    Clean,
    /// Manifest present but at least one artifact drifted (mismatch/missing).
    Drift,
    /// Manifest present but unparseable TOML: treat as drift needing repair.
    Unparseable,
}

/// Sentinel file marking a partial (interrupted) install.
const HARNESS_MARKER: &str = ".mdmv/verify.sh";

/// Classify the repository state at `repo_root` (state machine over the
/// manifest's presence, validity, and artifact digests).
pub fn verdict(repo_root: &Path) -> Result<Verdict> {
    let manifest_path = repo_root.join(Manifest::FILE);
    let harness_marker = repo_root.join(HARNESS_MARKER);
    let manifest_exists = manifest_path.is_file();
    let marker_present = harness_marker.is_file();

    if !manifest_exists {
        return Ok(if marker_present {
            Verdict::Interrupted
        } else {
            Verdict::Unmanaged
        });
    }

    // Absent (impossible given the existence check) and parse failures both
    // surface as Unparseable rather than a config abort.
    let Ok(manifest) = Manifest::read(repo_root) else {
        return Ok(Verdict::Unparseable);
    };
    let Some(manifest) = manifest else {
        return Ok(Verdict::Unparseable);
    };

    for artifact in &manifest.artifacts {
        let on_disk = repo_root.join(&artifact.path);
        let current = digest_file(&on_disk);
        let match_recorded = current.as_ref().is_ok_and(|d| *d == artifact.sha256);
        if !match_recorded {
            return Ok(Verdict::Drift);
        }
    }

    Ok(Verdict::Clean)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    fn entry(root: &Path, rel: &str) -> ArtifactEntry {
        let abs = root.join(rel);
        ArtifactEntry {
            path: rel.to_owned(),
            sha256: digest_file(&abs).unwrap_or_else(|_| sha256_hex(rel.as_bytes())),
        }
    }

    #[test]
    fn sha256_is_stable_and_well_formed() {
        assert_eq!(sha256_hex(b"hello"), sha256_hex(b"hello"));
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(sha256_hex(b"hello").len(), 64);
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn manifest_round_trip_toml() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mdmv = root.path().join(".mdmv");
        std::fs::create_dir_all(&mdmv)?;
        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname = \"x\"\n")?;

        let manifest = Manifest::new(
            "strict".to_owned(),
            "app-tier".to_owned(),
            "0.1.0".to_owned(),
            vec![entry(root.path(), "Cargo.toml")],
        );
        Manifest::write_last(root.path(), &manifest)?;
        assert!(root.path().join(Manifest::FILE).is_file());

        let loaded = Manifest::read(root.path())?;
        assert_eq!(loaded.as_ref().map(|m| m.version), Some(Manifest::VERSION));
        assert_eq!(loaded.map_or(String::new(), |m| m.gate_mode), "strict");

        // The rendered TOML carries every required field.
        let raw = std::fs::read_to_string(root.path().join(Manifest::FILE))?;
        assert!(raw.contains("sha256"));
        assert!(raw.contains("[artifacts]"));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn verdict_transitions() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mdmv = root.path().join(".mdmv");

        assert_eq!(verdict(root.path())?, Verdict::Unmanaged);

        std::fs::create_dir_all(&mdmv)?;
        std::fs::write(mdmv.join("verify.sh"), "#!/bin/sh\nexit 0\n")?;
        assert_eq!(verdict(root.path())?, Verdict::Interrupted);

        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname = \"x\"\n")?;
        let manifest = Manifest::new(
            "strict".to_owned(),
            "app-tier".to_owned(),
            "0.1.0".to_owned(),
            vec![entry(root.path(), "Cargo.toml")],
        );
        Manifest::write_last(root.path(), &manifest)?;
        assert_eq!(verdict(root.path())?, Verdict::Clean);

        // Drift: mutate a recorded artifact.
        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname = \"y\"\n")?;
        assert_eq!(verdict(root.path())?, Verdict::Drift);

        // Unparseable: corrupt state.toml.
        std::fs::write(mdmv.join("state.toml"), "not = = valid\n")?;
        assert_eq!(verdict(root.path())?, Verdict::Unparseable);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn read_returns_none_when_absent() -> Result<()> {
        let root = tempfile::tempdir()?;
        assert!(Manifest::read(root.path())?.is_none());
        Ok(())
    }
}
