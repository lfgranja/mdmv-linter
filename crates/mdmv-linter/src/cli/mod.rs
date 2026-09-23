//! Command-line interface skeleton and subcommand dispatch.
//!
//! The CLI surface is pinned by `contracts/cli-and-ci.md` §1. Handlers live in
//! sibling modules and return a typed [`ExitStatus`] (or an [`Error`]) that
//! `main` maps to a deterministic process exit code.

pub mod check;
pub mod doctor;
pub mod init;
pub mod metrics;
pub mod uninstall;
pub mod upgrade;

use clap::{Args, Parser, Subcommand};

use crate::domain::tier::Tier;
use crate::error::{ExitStatus, Result};

/// Top-level CLI parser.
#[derive(Debug, Parser)]
#[command(
    name = "mdmv-linter",
    version,
    about = "Deterministic pre-code governance harness installer for Rust repositories"
)]
pub struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The canonical subcommand set (`init`/`check`/`doctor`/`metrics`/`upgrade`/`uninstall`).
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Apply the governance harness to the current working directory.
    Init(InitArgs),
    /// Run the canonical verification script plus the fixture and policy layers.
    Check(CheckArgs),
    /// Report governance drift with an actionable remedy per finding.
    Doctor,
    /// Compare current violation counts against the committed baseline.
    Metrics(MetricsArgs),
    /// Refresh the installed policy (rules/tier) under confirmation.
    Upgrade(UpgradeArgs),
    /// Reverse the harness and restore the pre-mutation state.
    Uninstall(UninstallArgs),
}

/// Arguments for `init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Per-crate tier override, repeatable: `--tier <crate>=<tier>`.
    #[arg(
        long = "tier",
        value_name = "CRATE=TIER",
        value_parser = tier_value_parser,
        action = clap::ArgAction::Append
    )]
    pub tier: Vec<String>,

    /// Override the auto-derived gate mode: `ratchet` or `strict`.
    #[arg(long = "gate-mode", value_name = "ratchet|strict")]
    pub gate_mode: Option<String>,

    /// Required with `--gate-mode`; recorded in the append-only audit trail.
    #[arg(long = "gate-mode-reason", value_name = "TEXT")]
    pub gate_mode_reason: Option<String>,

    /// CI platform to generate: `github` (default), `gitlab`, or `forgejo`.
    #[arg(long = "ci", value_name = "github|gitlab|forgejo")]
    pub ci: Option<String>,

    /// Non-interactive confirmation for overwrite/merge.
    #[arg(long)]
    pub yes: bool,

    /// Emit the optional partner-engine integration file (`.casv.toml`).
    #[arg(long)]
    pub casv: bool,
}

/// Arguments for `check`.
#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Run only the fixture-conformance step.
    #[arg(long)]
    pub fixture: bool,

    /// Run only the minimum-policy-version gate.
    #[arg(long)]
    pub policy: bool,
}

/// Arguments for `metrics`.
#[derive(Debug, Args)]
pub struct MetricsArgs {
    /// Accept the current counts as the new baseline (only manual rewrite).
    #[arg(long)]
    pub accept: bool,

    /// Pre-answer the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments for `upgrade`.
#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Confirm non-interactively.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments for `uninstall` (alias `reset`).
#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Confirm non-interactively.
    #[arg(long)]
    pub yes: bool,
}

/// Validate the single `--tier` value grammar (`<crate>=<tier>`).
///
/// A bare value such as `--tier lib` is an unknown `--tier` value and must be
/// rejected by clap (exit `2`), naming the expected grammar.
fn tier_value_parser(raw: &str) -> std::result::Result<String, String> {
    let grammar = "--tier <crate>=<tier>";
    let (crate_name, tier_name) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid --tier value `{raw}`: expected grammar {grammar}"))?;

    if crate_name.is_empty() || tier_name.is_empty() {
        return Err(format!(
            "invalid --tier value `{raw}`: expected grammar {grammar}"
        ));
    }

    Tier::parse(tier_name)
        .map_err(|_| format!("invalid --tier value `{raw}`: unknown tier `{tier_name}`"))?;

    Ok(raw.to_owned())
}

/// Dispatch a parsed CLI to its handler.
pub fn run(cli: Cli) -> Result<ExitStatus> {
    match cli.command {
        Command::Init(args) => init::run(&args),
        Command::Check(args) => check::run(args),
        Command::Doctor => doctor::run(),
        Command::Metrics(args) => metrics::run(args),
        Command::Upgrade(args) => upgrade::run(args),
        Command::Uninstall(args) => uninstall::run(args),
    }
}
