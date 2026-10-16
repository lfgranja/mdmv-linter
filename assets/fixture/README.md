# mdmv-fixture: canary fixture (T038)

Standalone crate (`[workspace]` table here; the parent root `Cargo.toml`
carries `exclude = ["assets/fixture"]`). It is **expected to fail the lint
gate**: the US3 runner asserts one diagnostic per planted canary. Everything
outside the canaries stays documented and clean.

Each canary plants **exactly one** detected violation for one explicitly-named
`deny`/`forbid` leaf. Clippy-line canaries are inline modules in
`src/canary.rs`; compile-fail canaries abort the build, so they live behind
their own feature and are only compiled in an isolated pass.

## Canary map

`src/` line references are to `src/canary.rs` unless stated otherwise.
`expected_text` is the first line of the expected diagnostic.

| Canary | Lint group | Line ref | Expected text | Timing | Co-fires |
|---|---|---|---|---|---|
| `unwrap_used` | clippy (panic-safety, all tiers) | `canary.rs:13` | ``used `unwrap()` on an `Option` value`` | clippy-time | — |
| `expect_used` | clippy (panic-safety, all tiers) | `canary.rs:21` | ``used `expect()` on an `Option` value`` | clippy-time | — |
| `panic` | clippy (panic-safety, all tiers) | `canary.rs:30` | `` `panic` should not be present in production code`` | clippy-time | — |
| `panic_in_result_fn` | clippy (app tiers) | `canary.rs:40` (fn), `:42` (panic) | ``used `panic!()` or assertion in a function that returns `Result``` | clippy-time | `clippy::panic` (`canary.rs:42`) |
| `unimplemented` | clippy (app tiers) | `canary.rs:52` | `` `unimplemented` should not be present in production code`` | clippy-time | — |
| `print_stdout` | clippy (app tiers) | `canary.rs:60` | ``use of `println!``` | clippy-time | — |
| `print_stderr` | clippy (app tiers) | `canary.rs:68` | ``use of `eprintln!``` | clippy-time | — |
| `await_holding_lock` | clippy (app tiers) | `canary.rs:78` (guard), `:80` (await) | ``this `MutexGuard` is held across an await point`` | clippy-time | — |
| `dbg_macro` | clippy (common baseline) | `canary.rs:89` | ``the `dbg!` macro is intended as a debugging tool`` | clippy-time | — |
| `todo` | clippy (common baseline) | `canary.rs:98` | `` `todo` should not be present in production code`` | clippy-time | — |
| `allow_attributes` | clippy (all tiers) | `canary.rs:107` | ``#[allow] attribute found`` | clippy-time | inherent: any payload is fine |
| `allow_attributes_without_reason` | clippy (embedded) | `canary.rs:118` | `` `allow` attribute without specifying a reason`` | clippy-time | `clippy::allow_attributes` (`canary.rs:118`) |
| `large_futures` | clippy (embedded) | `canary.rs:129` | ``large future with a size of NNNNN bytes`` | clippy-time | `clippy::large_stack_arrays` (20 KiB literal, `canary.rs:129`) |
| `large_stack_arrays` | clippy (embedded) | `canary.rs:146` | ``allocating a local array larger than 16384 bytes`` | clippy-time | — |
| `mem_forget` | clippy (embedded) | `canary.rs:137` | ``usage of `mem::forget` on `Drop` type`` | clippy-time | — |
| `large_types_passed_by_value` | clippy (embedded) | `canary.rs:170` (`pub(crate)` sig) | ``this argument (NNNN byte) is passed by value, but might be more efficient if passed by reference (limit: NN byte)`` | clippy-time | — |
| `unreachable_pub` | rust (embedded/app) | `canary.rs:181` | ``unreachable `pub` item`` | rustc-time (deny) | — |
| `unreachable` | clippy (embedded panic-safety superset) | `canary.rs:200` | ``usage of the `unreachable!` macro`` | clippy-time | `unreachable_patterns` rustc warning `canary.rs:200` (expected, warn-only) |
| `indexing_slicing` | clippy (embedded panic-safety superset) | `canary.rs:210` | ``indexing may panic`` | clippy-time | — |
| `missing_docs` | rust (lib tier) | `canary.rs:217` | ``missing documentation for a function`` | rustc-time (deny) | — |
| `missing_debug_implementations` | rust (embedded) | `canary.rs:227` | ``type does not implement `std::fmt::Debug`...`` | rustc-time (deny) | — |
| `missing_copy_implementations` | rust (embedded) | `canary.rs:238` | ``type could implement `Copy`...`` | rustc-time (deny) | — |
| `unfulfilled_lint_expectations` | rust (embedded) | `canary.rs:251` | ``this lint expectation is unfulfilled`` | rustc-time (deny) | — |
| `unsafe_code` (`src/canary/unsafe_code.rs:9`) | rust (embedded, forbid) | feature `canary_compile_fail_unsafe` | ``usage of an `unsafe` block`` | **compile-time hard error**; feature-gated, isolated pass | — |
| `non_ascii_idents` (`src/canary/non_ascii_idents.rs:8`) | rust (embedded, forbid) | feature `canary_compile_fail_non_ascii` | ``identifier contains non-ASCII characters`` | **compile-time hard error**; feature-gated, isolated pass | — |
| `tests/test_surface.rs` `todo!()` (`tests/test_surface.rs:13`) | clippy (common baseline) | integration-test surface | `` `todo` should not be present in production code`` | clippy-time | — |

Co-firing notes:

- `panic_in_result_fn`/`panic`: the planted `panic!()` inside a `Result`-returning fn is by nature both violations. The US3 runner must expect both records.
- `allow_attributes_without_reason`/`allow_attributes`: an `#[allow]` without a reason is necessarily also an `#[allow]`. The first canary carries a reason so it fires only `allow_attributes`.
- `large_futures`: the awaited future's 20 KiB unit array literal also triggers `large_stack_arrays` at the same span.
- `unreachable`: the dead catch-all arm also yields the rustc `unreachable_patterns` (warn-by-default) warning; documented as expected noise.
- `tests/` canary: note the `lib.rs` `cfg_attr(test)` panic-safety exemptions do **not** apply to integration-test crates, so nothing can exempt `tests/`.

Runner semantics: every clippy above is deny via `[lints]`; the `rust` leaf
denies are native rustc denies so they can be observed in `rustc`/`clippy`
passes alike. Exhaustive per-leaf runner assertions land in US3.

## Embedded residual (accepted gaps, deliberately not modeled)

- `out_of_bounds_indexing` (constant out-of-bounds array index) is **not**
  covered; it is a hard compile-time error (`deny-by-default` rustc const
  check), so a single-clippy-pass canary cannot distinguish it.
- No `no_std`-specific stack/heap lint set is modeled in this fixture; the
  embedded tier's std-only lints are canaried, embedded-std-analysis is
  pending.
- SCRC (software-structural class review) and MISRA-adjacent rule mapping is
  pending: the fixture canaries machine-checkable leaves only.
- Lint thresholds (`future-size-threshold`, `pass-by-value-size-limit`,
  `stack-size-threshold`) mirror harness `clippy.toml` defaults; changing
  thresholds num-bounds the runner's `expected_text` matching.
