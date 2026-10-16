//! `init` command — apply the governance harness to the current directory.
//!
//! Orchestration follows `contracts/cli-and-ci.md` §1: anchor at cwd, validate
//! preconditions, take the advisory lock, capture the rollback snapshot, run the
//! pre-count canonical scan, derive/override the gate mode, apply the embedded
//! assets non-destructively, write the baseline on ratchet, and commit the
//! manifest last.

use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::InitArgs;
use crate::domain::baseline::Baseline;
use crate::domain::governance::GateMode;
use crate::domain::policy_version::PolicyVersion;
use crate::domain::tier::{self, CrateTierInput};
use crate::error::{Error, ExitStatus, Result};
use crate::infra::{
    assets, atomic_fs, cargo_meta, ci_platform,
    ci_platform::CiPlatformKind,
    hook_manager::{HookManager, LefthookManager},
    lock::RepoLock,
    manifest, merge, scan,
};

/// Run `init` anchored at the current working directory.
pub fn run(args: &InitArgs) -> Result<ExitStatus> {
    let repo = std::env::current_dir().map_err(Error::from)?;
    let report = install(&repo, args)?;
    emit_report(&report)?;
    Ok(ExitStatus::Success)
}

/// Summary printed at the end of an `init` run (FR-001).
struct Report {
    gate_mode: GateMode,
    total: u64,
    per_lint: Vec<(String, u64)>,
    artifacts: usize,
}

/// Apply the harness to `repo`.
fn install(repo: &Path, args: &InitArgs) -> Result<Report> {
    if !repo.join("Cargo.toml").is_file() {
        return Err(Error::config(format!(
            "no Cargo.toml at {}: run `mdmv-linter init` in a Rust repository root",
            repo.display()
        )));
    }

    let overrides = tier::parse_tier_overrides(&args.tier)?;
    let explicit_mode = resolve_explicit_mode(args)?;
    let platform = match args.ci.as_deref() {
        Some(raw) => CiPlatformKind::parse(raw)?,
        None => CiPlatformKind::default_platform(),
    };

    let mdmv_dir = repo.join(".mdmv");
    std::fs::create_dir_all(&mdmv_dir)?;

    let _lock = RepoLock::acquire_default(&mdmv_dir.join("lock"))?;

    // Pre-count BEFORE any mutation (FR-001 derivation input).
    let counts = scan::canonical_scan(repo)?;
    let derived = if counts.total == 0 {
        GateMode::Strict
    } else {
        GateMode::Ratchet
    };
    let gate_mode = explicit_mode.unwrap_or(derived);

    let workspace = cargo_meta::introspect(repo)?;

    // Snapshot the pre-mutation governance state (FR-017 rollback source).
    let prior = PriorState::capture(repo);
    let mut mutated = false;

    let mut artifacts: Vec<manifest::ArtifactEntry> = Vec::new();

    // 1) Per-crate tier presets (FR-004).
    let mut tiers: Vec<String> = Vec::new();
    for krate in &workspace.crates {
        let kinds: Vec<tier::TargetKind> = krate.targets.iter().map(|target| target.kind).collect();
        let input = CrateTierInput {
            name: krate.name.clone(),
            target_kinds: kinds,
        };
        let resolved = tier::resolve_tier(&input, &overrides)?;
        tiers.push(resolved.canonical_name().to_owned());
        let preset = asset(preset_path(resolved))?;
        if merge::merge_lints_into_cargo_manifest(&krate.manifest_path, &preset)? {
            mutated = true;
        }
        push_entry(&mut artifacts, repo, &krate.manifest_path)?;
    }
    let tier_summary = summarize_tiers(&tiers);

    // 2) Single-file artifacts (non-destructive; `--yes` to overwrite).
    for (relative, asset_name) in [
        (".mdmv/verify.sh", "verify.sh"),
        ("clippy.toml", "clippy.toml"),
        ("rust-toolchain.toml", "rust-toolchain.toml"),
        (".cargo/config.toml", "cargo-config.toml"),
        (".cargo/deny.toml", "deny.toml"),
        ("CONSTITUTION.md", "CONSTITUTION.md"),
    ] {
        let contents = asset(asset_name)?;
        mutated |= apply_file(repo, relative, &contents, args.yes, &mut artifacts)?;
    }

    // 3) Canary fixture tree.
    mutated |= apply_fixture(repo, args.yes, &mut artifacts)?;

    // 4) CI workflow + local hook.
    let ci_path = ci_platform::install(platform, repo)?;
    push_entry(&mut artifacts, repo, &ci_path)?;
    hook_manager_install(repo, &mut artifacts)?;

    // 5) Opt-in partner integration file (off by default, FR-005/C6).
    if args.casv {
        let contents = asset("casv.toml")?;
        mutated |= apply_file(repo, ".casv.toml", &contents, args.yes, &mut artifacts)?;
    }

    // 6) Baseline on ratchet (written by init, FR-012).
    if gate_mode == GateMode::Ratchet {
        let policy = PolicyVersion::current()?;
        let baseline = Baseline::from_counts(&counts, &resolved_any_tier(&tiers), &policy, now()?);
        let path = repo.join(Baseline::FILE);
        if !path.is_file() {
            atomic_fs::write_atomic_str(&path, &baseline.to_toml_string()?)?;
            mutated = true;
        }
        push_entry(&mut artifacts, repo, &path)?;
    }

    // 7) Override audit trail (FR-001, idempotent append).
    if let Some(mode) = explicit_mode {
        let reason = args.gate_mode_reason.as_deref().unwrap_or_default();
        mutated |= append_override(&mdmv_dir.join("gate-override.log"), mode, reason)?;
    }

    // Persist the rollback snapshot only when something actually changed, so a
    // no-op re-run stays byte-for-byte identical (FR-002).
    if mutated {
        prior.persist(repo)?;
    }

    // 8) Commit manifest LAST (FR-019).
    let policy = PolicyVersion::current()?;
    let committed = manifest::Manifest::new(
        gate_mode.canonical_name().to_owned(),
        tier_summary,
        policy.as_str(),
        artifacts,
    );
    manifest::Manifest::write_last(repo, &committed)?;

    Ok(Report {
        gate_mode,
        total: counts.total,
        per_lint: counts.per_lint.into_iter().collect(),
        artifacts: committed.artifacts.len(),
    })
}

/// Resolve the explicit `--gate-mode` override, enforcing the required reason.
fn resolve_explicit_mode(args: &InitArgs) -> Result<Option<GateMode>> {
    match args.gate_mode.as_deref() {
        None => Ok(None),
        Some(raw) => {
            let reason = args
                .gate_mode_reason
                .as_deref()
                .ok_or_else(|| Error::config("--gate-mode requires --gate-mode-reason <text>"))?;
            if reason.trim().is_empty() {
                return Err(Error::config("--gate-mode-reason must not be empty"));
            }
            Ok(Some(GateMode::parse(raw)?))
        }
    }
}

/// Read an embedded asset, failing with an actionable config error.
fn asset(name: &str) -> Result<String> {
    assets::read_to_string(name)
        .ok_or_else(|| Error::config(format!("embedded asset `{name}` is missing")))
}

/// Asset path for a tier preset.
fn preset_path(tier: tier::Tier) -> &'static str {
    match tier {
        tier::Tier::Lib => "presets/Cargo-lints-lib.toml",
        tier::Tier::App => "presets/Cargo-lints-app.toml",
        tier::Tier::Embedded => "presets/Cargo-lints-embedded.toml",
    }
}

/// Collapse the per-crate tiers into a manifest label.
fn summarize_tiers(tiers: &[String]) -> String {
    let first = tiers.first();
    match first {
        Some(name) if tiers.iter().all(|tier| tier == name) => name.clone(),
        Some(_) => "mixed".to_owned(),
        None => "unknown".to_owned(),
    }
}

/// Pick a representative tier for the baseline (first resolved crate).
fn resolved_any_tier(tiers: &[String]) -> tier::Tier {
    match tiers.first().map(String::as_str) {
        Some("lib") => tier::Tier::Lib,
        Some("embedded") => tier::Tier::Embedded,
        _ => tier::Tier::App,
    }
}

/// Apply a single file non-destructively (FR-003).
fn apply_file(
    repo: &Path,
    relative: &str,
    contents: &str,
    yes: bool,
    artifacts: &mut Vec<manifest::ArtifactEntry>,
) -> Result<bool> {
    let target = repo.join(relative);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if target.is_file() {
        let existing = std::fs::read_to_string(&target)?;
        if existing == contents {
            push_entry(artifacts, repo, &target)?;
            return Ok(false);
        }
        if !yes {
            return Err(Error::violation(format!(
                "{relative} already exists and differs; re-run with --yes to overwrite, or run `mdmv-linter doctor`"
            )));
        }
    }
    atomic_fs::write_atomic_str(&target, contents)?;
    push_entry(artifacts, repo, &target)?;
    Ok(true)
}

/// Copy the embedded canary fixture tree into `<repo>/fixture`.
fn apply_fixture(
    repo: &Path,
    yes: bool,
    artifacts: &mut Vec<manifest::ArtifactEntry>,
) -> Result<bool> {
    let mut mutated = false;
    for file in assets::iter_files() {
        let rel = file.path().to_string_lossy();
        if !rel.starts_with("fixture/") {
            continue;
        }
        let Some(contents) = file.contents_utf8() else {
            continue;
        };
        mutated |= apply_file(repo, &rel, contents, yes, artifacts)?;
    }
    Ok(mutated)
}

/// Install the local hook via the canonical `lefthook` adapter (FR-009).
fn hook_manager_install(repo: &Path, artifacts: &mut Vec<manifest::ArtifactEntry>) -> Result<()> {
    let manager = LefthookManager;
    let verify = repo.join(".mdmv").join("verify.sh");
    manager.install(repo, &verify)?;
    push_entry(artifacts, repo, &repo.join("lefthook.yml"))
}

/// Append an entry to the artifact list with the file's content digest.
fn push_entry(
    artifacts: &mut Vec<manifest::ArtifactEntry>,
    repo: &Path,
    path: &Path,
) -> Result<()> {
    let relative = path
        .strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    let digest = manifest::digest_file(path)?;
    artifacts.push(manifest::ArtifactEntry {
        path: relative,
        sha256: digest,
    });
    Ok(())
}

/// Pre-mutation governance bytes, persisted only when a mutation occurred.
struct PriorState {
    state: Option<Vec<u8>>,
    baseline: Option<Vec<u8>>,
}

impl PriorState {
    /// Read the current manifest/baseline bytes into memory (no side effect).
    fn capture(repo: &Path) -> Self {
        Self {
            state: read_optional(&repo.join(manifest::Manifest::FILE)),
            baseline: read_optional(&repo.join(Baseline::FILE)),
        }
    }

    /// Persist the captured bytes under `.mdmv/snapshot/` for `uninstall`.
    fn persist(&self, repo: &Path) -> Result<()> {
        if self.state.is_none() && self.baseline.is_none() {
            return Ok(());
        }
        let snapshot = repo.join(".mdmv").join("snapshot");
        std::fs::create_dir_all(&snapshot)?;
        if let Some(bytes) = &self.state {
            atomic_fs::write_atomic(&snapshot.join("state.toml"), bytes)?;
        }
        if let Some(bytes) = &self.baseline {
            atomic_fs::write_atomic(&snapshot.join("baseline.toml"), bytes)?;
        }
        Ok(())
    }
}

/// Read a file's bytes, or `None` if absent.
fn read_optional(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Append the override audit entry, deduplicated against the last one (FR-001).
fn append_override(path: &Path, mode: GateMode, reason: &str) -> Result<bool> {
    let decision = format!("{}\u{0}{reason}", mode.canonical_name());
    let sha = manifest::sha256_hex(decision.as_bytes());
    if let Ok(existing) = std::fs::read_to_string(path) {
        if existing
            .lines()
            .last()
            .is_some_and(|last| last.ends_with(&sha))
        {
            return Ok(false);
        }
    }
    let entry = format!("{}\t{}\t{reason}\t{sha}\n", now()?, mode.canonical_name());
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(entry.as_bytes())?;
    Ok(true)
}

/// Current UNIX timestamp as a string.
fn now() -> Result<String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::config(format!("system clock error: {err}")))?
        .as_secs();
    Ok(seconds.to_string())
}

/// Print the final report to stdout (explicit IO, never `println!`).
fn emit_report(report: &Report) -> Result<()> {
    let mut out = std::io::stdout().lock();
    writeln!(
        out,
        "mdmv-linter: installed {} governance ({} artifacts)",
        report.gate_mode.canonical_name(),
        report.artifacts
    )?;
    writeln!(out, "pre-existing violations: {}", report.total)?;
    for (lint, count) in &report.per_lint {
        writeln!(out, "  {lint}: {count}")?;
    }
    Ok(())
}
