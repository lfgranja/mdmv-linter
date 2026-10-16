//! Core library for the `mdmv-linter` deterministic governance harness installer.
//!
//! This crate is dogfooded under the same lint policy it distributes: the
//! `mdmv-linter` binary is an application-tier crate carrying the common
//! baseline plus the app-tier rules (and `pedantic = "deny"` per
//! `mdmv-rust-thin`).

pub mod cli;
pub mod domain;
pub mod error;
pub mod infra;

pub use error::{Error, ExitStatus, Result};
