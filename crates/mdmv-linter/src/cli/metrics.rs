//! `metrics` command — compare current counts against the baseline.

use crate::cli::MetricsArgs;
use crate::error::{Error, ExitStatus, Result};

/// Run `metrics`. Implementation lands in the US4 phase.
pub fn run(_args: MetricsArgs) -> Result<ExitStatus> {
    Err(Error::config("`metrics` is not implemented yet"))
}
