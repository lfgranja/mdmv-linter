//! Planted compile-fail canary for `rust::unsafe_code` (forbid — embedded
//! tier). This module is only built when the feature
//! `canary_compile_fail_unsafe` is enabled; the `unsafe` block then aborts
//! the whole crate compilation with a hard error, so no other canary can be
//! observed in that pass (README.md: compile-time vs clippy-time).

/// Plants the `rust::unsafe_code` violation: an `unsafe {}` block.
pub fn plant(value: u8) -> u8 {
    unsafe { value } // rust: unsafe_code (forbid → hard compile error)
}
