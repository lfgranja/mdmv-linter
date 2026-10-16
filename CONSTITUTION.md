# Project Constitution

> This file defines the immutable laws of this codebase. AI agents and human
> developers alike must follow these rules. No exceptions.

## Non-Negotiable Rules

### 1. Error Handling

- **No `unwrap()` or `expect()`** - Handle errors explicitly with `?` or match
- **No `panic!()`** - Return `Result` or `Option` instead
- **Use `thiserror`** for custom error types in libraries
- **Use `anyhow`** for error handling in applications
- **No panics inside `Result`-returning functions**

### 2. Logging & Output

- **No `println!()` or `eprintln!()`** - Use `tracing` for structured logging
- **No `dbg!()`** - Remove all debug macros before committing
- Log levels: `error` for failures, `warn` for recoverable issues, `info` for
  important events, `debug` for development, `trace` for detailed debugging

### 3. Code Completeness

- **No `todo!()`** - Implement the feature or create an issue
- **No `unimplemented!()`** - Provide a proper error or stub
- **No `#[allow(...)]`** - Fix the lint warning properly

### 4. Documentation

- All public functions must have doc comments
- Doc comments must include:
  - Brief description of what the function does
  - `# Errors` section if it returns `Result`
  - `# Examples` for complex functions

### 5. Testing

- Tests must exist for all public API
- Use `#[test]` for unit tests, `tests/` directory for integration tests
- Test error cases, not just happy paths
- Aim for meaningful coverage, not 100% line coverage

## Architecture Principles

### Modularity

- Each module should have a single, clear responsibility
- Prefer composition over inheritance
- Keep public APIs minimal

### Performance

- Profile before optimizing
- Prefer stack allocation over heap when practical
- Use `Arc` and `Rc` sparingly - prefer borrowing

### Async Code

- Never hold a `MutexGuard` across `.await`
- Prefer `tokio::sync` primitives over `std::sync` in async code
- Use structured concurrency with `JoinSet` or similar

### Dependencies

- Minimize dependencies - each one is a liability
- Prefer well-maintained crates with good security track records
- Pin major versions in `Cargo.toml`

## Code Style

- Run `cargo fmt` before committing
- Follow Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Use meaningful names - avoid single-letter variables except in iterators
- Keep functions under 50 lines when possible

## Rule → Enforcement (1:1)

| Rule | Enforcing lint / mechanism |
|---|---|
| No `unwrap()` | `clippy::unwrap_used` (deny, all tiers) |
| No `expect()` | `clippy::expect_used` (deny, all tiers) |
| No `panic!()` | `clippy::panic` (deny, all tiers) |
| No panics inside `Result` functions | `clippy::panic_in_result_fn` (deny, app/embedded) |
| No `todo!()` | `clippy::todo` (deny, all tiers) |
| No `unimplemented!()` | `clippy::unimplemented` (deny, app/embedded) |
| No `println!()` / stdout output | `clippy::print_stdout` (deny, app/embedded) |
| No `eprintln!()` / stderr output | `clippy::print_stderr` (deny, app/embedded) |
| No `dbg!()` | `clippy::dbg_macro` (deny, all tiers) |
| No `#[allow(...)]` | `clippy::allow_attributes` (deny, all tiers); `clippy::allow_attributes_without_reason` (deny, embedded) |
| No `process::exit` outside `main` | `clippy::exit` (deny, app/embedded) |
| Doc comments on public API | `missing_docs` (rust, deny, lib) |
| other lints conform to pedantic style | `clippy::pedantic` (warn lib/app, deny embedded) |
| No `unsafe` code | `unsafe_code` (rust, forbid, embedded) |
| ASCII-only identifiers | `non_ascii_idents` (rust, forbid, embedded) |
| Idiomatic Rust | `rust_2018_idioms`, `future_incompatible`, `nonstandard_style` (rust groups, deny, embedded) |
| Complete derive set on public types | `missing_debug_implementations`, `missing_copy_implementations` (rust, deny, embedded) |
| No dead lint expectations | `unfulfilled_lint_expectations` (rust, deny, embedded) |
| Clean rustdoc | `rustdoc::all` (rustdoc group, deny, embedded) |
| Formatting | `cargo fmt --all --check` (canonical gate step 1) |
| Aggregate gate | `.mdmv/verify.sh` — `cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test` |

### Exceptions

Every exception must be a surgical, justified suppression using
`#[expect(clippy::lint, reason = "...")]`. `#[allow]` is never permitted
(enforced by `clippy::allow_attributes`, whose use of a bare attribute is
rejected by the gate without exception).

### Test-surface residual note

The `tests/` surface is never exempt from the governor, except the
panic-safety set exercised inside `#[cfg(test)]` attributed modules: unit
tests may use the lints' targets only where the harness cannot demand
`Result` plumbing without invalidating the tests themselves. No other test
surface, helper, or fixture receives an exemption.

---

*This constitution is enforced by the lint configuration in `Cargo.toml`.
Violations will cause compilation to fail.*
