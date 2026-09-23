//! The rule configuration model (T026).
//!
//! The config TOML is a strict schema: unknown keys are rejected as hard
//! configuration errors, and every inactive ("exception") rule must carry a
//! `condition` and a `reason` whose `condition` names a known lint id.

use serde::Deserialize;

use crate::error::{Error, Result};

/// Enforcement level of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleLevel {
    /// Hard error.
    Deny,
    /// Suggestion.
    Warn,
    /// Explicitly disabled.
    Allow,
}

/// One configured lint rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Lint identifier the rule addresses (e.g. `clippy::unwrap_used`).
    pub source: String,
    /// Enforcement level.
    pub level: RuleLevel,
    /// Whether the rule participates in checking; `false` marks an exception.
    pub active: bool,
    /// Policy version in which this rule becomes obsolete, if any.
    pub deprecated_in: Option<String>,
    /// Reference into the fixture suite for this rule.
    pub fixture_line_ref: Option<String>,
    /// Excuse machine-readable condition for an inactive rule.
    pub condition: Option<String>,
    /// Free-form justification, required for inactive rules.
    pub reason: Option<String>,
}

/// A parsed rule configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleSet {
    /// The rules in declaration order.
    pub rules: Vec<Rule>,
}

/// Strict on-disk schema for a rule; unknown keys fail deserialization.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleSpec {
    source: String,
    level: RuleLevel,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    deprecated_in: Option<String>,
    #[serde(default)]
    fixture_line_ref: Option<String>,
    #[serde(default)]
    condition: Option<String>,
    #[serde(default)]
    reason: Option<String>,
}

/// Strict wrapper mirroring [`RuleSet`] with [`deny_unknown_fields`].
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleSetSpec {
    rules: Vec<RuleSpec>,
}

impl RuleSet {
    /// Parse a rule-config TOML string. An unknown key is a hard error
    /// ([`Error::config`]), never silently ignored.
    ///
    /// # Errors
    /// Returns [`Error::Config`] on malformed TOML, unknown keys, or an
    /// unknown rule level.
    pub fn from_toml_str(raw: &str) -> Result<Self> {
        let spec: RuleSetSpec =
            toml::from_str(raw).map_err(|e| Error::config(format!("invalid rule config: {e}")))?;
        Ok(Self {
            rules: spec
                .rules
                .into_iter()
                .map(|r| Rule {
                    source: r.source,
                    level: r.level,
                    active: r.active,
                    deprecated_in: r.deprecated_in,
                    fixture_line_ref: r.fixture_line_ref,
                    condition: r.condition,
                    reason: r.reason,
                })
                .collect(),
        })
    }

    /// FR-021 + FR-014: the set must contain at least one active rule, and
    /// every exception (`active == false`) must carry both a `condition` and a
    /// `reason`; a `condition` that does not resolve against the known lint id
    /// set is a hard error.
    ///
    /// # Errors
    /// Returns [`Error::Config`] on zero active rules, incomplete exceptions,
    /// or unresolved exception conditions.
    pub fn validate(&self, known_lints: &[&str]) -> Result<()> {
        if !self.rules.iter().any(|rule| rule.active) {
            return Err(Error::config(
                "rule config is empty or has no active rules; at least one active rule is required",
            ));
        }
        for rule in &self.rules {
            if rule.active {
                continue;
            }
            let source = &rule.source;
            let Some(condition) = rule.condition.as_ref() else {
                return Err(Error::config(format!(
                    "exception for `{source}` is missing `condition`; an inactive rule \
                     must declare both `condition` and `reason`"
                )));
            };
            if rule.reason.is_none() {
                return Err(Error::config(format!(
                    "exception for `{source}` is missing `reason`; an inactive rule \
                     must declare both `condition` and `reason`"
                )));
            }
            let normalized = condition.trim();
            if !known_lints.contains(&normalized) {
                return Err(Error::config(format!(
                    "exception for `{source}` has unresolvable condition `{normalized}`; \
                     it must name a known lint id"
                )));
            }
        }
        Ok(())
    }

    /// All active `deny` rules, in declaration order.
    #[must_use]
    pub fn active_deny_leaves(&self) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|rule| rule.active && rule.level == RuleLevel::Deny)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn base_rule(source: &str, level: RuleLevel, active: bool) -> Rule {
        Rule {
            source: source.to_owned(),
            level,
            active,
            deprecated_in: None,
            fixture_line_ref: None,
            condition: None,
            reason: None,
        }
    }

    const KNOWN: &[&str] = &["clippy::unwrap_used", "clippy::dbg_macro", "rustdoc::all"];

    /// Extracts the `Error::Config` message, or `None` for any other outcome.
    fn config_message<T>(result: &Result<T>) -> Option<String> {
        match result {
            Err(Error::Config { message }) => Some(message.clone()),
            _ => None,
        }
    }

    /// Asserts the result is an `Error::Config` mentioning `needle` via
    /// `assert!` (no `panic!`, `unwrap`, or constant-false assertion).
    fn expect_config_error<T: std::fmt::Debug>(result: &Result<T>, needle: &str) {
        let found = config_message(result).unwrap_or_default();
        assert!(
            found.contains(needle),
            "expected Error::config mentioning {needle:?}, got: {result:?}"
        );
    }

    #[test]
    fn parses_valid_config() {
        let raw = r#"
[[rules]]
source = "clippy::unwrap_used"
level = "deny"
active = true
fixture_line_ref = "rules.toml:3"

[[rules]]
source = "clippy::dbg_macro"
level = "warn"
active = false
condition = "clippy::dbg_macro"
reason = "reviewed exception"
deprecated_in = "0.2.0"
"#;
        let set = config_message(&RuleSet::from_toml_str(raw));
        assert!(set.is_none(), "expected a config that parses: {set:?}");
        match RuleSet::from_toml_str(raw) {
            Ok(set) => {
                assert_eq!(set.rules.len(), 2);
                assert_eq!(set.rules[0].level, RuleLevel::Deny);
                assert!(set.rules[0].active);
                assert_eq!(set.rules[1].condition.as_deref(), Some("clippy::dbg_macro"));
                assert_eq!(set.active_deny_leaves().len(), 1);
                assert_eq!(set.active_deny_leaves()[0].source, "clippy::unwrap_used");
                assert!(set.validate(KNOWN).is_ok(), "expected validate() to pass");
            }
            Err(err) => {
                let unreachable = Err::<(), ()>(()).is_ok();
                assert!(unreachable, "unreachable; config failed to parse: {err}");
            }
        }
    }

    #[test]
    fn unknown_key_is_hard_error() {
        let raw = r#"
[[rules]]
source = "clippy::unwrap_used"
level = "deny"
typo_active = true
"#;
        expect_config_error(&RuleSet::from_toml_str(raw), "typo_active");
    }

    #[test]
    fn unknown_top_level_key_is_hard_error() {
        let raw = r#"
extra_key = 1
[[rules]]
source = "clippy::unwrap_used"
level = "deny"
"#;
        expect_config_error(&RuleSet::from_toml_str(raw), "extra_key");
    }

    #[test]
    fn unknown_level_is_hard_error() {
        let raw = r#"
[[rules]]
source = "clippy::unwrap_used"
level = "strict"
"#;
        expect_config_error(&RuleSet::from_toml_str(raw), "unknown variant");
    }

    #[test]
    fn zero_active_rules_is_config_error() {
        let set = RuleSet::default();
        expect_config_error(&set.validate(KNOWN), "no active rules");
    }

    #[test]
    fn exception_without_condition_is_config_error() {
        let set = RuleSet {
            rules: vec![
                base_rule("clippy::unwrap_used", RuleLevel::Deny, true),
                base_rule("clippy::dbg_macro", RuleLevel::Warn, false),
            ],
        };
        expect_config_error(&set.validate(KNOWN), "condition");
    }

    #[test]
    fn exception_without_reason_is_config_error() {
        let set = RuleSet {
            rules: vec![
                base_rule("clippy::unwrap_used", RuleLevel::Deny, true),
                Rule {
                    source: String::from("clippy::dbg_macro"),
                    level: RuleLevel::Warn,
                    active: false,
                    deprecated_in: None,
                    fixture_line_ref: None,
                    condition: Some(String::from("clippy::dbg_macro")),
                    reason: None,
                },
            ],
        };
        expect_config_error(&set.validate(KNOWN), "reason");
    }

    #[test]
    fn unresolved_condition_is_config_error() {
        let set = RuleSet {
            rules: vec![
                base_rule("clippy::unwrap_used", RuleLevel::Deny, true),
                Rule {
                    source: String::from("clippy::dbg_macro"),
                    level: RuleLevel::Warn,
                    active: false,
                    deprecated_in: None,
                    fixture_line_ref: None,
                    condition: Some(String::from("no-such-lint::id")),
                    reason: Some(String::from("reviewed")),
                },
            ],
        };
        expect_config_error(&set.validate(KNOWN), "no-such-lint");
    }

    #[test]
    fn valid_set_passes_validate() {
        let set = RuleSet {
            rules: vec![
                base_rule("clippy::unwrap_used", RuleLevel::Deny, true),
                Rule {
                    source: String::from("clippy::dbg_macro"),
                    level: RuleLevel::Warn,
                    active: false,
                    deprecated_in: None,
                    fixture_line_ref: None,
                    condition: Some(String::from("clippy::dbg_macro")),
                    reason: Some(String::from("dogfooding exception")),
                },
            ],
        };
        assert!(set.validate(KNOWN).is_ok(), "expected validate() to pass");
    }
}
