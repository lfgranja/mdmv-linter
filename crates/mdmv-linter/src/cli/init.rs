//! `init` command — apply the governance harness to the current directory.

use crate::cli::InitArgs;
use crate::error::{Error, ExitStatus, Result};

/// Run `init`. Implementation lands in the US1 phase.
pub fn run(_args: InitArgs) -> Result<ExitStatus> {
    Err(Error::config("`init` is not implemented yet"))
}
