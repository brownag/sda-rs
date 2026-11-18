# Orchestration State 1

Plan: `planning/EXECUTION_PLAN_1.md`

## Baseline checklist

- [ ] `cargo fmt --check` — not runnable before crate scaffold; run after task 1.1.1.
- [ ] `cargo test` — not runnable before crate scaffold; run after task 1.1.1.
- [ ] In-process Axum tests use no fixed/shared TCP ports.
- [ ] SpatiaLite extension availability recorded; missing extension has deterministic unavailable-capability tests.
- Git baseline: repository on `master`, no commits at plan authoring; planning sources untracked. Do not switch branch or initialize Git.
- Conventions/ADR baseline: no repository `AGENTS.md`, `CONTRIBUTING.md`, accepted ADRs, prior plans, or test suites found.

## Wave board

| Wave | Tasks | Status |
|---|---|---|
| 1 | 1.1.1 | done |
| 2 | 1.2.1 | done |
| 3 | 1.2.2 | done |
| 4 | 1.3.2 | done |
| 5 | 1.3.1 | done |
| 6 | 1.4.1, 1.4.2 | done |
| 7 | 1.5.1 | in progress |

## Task reports

## 1.3.1
- Status: implemented and verified on `feature/sqlite-mvp`.
- Planning micro-plan: `b42cac3`; code commit: `f25031b` (`feat(routes): execute SDA tabular requests`).
- Files: six authorized source files (`routes.rs`, `request.rs`, `lib.rs`, `main.rs`, `executor/mod.rs`, `executor/sqlite.rs`).
- Configured SQLite path flows through app state; each request opens its own connection inside `spawn_blocking`, preserving batch session semantics.
- Supports JSON/form requests, case-insensitive fields, query echo, transpilation, execution, serialization, and structured errors.
- `cargo test routes::`: passed, 1 passed, 0 failed; mixed-case JSON/form fields, echo, missing/unsafe query, and configured temporary database response verified.
- `cargo fmt --check` and `git diff --check`: passed.
- Deviation: route handler exercised in process without TCP; no dependencies or out-of-scope files changed.
- Acceptance follow-up: `5fc775d` (`test(routes): exercise registered router endpoints`) adds `tower` as a dev-dependency and exercises `app(config).oneshot` for both paths, both encodings, configured DB, echo, missing and unsafe queries.
- Follow-up checks: `cargo test routes::` — 1 passed; `cargo fmt --check` and `git diff --check` passed.
- Follow-up resolves the initial handler-level-only test deviation; registered route wiring is now covered through `app(config).oneshot`.
- Orchestrator wave-boundary verification: `cargo fmt --check` passed; `cargo test` passed (14 passed, 0 failed; binary/doc suites 0 tests).

## 1.3.2
- Status: implemented and verified on `feature/sqlite-mvp`.
- Files: `src/serializer.rs`, `src/lib.rs` (serializer module registration only).
- Formats: JSON, JSON+COLUMNNAME, JSON+COLUMNNAME+METADATA, and XML; JSON values stringify non-NULL SQLite values and preserve NULL; table names retain result-set indexes; empty result sets are omitted.
- Metadata mappings are explicitly provisional and inferred from SQLite dynamic values; no exact SDA parity claim.
- `cargo test serializer::`: 4 passed, 0 failed (9 filtered); binary suite 0 tests.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Deviations: used std-only JSON encoding because `serde_json` is not a dependency and adding/changing dependency files was outside task scope.
- Planning and implementation commits are separate; no remote actions.
- Orchestrator wave-boundary verification: `cargo fmt --check` passed; `cargo test` passed (13 passed, 0 failed; binary/doc suites 0 tests).

## 1.1.1
- Status: implemented and verified on `feature/sqlite-mvp`.
- Planning commit: `0133d85` (`docs(planning): record task 1.1.1 micro-plan`).
- Files changed: `Cargo.toml`, `Cargo.lock`, `src/config.rs`, `src/lib.rs`, `src/main.rs`.
- Added CLI/env config, default/override unit test, router endpoints, CORS/tracing/body limit, and runnable server; no spatial initialization.
- `cargo check`: passed.
- `cargo test`: 1 passed, 0 failed (plus empty binary/doc test suites).
- `cargo fmt --check`: passed after formatting.
- Deviations: none.
- Gotchas: route handlers are intentional 501 stubs; later route task replaces them.

## 1.2.1
- Status: implemented and verified on `feature/sqlite-mvp`; checkout/worktree: `/home/andrew/workspace/sda-rs` (sole worktree).
- Planning commit: `56a5dac` (`docs(planning): sequence shared module registration`); completion update committed separately below.
- Code commit: `ca7e0e5` (`feat(executor): add request-local SQLite execution`).
- Files: `src/result.rs`, `src/executor/mod.rs`, `src/executor/sqlite.rs`, `src/lib.rs` (executor/result registration only).
- Tests: `cargo test executor::` — 4 passed, 0 failed (one unrelated test filtered); binary suite 0 tests.
- Additional checks: `cargo fmt --check` passed; `git diff --check` passed.
- Behavior: sequential statements share one in-memory connection; row result sets preserve column order, dynamic SQLite values/NULL, and empty-set columns; async API uses `spawn_blocking`.
- Deviations: none. Remaining untracked `target/` was pre-existing and untouched.
- Gotchas: `src/lib.rs` does not register `transpiler`; task 1.2.2 may add only its authorized module registration.

## 1.2.2
- Status: implemented and verified on `feature/sqlite-mvp`; worktree `/home/andrew/workspace/sda-rs`.
- API verified from `polyglot-sql 0.14.1`: `Dialect::parse`, `Dialect::transpile`, `Dialect::generate`, `ExpressionWalk::dfs`, `transform_all`.
- Files: `src/transpiler/{mod.rs,normalize.rs,security.rs}`, `src/lib.rs` (module registration only).
- `cargo test transpiler::`: 4 passed, 0 failed (5 filtered); binary suite 0 tests.
- `cargo fmt` and `git diff --check` passed.
- AST safety rejects DROP/ALTER/TRUNCATE/GRANT/EXEC/WAITFOR; permits only temporary `#name` CREATE TABLE.
- SQLite dialect pipeline handles bracket identifiers, TOP, IIF, STRING_AGG; AST pass maps ISNULL to COALESCE.
- Planning report and implementation commits are separate; no remote actions.
- Deviations: none.
- Orchestrator wave-boundary verification: `cargo fmt --check` passed; `cargo test` passed (9 passed, 0 failed; binary/doc suites 0 tests).

## 1.4.1
- Status: implemented and verified on `feature/sqlite-mvp`.
- Planning micro-plan: `20ec701`; authorized scope revision: `f57409a`; implementation commit: `15fe592` (`feat(transpiler): fold batch vars and temp tables`).
- Files: `src/transpiler/{mod.rs,normalize.rs,vars.rs,temp_tables.rs}` only; removed temporary AST inspection test; no shared module registration needed.
- Pipeline now runs validated source AST through variable folding and temp-table rewriting, then T-SQL-to-SQLite dialect conversion and SQLite normalization/generation.
- `cargo test transpiler::vars`: 2 passed; `cargo test transpiler::temp_tables`: 2 passed.
- `cargo test transpiler::`: 9 passed; `cargo fmt --check` and `git diff --check` passed.
- Unsupported forms reject: multi-item/unequal SET, assignments to undeclared vars, non-sole SELECT variable assignment, unbound vars, and non-`#temp` SELECT INTO.
- No remote actions; pre-existing `target/` preserved.

## 1.4.2
- Status: implemented and verified on `feature/sqlite-mvp`.
- Scope: `src/spatial.rs`, `src/lib.rs`, `src/routes.rs`, `src/executor/mod.rs`, `src/executor/sqlite.rs`, and `Cargo.toml` solely to enable rusqlite's existing `load_extension` feature; no new dependency or version change.
- Per-request blocking executor loads a configured extension under `LoadExtensionGuard`, verifies `spatialite_version()`, discovers optional `gpkg_geometry_columns` table/column/SRS metadata, and exposes `SpatialCapability` + effective geometry-column override.
- Plain SQLite remains valid with unavailable spatial capability and absent GeoPackage metadata. `require_spatial()` returns a typed unavailable error.
- `cargo test spatial::`: 3 passed; `cargo test routes::`: 2 passed; `cargo fmt --check`: passed; `git diff --check`: passed.
- SpatiaLite availability: no `mod_spatialite` extension found; only `libspatialite.so` shared library present, so extension smoke test was not applicable. Missing-extension behavior is covered deterministically.
- No AOI/UDF rewrites, transpiler/serializer changes, or remote actions. Existing untracked `target/` preserved.
- Review follow-up: `execute_query_blocking` now provides capability/metadata to a pre-transpile callback on the same request-local connection; spatial-required source or generated SQL is rejected with structured `SpatialError::Unavailable` before SQLite execution.
- Follow-up tests: `cargo test spatial::` — 5 passed; `cargo test routes::` — 3 passed; `cargo fmt --check` and `git diff --check` passed.
- Follow-up implementation and planning/report commits are recorded separately.
- Orchestrator wave-boundary verification after both tasks: `cargo fmt --check` passed; `cargo test` passed (27 passed, 0 failed; binary/doc suites 0 tests).

## 1.5.1
- Status: implementation committed, metadata-selection follow-up required before completion; code commit `704d7ed`, task report `30b5ec6`.
- Files: `src/transpiler/spatial.rs`, `src/transpiler/mod.rs`, `src/routes.rs`, `src/spatial.rs` (follow-up authorized), `tests/sda_mvp.rs`.
- Supports AOI construction/intersection, folded AOI variables, mukey TVF in supported IN-subquery/FROM/JOIN contexts, and dynamic geometry-column rewrites.
- Unsupported: nonliteral/correlated WKT, column-alias lists, qualified method-call TVF ASTs, and scalar contexts beyond sole projection; these return structured errors.
- `cargo test transpiler::spatial`: passed; `cargo test --test sda_mvp`: passed (2 tests); `cargo fmt --check` and `git diff --check`: passed.
- No loadable `mod_spatialite` was available. Conditional live-extension test early-returned; no live spatial execution is claimed.
- Orchestrator wave-boundary verification: `cargo fmt --check` passed; `cargo test` passed (33 unit tests + 2 integration tests, 0 failures; binary/doc suites 0 tests).
- Remaining acceptance gap: capability currently exposes only the first `gpkg_geometry_columns` row; follow-up must select `mupolygon` from multiple rows.
