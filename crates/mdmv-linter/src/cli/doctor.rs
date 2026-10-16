//! `doctor` command — report governance drift.

use crate::error::{Error, ExitStatus, Result};

/// Run `doctor`. Implementation lands in the US4 phase.
pub fn run() -> Result<ExitStatus> {
    Err(Error::config("`doctor` is not implemented yet"))
}
