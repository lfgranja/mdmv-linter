#!/usr/bin/env bash
# Append one entry to a component LOG.md (append-only, newest last).
# Source of truth: docs/log-convention.md
#
# Usage:
#   scripts/log.sh <LOG.md> <ran-start|ran-complete|ran-failed> <run> <actor> <summary...>
#
# Example:
#   scripts/log.sh crates/validator-core/LOG.md ran-complete \
#     validator-intake/2026-09-23-001 ci "42 files validated, 3 quarantined"

set -euo pipefail

log="${1:?usage: log.sh <LOG.md> <status> <run> <actor> <summary>}"
status="${2:?missing status}"
run="${3:?missing run id}"
actor="${4:?missing actor}"
shift 4
summary="$*"

case "$status" in
    ran-start|ran-complete|ran-failed) ;;
    *) echo "invalid status: '$status' (allowed: ran-start|ran-complete|ran-failed)" >&2; exit 2 ;;
esac
[ -n "$summary" ] || { echo "summary must not be empty" >&2; exit 2; }
[ -f "$log" ] || { echo "no such LOG.md: $log" >&2; exit 2; }

ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
    printf '\n## %s [%s]\n' "$ts" "$status"
    printf -- '- run: %s\n' "$run"
    printf -- '- actor: %s\n' "$actor"
    printf -- '- summary: %s\n' "$summary"
} >> "$log"
echo "appended [$status] to $log"
