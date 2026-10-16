//! Canonical scan: identical to the gate (contracts §1).

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::domain::baseline::ViolationCounts;
use crate::error::{Error, Result};

/// Run `cargo clippy --all-targets --message-format=json` and tally
/// diagnostics per lint code.
///
/// This is the canonical scan: the gate counts exactly what this function
/// counts. A nonzero process exit is expected whenever violations exist, so
/// it is never treated as an error; the error contract only covers spawn
/// failure and the case where cargo emits no parseable JSON at all.
///
/// # Errors
///
/// Returns [`Error::Config`] if `cargo clippy` cannot be spawned from
/// `repo_root`, or emits no parseable JSON lines.
pub fn canonical_scan(repo_root: &Path) -> Result<ViolationCounts> {
    let child = Command::new("cargo")
        .args(["clippy", "--all-targets", "--message-format=json"])
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| {
            Error::config(format!(
                "cannot spawn `cargo clippy` in {}: {err}",
                repo_root.display()
            ))
        })?;

    let stdout = child
        .stdout
        .ok_or_else(|| Error::config("cargo clippy stdout is unreachable"))?;

    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut total: u64 = 0;
    let mut parsed_lines: usize = 0;
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(Error::from)?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        parsed_lines += 1;
        if let Some(code) = diagnostic_code(&value) {
            *counts.entry(code).or_insert(0) += 1;
            total += 1;
        }
    }

    if parsed_lines == 0 {
        return Err(Error::config(format!(
            "`cargo clippy` emitted no parseable JSON in {}",
            repo_root.display()
        )));
    }

    Ok(ViolationCounts {
        total,
        per_lint: counts,
    })
}

/// Extract the lint code of one `message-format=json` record.
///
/// Diagnostics with `code.code == null` (plain rustc build warnings)
/// are attributed to the message level (e.g. `"warning"`).
fn diagnostic_code(value: &Value) -> Option<String> {
    if value.get("reason").and_then(Value::as_str) != Some("compiler-message") {
        return None;
    }

    let message = value.get("message")?;
    let code = message
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str);

    match code {
        Some(code) => {
            let code = code.trim();
            if code.is_empty() {
                None
            } else {
                Some(code.to_owned())
            }
        }
        None => Some(message.get("level").and_then(Value::as_str)?.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::diagnostic_code;

    #[test]
    fn lint_diagnostic_uses_code_code() {
        let value = json!({
            "reason": "compiler-message",
            "message": {
                "level": "error",
                "code": {"code": "clippy::unwrap_used", "explanation": null},
                "message": "used unwrap"
            }
        });
        assert_eq!(
            diagnostic_code(&value).as_deref(),
            Some("clippy::unwrap_used")
        );
    }

    #[test]
    fn build_warning_falls_back_to_level() {
        let value = json!({
            "reason": "compiler-message",
            "message": {
                "level": "warning",
                "code": null,
                "message": "unused variable: `x`"
            }
        });
        assert_eq!(diagnostic_code(&value).as_deref(), Some("warning"));
    }

    #[test]
    fn non_compiler_messages_are_skipped() {
        let value = json!({"reason": "compiler-artifact", "target": {"name": "x"}});
        assert_eq!(diagnostic_code(&value), None);
    }
}
