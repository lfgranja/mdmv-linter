//! Quickstart B — a second identical `init` changes nothing (T046).
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
fn second_identical_init_is_byte_identical() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.run(&["init", "--ci", "github", "--yes"])?.success();
    let first = sandbox.tree_digest()?;

    sandbox.run(&["init", "--ci", "github", "--yes"])?.success();
    let second = sandbox.tree_digest()?;

    assert_eq!(first, second, "second init must produce zero file changes");
    assert!(!sandbox.exists(".casv.toml"));
    Ok(())
}

#[test]
fn override_audit_log_is_deduplicated_on_rerun() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    let args = [
        "init",
        "--gate-mode",
        "strict",
        "--gate-mode-reason",
        "cleaning a legacy repo now",
        "--yes",
    ];

    sandbox.run(&args)?.success();
    let first = sandbox.read(".mdmv/gate-override.log")?.lines().count();

    sandbox.run(&args)?.success();
    let second = sandbox.read(".mdmv/gate-override.log")?.lines().count();

    assert_eq!(first, second, "identical override must not append");
    assert!(!sandbox.exists(".casv.toml"));
    Ok(())
}
