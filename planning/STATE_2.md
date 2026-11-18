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
| 2 | 2.2.1 Executor, spatial, routes dead code | pending | — |
| 2 | 2.2.2 Transpiler spatial cleanup and error classification | pending | — |

Wave-boundary verification: wave 1 — done. `cargo fmt --check` passed; `cargo test` with `SDA_TEST_SPATIALITE` set passed (37 unit + 3 integration, 0 failures).

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
