# LOG.md convention — append-only component logs

> **Owner:** `mdmv-linter` (this document is the source of truth).
> **Origin:** cherry-pick 1 from the CenterOS pattern (`LOG.md` per component).
> **Role:** durable, human- and agent-readable run history per component. It
> **complements** — never replaces — the orchestrator's event log (Fabro
> `events`/`logs`) and the harness trace store (`.wiki/raw/`).

---

## 1. Scope

- Every **active component** (crate, room, service, workflow) MAY have a `LOG.md`
  at its root.
- The **repository root** MAY have a `LOG.md` for system-level runs and changes.
- `LOG.md` is versioned with the code (plain Markdown, no tooling required).

## 2. Entry format

One entry = one level-2 heading with an ISO-8601 UTC timestamp and a status in
square brackets, followed by three bullet fields:

```markdown
## 2026-09-23T20:50:03Z [ran-complete]
- run: validator-intake/2026-09-23-001
- actor: ci
- summary: 42 files validated, 3 quarantined, handoff emitted
```

| Field | Rule |
|---|---|
| heading | `## <ISO-8601-UTC> [<status>]` — exact shape |
| `<ISO-8601-UTC>` | `YYYY-MM-DDTHH:MM:SSZ` (e.g. `date -u +%Y-%m-%dT%H:%M:%SZ`) |
| `<status>` | closed set: `ran-start`, `ran-complete`, `ran-failed` |
| `- run:` | run identifier or slug (stable, greppable) |
| `- actor:` | `ci`, `human`, or an agent/skill name |
| `- summary:` | one line; for failures, the reason (never empty) |

## 3. Rules

1. **Append-only.** Never edit or delete an entry; add a new one.
2. **Newest last.** Timestamps are non-decreasing down the file.
3. **Closed status set.** Only the three statuses above.
4. **Required fields.** Every entry has `run`, `actor`, `summary`.
5. **Log both ends.** A run writes `ran-start` when it begins and
   `ran-complete` or `ran-failed` when it ends.
6. **Log failures.** Do not silently swallow errors.
7. **Relative paths only** in entry content; never absolute paths.
8. **No secrets.** Never log credentials, tokens, or proprietary content.

## 4. Tooling

| Artifact | Purpose |
|---|---|
| `assets/LOG.template.md` | starting point for a new component `LOG.md` |
| `scripts/check-logs.sh` | validates rules 1–4 over all `LOG.md` under a root |
| `scripts/log.sh` | appends one entry (deterministic timestamp, status validation) |

Wire `check-logs.sh` into CI as a gate (see `mdmv-linter init`) so a malformed or
out-of-order log fails the build.

## 5. Adoption

A consuming repository vendors `assets/LOG.template.md`, `scripts/check-logs.sh`
and `scripts/log.sh`, creates `LOG.md` at the repo root and in each active
component, and adds the checker to its CI. The first adopter is **`crsdd-fabro`**
(Contaminated Room).
