//! Embedded asset access for templates shipped with the harness binary.

use include_dir::{include_dir, Dir};

/// The `assets/` tree at the repository root, embedded at compile time.
///
/// The crate lives at `crates/mdmv-linter`, so `../../assets` resolves to
/// the repository root. Files embedded under `assets/fixture/` are plain
/// bytes; they are never treated as a cargo member by the harness itself.
pub static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets");

/// Look up an embedded file by its path relative to the `assets/` root.
#[must_use]
pub fn get(path: &str) -> Option<&'static include_dir::File<'static>> {
    ASSETS.get_file(path)
}

/// Read an embedded file as UTF-8 text by its path relative to `assets/`.
#[must_use]
pub fn read_to_string(path: &str) -> Option<String> {
    get(path)
        .and_then(include_dir::File::contents_utf8)
        .map(ToOwned::to_owned)
}

/// Iterate every embedded asset file, recursively.
pub fn iter_files() -> impl Iterator<Item = &'static include_dir::File<'static>> {
    ASSETS.files()
}

#[cfg(test)]
mod tests {
    use super::{get, iter_files, read_to_string};

    #[test]
    fn embedded_tree_is_not_empty() {
        assert!(iter_files().next().is_some());
    }

    #[test]
    fn missing_asset_returns_none() {
        assert!(get("nonexistent-template.yml").is_none());
        assert!(read_to_string("nonexistent-template.yml").is_none());
    }
}
