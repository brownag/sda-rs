# Execution Plan 3 — SQLite MVP correctness gaps

## Goal & Scope

### Goal

Close the outstanding correctness gaps that block the SQLite/GeoPackage MVP: atomic batches (G16), isolated temporary tables (G17), correct variable declaration semantics (G18), scoped spatial metadata failures (G19), safe mukey geometry selection (G20), AST-level spatial constructor rewriting (G22), and shared-cache in-memory integration tests (G24). G16 remains first because it is the data-integrity prerequisite for trusting batch behavior. G1 and broader MVP acceptance work remain a follow-on orchestration after these focused gaps are closed.

### In scope

- The focused correctness gaps listed above, sequenced by dependency and file overlap.
- Tests that prove each goal's exit criteria; preserve existing behavior outside those criteria.

### Out of scope / deferred

- End-to-end MVP completion (G1), spatial fixture and suite work (G6/G7), and explicit live-extension acceptance (G23); schedule after the focused correctness tasks in this plan.
- Native SQLite spatial acceleration (G28), PostgreSQL/PostGIS (G10), the full SDA function catalog (G11), broader CLR spatial methods (G12), general APPLY rewrites (G13), additional SDA modes (G14), and wire comparison against live captures (G15).
- Spatial module decomposition (G25), dependency-list reconciliation (G26), and pruned commit-history cleanup (G27).
- Changing error status codes. The existing `ExecutorError::Sqlite` → 500 mapping is kept.
- Changing the request-local connection lifecycle or the `spawn_blocking` boundary.
- Nested transactions or savepoints for user-issued `BEGIN`/`COMMIT`. Security guardrails already reject transaction control; this plan does not add a new path for it. (Verify in recon; if user `BEGIN` reaches execution, report and stop.)

### Verbatim constraints

- Decision D3 (recorded in `planning/GOALS.md`, 2026-11-17): "A failed multi-statement batch rolls back all of its writes."
- GOALS.md G16 exit criteria: "A failing batch leaves rows unchanged and returns 500; a successful batch still commits."
- `planning/EXECUTION_PLAN_1.md` global rule: "Never execute raw multi-statement SQL as a single prepared statement... execute statements sequentially in the same request connection." This plan keeps that rule.

## Reconnaissance log

- Branch `feature/sqlite-mvp`; at plan expansion, HEAD is `35296b8` after integrating wave 1. The main worktree has a user-owned modification to `planning/GOALS.md` and pre-existing untracked `.claude/`; neither is part of this plan.
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
- 2026-11-17 — G16 is one task in wave 1. Keep its implementation narrow; the other correctness goals are subsequent waves because they have separate dependencies and file scopes.
- 2026-11-17 — Rollback is implemented with a transaction around `execute_on`, not by per-statement savepoints. This matches the batch unit and keeps the change small.
- 2026-11-17 — Extend this orchestration with G17–G20, G22, and G24 after the user noticed the original one-task decomposition omitted the remaining sequenced correctness goals. Keep G1 and broad integration-suite work for a follow-on orchestration.
- 2026-11-17 — Amend G22 scope for the parser limitation: retain only a narrow lexical compatibility shim for the exact `geometry::STGeomFromText` syntax outside string literals, because polyglot-sql rejects that syntax before AST parsing. The semantic spatial lowering must happen on the AST; ordinary SQL and string literals remain unchanged.
- 2026-11-17 — G20 SRID exception: when `mupolygon` metadata is absent but an explicit geometry-column override is configured, preserve the existing 4326 SRID fallback. Without an override, reject missing `mupolygon` metadata. This exception is user-directed.
- 2026-11-17 — Amend G20 scope to expose explicit geometry-override provenance from `SpatialCapability`; the current effective-column accessor cannot distinguish an explicit override from another table's discovered geometry column.

## DAG and wave table

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 3.1.1 | Transactional batch execution with rollback, plus tests. |
| 2 | 3.2.1 | G17: isolate `#temp` names from persistent tables. |
| 3 | 3.3.1, 3.3.2 | G18 variable declaration semantics; G19 scoped spatial metadata failures. Disjoint files. |
| 4 | 3.4.1 | G22: rewrite `geometry::STGeomFromText` on the parsed AST. |
| 5 | 3.5.1 | G20: reject unsafe mukey geometry/SRID fallback. Shares transpiler spatial files with G22, so follows it. |
| 6 | 3.6.1 | G24: migrate remaining integration tests to unique shared-cache in-memory databases. |

Dependencies: 3.1.1 → 3.2.1 → {3.3.1, 3.3.2} → 3.4.1 → 3.5.1 → 3.6.1. G18 and G19 have independent code/test files and may run in parallel. Other goals are sequenced to avoid known shared-file collisions. GOALS.md lists a cycle between G20 and G6. Per the user's decision, this plan places G20 before G6: G20's metadata rejection is verified at the transpiler and router layers without G6's live spatial fixture; G6 then consumes the established safe target behavior.

## Global hazards and execution rules

- Do not change `execute_query_blocking`'s spatial and preparation ordering. The checks at ~lines 95-104 stay in place and in order.
- Do not hold the transaction across the `prepare` callback. The transaction starts after `prepare` returns statements, inside `execute_on`'s caller.
- Commit only on full success. Any `?` inside the transaction scope must drop the transaction uncommitted.
- Keep `spawn_blocking` as the only blocking boundary.
- Do not run the full suite in workers. The orchestrator runs `cargo fmt --check` and full `cargo test` serially at the wave boundary.
- Use worktree isolation for multi-task waves. No remote Git actions.

## Task specifications

#### Task 3.1.1 — Roll back failed batches

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

#### Task 3.2.1 — Isolate temporary table names (G17)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/temp_tables.rs`, `src/executor/sqlite.rs` (focused regression test only), `src/transpiler/mod.rs` (update the existing generated-SQL assertion only).
- **Can create:** none.
- **Depends on:** 3.1.1.
- **Spec:** Give each recognized `#temp` table a generated SQLite temporary name that cannot shadow a persistent table. Rewrite references to that same `#name` within the batch to its generated name. Preserve session lifetime and existing unknown hash-table behavior. Add a regression with a persistent `sample` table proving `SELECT COUNT(*) FROM sample` continues to read the persistent table after `SELECT ... INTO #sample`; also verify an explicit `#sample` reference reads the temporary table. Cover `CREATE TABLE #x` followed by `INSERT INTO #x` as required by G17's exit criteria.
- **Do not touch:** `src/routes.rs`, `src/spatial.rs`, `src/transpiler/vars.rs`, `src/transpiler/spatial.rs`, `tests/sda_mvp.rs`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `fn rewrite`, `known.insert`, `CREATE TEMP TABLE`, `executes_multiple_statements_and_keeps_temp_tables_in_session`.
- **Acceptance:** `cargo test temp_tables::`; `cargo test executor::`; `cargo test folds_variables_and_select_into_batch`; `cargo fmt --check`; `git diff --check`.

#### Task 3.3.1 — Correct batch variable declarations (G18)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/vars.rs` (implementation and unit tests).
- **Can create:** none.
- **Depends on:** G4 (completed in plan 1).
- **Spec:** Substitute a declared variable's current value into another variable's default at declaration time (`DECLARE @b INT = 5; DECLARE @a INT = @b; SELECT @a` produces 5). Reject unbound variable defaults rather than turning them into NULL, and reject re-declaration instead of silently resetting an existing variable. Preserve established `SET` and select-assignment behavior.
- **Do not touch:** `src/transpiler/mod.rs`, `src/transpiler/spatial.rs`, `src/routes.rs`, tests outside `vars.rs`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `fn fold`, `DECLARE`, `folds_declare_set_and_select_assignment_case_insensitively`.
- **Acceptance:** `cargo test transpiler::vars::`; `cargo fmt --check`; `git diff --check`.

#### Task 3.3.2 — Scope spatial metadata failures (G19)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/spatial.rs`, `src/routes.rs` (focused regression test only).
- **Can create:** none.
- **Depends on:** G5 (completed in plan 1).
- **Spec:** Malformed or unavailable GeoPackage geometry metadata must not fail ordinary SQL requests such as `SELECT 1`; spatial requests must still return the existing structured metadata/capability error. Keep spatial initialization and error mapping compatible with current callers. Add a route regression covering both ordinary and spatial requests against malformed metadata.
- **Do not touch:** `src/executor/sqlite.rs`, `src/transpiler/*`, `tests/sda_mvp.rs`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `pub fn initialize`, `geometry_metadata_for_table`, `spatial_query_without_extension`, `ExecutorError`.
- **Acceptance:** `cargo test spatial::`; `cargo test routes::`; `cargo fmt --check`; `git diff --check`.

#### Task 3.4.1 — Rewrite spatial constructor calls on the AST (G22)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs` (AST rewrite, narrowly scoped parser-compatibility shim, and tests), `src/transpiler/mod.rs` (only the pre-parse shim invocation).
- **Can create:** none.
- **Depends on:** G4 (completed in plan 1).
- **Spec:** The installed T-SQL parser rejects `geometry::STGeomFromText(...)` before AST parsing. Keep the smallest parser-compatibility shim needed to change only that exact syntax to a parseable qualified call, preserving string literals; keep the semantic spatial function lowering on the parsed AST. Ordinary/non-spatial SQL must remain unchanged by the shim. Preserve existing AOI and spatial predicate behavior; add AST-level tests for constructor contexts, non-spatial statements, and string literals.
- **Do not touch:** `src/spatial.rs`, `src/routes.rs`, `tests/sda_mvp.rs`, other parts of `src/transpiler/mod.rs`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `normalize_static_calls`, `rewrite_spatial`, `STGeomFromText`, `parser_ast_rewrites_geometry_constructor_and_intersects_without_runtime_extension`.
- **Acceptance:** `cargo test transpiler::spatial::`; `cargo test transpiler::`; `cargo fmt --check`; `git diff --check`.

#### Task 3.5.1 — Reject unsafe mukey geometry fallback (G20)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs` (implementation/unit tests), `src/spatial.rs` (narrow override-provenance accessor and focused unit test only), `tests/sda_mvp.rs` (focused integration regression only).
- **Can create:** none.
- **Depends on:** G5; follows 3.4.1 because both goals touch `src/transpiler/spatial.rs`. G20 is scheduled before G6 to resolve the cycle in GOALS.md, as directed by the user.
- **Spec:** Distinguish an explicit configured geometry-column override from a geometry column merely discovered on another table. Add a narrow `SpatialCapability` accessor for override provenance without changing initialization behavior. If `mupolygon` metadata is missing and there is no explicit override, reject the mukey TVF with a structured error rather than selecting another layer. Add unit and integration coverage using a GeoPackage with one non-`mupolygon` geometry layer, both with and without an explicit override. When `mupolygon` metadata exists, use its discovered SRS. If metadata is absent but an explicit override exists, use the overridden column and preserve the existing SRID 4326 fallback, per the user's decision.
- **Do not touch:** `src/routes.rs`, `src/transpiler/mod.rs`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `SpatialCapability`, `geometry_override`, `effective_geometry_column`, `MukeyTarget`, `geometry_metadata_for_table("mupolygon")`, `mukey_rewrite_uses_mupolygon_metadata_not_first_geometry_row`.
- **Acceptance:** `cargo test spatial::`; `cargo test transpiler::spatial::`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_mvp`; `cargo fmt --check`; `git diff --check`.

#### Task 3.6.1 — Use shared-cache in-memory test databases (G24)

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/routes.rs` (test setup only), `tests/sda_mvp.rs` (test setup only).
- **Can create:** none.
- **Depends on:** 3.5.1; decision D1.
- **Spec:** Replace test-only temporary-file SQLite databases with unique named shared-cache in-memory URIs, keeping a holder connection alive for each test and preserving any needed non-database fixture inputs. Do not change production configuration or connection behavior. Confirm no test database path covered by G24 still uses a temporary file.
- **Do not touch:** production implementation outside test-only setup, `src/executor/sqlite.rs`, `src/transpiler/*`, `planning/*` except this task's micro-plan.
- **Grep anchors:** `Connection::open(&path)`, `NamedTempFile`, `fn app`, integration `#[tokio::test]` setup.
- **Acceptance:** `cargo test routes::`; `cargo test --test sda_mvp`; `cargo fmt --check`; `git diff --check`.

## Wave-boundary verification (orchestrator)

- Full serial `cargo test` with `SDA_TEST_SPATIALITE` set, and `cargo fmt --check`.
- For wave 1, confirm the new rollback tests failed on the pre-change code and pass after; report recorded in `STATE_3.md`.
- Run the full suite serially after each wave before marking it done.
