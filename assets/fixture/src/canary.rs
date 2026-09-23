//! Canary modules. One planted violation per canaried deny/forbid leaf lint;
//! every other pub surface in this crate stays clean.
//!
//! All canaries are inline modules. Compile-fail canaries remain separate
//! files (feature-gated) because their violations abort compilation of the
//! whole crate. Some canaries inherently co-fire a second lint; every
//! co-firing is documented in README.md.

/// Panic-safety set: `unwrap_used`.
pub mod unwrap_used {
    /// Plants the `clippy::unwrap_used` violation: `unwrap()` on an `Option`.
    pub fn plant(option: Option<u8>) -> u8 {
        option.unwrap() // clippy::unwrap_used
    }
}

/// Panic-safety set: `expect_used`.
pub mod expect_used {
    /// Plants the `clippy::expect_used` violation: `expect()` on an `Option`.
    pub fn plant(option: Option<u8>) -> u8 {
        option.expect("planted: clippy::expect_used")
    }
}

/// Panic-safety set: `panic`.
pub mod panic {
    /// Plants the `clippy::panic` violation: explicit `panic!()`.
    pub fn plant(flag: bool) -> u8 {
        if flag {
            panic!("planted: clippy::panic")
        }
        0
    }
}

/// app tier: `panic_in_result_fn` (inherently co-fires `clippy::panic`).
pub mod panic_in_result_fn {
    /// Plants the `clippy::panic_in_result_fn` violation: `panic!()` inside a
    /// fn returning `Result` (also fires `clippy::panic` — README co-firing).
    pub fn plant(flag: bool) -> Result<u8, ()> {
        if flag {
            panic!("planted: clippy::panic_in_result_fn")
        }
        Ok(0)
    }
}

/// app tier: `unimplemented`.
pub mod unimplemented {
    /// Plants the `clippy::unimplemented` violation: `unimplemented!()`.
    pub fn plant() -> u8 {
        unimplemented!("planted: clippy::unimplemented")
    }
}

/// app tier: `print_stdout`.
pub mod print_stdout {
    /// Plants the `clippy::print_stdout` violation: `println!`.
    pub fn plant(value: u8) {
        println!("planted: clippy::print_stdout ({value})");
    }
}

/// app tier: `print_stderr`.
pub mod print_stderr {
    /// Plants the `clippy::print_stderr` violation: `eprintln!`.
    pub fn plant(value: u8) {
        eprintln!("planted: clippy::print_stderr ({value})");
    }
}

/// app tier: `await_holding_lock`.
pub mod await_holding_lock {
    use std::sync::{Mutex, PoisonError};

    /// Plants the `clippy::await_holding_lock` violation: a `std::sync`
    /// `MutexGuard` held across an `.await` point.
    pub async fn plant(mutex: &Mutex<u8>) -> u8 {
        let guard = mutex.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = std::future::ready(()).await; // clippy::await_holding_lock
        *guard
    }
}

/// Common baseline: `dbg_macro`.
pub mod dbg_macro {
    /// Plants the `clippy::dbg_macro` violation: `dbg!()`.
    pub fn plant(value: u8) -> u8 {
        let _ = dbg!(value); // clippy::dbg_macro
        value
    }
}

/// Common baseline: `todo`.
pub mod todo {
    /// Plants the `clippy::todo` violation: `todo!()`.
    pub fn plant() -> u8 {
        todo!("planted: clippy::todo")
    }
}

/// embedded tier: `allow_attributes` (fires on any `#[allow]`, even with a
/// reason; `allow_attributes_without_reason` needs one *without* a reason).
pub mod allow_attributes {
    /// Plants the `clippy::allow_attributes` violation: an `#[allow]`
    /// attribute (carries a reason, so only `allow_attributes` fires).
    #[allow(clippy::unwrap_used, reason = "planted canary: allow_attributes")]
    pub fn plant() -> u8 {
        0
    }
}

/// embedded tier: `allow_attributes_without_reason`
/// (inherently co-fires `allow_attributes` — see README.md).
pub mod allow_attributes_without_reason {
    /// Plants the `clippy::allow_attributes_without_reason` violation: an
    /// `#[allow]` with no `reason` (also fires `allow_attributes`).
    #[allow(clippy::unwrap_used)]
    pub fn plant() -> u8 {
        0
    }
}

/// embedded tier: `large_futures`.
pub mod large_futures {
    /// Plants the `clippy::large_futures` violation: awaiting a future whose
    /// state machine exceeds the 16 KiB `future-size-threshold`.
    pub async fn plant() -> usize {
        std::future::ready([0u8; 20_000]).await.len() // clippy::large_futures
    }
}

/// embedded tier: `mem_forget`.
pub mod mem_forget {
    /// Plants the `clippy::mem_forget` violation: `std::mem::forget`.
    pub fn plant(value: Box<u8>) {
        std::mem::forget(value);
    }
}

/// embedded tier: `large_stack_arrays`.
pub mod large_stack_arrays {
    /// Plants the `clippy::large_stack_arrays` violation: a 100 KiB
    /// stack-allocated array.
    pub fn plant() -> usize {
        let buffer = [0u8; 100_000]; // clippy::large_stack_arrays
        buffer.len()
    }
}

/// embedded tier: `large_types_passed_by_value`.
pub mod large_types_passed_by_value {
    /// Oversized type moved by value into [`plant`].
    #[derive(Clone, Copy, Debug)]
    pub struct Payload {
        /// 1 KiB of inline data makes the struct exceed the by-value limit.
        pub blob: [u8; 1024],
    }

    /// Supplies the value to the planted violation. The lint skips exported
    /// fns (`avoid-breaking-exported-api`), so the violating signature lives
    /// in a `pub(crate)` fn.
    pub fn plant(payload: Payload) -> usize {
        plant_inner(payload)
    }

    /// Clippy skips exported fns for `large_types_passed_by_value`; this
    /// `pub(crate)` fn is not exported, so it is linted.
    // clippy::large_types_passed_by_value (Payload is 1 KiB, limit 256 B)
    pub(crate) fn plant_inner(payload: Payload) -> usize {
        payload.blob.len()
    }
}

/// lib tier: `unreachable_pub` (needs a `pub` item in a private module).
pub mod unreachable_pub {
    /// Private module whose pub item is unreachable from the crate root.
    mod hidden {
        /// Plants the `clippy::unreachable_pub` violation: `pub` item in a
        /// private module.
        pub fn plant(value: u8) -> u8 {
            value
        }
    }

    /// Supplies the value to the planted violation. `pub(crate)` silences
    /// nothing else; the canary itself stays in `hidden`.
    pub fn plant(value: u8) -> u8 {
        hidden::plant(value) // clippy::unreachable_pub (on `hidden::plant`)
    }
}

/// embedded tier: `unreachable` (embedded panic-safety superset).
pub mod unreachable {
    /// Plants the `clippy::unreachable` violation: `unreachable!()` in an
    /// arm that matches an infallible case (always dead).
    pub fn plant(point: (u8, u8)) -> u8 {
        match point {
            (x, y) => x.wrapping_add(y),
            _ => unreachable!("planted: clippy::unreachable"), // clippy::unreachable
        }
    }
}

/// embedded tier: `indexing_slicing` (embedded panic-safety superset).
pub mod indexing_slicing {
    /// Plants the `clippy::indexing_slicing` violation: unguarded slice
    /// indexing.
    pub fn plant(values: &[u8]) -> u8 {
        values[0] // clippy::indexing_slicing
    }
}

/// rust: `missing_docs` (lib tier; ucb deny).
pub mod missing_docs {
    // rust: missing_docs (this `pub` fn carries no doc comment)
    pub fn planted_without_doc(value: u8) -> u8 {
        value
    }
}

/// rust: `missing_debug_implementations` (embedded tier deny).
pub mod missing_debug_implementations {
    /// Plants the `rust::missing_debug_implementations` violation: a pub
    /// struct with no `Debug` impl (derives `Copy` so only one lint co-fires).
    #[derive(Clone, Copy)]
    pub struct NoDebug {
        /// Documented so only `missing_debug_implementations` fires.
        pub value: u8,
    }
}

/// rust: `missing_copy_implementations` (embedded tier deny).
pub mod missing_copy_implementations {
    /// Plants the `rust::missing_copy_implementations` violation: a pub unit
    /// struct that could be `Copy` but derives nothing (manual `Debug` so
    /// only one lint co-fires).
    pub struct Copyable;

    impl std::fmt::Debug for Copyable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Copyable")
        }
    }
}

/// rust: `unfulfilled_lint_expectations` (embedded tier deny).
pub mod unfulfilled_lint_expectations {
    /// Plants the `rust::unfulfilled_lint_expectations` violation: an
    /// `#[expect]` on code that never trips the expected lint.
    #[expect(clippy::unwrap_used, reason = "planted canary: unfulfilled expectation")]
    pub fn plant(value: u8) -> u8 {
        value
    }
}

/// Compile-fail canaries (feature-gated hard compile errors).
#[cfg(feature = "canary_compile_fail_unsafe")]
pub mod unsafe_code;

/// Compile-fail canary: non-ASCII identifier (forbid → parse-error abort).
#[cfg(feature = "canary_compile_fail_non_ascii")]
pub mod non_ascii_idents;
