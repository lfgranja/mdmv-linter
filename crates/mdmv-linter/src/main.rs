//! Binary entrypoint for the `mdmv-linter` governance harness installer.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use mdmv_linter::cli::{self, Cli};

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli::run(cli) {
        Ok(status) => status.exit_code(),
        Err(error) => {
            let _ = writeln!(std::io::stderr().lock(), "mdmv-linter: {error}");
            error.exit_status().exit_code()
        }
    }
}
