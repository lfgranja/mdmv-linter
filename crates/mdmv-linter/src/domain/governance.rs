//! Governance state of a managed repository: gate mode, drift ledger, and the
//! canonical install layout under `.mdmv/`.

use std::path::{Path, PathBuf};

use crate::domain::baseline::Baseline;
use crate::domain::tier::Tier;
use crate::error::{Error, Result};

/// Enforcement mode of the non-worsening gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateMode {
    /// Gate denies every violation outright.
    Strict,
    /// Gate enforces the baseline ratchet (no metric may worsen).
    Ratchet,
}

impl GateMode {
    /// Canonical lowercase name (`strict` / `ratchet`).
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Ratchet => "ratchet",
        }
    }

    /// Parse a mode from its canonical name.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "strict" => Ok(Self::Strict),
            "ratchet" => Ok(Self::Ratchet),
            other => Err(Error::config(format!(
                "unknown gate mode {other:?}: expected \"strict\" or \"ratchet\""
            ))),
        }
    }
}

/// Kind of harness drift detected during verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftKind {
    /// Installed toolchain diverges from the pinned toolchain.
    Toolchain,
    /// A lint rule no longer has any effect.
    DeadRule,
    /// A required CI template is missing.
    CiMissing,
    /// An installed CI template diverges from its canonical source.
    CiDivergent,
    /// A required hook is missing.
    HookMissing,
    /// The baseline is stale relative to recorded policy/tier.
    StaleBaseline,
    /// Threshold configuration drifted from policy.
    ThresholdDrift,
    /// The install process completed only partially.
    InterruptedInstall,
}

impl DriftKind {
    /// Canonical snake_case name used in machine-readable drift output.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Toolchain => "toolchain",
            Self::DeadRule => "dead_rule",
            Self::CiMissing => "ci_missing",
            Self::CiDivergent => "ci_divergent",
            Self::HookMissing => "hook_missing",
            Self::StaleBaseline => "stale_baseline",
            Self::ThresholdDrift => "threshold_drift",
            Self::InterruptedInstall => "interrupted_install",
        }
    }
}

/// A single detected drift with its human-readable remedy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriftEntry {
    /// Which kind of drift was detected.
    pub kind: DriftKind,
    /// Concrete description of what diverged.
    pub detail: String,
    /// Actionable remedy for the operator.
    pub remedy: String,
}

/// Collected drift detected during a verification run.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DriftReport {
    /// All drift entries detected so far.
    pub entries: Vec<DriftEntry>,
}

impl DriftReport {
    /// Whether no drift was detected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append a drift entry.
    pub fn push(&mut self, kind: DriftKind, detail: String, remedy: String) {
        self.entries.push(DriftEntry {
            kind,
            detail,
            remedy,
        });
    }

    /// Process exit code: `0` when clean, `1` when drift was detected.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.is_empty())
    }
}

/// Snapshot of drift status for a managed repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftStatus {
    /// No drift detected on the last check.
    Clean,
    /// Drift was detected on the last check.
    DriftDetected,
    /// Drift has not been checked yet.
    Unknown,
}

/// Governance layout of a managed repository.
#[derive(Debug, Clone)]
pub struct RepoGovernance {
    /// Absolute path of the repository root.
    pub repo_path: PathBuf,
    /// Enforcement tier of the repository.
    pub tier: Tier,
    /// Gate mode in force for the repository.
    pub gate_mode: GateMode,
    /// Deployed baseline, when the repository has one.
    pub baseline: Option<Baseline>,
    /// ISO-8601 timestamp when the harness was installed.
    pub installed_at: String,
    /// Canonical verify script path (`<repo>/.mdmv/verify.sh`).
    pub verify_script_path: PathBuf,
    /// Canonical hook location.
    pub hook_path: PathBuf,
    /// CI template names installed for the repository.
    pub ci_templates: Vec<String>,
    /// Copy of the governing skill, when installed.
    pub skill_ref: Option<PathBuf>,
    /// Path of the governance constitution document.
    pub constitution_path: PathBuf,
    /// Path of the canary fixture root.
    pub fixture_path: PathBuf,
    /// Last known drift status.
    pub drift_status: DriftStatus,
    /// Path of the install lock controlling concurrent mutations
    /// (`<repo>/.mdmv/lock`).
    pub lock_path: PathBuf,
    /// Whether the install lock is currently held.
    pub locked: bool,
    /// Path of the install state file.
    pub snapshot_path: PathBuf,
    /// Path of the gate override audit log
    /// (`<repo>/.mdmv/gate-override.log`).
    pub override_log_path: PathBuf,
}

/// Append `.mdmv/<relative>` to a repository root path.
fn dot_mdmv_child(repo_path: &Path, relative: &str) -> PathBuf {
    repo_path.join(".mdmv").join(relative)
}

impl RepoGovernance {
    /// Build governance for a repository: tier and gate mode are explicit,
    /// the remaining layout splits across canonical `.mdmv/` paths starting
    /// locked with `Unknown` drift and no baseline recorded.
    ///
    /// Canonical paths: `.mdmv/verify.sh`, `.mdmv/lock`,
    /// `.mdmv/state.toml`, `.mdmv/gate-override.log`.
    pub fn new(repo_path: PathBuf, tier: Tier, gate_mode: GateMode) -> Self {
        let verify_script_path = dot_mdmv_child(&repo_path, "verify.sh");
        let hook_path = dot_mdmv_child(&repo_path, "hooks");
        let constitution_path = dot_mdmv_child(&repo_path, "constitution.md");
        let fixture_path = dot_mdmv_child(&repo_path, "fixtures");
        let lock_path = dot_mdmv_child(&repo_path, "lock");
        let snapshot_path = dot_mdmv_child(&repo_path, "state.toml");
        let override_log_path = dot_mdmv_child(&repo_path, "gate-override.log");
        Self {
            repo_path,
            tier,
            gate_mode,
            baseline: None,
            installed_at: String::new(),
            verify_script_path,
            hook_path,
            ci_templates: Vec::new(),
            skill_ref: None,
            constitution_path,
            fixture_path,
            drift_status: DriftStatus::Unknown,
            lock_path,
            locked: true,
            snapshot_path,
            override_log_path,
        }
    }

    /// Whether `.mdmv/lock` is the canonical lock location.
    #[must_use]
    pub fn lock_path_is_canonical(&self) -> bool {
        self.lock_path.ends_with(Path::new(".mdmv").join("lock"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic on failure"
    )]
    fn gate_mode_parses_canonical_names() -> Result<()> {
        assert_eq!(GateMode::parse("strict")?, GateMode::Strict);
        assert_eq!(GateMode::parse("ratchet")?, GateMode::Ratchet);
        Ok(())
    }

    #[test]
    fn gate_mode_rejects_unknown_names() {
        assert!(GateMode::parse("bogus").is_err());
        assert!(GateMode::parse("Strict").is_err());
        assert!(GateMode::parse("").is_err());
    }

    #[test]
    fn gate_mode_canonical_names() {
        assert_eq!(GateMode::Strict.canonical_name(), "strict");
        assert_eq!(GateMode::Ratchet.canonical_name(), "ratchet");
    }

    #[test]
    fn empty_drift_report_exits_zero() {
        let report = DriftReport::default();
        assert!(report.is_empty());
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn pushed_drift_report_exits_one() {
        let mut report = DriftReport::default();
        report.push(
            DriftKind::CiMissing,
            "ci/pedantic.yml missing".to_owned(),
            "re-run mdmv-linter install".to_owned(),
        );
        assert!(!report.is_empty());
        assert_eq!(report.exit_code(), 1);
        assert_eq!(
            report.entries.first().map(|e| e.kind),
            Some(DriftKind::CiMissing)
        );
        assert_eq!(DriftKind::CiMissing.canonical_name(), "ci_missing");
    }

    #[test]
    fn new_uses_canonical_lock_path() {
        let gov = RepoGovernance::new(PathBuf::from("/tmp/repo"), Tier::App, GateMode::Ratchet);
        assert!(gov.lock_path_is_canonical());
        assert_eq!(gov.lock_path, PathBuf::from("/tmp/repo/.mdmv/lock"));
        assert_eq!(
            gov.verify_script_path,
            PathBuf::from("/tmp/repo/.mdmv/verify.sh")
        );
        assert_eq!(
            gov.snapshot_path,
            PathBuf::from("/tmp/repo/.mdmv/state.toml")
        );
        assert_eq!(
            gov.override_log_path,
            PathBuf::from("/tmp/repo/.mdmv/gate-override.log")
        );
        assert!(gov.locked);
        assert_eq!(gov.drift_status, DriftStatus::Unknown);
        assert!(gov.baseline.is_none());
    }

    #[test]
    fn noncanonical_lock_path_is_detected() {
        let mut gov =
            RepoGovernance::new(PathBuf::from("/tmp/repo"), Tier::Embedded, GateMode::Strict);
        gov.lock_path = PathBuf::from("/tmp/repo/lock");
        assert!(!gov.lock_path_is_canonical());
    }
}
