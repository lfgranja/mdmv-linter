//! Infrastructure adapters: filesystem, locking, manifests, and merges.
//!
//! Every module here is side-effectful by necessity (it installs and verifies
//! harness state), but all writes are atomic and all state transitions are
//! crash-recoverable via `crate::infra::manifest`.

pub mod assets;
pub mod atomic_fs;
pub mod cargo_meta;
pub mod ci_platform;
pub mod hook_manager;
pub mod lock;
pub mod manifest;
pub mod merge;
pub mod scan;
