//! `check` command — run the canonical gate plus fixture/policy layers.

use crate::cli::CheckArgs;
use crate::error::{Error, ExitStatus, Result};

/// Run `check`. Implementation lands in the US2 phase.
pub fn run(_args: CheckArgs) -> Result<ExitStatus> {
    Err(Error::config("`check` is not implemented yet"))
}
