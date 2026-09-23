//! Quickstart A — clean repo derives `strict` and installs every artifact (T045).
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
fn clean_repo_installs_strict_governance() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(CLEAN_LIB)?;
    sandbox.run(&["init", "--ci", "github", "--yes"])?.success();

    for artifact in [
        "clippy.toml",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        ".cargo/deny.toml",
        ".mdmv/verify.sh",
        ".github/workflows/harness-verify.yml",
        "CONSTITUTION.md",
        "lefthook.yml",
        "fixture/clippy.toml",
        ".mdmv/state.toml",
    ] {
        assert!(sandbox.exists(artifact), "missing artifact: {artifact}");
    }

    assert!(
        !sandbox.exists(".casv.toml"),
        ".casv.toml must be opt-in (FR-005)"
    );
    assert!(
        !sandbox.exists(".mdmv/baseline.toml"),
        "strict mode has no baseline by design"
    );

    let manifest = sandbox.read(".mdmv/state.toml")?;
    assert!(manifest.contains("gate_mode = \"strict\""));
    Ok(())
}
