//! Non-destructive install (T048): differing config aborts; merges preserve keys.
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
fn differing_existing_file_aborts_without_yes() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    let original = "cognitive-complexity-threshold = 99\n";
    sandbox.write("clippy.toml", original)?;

    sandbox.run(&["init"])?.failure().code(1);

    assert_eq!(sandbox.read("clippy.toml")?, original);
    assert!(
        !sandbox.exists(".mdmv/state.toml"),
        "abort must not commit a manifest"
    );
    Ok(())
}

#[test]
fn merge_preserves_existing_lint_keys() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.write(
        "Cargo.toml",
        "[package]\nname = \"sandbox\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [lints.clippy]\n\
         unwrap_used = \"warn\"\n",
    )?;

    sandbox.run(&["init", "--yes"])?.success();

    let manifest = sandbox.read("Cargo.toml")?;
    assert!(
        manifest.contains("unwrap_used = \"warn\""),
        "existing user lint must not be overwritten"
    );
    assert!(
        manifest.contains("dbg_macro"),
        "missing preset lints must be merged in"
    );
    Ok(())
}
