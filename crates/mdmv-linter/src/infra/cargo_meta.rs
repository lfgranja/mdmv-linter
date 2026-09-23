//! Workspace introspection built on `cargo_metadata`.

use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;

use crate::domain::tier::TargetKind;
use crate::error::{Error, Result};

/// One build target of one crate.
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Target name as declared in the manifest.
    pub name: String,
    /// Reduced library/binary classification.
    pub kind: TargetKind,
}

/// One member crate of the workspace.
#[derive(Debug, Clone)]
pub struct CrateInfo {
    /// Package name.
    pub name: String,
    /// Absolute path to `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Library and binary targets, ignores examples/tests/benches.
    pub targets: Vec<TargetInfo>,
    /// Package edition as a string (e.g. `"2021"`).
    pub edition: String,
    /// Minimum supported Rust version, if declared.
    pub rust_version: Option<String>,
}

/// Full workspace view used by the tier resolution logic.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Workspace root directory.
    pub root: PathBuf,
    /// Every workspace member crate, in `cargo metadata` order.
    pub crates: Vec<CrateInfo>,
    /// Whether the workspace manifest declares `[lints] workspace = true`.
    pub workspace_lints_present: bool,
}

/// Map raw cargo target kind strings to a domain [`TargetKind`].
///
/// `bin` wins over library-ish markers, and a plain `kind` annotation
/// beats a `crate_types` annotation, because cargo reports both arrays
/// for a single target and [`TargetKind::Bin`] is the more actionable
/// classification for tier gating.
///
/// # Examples
///
/// ```
/// use mdmv_linter::infra::cargo_meta::classify_target;
/// use mdmv_linter::domain::tier::TargetKind;
///
/// assert_eq!(classify_target(&["bin"], &[]), Some(TargetKind::Bin));
/// assert_eq!(classify_target(&["lib"], &["rlib"]), Some(TargetKind::Lib));
/// assert_eq!(classify_target(&["example"], &[]), None);
/// ```
#[must_use]
pub fn classify_target(kind: &[&str], crate_types: &[&str]) -> Option<TargetKind> {
    if kind.contains(&"bin") || crate_types.contains(&"bin") {
        Some(TargetKind::Bin)
    } else if is_library_marker(kind) || is_library_marker(crate_types) {
        Some(TargetKind::Lib)
    } else {
        None
    }
}

/// Does the string array contain a library-shaped crate-type marker?
fn is_library_marker(kind: &[&str]) -> bool {
    kind.iter().any(|k| {
        matches!(
            *k,
            "lib" | "rlib" | "cdylib" | "proc-macro" | "dylib" | "staticlib"
        )
    })
}

/// Run `cargo metadata` at `repo_root` and reduce it to a [`WorkspaceInfo`].
///
/// # Errors
///
/// Returns [`Error::Config`] if there is no `Cargo.toml` at `repo_root`
/// (or the workspace manifest cannot be inspected), and propagates the
/// `cargo metadata` failure otherwise.
pub fn introspect(repo_root: &Path) -> Result<WorkspaceInfo> {
    let manifest = repo_root.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(Error::config(format!(
            "no Cargo.toml at {}: cannot introspect the workspace",
            manifest.display()
        )));
    }

    let metadata = MetadataCommand::new().manifest_path(&manifest).exec()?;

    let workspace_manifest = metadata.workspace_root.join("Cargo.toml");
    let workspace_lints_present = workspace_lints_declared(workspace_manifest.as_std_path())
        .map_err(|err| {
            Error::config(format!(
                "cannot read workspace manifest {workspace_manifest}: {err}"
            ))
        })?;

    let member_ids = &metadata.workspace_members;
    let crates = metadata
        .packages
        .iter()
        .filter(|pkg| member_ids.contains(&pkg.id))
        .map(crate_info)
        .collect();

    Ok(WorkspaceInfo {
        root: metadata.workspace_root.as_std_path().to_path_buf(),
        crates,
        workspace_lints_present,
    })
}

/// Reduce one cargo package to a [`CrateInfo`].
fn crate_info(pkg: &cargo_metadata::Package) -> CrateInfo {
    let targets = pkg
        .targets
        .iter()
        .filter_map(|target| {
            let kinds: Vec<&str> = target.kind.iter().map(String::as_str).collect();
            let crate_types: Vec<&str> = target.crate_types.iter().map(String::as_str).collect();
            classify_target(&kinds, &crate_types).map(|kind| TargetInfo {
                name: target.name.clone(),
                kind,
            })
        })
        .collect();

    let rust_version = pkg.rust_version.as_ref().map(ToString::to_string);

    CrateInfo {
        name: pkg.name.clone(),
        manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
        targets,
        edition: pkg.edition.to_string(),
        rust_version,
    }
}

/// Check whether the workspace manifest declares `[lints] workspace = true`.
fn workspace_lints_declared(manifest: &Path) -> Result<bool> {
    let raw = std::fs::read_to_string(manifest)?;
    let value: toml::Value = toml::from_str(&raw)?;
    Ok(value
        .get("lints")
        .and_then(|lints| lints.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::classify_target;
    use crate::domain::tier::TargetKind;

    #[test]
    fn bin_maps_to_bin() {
        assert_eq!(classify_target(&["bin"], &[]), Some(TargetKind::Bin));
        assert_eq!(classify_target(&["lib"], &["bin"]), Some(TargetKind::Bin));
    }

    #[test]
    fn library_kinds_map_to_lib() {
        assert_eq!(
            classify_target(&["lib"], &["rlib", "proc-macro"]),
            Some(TargetKind::Lib)
        );
        assert_eq!(
            classify_target(&["lib"], &["cdylib"]),
            Some(TargetKind::Lib)
        );
        assert_eq!(classify_target(&[], &["proc-macro"]), Some(TargetKind::Lib));
    }

    #[test]
    fn foreign_kinds_are_ignored() {
        assert_eq!(classify_target(&["example"], &[]), None);
        assert_eq!(classify_target(&["test"], &["bench"]), None);
        assert_eq!(classify_target(&["custom-build"], &[]), None);
    }
}
