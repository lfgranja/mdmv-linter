//! Workspace per-crate tier resolution (T049, FR-004 / FR-021).
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

fn member_toml(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
}

#[test]
fn workspace_resolves_tiers_per_crate() -> TestResult<()> {
    let sandbox = Sandbox::empty()?;
    sandbox.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"lib_crate\", \"app_crate\"]\n",
    )?;
    sandbox.write("lib_crate/Cargo.toml", &member_toml("lib_crate"))?;
    sandbox.write("lib_crate/src/lib.rs", CLEAN_LIB)?;
    sandbox.write("app_crate/Cargo.toml", &member_toml("app_crate"))?;
    sandbox.write("app_crate/src/main.rs", "fn main() {}\n")?;

    sandbox.run(&["init", "--yes"])?.success();

    let lib_manifest = sandbox.read("lib_crate/Cargo.toml")?;
    let app_manifest = sandbox.read("app_crate/Cargo.toml")?;
    assert!(lib_manifest.contains("missing_docs"), "lib gets lib rules");
    assert!(
        !app_manifest.contains("missing_docs"),
        "app must not inherit lib-only rules"
    );
    assert!(
        app_manifest.contains("print_stdout"),
        "app gets app-tier rules"
    );
    Ok(())
}

#[test]
fn embedded_requires_explicit_declaration() -> TestResult<()> {
    let sandbox = Sandbox::empty()?;
    sandbox.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"lib_crate\"]\n",
    )?;
    sandbox.write("lib_crate/Cargo.toml", &member_toml("lib_crate"))?;
    sandbox.write("lib_crate/src/lib.rs", CLEAN_LIB)?;

    sandbox.run(&["init", "--yes"])?.success();
    let default_manifest = sandbox.read("lib_crate/Cargo.toml")?;
    assert!(
        !default_manifest.contains("unsafe_code"),
        "embedded rules require an explicit declaration"
    );

    sandbox
        .run(&["init", "--tier", "lib_crate=embedded", "--yes"])?
        .success();
    let embedded_manifest = sandbox.read("lib_crate/Cargo.toml")?;
    assert!(embedded_manifest.contains("unsafe_code"));
    Ok(())
}

#[test]
fn crate_resolving_to_no_tier_is_an_error() -> TestResult<()> {
    let sandbox = Sandbox::empty()?;
    sandbox.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"examples_only\"]\n",
    )?;
    sandbox.write(
        "examples_only/Cargo.toml",
        "[package]\nname = \"examples_only\"\nversion = \"0.1.0\"\n\
         edition = \"2021\"\n\n[[example]]\nname = \"demo\"\npath = \"examples/demo.rs\"\n",
    )?;
    sandbox.write("examples_only/examples/demo.rs", "fn main() {}\n")?;

    sandbox.run(&["init", "--yes"])?.failure().code(2);
    assert!(!sandbox.exists(".mdmv/state.toml"));
    Ok(())
}
