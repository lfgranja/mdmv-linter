//! `upgrade` command — refresh the installed policy under confirmation.

use crate::cli::UpgradeArgs;
use crate::error::{Error, ExitStatus, Result};

/// Run `upgrade`. Implementation lands in the US5 phase.
pub fn run(_args: UpgradeArgs) -> Result<ExitStatus> {
    Err(Error::config("`upgrade` is not implemented yet"))
}
