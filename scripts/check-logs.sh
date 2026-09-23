#!/usr/bin/env bash
# ============================================
# LOG.md APPEND-ONLY CHECKER — MDMV convention
# ============================================
# Source of truth: docs/log-convention.md
# Validates every component LOG.md under <root>:
#   1. heading shape: `## <ISO-8601-UTC> [<status>]`
#   2. status in {ran-start, ran-complete, ran-failed}
#   3. required fields: `- run:`, `- actor:`, `- summary:`
#   4. timestamps non-decreasing (append-only; newest last)
#
# Usage: scripts/check-logs.sh [root]     (default: .)
# Exit:  0 = ok (or no LOG.md found), 1 = violation(s).

set -uo pipefail

ROOT="${1:-.}"
VIOLATIONS=0
FILES=0

prev_ts=""
in_entry=0
in_comment=0
entry_line=0
have_run=0
have_actor=0
have_summary=0
file=""

flush_entry() {
    [ "$in_entry" -eq 0 ] && return 0
    local missing=""
    [ "$have_run" -eq 0 ] && missing="$missing run"
    [ "$have_actor" -eq 0 ] && missing="$missing actor"
    [ "$have_summary" -eq 0 ] && missing="$missing summary"
    if [ -n "$missing" ]; then
        echo "FAIL: $file:$entry_line — entry missing field(s):$missing"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
}

while IFS= read -r f; do
    FILES=$((FILES + 1))
    file="$f"
    prev_ts=""; in_entry=0; in_comment=0; have_run=0; have_actor=0; have_summary=0
    n=0
    while IFS= read -r line || [ -n "$line" ]; do
        n=$((n + 1))
        # Skip HTML comments (templates carry a commented example entry).
        if [ "$in_comment" -eq 1 ]; then
            [[ "$line" == *"-->"* ]] && in_comment=0
            continue
        fi
        if [[ "$line" == *"<!--"* ]]; then
            [[ "$line" == *"-->"* ]] || in_comment=1
            continue
        fi
        if [[ "$line" =~ ^##[[:space:]]([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z)[[:space:]]\[(ran-start|ran-complete|ran-failed)\]$ ]]; then
            flush_entry
            ts="${BASH_REMATCH[1]}"
            if [ -n "$prev_ts" ] && [[ "$ts" < "$prev_ts" ]]; then
                echo "FAIL: $file:$n — timestamp '$ts' < previous '$prev_ts' (append-only: newest last)"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
            prev_ts="$ts"; entry_line=$n
            in_entry=1; have_run=0; have_actor=0; have_summary=0
        elif [[ "$line" == "## "* ]]; then
            flush_entry
            in_entry=0
            echo "FAIL: $file:$n — malformed heading (expected '## <ISO-8601-UTC> [<status>]'): $line"
            VIOLATIONS=$((VIOLATIONS + 1))
        elif [ "$in_entry" -eq 1 ]; then
            case "$line" in
                "- run:"*) have_run=1 ;;
                "- actor:"*) have_actor=1 ;;
                "- summary:"*) have_summary=1 ;;
            esac
        fi
    done < "$f"
    flush_entry
done < <(find "$ROOT" \( -name target -o -name .git -o -name node_modules \) -prune -o -name 'LOG.md' -type f -print 2>/dev/null | sort)

if [ "$FILES" -eq 0 ]; then
    echo "check-logs: no LOG.md found (vacuous pass)"
    exit 0
fi
if [ "$VIOLATIONS" -eq 0 ]; then
    echo "PASS: check-logs — $FILES LOG.md file(s) valid"
    exit 0
fi
echo "FAIL: check-logs — $VIOLATIONS violation(s)"
exit 1
