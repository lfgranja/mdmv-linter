//! Quickstart C — legacy repo derives `ratchet` and records a baseline (T047).
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

use common::{Sandbox, TestResult};

fn legacy_source() -> String {
    let body = concat!(
        "    let value_0 = 0;\n",
        "    let value_1 = 1;\n",
        "    let value_2 = 2;\n",
        "    let value_3 = 3;\n",
        "    let value_4 = 4;\n",
        "    let value_5 = 5;\n",
        "    let value_6 = 6;\n",
        "    let value_7 = 7;\n",
        "    let value_8 = 8;\n",
        "    let value_9 = 9;\n",
        "    let value_10 = 10;\n",
        "    let value_11 = 11;\n",
    );
    format!(
        "//! Legacy crate.\n/// Returns zero.\n#[must_use]\npub fn f() -> u32 {{\n{body}    0\n}}\n"
    )
}

#[test]
fn legacy_repo_derives_ratchet_with_baseline() -> TestResult<()> {
    let sandbox = Sandbox::single_crate(&legacy_source())?;
    sandbox.run(&["init", "--yes"])?.success();

    let manifest = sandbox.read(".mdmv/state.toml")?;
    assert!(manifest.contains("gate_mode = \"ratchet\""));

    let baseline_text = sandbox.read(".mdmv/baseline.toml")?;
    let baseline: toml::Value = toml::from_str(&baseline_text)?;
    let total = baseline
        .get("total_violations")
        .and_then(toml::Value::as_integer)
        .unwrap_or(-1);
    let per_lint_sum: i64 = baseline
        .get("per_lint")
        .and_then(toml::Value::as_table)
        .map_or(0, |table| {
            table.values().filter_map(toml::Value::as_integer).sum()
        });

    assert!(total > 0, "legacy repo must record a non-zero baseline");
    assert_eq!(total, per_lint_sum, "total must equal the sum of per-lint");
    Ok(())
}
