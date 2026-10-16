//! Integration-test canary proving the `tests/` directory is never exempt
//! from the lint gate (FR: `tests/` violations must still be flagged).
//!
//! Note: integration tests are compiled as their own crate — the
//! `#![cfg_attr(test, allow(...))]` exemptions declared in `src/lib.rs` do
//! NOT apply to this file. The planted `todo!()` below is a non-panic-style
//! violation that MUST still be reported when the runner lints `tests/`.
//!
//! Exhaustive per-leaf runner assertions land in US3.

/// Plants a `clippy::todo` violation in the integration-test surface.
pub fn plant() -> u8 {
    todo!("planted: clippy::todo in tests/ (must still be flagged)")
}
