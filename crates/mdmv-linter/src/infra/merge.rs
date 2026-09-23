//! Format-preserving TOML merges (T022) via `toml_edit`.
//!
//! Two merge surfaces:
//!
//! - [`merge_lints_into_cargo_manifest`]: adds only the `[lints.*]` tables
//!   that the preset provides; any key the user already set is left verbatim.
//! - [`merge_missing_keys`]: generic "top up a config file with keys the user
//!   does not already have" (e.g. clippy.toml).
//!
//! Both preserve comments, inline whitespace, and key ordering, because edits
//! go through `toml_edit::DocumentMut` and only *insertions* are performed.

use std::path::Path;

use toml_edit::{Item, Table};

use crate::error::{Error, Result};
use crate::infra::atomic_fs;

/// Merge `[lints.*]` tables from `preset_toml` into the Cargo.toml at `path`.
///
/// Only missing keys are added; an existing user-set lint key is never
/// overwritten (non-destructive). Returns whether the document changed. If the
/// file does not exist it is created from the preset fragment.
pub fn merge_lints_into_cargo_manifest(path: &Path, preset_toml: &str) -> Result<bool> {
    if !path.is_file() {
        atomic_fs::write_atomic_str(path, preset_toml)?;
        return Ok(true);
    }

    let raw = std::fs::read_to_string(path)
        .map_err(|e| Error::config(format!("cannot read {}: {e}", path.display())))?;

    let preset = parse_document_string(preset_toml, path)?;
    let preset_lints = match preset.get("lints") {
        Some(item) if item.is_table() => item.clone(),
        _ => {
            return Err(Error::config(
                "preset fragment has no [lints.*] tables; nothing to merge",
            ));
        }
    };

    let mut doc = raw
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| Error::config(format!("cannot parse {}: {e}", path.display())))?;

    let changed = if doc.get("lints").is_none() {
        doc.insert("lints", preset_lints);
        true
    } else {
        let target_item = doc
            .get_mut("lints")
            .ok_or_else(|| Error::config(format!("cannot reach [lints] in {}", path.display())))?;
        if !target_item.is_table() {
            return Err(Error::config(format!(
                "\"lints\" in {} is not a table (it is a {}); refusing to merge",
                path.display(),
                target_item.type_name()
            )));
        }
        let target_table = target_item.as_table_mut().ok_or_else(|| {
            Error::config(format!(
                "[lints] table entry is malformed in {}",
                path.display()
            ))
        })?;
        let source_table = preset_lints
            .as_table()
            .ok_or_else(|| Error::config("preset [lints] entry is not a table"))?;
        merge_missing_into_table(target_table, source_table, path)?
    };

    if changed {
        atomic_fs::write_atomic_str(path, &doc.to_string())?;
    }
    Ok(changed)
}

/// Merge every missing top-level (and nested-table) key from `asset_toml`
/// into the TOML file at `path` without touching existing keys or formatting.
///
/// Returns whether the document changed. If the target file does not exist it
/// is created with the asset content.
pub fn merge_missing_keys(path: &Path, asset_toml: &str) -> Result<bool> {
    if !path.is_file() {
        atomic_fs::write_atomic_str(path, asset_toml)?;
        return Ok(true);
    }

    let raw = std::fs::read_to_string(path)
        .map_err(|e| Error::config(format!("cannot read {}: {e}", path.display())))?;

    let asset = parse_document_string(asset_toml, path)?;
    let mut doc = raw
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| Error::config(format!("cannot parse {}: {e}", path.display())))?;

    let mut changed = false;
    for (key, incoming) in asset.as_table() {
        changed |= apply_key(doc.as_table_mut(), key, incoming, path)?;
    }

    if changed {
        atomic_fs::write_atomic_str(path, &doc.to_string())?;
    }
    Ok(changed)
}

/// Parse `contents` as a standalone TOML document.
fn parse_document_string(contents: &str, origin: &Path) -> Result<toml_edit::DocumentMut> {
    contents
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| Error::config(format!("cannot parse TOML for {}: {e}", origin.display())))
}

/// Apply one incoming item onto `target` for `key`, bridging the two merge
/// entry points: adding a missing key, or descending into nested tables.
fn apply_key(
    target: &mut toml_edit::Table,
    key: &str,
    incoming: &Item,
    origin: &Path,
) -> Result<bool> {
    let Some(existing) = target.get_mut(key) else {
        // Missing entirely: bring the value in, untouched by the target's style.
        target.insert(key, incoming.clone());
        return Ok(true);
    };

    // Existing non-table value: never overwrite user-set values. Existing
    // table plus incoming non-table value: likewise refuse to clobber.
    if !existing.is_table() || incoming.as_table().is_none() {
        return Ok(false);
    }

    let source = incoming.as_table().ok_or_else(|| {
        Error::config(format!(
            "inconsistent preset key \"{key}\" for {}",
            origin.display()
        ))
    })?;
    let nested = existing.as_table_mut().ok_or_else(|| {
        Error::config(format!(
            "inconsistent table state for key \"{key}\" in {}",
            origin.display()
        ))
    })?;
    merge_missing_into_table(nested, source, origin)
}

/// Insert only sub-keys of `source` absent from `target` (recursive bottom of
/// both merge entry points).
fn merge_missing_into_table(target: &mut Table, source: &Table, origin: &Path) -> Result<bool> {
    let mut changed = false;
    for (key, incoming) in source {
        changed |= apply_key(target, key, incoming, origin)?;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    const PRESET_CARGO: &str = r#"
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
pedantic = "deny"
"#;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn cargo_merge_adds_missing_and_preserves_user_keys() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("Cargo.toml");
        std::fs::write(
            &path,
            "[package]\nname = \"x\"\nedition = \"2021\"\n\n# keep this comment\n[lints.clippy]\nunwrap_used = \"allow\"\n",
        )?;

        let changed = merge_lints_into_cargo_manifest(&path, PRESET_CARGO)?;
        assert!(changed);

        let out = std::fs::read_to_string(&path)?;
        // User choice survives verbatim.
        assert!(out.contains("unwrap_used = \"allow\""), "{out}");
        // Comment survives verbatim.
        assert!(out.contains("# keep this comment"), "{out}");
        // `[package]` survives verbatim.
        assert!(out.contains("edition = \"2021\""), "{out}");
        // Missing keys are added.
        assert!(out.contains("unsafe_code"), "{out}");
        assert!(out.contains("pedantic"), "{out}");
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn cargo_merge_second_pass_is_noop() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("Cargo.toml");

        assert!(merge_lints_into_cargo_manifest(&path, PRESET_CARGO)?);
        let first_pass = std::fs::read_to_string(&path)?;

        assert!(!merge_lints_into_cargo_manifest(&path, PRESET_CARGO)?);
        assert_eq!(std::fs::read_to_string(&path)?, first_pass);
        Ok(())
    }

    const CLIPPY_ASSET: &str = "msrv = \"1.98\"\ntype-complexity-threshold = 96\n";

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn merge_missing_keys_creates_absent_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("clippy.toml");

        let changed = merge_missing_keys(&path, CLIPPY_ASSET)?;
        assert!(changed);
        assert_eq!(std::fs::read_to_string(&path)?, CLIPPY_ASSET);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "tests may panic on assertion failure"
    )]
    fn merge_missing_keys_adds_only_new() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("clippy.toml");
        std::fs::write(&path, "# user comment\nmsrv = \"1.85\"\n")?;

        let changed = merge_missing_keys(&path, "msrv = \"1.98\"\navoid-break = true\n")?;
        assert!(changed);

        let out = std::fs::read_to_string(&path)?;
        assert!(out.contains("msrv = \"1.85\""), "user value kept: {out}");
        assert!(out.contains("# user comment"), "{out}");
        assert!(out.contains("avoid-break"), "{out}");
        Ok(())
    }
}
