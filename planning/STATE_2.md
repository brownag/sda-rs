# State 2 — Review Cleanup

Plan: `planning/EXECUTION_PLAN_2.md`. Goals: `planning/GOALS.md` [C1]. Source review: 2026-11-17, `master...feature/sqlite-mvp`.

## Baseline (recorded before wave 1)

- Branch: `feature/sqlite-mvp`; working tree clean (only ignored `target/`).
- Last known green (STATE_1, orchestrator wave-boundary verification): `cargo fmt --check` passed; `cargo test` passed (33 unit + 3 integration, 0 failures) with `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so`.
- SpatiaLite: `/usr/lib/x86_64-linux-gnu/mod_spatialite.so` installed locally; live spatial test runs when `SDA_TEST_SPATIALITE` is set.

## Wave board

| Wave | Task | Status | Code commit |
|---|---|---|---|
| 1 | 2.1.1 Serializer cleanup, XML validation, result keys | done | `5193bd1` (worker commit `d62fcbd`) |
| 1 | 2.1.2 Security denylist, tracing, request visitor | done | `a51062b` (worker commit `9017dca`) |
| 2 | 2.2.1 Executor, spatial, routes dead code | done | `07150f4` (worker commit `166e4f8`) |
| 2 | 2.2.2 Transpiler spatial cleanup and error classification | done | `fe72f81` (worker commit `ce8d980`) |

Wave-boundary verification: wave 1 — done. `cargo fmt --check` passed; `cargo test` with `SDA_TEST_SPATIALITE` set passed (37 unit + 3 integration, 0 failures). Wave 2 — done. `cargo fmt --check` passed; full `cargo test` with `SDA_TEST_SPATIALITE` set passed (38 unit + 3 integration, 0 failures). Dead-name grep leaves one expected match: the `geometries.first()` fallback in `src/spatial.rs:162`.

Plan 2 status: complete. All four tasks are verified and committed on `feature/sqlite-mvp`.

## Task reports

## 2.1.1
- Status: done, verified; worker commit `d62fcbd` cherry-picked as `5193bd1`.
- Files: `src/serializer.rs` (duplicate varchar arm removed; `SerializeError`; ASCII XML name validation; non-empty-ordinal keys per D4); `src/routes.rs` (call site only: `Err` → 400).
- Tests: `cargo test serializer::` 6 passed.
- Scope amendment (approved by orchestrator, recorded in EXECUTION_PLAN_2.md): `serialize` is infallible and `routes.rs` was its only caller; the 400 requirement needed the one-line call-site change in this task. Task 2.2.1 still owns the rest of `routes.rs`.
- Decisions applied: ASCII-only XML names; the existing multi-set test expectation changed `Table2` → `Table1`.

## 2.1.2
- Status: done, verified; worker commit `9017dca` cherry-picked as `a51062b`.
- Files: `src/transpiler/security.rs` (Pragma/Attach denied, two tests), `src/main.rs` (tracing subscriber, RUST_LOG default info), `src/request.rs` (unit struct), `Cargo.toml` (+1 line), `Cargo.lock` (+75 lines, no bumps).
- Tests: `cargo test transpiler::security` 2 passed; the parser does produce both variants, so the direct `validate` fallback was not needed.
- Note: `cargo test request::` runs 0 tests in this repo.

## 2.2.1
- Status: done, verified; worker commit `166e4f8` cherry-picked as `07150f4`.
- Files: `src/executor/sqlite.rs` (SqliteExecutor, execute_blocking, execute_spatial_blocking deleted; single `:memory:` branch; `open_initialized` helper), `src/executor/mod.rs`, `src/spatial.rs` (geometry field and geometry_metadata() removed), `src/routes.rs` (extension seam removed; test renamed and body asserted as `{"Table":[["1"]]}`).
- Tests: executor 5, spatial 11, routes 3, sda_mvp 3 passed. Dead-name grep: one expected match (`src/spatial.rs:162`).

## 2.2.2
- Status: done, verified; worker commit `ce8d980` cherry-picked as `fe72f81`.
- Files: `src/transpiler/spatial.rs` only. Dead branch and `let _` removed; arity check extracted; `MukeyTarget { column, srid: i32 }` built once; `classify_error` maps `InvalidInput` to `Rejected`.
- Tests: transpiler::spatial 7 passed (including the new classification test); sda_mvp 3 passed, unchanged.
