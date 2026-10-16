//! `uninstall` command (alias `reset`) — reverse the harness.

use crate::cli::UninstallArgs;
use crate::error::{Error, ExitStatus, Result};

/// Run `uninstall`. Implementation lands in the US6 phase.
pub fn run(_args: UninstallArgs) -> Result<ExitStatus> {
    Err(Error::config("`uninstall` is not implemented yet"))
}
