# Execution Plan 3 — G16: A failed batch commits nothing

## Goal & Scope

### Goal

Make multi-statement SQL batches atomic. When any statement in a request's batch fails, every write made by earlier statements in that batch is rolled back, the request returns HTTP 500, and the database is unchanged. A successful batch still commits. This is GOALS.md goal `[G16]`, the circled goal, which closes the only data-integrity defect from the 2026-11-17 review.

### In scope

- The per-request execution path in `src/executor/sqlite.rs`: wrap statement execution in one transaction with rollback on error.
- Tests for the failing-batch rollback, the successful-batch commit, and the existing temp-table session behavior.

### Out of scope / deferred

- `#temp` name isolation (G17), `DECLARE` substitution (G18), metadata failure scoping (G19), mukey fallback (G20), AST-level `STGeomFromText` (G22). Next plans.
- Changing error status codes. The existing `ExecutorError::Sqlite` → 500 mapping is kept.
- Changing the request-local connection lifecycle or the `spawn_blocking` boundary.
- Nested transactions or savepoints for user-issued `BEGIN`/`COMMIT`. Security guardrails already reject transaction control; this plan does not add a new path for it. (Verify in recon; if user `BEGIN` reaches execution, report and stop.)

### Verbatim constraints

- Decision D3 (recorded in `planning/GOALS.md`, 2026-11-17): "A failed multi-statement batch rolls back all of its writes."
- GOALS.md G16 exit criteria: "A failing batch leaves rows unchanged and returns 500; a successful batch still commits."
- `planning/EXECUTION_PLAN_1.md` global rule: "Never execute raw multi-statement SQL as a single prepared statement... execute statements sequentially in the same request connection." This plan keeps that rule.

## Reconnaissance log

- Branch `feature/sqlite-mvp`; working tree clean except untracked `.claude/` (worktree directory, not part of this plan). HEAD `57c7a8f`.
- Baseline (plan 2, wave 2 boundary): `cargo fmt --check` passed; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (38 unit + 3 integration, 0 failures). Record this as the pre-wave baseline.
- `src/executor/sqlite.rs` is 232 lines. Anchors:
  - `execute_on` at ~line 132: sequential `connection.prepare` / `execute` / `query` per statement, with no transaction. Autocommit means each statement commits on its own.
  - `execute_query_blocking` at ~line 82: calls `execute_on(&connection, &statements)?` at ~line 105. Preparation and spatial checks run before execution, so they are already outside the batch.
  - `execute` (public, used by tests) at ~line 50 also reaches `execute_on`. Confirm during analysis.
  - Test module at ~line 169; existing tests include `executes_multiple_statements_and_keeps_temp_tables_in_session` (~line 192), which must still pass.
- Error path: `ExecutorError` has `From<rusqlite::Error>` (~line 32). `src/routes.rs:96` maps only `Preparation` to 400; execution errors go to 500 at the existing arm. Do not change routes.
- rusqlite 0.32 (`Cargo.toml`). Reference pattern: `Connection::unchecked_transaction(&self) -> Result<Transaction>`; `Transaction` derefs to `Connection`; dropping an uncommitted `Transaction` rolls back. Verify the exact API in the installed crate source (`~/.cargo/registry/src/*/rusqlite-0.32*`) before implementation.
- SQLite supports transactional DDL, so `CREATE TEMP TABLE` inside the batch rolls back with the batch. Temp tables are per-connection, so the session-visibility behavior is unchanged.
- Test infrastructure: `cargo test` is in-process with `:memory:` for unit tests and `tests/sda_mvp.rs` uses the Axum router via `oneshot`. Decision D1 (shared-cache in-memory DBs) applies. Each request opens its own connection, so a plain `:memory:` DB disappears between requests. Use a named shared-cache URI instead, `file:g16_<unique>?mode=memory&cache=shared`, and keep one holder connection open for the whole test so the DB persists. Verify during analysis that `Connection::open` in rusqlite 0.32 accepts URIs (its default flags include `SQLITE_OPEN_URI`); if not, stop and report.
- No `docs/decisions/`, `AGENTS.md`, or `CONTRIBUTING.md` in the repo. Standards source is `planning/` and the commit template.

## Dated decisions

- 2026-11-17 — D3: a failed multi-statement batch rolls back all of its writes (from `planning/GOALS.md`).
- 2026-11-17 — Track planning artifacts; commit the approved plan with `docs(planning):`, separate from code.
- 2026-11-17 — G16 is one task in one wave. The change is confined to one function and its tests, so a second task would only add coordination cost.
- 2026-11-17 — Rollback is implemented with a transaction around `execute_on`, not by per-statement savepoints. This matches the batch unit and keeps the change small.

## DAG and wave table

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 3.1.1 | Transactional batch execution with rollback, plus tests. |

No dependencies. Single task, so no file-disjointness question arises.

## Global hazards and execution rules

- Do not change `execute_query_blocking`'s spatial and preparation ordering. The checks at ~lines 95-104 stay in place and in order.
- Do not hold the transaction across the `prepare` callback. The transaction starts after `prepare` returns statements, inside `execute_on`'s caller.
- Commit only on full success. Any `?` inside the transaction scope must drop the transaction uncommitted.
- Keep `spawn_blocking` as the only blocking boundary.
- Do not run the full suite in workers. The orchestrator runs `cargo fmt --check` and full `cargo test` serially at the wave boundary.
- Worktree isolation is used for the single task. No remote Git actions.

## Task specifications

### Task 3.1.1 — Roll back failed batches

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/executor/sqlite.rs` (transaction and unit tests), `tests/sda_mvp.rs` (router-level rollback test only).
- **Can create:** none.
- **Depends on:** none.
- **Spec:**
  1. In `execute_on` (~line 132), run all statements inside one transaction. Use `connection.unchecked_transaction()` (or the rusqlite 0.32 equivalent verified in analysis). Execute statements with the existing loop body unchanged. On any error, the `?` returns and the uncommitted transaction rolls back. On success, call `commit()` and return the result sets. Map the commit error through the existing `From<rusqlite::Error>`.
  2. Do not change `execute_query_blocking`'s ordering, the spatial checks, or the `ExecutorError` variants.
  3. Add a unit test in the existing tests module using a named shared-cache in-memory URI (`file:g16_<unique>?mode=memory&cache=shared`, per D1). Hold one connection open for the test's lifetime. Create `t(x INTEGER)` with a seed row through the holder, then call `execute` with the URI and `["UPDATE t SET x = 2", "SELECT * FROM missing"]`. Expect `Err`, then query `x` through the holder and expect the seed value.
  4. Add a unit test: a successful multi-statement batch (`UPDATE` then `SELECT`) commits, and a fresh connection sees the update.
  5. Add an integration test in `tests/sda_mvp.rs` that uses a named shared-cache in-memory URI (D1), holds a connection open for the test, and seeds `t`. It POSTs `UPDATE t SET x = 2; SELECT * FROM missing` through the router and asserts HTTP 500, then checks through the holder that `x` is unchanged. Follow the existing `oneshot` pattern in that file and its `Config` setup.
  6. Keep the existing `executes_multiple_statements_and_keeps_temp_tables_in_session` test passing unchanged.
- **Do not touch:** `src/routes.rs`, `src/spatial.rs`, `src/transpiler/*`, `src/serializer.rs`, `Cargo.toml`, the other tests in `tests/sda_mvp.rs`, `planning/*` (except this task's micro-plan).
- **Grep anchors:** `fn execute_on`, `fn execute_query_blocking`, `fn execute(`, `executes_multiple_statements_and_keeps_temp_tables_in_session`, `fn app(`, `oneshot`.
- **Acceptance:**
  - `cargo test executor::`
  - `cargo test --test sda_mvp`
  - `cargo fmt --check`
  - `git diff --check`
  - Wave-boundary (orchestrator): `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serial, all green.

## Wave-boundary verification (orchestrator)

- Full serial `cargo test` with `SDA_TEST_SPATIALITE` set, and `cargo fmt --check`.
- Confirm the new rollback tests fail on the pre-change code (the worker should record a red run before the change), then pass after.
- Confirm `git diff master...HEAD -- src/routes.rs` shows no change from this task.
