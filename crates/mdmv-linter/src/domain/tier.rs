//! Crate tier classification, overrides, and FR-004/FR-021 resolution.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Governance tier for a crate (T024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Shared library tier.
    Lib,
    /// Binary/application tier.
    App,
    /// Embedded/no-std tier.
    Embedded,
}

impl Tier {
    /// Parse a tier name, accepting canonical names and warn-and-mapping the
    /// deprecated aliases (`minimal` -> Lib, `standard` -> App,
    /// `maximum` -> Embedded).
    ///
    /// # Errors
    /// Returns [`Error::Config`] for unknown tier names.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "lib" => Ok(Self::Lib),
            "app" => Ok(Self::App),
            "embedded" => Ok(Self::Embedded),
            kind @ ("minimal" | "standard" | "maximum") if Self::is_deprecated_alias(kind) => {
                Self::map_deprecated_alias(kind)
                    .ok_or_else(|| Error::config(format!("unknown tier `{raw}`")))
            }
            other => Err(Error::config(format!(
                "unknown tier `{other}`; expected one of `lib`, `app`, `embedded`"
            ))),
        }
    }

    /// The canonical, user-facing name of this tier.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Lib => "lib",
            Self::App => "app",
            Self::Embedded => "embedded",
        }
    }

    /// True when `raw` is a deprecated alias (`minimal`, `standard`,
    /// `maximum`), all of which still parse with a warning.
    #[must_use]
    pub fn is_deprecated_alias(raw: &str) -> bool {
        matches!(raw.trim(), "minimal" | "standard" | "maximum")
    }

    /// Map a deprecated alias to its canonical tier, if it is one.
    #[must_use]
    pub fn map_deprecated_alias(raw: &str) -> Option<Self> {
        match raw.trim() {
            "minimal" => Some(Self::Lib),
            "standard" => Some(Self::App),
            "maximum" => Some(Self::Embedded),
            _ => None,
        }
    }
}

/// An explicit per-crate tier override from `--tier <crate>=<tier>` (T025).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierOverride {
    /// Crate name the override applies to.
    pub crate_name: String,
    /// The tier forced onto that crate.
    pub tier: Tier,
}

/// Parse the repeatable `--tier` CLI values.
///
/// Each value must follow the grammar `--tier <crate>=<tier>`; a value without
/// `=`, an empty crate name, or an unknown/empty tier is a config error.
///
/// # Errors
/// Returns [`Error::Config`] with an actionable message.
pub fn parse_tier_overrides(values: &[String]) -> Result<Vec<TierOverride>> {
    values
        .iter()
        .map(|value| parse_tier_override(value))
        .collect()
}

fn parse_tier_override(value: &str) -> Result<TierOverride> {
    let Some((crate_name_raw, tier_raw)) = value.split_once('=') else {
        return Err(Error::config(format!(
            "unknown --tier value `{value}`; expected the grammar `--tier <crate>=<tier>`, \
             e.g. `--tier my-crate=lib`"
        )));
    };
    let crate_name = crate_name_raw.trim();
    if crate_name.is_empty() {
        return Err(Error::config(unknown_empty_value_error_message(
            value,
            "empty crate name",
        )));
    }
    let tier = Tier::parse(tier_raw).map_err(|_| {
        Error::config(unknown_empty_value_error_message(
            value,
            "unknown tier value",
        ))
    })?;
    Ok(TierOverride {
        crate_name: crate_name.to_owned(),
        tier,
    })
}

fn unknown_empty_value_error_message(value: &str, problem: &str) -> String {
    format!(
        "invalid --tier value `{value}` ({problem}); expected the grammar `--tier <crate>=<tier>`, \
         e.g. `--tier my-crate=lib`"
    )
}

/// Targets discovered for a crate by the metadata collector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// A `lib` target.
    Lib,
    /// A `bin` target.
    Bin,
}

/// Input for FR-004 tier resolution.
#[derive(Debug, Clone)]
pub struct CrateTierInput {
    /// Crate name (used for override lookup).
    pub name: String,
    /// Target kinds discovered for that crate.
    pub target_kinds: Vec<TargetKind>,
}

/// FR-004 resolution: an explicit override wins; otherwise a crate with a lib
/// target is Lib, a crate with only bin targets is App, and Embedded is only
/// ever reachable through an explicit override.
///
/// # Errors
/// Returns [`Error::Config`] (FR-021) when no override exists and no target
/// kind yields a resolution.
pub fn resolve_tier(input: &CrateTierInput, overrides: &[TierOverride]) -> Result<Tier> {
    if let Some(overridden) = overrides
        .iter()
        .find(|candidate| candidate.crate_name == input.name)
    {
        return Ok(overridden.tier);
    }
    if input.target_kinds.contains(&TargetKind::Lib) {
        return Ok(Tier::Lib);
    }
    if input.target_kinds.contains(&TargetKind::Bin) {
        return Ok(Tier::App);
    }
    Err(Error::config(format!(
        "cannot resolve tier for crate `{}`: no targets, and no `--tier` override given; \
         pass `--tier {}=<lib|app|embedded>` explicitly",
        input.name, input.name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    /// Extracts the `Error::Config` message, or `None` for any other outcome.
    fn config_message<T>(result: &Result<T>) -> Option<String> {
        match result {
            Err(Error::Config { message }) => Some(message.clone()),
            _ => None,
        }
    }

    /// Asserts the result is `Ok(value)` via `assert!` (no `panic_in_result_fn`,
    /// no `unwrap`, and no constant-false assertion).
    fn assert_resolves<T: PartialEq + std::fmt::Debug>(result: &Result<T>, expected: &T) {
        let ok = matches!(result, Ok(value) if value == expected);
        assert!(ok, "expected Ok({expected:?}), got: {result:?}");
    }

    #[test]
    fn parses_canonical_names() {
        assert_resolves(&Tier::parse("lib"), &Tier::Lib);
        assert_resolves(&Tier::parse("app"), &Tier::App);
        assert_resolves(&Tier::parse("embedded"), &Tier::Embedded);
        assert_eq!(Tier::Embedded.canonical_name(), "embedded");
        assert_eq!(Tier::Lib.canonical_name(), "lib");
    }

    #[test]
    fn maps_deprecated_aliases() {
        assert!(Tier::is_deprecated_alias("standard"));
        assert!(!Tier::is_deprecated_alias("lib"));
        assert_eq!(Tier::map_deprecated_alias("minimal"), Some(Tier::Lib));
        assert_eq!(Tier::map_deprecated_alias("standard"), Some(Tier::App));
        assert_eq!(Tier::map_deprecated_alias("maximum"), Some(Tier::Embedded));
        assert_eq!(Tier::map_deprecated_alias("app"), None);
        assert_resolves(&Tier::parse("maximum"), &Tier::Embedded);
    }

    #[test]
    fn unknown_tier_is_config_error() {
        let result = Tier::parse("ultra");
        let message = config_message(&result).unwrap_or_default();
        assert!(
            message.contains("unknown tier `ultra`") && message.contains("`app`"),
            "message was: {message}"
        );
    }

    #[test]
    fn bare_tier_value_is_rejected() {
        let values = vec![String::from("lib")];
        let result = parse_tier_overrides(&values);
        let message = config_message(&result).unwrap_or_default();
        assert!(
            message.contains("unknown --tier value `lib`")
                && message.contains("--tier <crate>=<tier>"),
            "message was: {message}"
        );
    }

    #[test]
    fn rejects_empty_crate_and_tier() {
        for bad in ["=lib", "my-crate=", "my-crate=ultra"] {
            let result = parse_tier_overrides(&[String::from(bad)]);
            let message = config_message(&result).unwrap_or_default();
            assert!(message.contains(bad), "message was: {message}");
        }
    }

    #[test]
    fn parses_valid_overrides() {
        let values = vec![String::from("my-crate=lib"), String::from("other=maximum")];
        let overrides = if let Ok(overrides) = parse_tier_overrides(&values) {
            overrides
        } else {
            assert!(
                Err::<(), ()>(()).is_ok(),
                "unreachable; overrides failed to parse"
            );
            Vec::new()
        };
        assert_eq!(
            overrides.as_slice(),
            [
                TierOverride {
                    crate_name: String::from("my-crate"),
                    tier: Tier::Lib,
                },
                TierOverride {
                    crate_name: String::from("other"),
                    tier: Tier::Embedded,
                },
            ]
            .as_slice()
        );
    }

    #[test]
    fn resolution_override_wins() {
        let input = CrateTierInput {
            name: String::from("my-crate"),
            target_kinds: vec![TargetKind::Lib, TargetKind::Bin],
        };
        let overrides = vec![TierOverride {
            crate_name: String::from("my-crate"),
            tier: Tier::Embedded,
        }];
        assert_resolves(&resolve_tier(&input, &overrides), &Tier::Embedded);
    }

    #[test]
    fn resolution_lib_wins_over_bin() {
        let input = CrateTierInput {
            name: String::from("my-crate"),
            target_kinds: vec![TargetKind::Bin, TargetKind::Lib],
        };
        assert_resolves(&resolve_tier(&input, &[]), &Tier::Lib);
    }

    #[test]
    fn resolution_bin_only_is_app() {
        let input = CrateTierInput {
            name: String::from("my-crate"),
            target_kinds: vec![TargetKind::Bin],
        };
        assert_resolves(&resolve_tier(&input, &[]), &Tier::App);
    }

    #[test]
    fn resolution_unresolved_is_config_error() {
        let input = CrateTierInput {
            name: String::from("my-crate"),
            target_kinds: Vec::new(),
        };
        let result = resolve_tier(&input, &[]);
        let message = config_message(&result).unwrap_or_default();
        assert!(
            message.contains("my-crate") && message.contains("--tier"),
            "message was: {message}"
        );
    }
}
