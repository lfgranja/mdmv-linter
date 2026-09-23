//! Repo-wide violation counts and the `.mdmv/baseline.toml` baseline.
//!
//! The baseline is the input to the FR-012 non-worsening gate: current scan
//! metrics are compared against the recorded counts and the gate fails only
//! when metrics worsen (`worsened` dominates).

use std::collections::BTreeMap;

use crate::domain::policy_version::PolicyVersion;
use crate::domain::tier::Tier;
use crate::error::{Error, Result};

/// Per-lint violation counts plus the repo-wide total from a canonical scan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViolationCounts {
    /// Total number of violations across all lints.
    pub total: u64,
    /// Violation count keyed by lint name.
    pub per_lint: BTreeMap<String, u64>,
}

impl ViolationCounts {
    /// Build counts from `(lint, count)` pairs; `total` is the sum.
    pub fn from_pairs<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (String, u64)>,
    {
        let mut total: u64 = 0;
        let mut per_lint = BTreeMap::new();
        for (lint, count) in pairs {
            total = total.saturating_add(count);
            per_lint.insert(lint, count);
        }
        Self { total, per_lint }
    }
}

fn sum_counts(per_lint: &BTreeMap<String, u64>) -> u64 {
    per_lint.values().copied().fold(0, u64::saturating_add)
}

/// Serialized baseline recorded from the canonical scan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Baseline {
    /// Repo-wide violation total; must equal the sum over `per_lint`.
    pub total_violations: u64,
    /// Violation count keyed by lint name.
    pub per_lint: BTreeMap<String, u64>,
    /// ISO-8601 timestamp at which the baseline was recorded.
    pub recorded_at: String,
    /// Canonical tier name at recording time.
    pub tier: String,
    /// Canonical policy version at recording time.
    pub policy_version: String,
}

impl Baseline {
    /// Canonical file path, relative to the repository root.
    pub const FILE: &'static str = ".mdmv/baseline.toml";

    /// Parse a baseline from TOML text.
    pub fn from_toml_str(raw: &str) -> Result<Self> {
        toml::from_str(raw)
            .map_err(|err| Error::config(format!("failed to parse {} as TOML: {err}", Self::FILE)))
    }

    /// Serialize the baseline to TOML text.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string(self)
            .map_err(|err| Error::config(format!("failed to serialize baseline as TOML: {err}")))
    }

    /// Structural validation: the declared total must equal the per-lint sum.
    pub fn validate(&self) -> Result<()> {
        match self.total_violations {
            expected if expected == sum_counts(&self.per_lint) => Ok(()),
            _ => Err(Error::config(format!(
                "{} is inconsistent: total_violations={} does not equal declared per-lint sum {}",
                Self::FILE,
                self.total_violations,
                sum_counts(&self.per_lint)
            ))),
        }
    }

    /// Build a baseline from scan output.
    pub fn from_counts(
        counts: &ViolationCounts,
        tier: &Tier,
        policy: &PolicyVersion,
        recorded_at: String,
    ) -> Self {
        Self {
            total_violations: counts.total,
            per_lint: counts.per_lint.clone(),
            recorded_at,
            tier: tier.canonical_name().to_owned(),
            policy_version: policy.to_string(),
        }
    }

    /// Extract the metrics view of this baseline.
    pub fn counts(&self) -> ViolationCounts {
        ViolationCounts {
            total: self.total_violations,
            per_lint: self.per_lint.clone(),
        }
    }
}

/// Outcome of the FR-012 non-worsening comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsStatus {
    /// Current metrics are strictly better than the baseline.
    Improved,
    /// At least one metric is worse than the baseline.
    Worsened,
    /// Current metrics equal the baseline.
    Stable,
}

impl MetricsStatus {
    /// Deterministic status token used in gate output.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Improved => "improved",
            Self::Worsened => "worsened",
            Self::Stable => "stable",
        }
    }

    /// Process exit code: `1` for worsened metrics, `0` otherwise.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Worsened => 1,
            Self::Improved | Self::Stable => 0,
        }
    }
}

/// Compare current scan counts against baseline counts.
///
/// `worsened` dominates: any lint whose current count exceeds the baseline
/// count, or a higher total, yields [`MetricsStatus::Worsened`].
#[must_use]
pub fn compare(current: &ViolationCounts, baseline: &ViolationCounts) -> MetricsStatus {
    let mut worsened = false;
    let mut improved = false;
    let lints = current.per_lint.keys().chain(baseline.per_lint.keys());
    for lint in lints {
        let now = current.per_lint.get(lint).copied().unwrap_or(0);
        let then = baseline.per_lint.get(lint).copied().unwrap_or(0);
        match now.cmp(&then) {
            std::cmp::Ordering::Greater => worsened = true,
            std::cmp::Ordering::Less => improved = true,
            std::cmp::Ordering::Equal => {}
        }
    }
    match current.total.cmp(&baseline.total) {
        std::cmp::Ordering::Greater => worsened = true,
        std::cmp::Ordering::Less => improved = true,
        std::cmp::Ordering::Equal => {}
    }
    if worsened {
        MetricsStatus::Worsened
    } else if improved {
        MetricsStatus::Improved
    } else {
        MetricsStatus::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u64)]) -> ViolationCounts {
        ViolationCounts::from_pairs(
            pairs
                .iter()
                .map(|(lint, count)| ((*lint).to_owned(), *count)),
        )
    }

    fn sample(pairs: &[(&str, u64)], total: u64) -> Baseline {
        Baseline {
            total_violations: total,
            per_lint: counts(pairs).per_lint,
            recorded_at: "2026-09-23T00:00:00Z".to_owned(),
            tier: "app".to_owned(),
            policy_version: "1.0.0".to_owned(),
        }
    }

    #[test]
    fn validate_accepts_consistent_total() {
        let baseline = sample(&[("lint.clone", 3), ("lint.doc", 4)], 7);
        assert!(baseline.validate().is_ok());
    }

    #[test]
    fn validate_rejects_inconsistent_total() {
        let baseline = sample(&[("lint.clone", 3)], 9);
        assert!(baseline.validate().is_err());
    }

    #[test]
    fn validate_accepts_zero_total() {
        let baseline = Baseline {
            total_violations: 0,
            per_lint: BTreeMap::new(),
            recorded_at: "t".to_owned(),
            tier: "lib".to_owned(),
            policy_version: "1.0.0".to_owned(),
        };
        assert!(baseline.validate().is_ok());
    }

    #[test]
    fn from_pairs_totals_the_sum() {
        let c = counts(&[("a", 2), ("b", 5)]);
        assert_eq!(c.total, 7);
        assert_eq!(c.per_lint.get("b").copied(), Some(5));
    }

    #[test]
    fn worsened_dominates_improved() {
        let baseline = counts(&[("lint.a", 3), ("lint.b", 1)]);
        let current = counts(&[("lint.a", 1), ("lint.b", 4)]);
        assert_eq!(compare(&current, &baseline), MetricsStatus::Worsened);
    }

    #[test]
    fn stable_when_counts_match() {
        let baseline = counts(&[("lint.a", 3)]);
        let current = counts(&[("lint.a", 3)]);
        assert_eq!(compare(&current, &baseline), MetricsStatus::Stable);
    }

    #[test]
    fn improved_when_all_counts_drop() {
        let baseline = counts(&[("lint.a", 3), ("lint.b", 2)]);
        let current = counts(&[("lint.a", 1), ("lint.b", 0)]);
        assert_eq!(compare(&current, &baseline), MetricsStatus::Improved);
    }

    #[test]
    fn worse_total_alone_is_worsened() {
        let baseline = counts(&[("lint.a", 2)]);
        let current = counts(&[("lint.new", 1), ("lint.a", 2)]);
        assert_eq!(compare(&current, &baseline), MetricsStatus::Worsened);
    }

    #[test]
    fn status_tokens_and_exit_codes() {
        assert_eq!(MetricsStatus::Improved.token(), "improved");
        assert_eq!(MetricsStatus::Worsened.token(), "worsened");
        assert_eq!(MetricsStatus::Stable.token(), "stable");
        assert_eq!(MetricsStatus::Worsened.exit_code(), 1);
        assert_eq!(MetricsStatus::Stable.exit_code(), 0);
        assert_eq!(MetricsStatus::Improved.exit_code(), 0);
    }

    #[test]
    fn from_counts_marshals_tier_and_policy() {
        let c = counts(&[("lint.a", 3)]);
        let tier = match Tier::parse("app") {
            Ok(t) => t,
            Err(_) => Tier::App,
        };
        let policy = PolicyVersion(semver::Version::new(1, 0, 0));
        let baseline = Baseline::from_counts(&c, &tier, &policy, "t".to_owned());
        assert_eq!(baseline.counts(), c);
        assert_eq!(baseline.tier, "app");
        assert_eq!(baseline.policy_version, "1.0.0");
        assert!(baseline.validate().is_ok());
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic on failure"
    )]
    fn toml_round_trip() -> Result<()> {
        let baseline = sample(&[("lint.a", 3)], 3);
        let text = baseline.to_toml_string()?;
        let parsed = Baseline::from_toml_str(&text)?;
        assert_eq!(parsed.total_violations, 3);
        assert_eq!(parsed.per_lint.get("lint.a").copied(), Some(3));
        Ok(())
    }
}
