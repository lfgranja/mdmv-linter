//! Self-verifying canary fixture for the mdmv-linter harness (T038).
//!
//! The crate isolates exactly one planted violation per explicitly-named
//! `deny`/`forbid` leaf lint of the normative per-tier table
//! (contracts/cli-and-ci.md §1). Every remaining pub surface is documented and
//! intentionally clean: only the canary modules violate. The fixture is
//! EXPECTED to fail the gate — the runner (US3) asserts each record.
//!
//! Groups (`all`, `nursery`, `cargo`, `pedantic`, `rustdoc::all`,
//! `rust_2018_idioms`, `future_incompatible`, `nonstandard_style`) are not
//! canaried individually: groups are covered by their leaves (deliberate,
//! documented interpretation — see README.md).

#![cfg_attr(test, allow(clippy::unwrap_used, reason = "Panic-safety set is exempt in #[cfg(test)] code (FR-006)"))]
#![cfg_attr(test, allow(clippy::expect_used, reason = "Panic-safety set exemption (FR-006)"))]
#![cfg_attr(test, allow(clippy::panic, reason = "Panic-safety set exemption (FR-006)"))]
#![cfg_attr(test, allow(clippy::unreachable, reason = "Embedded panic-safety superset exemption (FR-006)"))]
#![cfg_attr(test, allow(clippy::indexing_slicing, reason = "Embedded panic-safety superset exemption (FR-006)"))]

/// Clean baseline: saturating increment, no lints fired.
pub fn bump(value: u8) -> u8 {
    value.saturating_add(1)
}

/// Clean baseline record type with a full derive set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanRecord {
    /// Upper bound honored by the fuzzer.
    pub limit: u8,
}

/// Canary modules: one planted violation per canaried leaf lint.
pub mod canary;
