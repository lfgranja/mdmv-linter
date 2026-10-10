---

description: "Task list for Harness determinístico de governança para Rust (mdmv-linter)"
---

# Tasks: Harness determinístico de governança para Rust (`mdmv-linter`)

**Input**: Design documents from `/specs/001-deterministic-rust-harness/`

**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/cli-and-ci.md, quickstart.md, `.specify/memory/constitution.md` v1.0.3

**Tests**: REQUIRED. The project constitution (Development Workflow) mandates TDD, and plan.md
pins `cargo test` + `assert_cmd`/`predicates` contract tests + `tempfile` sandbox integration
tests. Test tasks are included per story and MUST fail before implementation.

**Organization**: Tasks are grouped by user story (spec.md §2, Stories 1–6) so each story is
independently implementable and testable. The deterministic artifact (`assets/`, ROADMAP F1) is a
blocking prerequisite for every story and lives in the Foundational phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: `[US1]`…`[US6]` maps to spec.md Stories 1–6
- Every task names an exact file path

## Path Conventions

- Workspace root: `/` (dogfood artifacts at repo root)
- CLI crate: `crates/mdmv-linter/src/`
- Distributable assets (single source of truth): `assets/`
- Tests: `crates/mdmv-linter/tests/{contract,integration,fixtures}/`
- Skill/eval: `skills/mdmv-linter/`, `evals/trigger-set/v1/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust workspace and dogfood configuration.

- [X] T001 Create workspace root `Cargo.toml` with `[workspace]` (`resolver = "2"`, member `crates/mdmv-linter`) and `[workspace.lints]` carrying ONLY the rules common to all tiers (common baseline); per-crate tier rules belong in each member's `[lints]` (`workspace = true` + additions) per `contracts/cli-and-ci.md` §1 in `Cargo.toml`
- [X] T002 Create binary crate manifest `crates/mdmv-linter/Cargo.toml` (edition 2021, `rust-version = "1.81"`, `[lints] workspace = true`) in `crates/mdmv-linter/Cargo.toml`
- [X] T003 Create crate entrypoints with module declarations only in `crates/mdmv-linter/src/main.rs` and `crates/mdmv-linter/src/lib.rs`
- [X] T004 [P] Create dogfood `rust-toolchain.toml` pinning the gate toolchain (`>= 1.97`) with `components = ["clippy", "rustfmt"]` in `rust-toolchain.toml`
- [X] T005 [P] Create dogfood global `clippy.toml` with thresholds `cognitive-complexity-threshold = 15`, `type-complexity-threshold = 200`, `too-many-lines-threshold = 100`, and `doc-valid-idents` in `clippy.toml`
- [X] T006 [P] Create dogfood `.cargo/config.toml` with `[build] warnings = "warn"` in `.cargo/config.toml`
- [X] T007 [P] Create dogfood supply-chain config `.cargo/deny.toml` (advisories, licenses, bans, sources) in `.cargo/deny.toml`
- [X] T008 Add runtime dependencies (clap v4 derive, serde, toml, toml_edit, cargo_metadata, serde_json, tempfile, rustix, fs4, semver, sha2, include_dir, anyhow, thiserror) in `crates/mdmv-linter/Cargo.toml`
- [X] T009 Add dev-dependencies (`assert_cmd`, `predicates`, `tempfile`) and create test directories `crates/mdmv-linter/tests/{contract,integration,fixtures}` in `crates/mdmv-linter/Cargo.toml`
- [X] T010 Update `.gitignore` for build/session dirs (`/target`, `.omo/`, `graphify-out/`, `*.xml`) in `.gitignore`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure + the deterministic artifact (`assets/`, ROADMAP F1). Every user
story depends on these; no story work may begin until this phase is complete.

**⚠️ CRITICAL**: `assets/` is the single source of truth and is embedded in the binary — it must
exist and be valid before `init` (US1) can apply anything.

### Core infrastructure

- [ ] T011 Implement typed domain errors and deterministic exit-code mapping (0/1/2/3) per `contracts/cli-and-ci.md` §1 in `crates/mdmv-linter/src/error.rs`
- [ ] T012 Implement the clap-derive CLI skeleton with `init/check/doctor/metrics/upgrade/uninstall` subcommands and the `--tier <crate>=<tier>` custom value parser in `crates/mdmv-linter/src/cli/mod.rs`
- [ ] T013 Wire `main.rs` to parse args, dispatch to command handlers, and translate `error.rs` errors into process exit codes in `crates/mdmv-linter/src/main.rs`
- [ ] T014 [P] Implement atomic writes (tempfile persist + `File::sync_all` + `rustix` directory fsync) in `crates/mdmv-linter/src/infra/atomic_fs.rs`
- [ ] T015 [P] Implement the advisory lock (`.mdmv/lock` via `fs4`, bounded 10 s wait, exit `2` with locked-state message) in `crates/mdmv-linter/src/infra/lock.rs`
- [ ] T016 [P] Implement the commit manifest `.mdmv/state.toml` (version, artifact list, sha2 digests, gate/tier/policy version; written last) in `crates/mdmv-linter/src/infra/manifest.rs`
- [ ] T017 [P] Implement cargo introspection (workspace members, targets, MSRV, edition, `[workspace.lints]`) via `cargo_metadata` in `crates/mdmv-linter/src/infra/cargo_meta.rs`
- [ ] T018 [P] Implement the canonical scan (`cargo clippy --all-targets --message-format=json`, count per `code.code`, include rustc build warnings) in `crates/mdmv-linter/src/infra/scan.rs`
- [ ] T019 [P] Implement asset embedding over `assets/` with `include_dir!` in `crates/mdmv-linter/src/infra/assets.rs`
- [ ] T020 [P] Implement the `HookManager` trait plus the `lefthook` adapter (no binary dependency in the target repo; `husky-rs` adapter deferred) in `crates/mdmv-linter/src/infra/hook_manager.rs`
- [ ] T021 [P] Implement the `CiPlatform` trait plus github/gitlab/forgejo adapters in `crates/mdmv-linter/src/infra/ci_platform.rs`
- [ ] T022 [P] Implement format-preserving config merge (`toml_edit`) for existing manifests in `crates/mdmv-linter/src/infra/merge.rs`

### Domain entities (data-model.md)

- [ ] T023 [P] Implement `PolicyVersion` (SemVer seal, parse/compare, minimum-version gate) in `crates/mdmv-linter/src/domain/policy_version.rs`
- [ ] T024 [P] Implement `Tier` (lib|app|embedded, deprecated `minimal/standard/maximum` warn-and-map aliases, ≥1 active rule) in `crates/mdmv-linter/src/domain/tier.rs`
- [ ] T025 [P] Implement per-crate tier resolution (lib→biblioteca, `[[bin]]`→app, embedded explicit, `--tier` override, unresolved = error) in `crates/mdmv-linter/src/domain/tier.rs`
- [ ] T026 [P] Implement `RuleSet` (levels deny|warn|allow, active flag, exception `condition`+`reason`, unknown key = hard error) in `crates/mdmv-linter/src/domain/rule_set.rs`
- [ ] T027 [P] Implement `Baseline` (`.mdmv/baseline.toml`: total + per-lint + tier + policy_version; `total == sum(per_lint)`; mode-aware absence) in `crates/mdmv-linter/src/domain/baseline.rs`
- [ ] T028 [P] Implement the `Fixture` entity (full violation records `lint_group`+`severity`+`line_ref`+`expected_text`) in `crates/mdmv-linter/src/domain/fixture.rs`
- [ ] T029 [P] Implement `RepoGovernance` (fields, snapshot path, state transitions) in `crates/mdmv-linter/src/domain/governance.rs`

### Deterministic artifact (`assets/`, ROADMAP F1)

- [ ] T030 Author tier presets `assets/presets/Cargo-lints-lib.toml`, `assets/presets/Cargo-lints-app.toml`, `assets/presets/Cargo-lints-embedded.toml` conforming to the normative per-tier table in `contracts/cli-and-ci.md` §1 (retire legacy `minimal/standard/maximum.toml`; no physical copies) in `assets/presets/`
- [ ] T031 [P] Author `assets/clippy.toml` (global thresholds 15/200/100, `allow-*-in-tests` limited to the panic-safety set, `allowed-duplicate-crates`, `doc-valid-idents`) in `assets/clippy.toml`
- [ ] T032 [P] Author the toolchain template (pin + clippy/rustfmt components) in `assets/rust-toolchain.toml`
- [ ] T033 [P] Author `assets/cargo-config.toml` (`[build] warnings = "warn"`) in `assets/cargo-config.toml`
- [ ] T034 [P] Author `assets/deny.toml` (advisories, licenses, bans, sources) in `assets/deny.toml`
- [ ] T035 [P] Author the canonical `assets/verify.sh` (order: `cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test`; fixture runs as a separate pinned step, not here) in `assets/verify.sh`
- [ ] T036 [P] Author `assets/casv.toml` (opt-in partner-engine integration template) in `assets/casv.toml`
- [ ] T037 [P] Author `assets/CONSTITUTION.md` (1:1 rule→lint mapping, `#[expect(reason = "...")]`) in `assets/CONSTITUTION.md`
- [ ] T038 Author the canary fixture crate (clean code that compiles + one planted violation per **deny/forbid leaf** rule with full records — groups covered by their leaves, `allow`/default cells excluded + a `tests/` case that must still fail + fixture `clippy.toml`) in `assets/fixture/`
- [ ] T039 [P] Author the GitHub workflow template (jobs `gate-lints`, `msrv-compat`, `tests` + aggregator `harness-verify` with `if: always()` and `needs`) in `assets/workflows/github/harness-verify.yml`
- [ ] T040 [P] Author the GitLab CI template (same 3 jobs + aggregator) in `assets/workflows/gitlab/.gitlab-ci.yml`
- [ ] T041 [P] Author the Forgejo workflow template (same 3 jobs + aggregator) in `assets/workflows/forgejo/harness-verify.yml`
- [ ] T042 Author the hook template running `.mdmv/verify.sh` at pre-commit (lefthook) in `assets/lefthook.yml`
- [ ] T043 Apply the dogfood artifacts to this repo (`.mdmv/verify.sh`, `.github/workflows/harness-verify.yml`, root `CONSTITUTION.md`) from the assets in `assets/` (FR-016)

**Checkpoint**: Infrastructure and the deterministic artifact are ready — user story phases can begin.

---

## Phase 3: User Story 1 - Aplicar governança em um repo Rust existente (Priority: P1) 🎯 MVP

**Goal**: `mdmv-linter init` transforms an existing Rust repo into a fully governed repo in one
run (< 30 s), non-destructively, deriving `strict`/`ratchet` from the pre-existing violation count,
resolving tiers per crate, writing the commit manifest last, and reporting the count.

**Independent Test**: Run `init` on a clean sandbox repo (quickstart A) and a legacy sandbox
(quickstart C); assert artifacts present, gate mode derived, baseline written for ratchet, and a
second identical run produces `git diff --stat` empty (quickstart B).

### Tests for User Story 1 (TDD — write first, must fail)

- [ ] T044 [P] [US1] Contract test for `init` help, `--tier` grammar, and exit codes in `crates/mdmv-linter/tests/contract/init_cli.rs`
- [ ] T045 [P] [US1] Integration test quickstart A (clean repo → strict, all artifacts present, exit 0) in `crates/mdmv-linter/tests/integration/init_clean.rs` (assert `.casv.toml` is absent without `--casv`, FR-005)
- [ ] T046 [P] [US1] Integration test quickstart B (idempotency: second identical `init` = zero file changes; includes a `--gate-mode --gate-mode-reason` run asserting `.mdmv/gate-override.log` is not appended on re-run) in `crates/mdmv-linter/tests/integration/init_idempotent.rs` (assert `.casv.toml` stays absent across both runs)
- [ ] T047 [P] [US1] Integration test quickstart C (legacy repo → derived ratchet + `.mdmv/baseline.toml` written) in `crates/mdmv-linter/tests/integration/init_ratchet.rs`
- [ ] T048 [P] [US1] Integration test non-destructive merge/abort on pre-existing config (FR-003) in `crates/mdmv-linter/tests/integration/init_nonclobber.rs`
- [ ] T049 [P] [US1] Integration test workspace per-crate tier resolution and root baseline (FR-004) in `crates/mdmv-linter/tests/integration/init_workspace.rs` (assert `embedded` requires explicit declaration and an unresolved crate is an error, FR-004)
- [ ] T050 [P] [US1] Golden-file tests of generated artifacts against `contracts/cli-and-ci.md` §1 in `crates/mdmv-linter/tests/fixtures/golden/`

### Implementation for User Story 1

- [ ] T051 [US1] Implement `init` orchestration (anchor cwd, acquire lock, capture rollback snapshot before mutation, pre-count scan, derive gate mode, apply assets, write manifest last, report count) in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T052 [US1] Implement non-destructive merge + TTY prompt + `--yes` + abort exit `1` for existing configs in `crates/mdmv-linter/src/cli/init.rs` (uses `src/infra/merge.rs`)
- [ ] T053 [US1] Implement per-crate tier application (member preset into each manifest, common root baseline) in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T054 [US1] Implement gate-mode derivation, `--gate-mode` override, and initial baseline write on ratchet in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T054a [US1] Implement the gate-mode override audit trail (require `--gate-mode-reason`; reject `--gate-mode` without it with exit `2` and no mutation; append `.mdmv/gate-override.log` with timestamp + reason + SHA **only when the decision differs from the last recorded entry — an identical re-run appends nothing, preserving FR-002**; echo to stdout) in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T055 [US1] Implement hook installation via `HookManager` in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T056 [US1] Implement `--casv` opt-in emission of `.casv.toml` (off by default, no auto-detection) in `crates/mdmv-linter/src/cli/init.rs`
- [ ] T057 [US1] Implement the `< 30 s` path and the final pre-existing-violation report in `crates/mdmv-linter/src/cli/init.rs`

**Checkpoint**: US1 is fully functional and independently testable — this is the MVP.

---

## Phase 4: User Story 2 - Rejeição automática do fora-do-padrão (Priority: P2)

**Goal**: The generated CI blocks off-standard PRs through the required check `harness-verify`,
runs the same canonical `.mdmv/verify.sh` as the local hook, enforces the supply-chain gate, and
blocks ratchet worsening.

**Independent Test**: Quickstart E — bypass the local hook (`git commit --no-verify`), push, and
assert CI still blocks; quickstart C — increase a per-lint count and assert `exit 1`.

### Tests for User Story 2 (TDD — write first, must fail)

- [ ] T058 [P] [US2] Contract test for `check` exit codes and canonical-script invocation in `crates/mdmv-linter/tests/contract/check_cli.rs`
- [ ] T059 [P] [US2] Integration test quickstart E (parity: local bypass does not escape CI block) in `crates/mdmv-linter/tests/integration/parity.rs`
- [ ] T060 [P] [US2] Integration test ratchet blocking on increased per-lint count (`exit 1`) in `crates/mdmv-linter/tests/integration/ratchet_block.rs`
- [ ] T061 [P] [US2] Golden tests of the rendered workflow per platform (3 jobs + `harness-verify`, `CARGO_BUILD_WARNINGS`) in `crates/mdmv-linter/tests/fixtures/ci/` (assert `gate-lints` exports `CARGO_BUILD_WARNINGS=deny` and `msrv-compat` exports `warn`, FR-008)

### Implementation for User Story 2

- [ ] T062 [US2] Implement the `check` command (runs `.mdmv/verify.sh` plus the fixture-conformance runner; `--fixture` runs only the fixture step; `--policy` runs only the minimum-policy-version gate; deterministic failure codes) in `crates/mdmv-linter/src/cli/check.rs`
- [ ] T063 [US2] Implement CI platform selection and workflow rendering/installation via `CiPlatform` in `crates/mdmv-linter/src/infra/ci_platform.rs` and `crates/mdmv-linter/src/cli/init.rs`
- [ ] T063a [US2] Wire the fixture-conformance step into the generated `gate-lints` job after `.mdmv/verify.sh` (fixture not a workspace member) in `crates/mdmv-linter/src/infra/ci_platform.rs` **(depends on T073a — the fixture runner; only the rendered step text can land before it)**
- [ ] T063b [US5] Wire the minimum-policy-version gate into the generated `gate-lints` job as `mdmv-linter check --policy` after the fixture step (stale policy → exit 2, job blocked) in `crates/mdmv-linter/src/infra/ci_platform.rs` **(depends on T089 — lands with US5, after the policy gate exists)**
- [ ] T064 [US2] Implement the required-status-check aggregator and parity guarantee in the generated workflows (branch context mirrors `harness-verify`) in `crates/mdmv-linter/src/infra/ci_platform.rs`
- [ ] T065 [US2] Wire the blocking supply-chain gate (`cargo deny check` + `.cargo/deny.toml` in the `tests` job) into the generated CI in `crates/mdmv-linter/src/infra/ci_platform.rs`
- [ ] T065a [P] [US2] Integration test supply-chain gate blocks each FR-005 category (plant a disallowed license, a banned crate and a vulnerability advisory in a sandbox; assert the `tests`-job step fails and the merge is blocked → exit `1`) in `crates/mdmv-linter/tests/integration/supply_chain.rs`

**Checkpoint**: US1 and US2 both work independently.

---

## Phase 5: User Story 3 - Anti-apodrecimento das regras (Priority: P3)

**Goal**: The canary fixture fails loudly when a rule is renamed/removed, unknown config keys are
hard errors, and threshold values are asserted — the config refuses to rot silently.

**Independent Test**: Quickstart D — simulate a renamed lint and assert `check` exits `2` pointing
at the dead rule; plant an unknown key and assert a hard error.

### Tests for User Story 3 (TDD — write first, must fail)

- [ ] T066 [P] [US3] Integration test quickstart D (renamed/removed lint → fixture failure, exit 2) in `crates/mdmv-linter/tests/integration/fixture_anti_rot.rs`
- [ ] T067 [P] [US3] Integration test unknown config key → hard error (never silently ignored) in `crates/mdmv-linter/tests/integration/unknown_key.rs`
- [ ] T068 [P] [US3] Integration test fixture value-assertion of the generated `clippy.toml` asset thresholds (live file is doctor drift) in `crates/mdmv-linter/tests/integration/fixture_thresholds.rs`
- [ ] T069 [P] [US3] Integration test one canary violation per deny/forbid leaf rule fails with the expected rule in `crates/mdmv-linter/tests/integration/fixture_canary.rs`
- [ ] T069a [P] [US3] Integration test FR-006 test-surface exemptions (panic-safety exempt only in `#[cfg(test)]`; non-panic restriction rules still fail; `tests/` directory never exempt) in `crates/mdmv-linter/tests/integration/test_exemptions.rs`

### Implementation for User Story 3

- [ ] T070 [US3] Implement the fixture runner (clean crate compiles + each canary violation asserted with its full record) in `crates/mdmv-linter/src/domain/fixture.rs`
- [ ] T071 [US3] Implement unknown-key hard-error validation for rule config in `crates/mdmv-linter/src/domain/rule_set.rs`
- [ ] T072 [US3] Implement the threshold value-assertion layer (assert the generated/golden `clippy.toml` asset = 15/200/100; never the live repo file) in `crates/mdmv-linter/src/domain/fixture.rs`
- [ ] T073 [US3] Implement fixture failure → exit mapping (violations → `1`, config error → `2`) in `crates/mdmv-linter/src/error.rs`
- [ ] T073a [US3] Wire the fixture-conformance runner into `check` (`--fixture` selector; fixture excluded from the workspace) in `crates/mdmv-linter/src/cli/check.rs`

**Checkpoint**: US1–US3 independently functional.

---

## Phase 6: User Story 4 - Diagnóstico e saúde contínua (Priority: P4)

**Goal**: `doctor` reports every drift (toolchain, dead rules, CI, hook, stale baseline,
interrupted install) with an actionable remedy; `metrics` reports total + per-lint + status and
blocks worsening, mode-aware on baseline absence, with `--accept` under confirmation.

**Independent Test**: Quickstart F — plant all drifts and assert 100% reported; quickstart G —
`metrics` in `strict` with no baseline → `stable`, exit `0`, no write.

### Tests for User Story 4 (TDD — write first, must fail)

- [ ] T074 [P] [US4] Contract test for the `metrics` output schema (total + per-lint + status, no score field) in `crates/mdmv-linter/tests/contract/metrics_schema.rs`
- [ ] T075 [P] [US4] Integration test quickstart G (metrics zero-state, strict, no baseline write) in `crates/mdmv-linter/tests/integration/metrics_zerostate.rs`
- [ ] T076 [P] [US4] Integration test quickstart F (doctor detects all 8 planted drifts: toolchain, dead rule, CI missing, CI divergent, hook, threshold drift, stale baseline, interrupted install) in `crates/mdmv-linter/tests/integration/doctor_drift.rs`
- [ ] T076a [P] [US4] Contract test for `doctor` exit codes (0 no drift / 1 drift / 2 config) in `crates/mdmv-linter/tests/contract/doctor_cli.rs`
- [ ] T077 [P] [US4] Integration test mode-aware missing baseline (`strict` → 0; `ratchet` → 2) in `crates/mdmv-linter/tests/integration/metrics_modeaware.rs`
- [ ] T078 [P] [US4] Integration test `metrics --accept` confirmation paths (`--yes`→0, no TTY→1, declined→3) in `crates/mdmv-linter/tests/integration/metrics_accept.rs`

### Implementation for User Story 4

- [ ] T079 [US4] Implement the `metrics` command (canonical scan, baseline comparison, status tokens `improved|worsened|stable`, exit 0/1/2) in `crates/mdmv-linter/src/cli/metrics.rs`
- [ ] T080 [US4] Implement mode-aware baseline absence handling in `crates/mdmv-linter/src/domain/baseline.rs` and `crates/mdmv-linter/src/cli/metrics.rs`
- [ ] T081 [US4] Implement `metrics --accept` (confirmation prompt, `--yes`, atomic baseline rewrite, no plain-write) in `crates/mdmv-linter/src/cli/metrics.rs`
- [ ] T082 [US4] Implement `doctor` drift detection (toolchain, dead rules, CI, hook, threshold drift, stale baseline, interrupted install) with remedies in `crates/mdmv-linter/src/cli/doctor.rs`
- [ ] T083 [US4] Implement the doctor verdict state machine over the manifest (`unmanaged` / interrupted / clean / drift / unparseable) in `crates/mdmv-linter/src/infra/manifest.rs`

**Checkpoint**: US1–US4 independently functional.

---

## Phase 7: User Story 5 - Orientação em prosa, constituição e ciclo de vida da política (Priority: P5)

**Goal**: The thin skill and 1:1 constitution are aligned to distributed rules, exceptions become
errors when their condition stops resolving, and `upgrade` evolves the policy under confirmation
with baseline rotation while `init` never bumps the version.

**Independent Test**: Measure the skill against `trigger-set/v1` (recall ≥ 80% AND precision
≥ 85%); run `upgrade` on a stale policy and assert the diff, confirmation gate, and atomic
baseline rotation.

### Tests for User Story 5 (TDD — write first, must fail)

- [ ] T084 [P] [US5] Contract test for `upgrade` exit codes (0/1/2/3) in `crates/mdmv-linter/tests/contract/upgrade_cli.rs`
- [ ] T084a [P] [US5] Contract test asserting the generated `gate-lints` job runs the policy-version step and rejects a stale policy with exit `2` in `crates/mdmv-linter/tests/contract/policy_gate_ci.rs`
- [ ] T085 [P] [US5] Integration test `upgrade` rule diff + confirm/abort + baseline rotation in `crates/mdmv-linter/tests/integration/upgrade.rs`
- [ ] T086 [P] [US5] Integration test minimum-policy-version gate rejects a stale installed policy in `crates/mdmv-linter/tests/integration/policy_gate.rs`
- [ ] T087 [P] [US5] Integration test exception whose `condition` no longer resolves → exit `2` in `crates/mdmv-linter/tests/integration/exception_lifecycle.rs`

### Implementation for User Story 5

- [ ] T088 [US5] Implement the `upgrade` command (detect installed policy, show rule diff, capture rollback snapshot, confirmed atomic rewrite + baseline rotation) in `crates/mdmv-linter/src/cli/upgrade.rs`
- [ ] T089 [US5] Implement the PolicyVersion minimum-version gate and the SemVer seal emitted in generated artifacts in `crates/mdmv-linter/src/domain/policy_version.rs`
- [ ] T090 [US5] Implement exception lifecycle resolution (every cited `condition` must resolve, else config error exit `2`) in `crates/mdmv-linter/src/domain/rule_set.rs`
- [ ] T091 [US5] Author the thin skill (≤250 lines: triggers in `description`, tier decision, pointers to `assets/`, `check`/`doctor`, declared residual) in `skills/mdmv-linter/SKILL.md`
- [ ] T092 [P] [US5] Author `skills/mdmv-linter/references/` (≤5 files: lints/priorities, panics + residual, thiserror/anyhow, `#[expect(reason)]` exceptions, gate/msrv flow) in `skills/mdmv-linter/references/`
- [ ] T093 [US5] Build the trigger-test harness (`trigger-set/v1`: ≥50 scenarios × ≥5 runs, fixed competing-skill pool) with the publish gate (recall ≥ 80% AND precision ≥ 85%) in `evals/trigger-set/v1/`
- [ ] T094 [US5] Verify `assets/CONSTITUTION.md` maps 1:1 (rule→lint) to the distributed presets and retire the legacy `skills/rust-magic-linter/` assets in `skills/rust-magic-linter/`

**Checkpoint**: US1–US5 independently functional.

---

## Phase 8: User Story 6 - Ciclo de vida e robustez (Priority: P6)

**Goal**: Every mutation is reversible via `uninstall`/`reset` (snapshot-first), failed
preconditions leave the repo byte-identical, interrupted runs are complete-or-absent, concurrent
invocations serialize on the lock, and empty presets / unresolved tiers are hard errors.

**Independent Test**: Quickstart I — `uninstall` restores the snapshot and removes artifacts
(decline → exit `3`, repo unchanged); plant invalid preconditions and assert exit `2` with no
mutation.

### Tests for User Story 6 (TDD — write first, must fail)

- [ ] T095 [P] [US6] Contract test for `uninstall` exit codes (0/2/3, never 1) in `crates/mdmv-linter/tests/contract/uninstall_cli.rs`
- [ ] T096 [P] [US6] Integration test quickstart I (snapshot restore + artifact removal; decline → 3, unchanged) in `crates/mdmv-linter/tests/integration/uninstall.rs`
- [ ] T097 [P] [US6] Integration test interrupted install detection + recovery (FR-019) in `crates/mdmv-linter/tests/integration/interrupted.rs`
- [ ] T098 [P] [US6] Integration test concurrent invocations wait then exit `2` on lock timeout (FR-020) in `crates/mdmv-linter/tests/integration/concurrency.rs`
- [ ] T099 [P] [US6] Integration test invalid preconditions (missing `Cargo.toml`, invalid `--tier`, invalid `--gate-mode`) → exit `2`, no mutation (FR-018) in `crates/mdmv-linter/tests/integration/preconditions.rs`
- [ ] T100 [P] [US6] Integration test empty preset / crate resolving to no tier → explicit error, no mutation (FR-021) in `crates/mdmv-linter/tests/integration/empty_preset.rs`

### Implementation for User Story 6

- [ ] T101 [US6] Implement `uninstall`/`reset` (snapshot-first restore of gate-mode/tier/baseline/policy version, then remove all artifacts, confirmation + `--yes`) in `crates/mdmv-linter/src/cli/uninstall.rs`
- [ ] T102 [US6] Implement snapshot capture/restore as one unit for gate-mode/tier/baseline/policy version in `crates/mdmv-linter/src/domain/governance.rs` and `crates/mdmv-linter/src/infra/manifest.rs`
- [ ] T103 [US6] Implement precondition validations (missing manifest, unknown `--tier`, invalid `--gate-mode`) with no mutation in `crates/mdmv-linter/src/cli/mod.rs`
- [ ] T104 [US6] Implement the FR-021 validator (empty preset / unresolved tier) in `crates/mdmv-linter/src/domain/rule_set.rs` and `crates/mdmv-linter/src/domain/tier.rs`
- [ ] T105 [US6] Implement the interrupted-install recovery path (safe idempotent re-run or actionable `doctor` remedy) in `crates/mdmv-linter/src/cli/doctor.rs`

**Checkpoint**: All six user stories are independently functional.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Dogfood, end-to-end validation, and release readiness.

- [ ] T106 [P] Run quickstart scenarios A–I end-to-end and record observed outcomes in `specs/001-deterministic-rust-harness/quickstart.md`
- [ ] T107 [P] Dogfood: ensure this repo passes its own `.mdmv/verify.sh` (fmt/clippy/test/deny) at the repo root (FR-016)
- [ ] T108 [P] Update `README.md` with the one-command install header and usage summary in `README.md`
- [ ] T109 [P] Audit every command/flag/exit code against `contracts/cli-and-ci.md` §1 and fix discrepancies in `crates/mdmv-linter/src/cli/`
- [ ] T110 Run the full local gate (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`) and confirm green at the repo root

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup — **BLOCKS all user stories** (assets + infra).
- **User Stories (Phases 3–8)**: All depend on Foundational completion. Proceed in priority order
  (US1 → US2 → US3 → US4 → US5 → US6). Once Foundational is done, US4 and US6 can start in
  parallel immediately; US2 lands before US3 (T073a needs T062) and before US5 (T089 supplies
  T063b's policy gate), so US3 and US5 staff up after US2 — not simultaneously with it.
- **Polish (Phase 9)**: Depends on all desired stories.

### User Story Dependencies

- **US1 (P1)**: Foundational only — no story dependencies. MVP.
- **US2 (P2)**: Foundational; consumes the `check`/CI surface built alongside US1 artifacts, but
  is independently testable (parity + ratchet block).
- **US3 (P3)**: Foundational; depends on US2's `check` surface (T062) for T073a's fixture wiring; otherwise independent (fixture/rule validation).
- **US4 (P4)**: Foundational; independent (doctor + metrics), reads the baseline US1 writes.
- **US5 (P5)**: Foundational; `upgrade` builds on US4's baseline/doctor but is independently
  testable via the policy gate and trigger eval.
- **US6 (P6)**: Foundational; touches the mutating commands from US1/US4/US5 but is independently
  testable via `uninstall`, lock, and precondition scenarios.

### Within Each User Story

- Tests MUST be written and FAIL before implementation (constitution: TDD obrigatório).
- Entities/infra before services; services before command wiring.
- Story complete and independently validated before moving to the next priority.

### Parallel Opportunities

- All Setup tasks marked `[P]` (T004–T007) can run in parallel.
- Foundational `[P]` tasks are grouped into independent file sets: infra (T014–T022), domain
  (T023–T029), assets (T031–T041) — these three groups can run in parallel once T001–T013 exist.
- Once Foundational completes, US1–US6 can be staffed in parallel (see per-story `[P]` tests).

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests together (write first, must fail):
Task: "Contract test for init in crates/mdmv-linter/tests/contract/init_cli.rs"
Task: "Integration test quickstart A in crates/mdmv-linter/tests/integration/init_clean.rs"
Task: "Integration test quickstart B in crates/mdmv-linter/tests/integration/init_idempotent.rs"
Task: "Integration test quickstart C in crates/mdmv-linter/tests/integration/init_ratchet.rs"
```

## Parallel Example: Foundational Assets

```bash
# Independent asset files can be authored together:
Task: "Author assets/clippy.toml"
Task: "Author assets/rust-toolchain.toml"
Task: "Author assets/cargo-config.toml"
Task: "Author assets/deny.toml"
Task: "Author assets/verify.sh"
Task: "Author assets/workflows/github/harness-verify.yml"
Task: "Author assets/workflows/gitlab/.gitlab-ci.yml"
Task: "Author assets/workflows/forgejo/harness-verify.yml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (CRITICAL — assets + infra block everything).
3. Complete Phase 3: US1 (`init` on clean and legacy repos, idempotent, non-destructive).
4. **STOP and VALIDATE**: run quickstart A/B/C and confirm.
5. Demo the one-command install.

### Incremental Delivery

1. Setup + Foundational → foundation ready.
2. US1 → MVP (installer).
3. US2 → CI rejection + ratchet.
4. US3 → anti-rot fixture.
5. US4 → doctor + metrics.
6. US5 → skill + constitution + policy lifecycle.
7. US6 → reversibility + robustness.
8. Polish → dogfood green, quickstart A–I recorded, release ready.

### Parallel Team Strategy

With multiple developers after Foundational: one each on US4 (doctor/metrics) and US6
(reversibility/robustness) immediately; US2 (CI/parity) starts alongside, then US3 (fixture) and
US5 (skill/upgrade) begin once US2's `check` surface (T062) is available.

---

## Notes

- `[P]` = different files, no dependencies on incomplete tasks.
- `[Story]` labels map tasks to spec.md Stories 1–6 for traceability.
- Tests are mandated by the constitution (TDD) and MUST fail before implementation.
- `assets/` is the single source of truth (embedded via `include_dir!`) — never duplicate config
  into the skill.
- Commit after each task or logical group; stop at any checkpoint to validate a story independently.
