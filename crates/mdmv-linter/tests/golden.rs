//! Golden-file tests for generated artifacts (T050).
//!
//! The delivered `clippy.toml` and `.mdmv/verify.sh` are compared byte-for-byte
//! against goldens frozen from the normative contracts, and the complexity
//! thresholds are asserted explicitly (contracts/cli-and-ci.md §1).
#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions may panic on failure"
)]

#[expect(
    clippy::allow_attributes,
    reason = "shared test helper module intentionally allows dead_code"
)]
#[allow(dead_code)]
mod common;

use std::path::PathBuf;

use common::{Sandbox, TestResult, CLEAN_LIB};

fn golden(name: &str) -> TestResult<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("golden")
        .join(name);
    Ok(std::fs::read_to_string(path)?)
}

#[test]
fn generated_clippy_toml_matches_golden() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.run(&["init", "--yes"])?.success();

    let generated = sandbox.read("clippy.toml")?;
    assert_eq!(generated, golden("clippy.toml")?);
    assert!(generated.contains("cognitive-complexity-threshold = 15"));
    assert!(generated.contains("type-complexity-threshold = 200"));
    assert!(generated.contains("too-many-lines-threshold = 100"));
    Ok(())
}

#[test]
fn generated_verify_script_matches_golden() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.run(&["init", "--yes"])?.success();

    let generated = sandbox.read(".mdmv/verify.sh")?;
    assert_eq!(generated, golden("verify.sh")?);
    let fmt = generated.find("cargo fmt --all --check");
    let clippy = generated.find("cargo clippy --all-targets -- -D warnings");
    let test = generated.find("cargo test");
    assert!(fmt < clippy && clippy < test, "canonical order is pinned");
    Ok(())
}

#[test]
fn tier_presets_match_golden() -> TestResult<()> {
    for (tier, name) in [
        ("lib", "Cargo-lints-lib.toml"),
        ("app", "Cargo-lints-app.toml"),
        ("embedded", "Cargo-lints-embedded.toml"),
    ] {
        let embedded = mdmv_linter::infra::assets::read_to_string(&format!("presets/{name}"));
        let embedded = embedded.unwrap_or_default();
        assert_eq!(embedded, golden(name)?, "preset drifted for tier {tier}");
    }
    Ok(())
}
