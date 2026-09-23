//! Planted compile-fail canary for `rust::non_ascii_idents` (forbid —
//! embedded tier). This module is only built when the feature
//! `canary_compile_fail_non_ascii` is enabled; the non-ASCII identifier then
//! aborts the whole crate compilation with a hard error, so no other canary
//! can be observed in that pass (README.md: compile-time vs clippy-time).

/// Plants the `rust::non_ascii_idents` violation: a non-ASCII identifier.
pub fn plant(valeur_élevée: u8) -> u8 {
    valeur_élevée // rust: non_ascii_idents (forbid → hard compile error)
}
