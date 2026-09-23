//! Typed domain errors and their deterministic mapping to process exit codes.
//!
//! Exit codes are pinned by `contracts/cli-and-ci.md` §1: `0` pass, `1`
//! violation or non-interactive abort, `2` configuration/environment error,
//! `3` declined interactive confirmation.

use std::process::ExitCode;

use thiserror::Error;

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Process exit status, one per contract exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// `0`: all gates pass; no drifts; fixture passes.
    Success,
    /// `1`: a violation, or a non-interactive abort without mutation.
    Violation,
    /// `2`: configuration or environment error; no mutation performed.
    Config,
    /// `3`: an interactive confirmation was declined.
    Declined,
}

impl ExitStatus {
    /// The numeric exit code for this status.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Violation => 1,
            Self::Config => 2,
            Self::Declined => 3,
        }
    }

    /// The process exit code for this status.
    #[must_use]
    pub fn exit_code(self) -> ExitCode {
        ExitCode::from(self.code())
    }
}

/// Typed errors raised by the harness.
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration or environment problem (exit `2`); no mutation performed.
    #[error("configuration error: {message}")]
    Config {
        /// Human-readable, actionable explanation.
        message: String,
    },

    /// A rule violation or non-interactive abort (exit `1`).
    #[error("violation: {message}")]
    Violation {
        /// Human-readable explanation.
        message: String,
    },

    /// The operator declined an interactive confirmation (exit `3`).
    #[error("declined: {message}")]
    Declined {
        /// Human-readable explanation.
        message: String,
    },

    /// Underlying I/O failure (exit `2`).
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// TOML deserialization failure (exit `2`).
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    /// Format-preserving TOML edit failure (exit `2`).
    #[error(transparent)]
    TomlEdit(#[from] toml_edit::TomlError),

    /// JSON parsing failure (exit `2`).
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// SemVer parsing failure (exit `2`).
    #[error(transparent)]
    Semver(#[from] semver::Error),

    /// `cargo metadata` invocation failure (exit `2`).
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),
}

impl Error {
    /// Build a configuration error from anything displayable.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Build a violation error from anything displayable.
    pub fn violation(message: impl Into<String>) -> Self {
        Self::Violation {
            message: message.into(),
        }
    }

    /// Build a declined-confirmation error from anything displayable.
    pub fn declined(message: impl Into<String>) -> Self {
        Self::Declined {
            message: message.into(),
        }
    }

    /// The deterministic exit status for this error.
    #[must_use]
    pub const fn exit_status(&self) -> ExitStatus {
        match self {
            Self::Violation { .. } => ExitStatus::Violation,
            Self::Declined { .. } => ExitStatus::Declined,
            Self::Config { .. }
            | Self::Io(_)
            | Self::Toml(_)
            | Self::TomlEdit(_)
            | Self::Json(_)
            | Self::Semver(_)
            | Self::CargoMetadata(_) => ExitStatus::Config,
        }
    }
}
