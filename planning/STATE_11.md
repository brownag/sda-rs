# State 11 — Test-Only DuckDB T-SQL Pipeline Spike

Plan: `planning/EXECUTION_PLAN_11.md`

## Baseline

- Repository/worktree: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-production-backend-task`
- Starting HEAD: `a859c41`
- Planning artifacts are tracked and not ignored; user approved tracking Plan 11.
- At initial inspection, preserve modified `planning/GOALS.md` and pre-existing untracked `planning/tasks/10.5.2.todo.md`, `planning/tasks/10.6.1.todo.md`; do not stage broad paths. User approved tracking Plan 11; `GOALS.md`, `EXECUTION_PLAN_11.md`, `PROMPT_TEMPLATE_11.md`, and `STATE_11.md` are staged for the focused planning-only commit.
- `cargo fmt --check`: passed on 2026-11-20.
- `cargo test`: passed on 2026-11-20 (75 unit + 30 integration; 0 failures).
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed on 2026-11-20 (75 unit + 30 integration; 0 failures).
- `git diff --check`: passed before Plan 11 drafting.
- Existing code has no DuckDB dependency; `polyglot-sql` already supports the DuckDB dialect. Scratch DuckDB `=1.20000.2` built offline in Plan 10.
- Accepted ADRs 0001–0007 remain in force. No production source implementation or runtime backend exists.

## Decisions / Approval Gates

- **2026-11-20 — Goal:** Advance G47 as a non-production T-SQL → DuckDB SQL → DuckDB execution → SDA-serializer slice.
- **2026-11-20 — Reachability:** Test-only/internal; no public route or runtime backend selector.
- **2026-11-20 — Dependency:** `duckdb = "=1.20000.2"` authorized as a dev-dependency only; do not include DuckDB code in normal binaries.
- **2026-11-20 — Contract subset:** TOP/bracketed identifiers/ISNULL/IIF, captured synthetic soilDB join, STRING_AGG, and temp-table batch with CTAS count suppressed; unsupported or lossy value types must be rejected explicitly.
- **2026-11-20 — Execution:** Serial on the current branch; required small-worker model `opencode-go/deepseek-v4.1-flash` with one escalation maximum to `openai/gpt-6-luna`. No worktrees.
- **2026-11-20 — Artifacts:** Track plan files and keep planning-only commits separate from implementation.
- **2026-11-20 — Plan approval:** User approved the four-wave decomposition; implementation has not started and remains gated by each task's micro-plan review.
- **2026-11-20 — Regression gate:** Targeted DuckDB test offline, then formatting, default full suite, and configured-SpatiaLite full suite serially.
- No production security/resource controls, spatial support, performance claim, runtime integration, or production enablement authorized.

## Wave Board

| Wave | Task | Status | Notes |
|---|---|---|---|
| 1 | 11.1.1 | done | Pinned dev-dependency and in-memory DuckDB engine smoke test; all serial wave gates passed. |
| 2 | 11.2.1 | done | Raw T-SQL-to-DuckDB translation assertions, including the IIF gap; all serial wave gates passed. |
| 3 | 11.3.1 | done | Supported DuckDB query execution, value mapping, SDA JSON, and SQLite comparison; all serial wave gates passed. |
| 4 | 11.4.1 | pending | Test-only IIF/batch adaptations, unsupported type assertion, and spike report. |

## Task Reports

<!-- Append concise worker reports below. -->

## 11.1.1

- Status: done; worker verification and orchestrator wave gates passed.
- Files changed: `Cargo.toml` (bundled `duckdb = "=1.20000.2"` dev-dependency), `Cargo.lock` (DuckDB subtree added; existing versions unchanged), `tests/duckdb_pipeline_contract.rs` (single in-memory scalar smoke test).
- Implementation commit: `29087fb` (`test(duckdb): add pinned dev dependency and engine smoke test`). Planning/state edits and the two Plan 10 task drafts are excluded.
- Targeted command: `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` -> 1 passed, 0 failed.
- Build check: `cargo build --offline` -> succeeded; `cargo tree -e normal` and `ldd target/debug/sda-rs` contain no DuckDB, so the dev-dependency stays out of normal builds.
- `git diff --check` passed. No `src/`, existing tests/fixtures, runtime, CI, or ADR changes.
- Deviations: used `features = ["bundled"]` as approved; feature-less pin would attempt a network download and fail offline.
- Limitations: engine/API smoke only; no T-SQL translation, value mapping, SDA serialization, IIF/CTAS handling, performance, or production claims.
- Worker model: `opencode-go/deepseek-v4.1-flash`; no escalation. Token/cost telemetry unavailable.
- Orchestrator gates: targeted DuckDB test passed (1); `cargo fmt --check` and `git diff --check` passed; default and configured-SpatiaLite `cargo test` each passed (75 unit + 31 integration, 0 failures).

## 11.2.1

- Status: done; worker verification and orchestrator wave gates passed.
- Files changed: `tests/duckdb_pipeline_contract.rs` only (4 new tests: TOP+ISNULL, bracketed identifiers, STRING_AGG, unresolved IIF; existing smoke test preserved).
- Implementation commit: `d710380` (`test(duckdb): assert raw T-SQL to DuckDB dialect output`). TODO and state edits excluded; Plan 10 artifacts untouched.
- Targeted command: `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` -> 5 passed, 0 failed.
- `git diff --check` passed; `rustfmt --check` on the test file passed. No `Cargo.toml`/`Cargo.lock`, `src/`, fixture, runtime, CI, or ADR changes.
- Behavior: T-SQL `TOP` -> `LIMIT`, `ISNULL` -> `COALESCE`, `[ident]` -> `"ident"`, `STRING_AGG` -> `LISTAGG`; raw IIF output remains `IIF(...)` (unresolved) with no CASE rewrite.
- Deviations: see the micro-plan deviation below.
- Limitations: translation-only assertions; no DuckDB execution, result mapping, SDA serialization, IIF adaptation, CTAS handling, performance, or production claims.
- Worker model: `opencode-go/deepseek-v4.1-flash`; no escalation. Token/cost telemetry unavailable.
- Orchestrator gates: targeted DuckDB test passed (5); `cargo fmt --check` and `git diff --check` passed; default and configured-SpatiaLite `cargo test` each passed (75 unit + 31 integration, 0 failures).
- Micro-plan deviation: supported translations were split into focused tests rather than one table-driven test; assertions and task scope are unchanged.

## 11.3.1

- Status: done; worker verification and orchestrator wave gates passed.
- Files changed: `tests/duckdb_pipeline_contract.rs` only (test-only synthetic DuckDB/SQLite schemas, DuckDB value adapter, DuckDB and SQLite execution helpers, 4 execution/comparison tests; 5 existing tests preserved).
- Implementation commit: `486e568` (`test(duckdb): execute supported queries and compare SDA JSON`). TODO and state edits excluded; Plan 10 artifacts untouched.
- Targeted command: `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` -> 9 passed, 0 failed.
- `rustfmt --check tests/duckdb_pipeline_contract.rs` and `git diff --check` passed. No `Cargo.toml`/`Cargo.lock`, `src/`, fixture, runtime, CI, or ADR changes.
- Behavior: TOP/brackets/ISNULL, captured muaggatt join, STRING_AGG, and a NULL-preserving bracket case serialize byte-identically on DuckDB and the existing SQLite transpiler/executor/serializer path; column names/order and NULL vs text are asserted.
- Mapping: NULL/booleans/signed and small unsigned integers/floats/doubles/text/blobs map to existing `ResultSet` values; UBigInt/Decimal map to text; all other DuckDB types are explicitly rejected (test-only adapter).
- SQLite comparison uses the real `sda_rs::transpiler::transpile` + `sda_rs::executor::execute` on a seeded shared-memory database (deviation from the Plan 10 hand-rolled loop; task-required).
- Deviations: none beyond that documented SQLite-path choice.
- Limitations: no IIF lowering/execution, no temp-table/CTAS handling, no additional typed values, no performance/security claims; the adapter is not a production mapping contract.
- Worker model: `opencode-go/deepseek-v4.1-flash`; no escalation. Token/cost telemetry unavailable.
- Orchestrator gates: targeted DuckDB test passed (9); `cargo fmt --check` and `git diff --check` passed; default and configured-SpatiaLite `cargo test` each passed (75 unit + 35 integration, 0 failures).

## 11.4.1

- Status: done (worker verification complete); awaiting orchestrator wave gates.
- Files changed: `tests/duckdb_pipeline_contract.rs` (test-only IIF lowering, statement/batch collection, CTAS count suppression, temporal rejection test; 9 existing tests preserved); created `planning/reports/duckdb-test-only-spike.md`; updated this TODO and state.
- Implementation commit: `0a98da1` (`test(duckdb): adapt IIF and batch gaps in test pipeline`). TODO/report/state kept out of that commit; planning artifacts committed separately.
- Targeted command: `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` -> 12 passed, 0 failed.
- `rustfmt --check tests/duckdb_pipeline_contract.rs` and `git diff --check` passed. No `Cargo.toml`/`Cargo.lock`, `src/`, fixture, runtime, CI, ADR, or Plan 10 artifact changes.
- Behavior: raw IIF output remains unresolved and fails on DuckDB prepare (`iif does not exist`); test-only lowering to `CASE WHEN ... THEN ... ELSE ... END` matches SQLite JSON exactly. Raw batch yields `{"Table":[["2"]],"Table1":[["2"],["3"]]}`; test-only CTAS filter adapts it to `{"Table":[["2"],["3"]]}`, matching SQLite.
- Rejection: executed temporal value (`Date32`) is rejected by the adapter with `unsupported/lossy`; no silent stringification.
- Deviations: the raw-IIF failure is asserted at `prepare`, where DuckDB reports the missing function (execution is never reached). No scope change.
- Limitations: IIF lowering and CTAS suppression are test-only, probe-scoped workarounds; no production transpiler/executor/serializer change, typed-value contract, spatial support, performance, or security claim.
- Worker model: `opencode-go/deepseek-v4.1-flash`; no escalation. Token/cost telemetry unavailable.

## Worker Usage

- Record worker model, escalation, and token/cost telemetry availability after task completion.
