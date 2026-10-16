#!/bin/sh
# Canonical mdmv-linter verification gate (POSIX sh).
# Order is normative and must not change:
#   1. cargo fmt --all --check
#   2. cargo clippy --all-targets -- -D warnings
#   3. cargo test
set -eu

cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
