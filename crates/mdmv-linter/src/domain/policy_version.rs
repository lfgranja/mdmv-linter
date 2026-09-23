//! Policy version handling and the FR-015 compatibility gate.

use std::fmt;

use crate::error::{Error, Result};

/// A semver version carried by a policy bundle.
///
/// Ordering is derived from the inner [`semver::Version`], which already
/// satisfies `PartialOrd`/`Ord` component-wise (major/minor/patch/pre).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PolicyVersion(pub semver::Version);

impl PolicyVersion {
    /// The policy version the shipped harness understands natively.
    pub const CURRENT: &'static str = "0.1.0";

    /// Parse a raw semver string into a policy version.
    ///
    /// # Errors
    /// Returns [`Error::Config`] when `raw` is not a valid semver version.
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        semver::Version::parse(trimmed)
            .map(Self)
            .map_err(|e| Error::config(format!("invalid policy version `{trimmed}`: {e}")))
    }

    /// The version this harness ships with.
    ///
    /// # Errors
    /// Returns [`Error::Config`] only if the built-in constant is malformed,
    /// which would be a packaging defect surfaced as a config error.
    pub fn current() -> Result<Self> {
        Self::parse(Self::CURRENT)
    }

    /// Canonical string form of this version.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// FR-015 gate: this policy is accepted by `tool` iff its major is greater
    /// than the tool's major, or the majors are equal and its minor is greater
    /// than or equal to the tool's minor. The patch component is ignored.
    #[must_use]
    pub fn is_accepted_by(&self, tool: &Self) -> bool {
        let (a, b) = (&self.0, &tool.0);
        a.major > b.major || (a.major == b.major && a.minor >= b.minor)
    }
}

impl fmt::Display for PolicyVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper for `Ok` results, avoiding `unwrap`/`panic` (both deny here).
    fn assert_resolves(result: &Result<PolicyVersion>, expected: &str) {
        let ok = matches!(result, Ok(version) if version.as_str() == expected);
        assert!(ok, "expected Ok({expected}), got: {result:?}");
    }

    #[test]
    fn parse_accepts_valid_semver() {
        assert_resolves(&PolicyVersion::parse("1.2.3"), "1.2.3");
    }

    #[test]
    fn parse_rejects_non_semver() {
        assert!(PolicyVersion::parse("not-a-version").is_err());
        assert!(PolicyVersion::parse("").is_err());
    }

    #[test]
    fn current_matches_constant() {
        assert_resolves(&PolicyVersion::current(), PolicyVersion::CURRENT);
    }

    #[test]
    fn display_matches_as_str() {
        let result = PolicyVersion::current();
        let text = match &result {
            Ok(version) => version.to_string(),
            Err(_) => String::new(),
        };
        let as_str = match &result {
            Ok(version) => version.as_str(),
            Err(_) => String::new(),
        };
        assert_eq!(text, as_str);
    }

    #[test]
    fn acceptance_ignores_patch() {
        let tool = PolicyVersion::parse("0.1.0");
        let accepted = match &tool {
            Ok(tool) => match PolicyVersion::parse("0.1.9") {
                Ok(policy) => policy.is_accepted_by(tool),
                Err(_) => false,
            },
            Err(_) => false,
        };
        assert!(accepted, "same minor, differing patch must be accepted");
    }

    #[test]
    fn acceptance_major_minor_edges() {
        let tool = match PolicyVersion::parse("1.2.0") {
            Ok(tool) => tool,
            Err(_) => PolicyVersion(semver::Version::new(1, 2, 0)),
        };
        let accepts = |candidate: &str| -> bool {
            match PolicyVersion::parse(candidate) {
                Ok(policy) => policy.is_accepted_by(&tool),
                Err(_) => false,
            }
        };
        let rejects = |candidate: &str| -> bool {
            match PolicyVersion::parse(candidate) {
                Ok(policy) => !policy.is_accepted_by(&tool),
                Err(_) => false,
            }
        };
        assert!(rejects("1.1.99"), "lower minor on same major is rejected");
        assert!(accepts("1.2.777"), "equal minor is accepted");
        assert!(accepts("1.3.0"), "higher minor is accepted");
        assert!(accepts("2.0.0"), "higher major, lower minor is accepted");
        assert!(rejects("0.9.0"), "lower major is rejected");
    }

    #[test]
    fn ordering_follows_inner_semver() {
        let results = [
            PolicyVersion::parse("1.0.0"),
            PolicyVersion::parse("1.0.1"),
            PolicyVersion::parse("0.1.0"),
        ];
        if results.iter().all(Result::is_ok) {
            let values: Vec<PolicyVersion> = results.into_iter().flatten().collect();
            assert!(
                values[0] < values[1] && values[2] < values[0],
                "semver-derived ordering violated"
            );
        } else {
            let parse_works = PolicyVersion::parse("1.0.0").is_ok();
            assert!(parse_works, "unreachable; ordering test failed to parse");
        }
    }
}
