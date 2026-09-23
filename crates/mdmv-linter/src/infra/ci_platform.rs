//! CI platform rendering and installation (Wave 1: pipeline-level files).

use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::infra::assets;

/// Supported CI platforms for harness workflow installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiPlatformKind {
    /// GitHub Actions.
    Github,
    /// GitLab CI.
    Gitlab,
    /// Forgejo Actions.
    Forgejo,
}

impl CiPlatformKind {
    /// Parse a command-line platform name: `github`, `gitlab`, `forgejo`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] for any unknown platform name.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "forgejo" => Ok(Self::Forgejo),
            unknown => Err(Error::config(format!(
                "unknown ci platform `{unknown}`; expected github, gitlab, or forgejo"
            ))),
        }
    }

    /// The platform used when the operator does not specify one.
    #[must_use]
    pub const fn default_platform() -> Self {
        Self::Github
    }

    /// The canonical lowercase name for this platform.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Forgejo => "forgejo",
        }
    }

    /// Workflow destination path, relative to the repository root.
    #[must_use]
    pub const fn relative_path(self) -> &'static str {
        match self {
            Self::Github => ".github/workflows/harness-verify.yml",
            Self::Gitlab => ".gitlab-ci.yml",
            Self::Forgejo => ".forgejo/workflows/harness-verify.yml",
        }
    }

    /// Embedded asset template path, relative to the `assets/` root.
    #[must_use]
    pub const fn asset_path(self) -> &'static str {
        match self {
            Self::Github => "workflows/github/harness-verify.yml",
            Self::Gitlab => "workflows/gitlab/.gitlab-ci.yml",
            Self::Forgejo => "workflows/forgejo/harness-verify.yml",
        }
    }
}

impl fmt::Display for CiPlatformKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

/// One workflow file ready to be installed in a target repository.
#[derive(Debug, Clone)]
pub struct Workflow {
    /// Destination path, relative to the repository root.
    pub relative_path: String,
    /// Workflow file contents.
    pub contents: String,
}

/// A CI platform capable of rendering and installing its workflow.
pub trait CiPlatform {
    /// Which platform this instance drives.
    fn kind(&self) -> CiPlatformKind;

    /// Render the workflow contents for `repo_root`.
    ///
    /// # Errors
    ///
    /// Returns an error when the embedded template is missing or the
    /// workflow file cannot be written.
    fn render(&self, repo_root: &Path) -> Result<Workflow>;

    /// Write the rendered workflow into `repo_root` and return the
    /// absolute destination path.
    ///
    /// # Errors
    ///
    /// Returns an error when the workflow file cannot be written.
    fn install(&self, repo_root: &Path) -> Result<PathBuf>;
}

/// GitHub Actions renderer.
#[derive(Debug, Default, Clone, Copy)]
pub struct GithubPlatform;

/// GitLab CI renderer.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitlabPlatform;

/// Forgejo Actions renderer.
#[derive(Debug, Default, Clone, Copy)]
pub struct ForgejoPlatform;

impl CiPlatform for GithubPlatform {
    fn kind(&self) -> CiPlatformKind {
        CiPlatformKind::Github
    }

    fn render(&self, _repo_root: &Path) -> Result<Workflow> {
        render_kind(CiPlatformKind::Github)
    }

    fn install(&self, repo_root: &Path) -> Result<PathBuf> {
        install_kind(CiPlatformKind::Github, repo_root)
    }
}

impl CiPlatform for GitlabPlatform {
    fn kind(&self) -> CiPlatformKind {
        CiPlatformKind::Gitlab
    }

    fn render(&self, _repo_root: &Path) -> Result<Workflow> {
        render_kind(CiPlatformKind::Gitlab)
    }

    fn install(&self, repo_root: &Path) -> Result<PathBuf> {
        install_kind(CiPlatformKind::Gitlab, repo_root)
    }
}

impl CiPlatform for ForgejoPlatform {
    fn kind(&self) -> CiPlatformKind {
        CiPlatformKind::Forgejo
    }

    fn render(&self, _repo_root: &Path) -> Result<Workflow> {
        render_kind(CiPlatformKind::Forgejo)
    }

    fn install(&self, repo_root: &Path) -> Result<PathBuf> {
        install_kind(CiPlatformKind::Forgejo, repo_root)
    }
}

/// Factory: build the platform renderer for `kind`.
#[must_use]
pub fn platform(kind: CiPlatformKind) -> Box<dyn CiPlatform> {
    match kind {
        CiPlatformKind::Github => Box::new(GithubPlatform),
        CiPlatformKind::Gitlab => Box::new(GitlabPlatform),
        CiPlatformKind::Forgejo => Box::new(ForgejoPlatform),
    }
}

/// Render the workflow for `kind` from the embedded asset template.
///
/// # Errors
///
/// Returns [`Error::Config`] when the embedded template is missing.
pub fn render(kind: CiPlatformKind, _repo_root: &Path) -> Result<Workflow> {
    render_kind(kind)
}

/// Write the rendered workflow for `kind` as
/// `<repo_root>/<relative_path>` and return the absolute path.
///
/// # Errors
///
/// Returns an error when the embedded template is missing or the
/// destination directory cannot be prepared.
pub fn install(kind: CiPlatformKind, repo_root: &Path) -> Result<PathBuf> {
    install_kind(kind, repo_root)
}

/// Worker behind [`install`], shared by the trait implementations.
fn install_kind(kind: CiPlatformKind, repo_root: &Path) -> Result<PathBuf> {
    let workflow = render_kind(kind)?;
    let target = repo_root.join(&workflow.relative_path);
    let Some(parent) = target.parent() else {
        return Err(Error::config(format!(
            "cannot resolve parent directory of {}",
            target.display()
        )));
    };
    std::fs::create_dir_all(parent)?;
    write_atomically(&target, &workflow.contents)?;
    Ok(target)
}

/// Render the workflow for a concrete kind, using the embedded assets.
fn render_kind(kind: CiPlatformKind) -> Result<Workflow> {
    let contents = read_template(kind.asset_path())?;
    Ok(Workflow {
        relative_path: kind.relative_path().to_owned(),
        contents,
    })
}

/// Read an embedded workflow template as UTF-8 text.
///
/// # Errors
///
/// Returns [`Error::Config`] when the embedded template is missing.
fn read_template(asset_path: &str) -> Result<String> {
    assets::read_to_string(asset_path).ok_or_else(|| {
        Error::config(format!(
            "embedded workflow template `{asset_path}` is missing"
        ))
    })
}

/// Write file contents through a sibling temporary file, then rename it
/// into place, so an interrupted write never leaves a truncated file.
fn write_atomically(target: &Path, contents: &str) -> Result<()> {
    use std::io::Write;

    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();
    let file_name = target
        .file_name()
        .map(std::ffi::OsStr::to_os_string)
        .unwrap_or_default();

    let temp = tempfile::NamedTempFile::new_in(&parent)?;
    temp.as_file().write_all(contents.as_bytes())?;
    temp.as_file().flush()?;

    temp.persist(parent.join(file_name))
        .map_err(|err| err.error)
        .map_err(Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{platform, render, CiPlatformKind};

    #[test]
    fn parse_accepts_known_platforms() {
        assert_eq!(
            CiPlatformKind::parse("github").ok(),
            Some(CiPlatformKind::Github)
        );
        assert_eq!(
            CiPlatformKind::parse("gitlab").ok(),
            Some(CiPlatformKind::Gitlab)
        );
        assert_eq!(
            CiPlatformKind::parse(" forgejo ").ok(),
            Some(CiPlatformKind::Forgejo)
        );
    }

    #[test]
    fn parse_rejects_unknown_platform() {
        match CiPlatformKind::parse("circleci") {
            Err(err) => assert!(err.to_string().contains("circleci")),
            Ok(got) => assert_eq!(
                got,
                CiPlatformKind::Github,
                "expected parse failure for unknown platform"
            ),
        }
    }

    #[test]
    fn default_platform_is_github() {
        assert_eq!(CiPlatformKind::default_platform(), CiPlatformKind::Github);
    }

    #[test]
    fn canonical_names_are_lowercase() {
        assert_eq!(CiPlatformKind::Github.canonical_name(), "github");
        assert_eq!(CiPlatformKind::Gitlab.canonical_name(), "gitlab");
        assert_eq!(CiPlatformKind::Forgejo.canonical_name(), "forgejo");
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic on failure"
    )]
    fn render_returns_non_empty_contents_for_each_platform() -> Result<(), crate::error::Error> {
        for kind in [
            CiPlatformKind::Github,
            CiPlatformKind::Gitlab,
            CiPlatformKind::Forgejo,
        ] {
            let workflow = render(kind, Path::new("/tmp/unused"))?;
            assert!(!workflow.contents.is_empty());
            assert!(workflow.relative_path.starts_with('.'));
            assert_eq!(
                Path::new(&workflow.relative_path).extension(),
                Some(std::ffi::OsStr::new("yml"))
            );
        }
        Ok(())
    }

    #[test]
    fn factory_dispatches_to_matching_kind() {
        for kind in [
            CiPlatformKind::Github,
            CiPlatformKind::Gitlab,
            CiPlatformKind::Forgejo,
        ] {
            let instance = platform(kind);
            assert_eq!(instance.kind(), kind);
        }
    }
}
