//! Contract tests for the `init` CLI surface (T044).
#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions may panic on failure"
)]

#[expect(
    clippy::allow_attributes,
    reason = "shared test helper module intentionally allows dead_code"
)]
#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

use common::{Sandbox, TestResult, CLEAN_LIB};

#[test]
fn init_help_documents_tier_grammar() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox
        .run(&["init", "--help"])?
        .success()
        .stdout(predicates::str::contains("--tier <CRATE=TIER>"));
    Ok(())
}

#[test]
fn bare_tier_value_is_an_unknown_value() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox
        .run(&["init", "--tier", "lib", "--yes"])?
        .failure()
        .code(2)
        .stderr(predicates::str::contains("--tier <crate>=<tier>"));
    assert!(!sandbox.exists(".mdmv/state.toml"));
    Ok(())
}

#[test]
fn missing_cargo_manifest_is_exit_2_without_mutation() -> TestResult<()> {
    let sandbox = Sandbox::empty()?;
    sandbox
        .run(&["init", "--yes"])?
        .failure()
        .code(2)
        .stderr(predicates::str::contains("Cargo.toml"));
    assert!(!sandbox.exists(".mdmv"));
    Ok(())
}

#[test]
fn gate_mode_without_reason_is_exit_2() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox
        .run(&["init", "--gate-mode", "strict", "--yes"])?
        .failure()
        .code(2);
    assert!(!sandbox.exists(".mdmv/state.toml"));
    Ok(())
}

#[test]
fn unknown_subcommand_is_exit_2() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.run(&["frobnicate"])?.failure().code(2);
    Ok(())
}
