# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-17
Source State: STATE_2.md (plan 2 complete); code review of `master...feature/sqlite-mvp` (2026-11-17)

## Circled Goal (Active)

- **[G16]** A failed batch commits nothing: multi-statement execution runs in one transaction and rolls back on any statement error.
  - Why now: This is the only item from the review that can lose or corrupt data, and it is confined to the executor. Batch tests in G7 cannot be trusted until it is fixed. It is also small enough to finish in one plan.
  - Exit criteria: A failing batch leaves rows unchanged and returns 500; a successful batch still commits. Decision D3 governs the rule. Test covers `UPDATE t SET x=1; SELECT * FROM missing` (expect zero rows changed) and a successful multi-statement batch.
  - First action: Write the failing rollback test in `src/executor/sqlite.rs` tests, then wrap `execute_on` in `BEGIN`/`COMMIT` with rollback on error.

## Sequenced Goals (Dependency DAG)

- **[G1]** A SQLite/GeoPackage-focused SDA tabular server executes a spatial MVP end to end: it accepts JSON and form-encoded POST requests, transpiles supported T-SQL, runs against SQLite with optional SpatiaLite support and GeoPackage geometry metadata discovery, and returns SDA-compatible output.
  - Depends on: G16, G17, G18, G19, G20 (the remaining correctness gaps)
  - Why now: This proves the highest-risk end-to-end path before investing in broader compatibility or another backend.
  - Exit criteria: Automated local tests cover request parsing, security rejection, basic query execution, `TOP`/`ISNULL`/`IIF`/`STRING_AGG` and bracketed identifiers, `DECLARE @aoi` with `STGeomFromText` and `STIntersects`, the `SDA_Get_Mukey_from_intersection_with_WktWgs84` function in supported query contexts, `#temp` batches, and exact `JSON+COLUMNNAME+METADATA` structure. `cargo fmt --check` and `cargo test` pass; spatial tests exercise SpatiaLite when available and otherwise verify the documented unavailable-capability behavior.

- **[G17]** `#temp` names cannot shadow persistent tables.
  - Depends on: G16 (same executor path; batch semantics settled first)
  - Evidence: State-machine review item 2. `SELECT COUNT(*) FROM sample` after `#sample` reads the temp table (`src/transpiler/temp_tables.rs:23-53`).
  - Exit criteria: A persistent `sample` table is unaffected by `SELECT ... INTO #sample`. Test covers that case and `CREATE TABLE #x` / `INSERT INTO #x`.

- **[G18]** Batch variables fold correctly: `DECLARE` defaults substitute at declaration time, and re-`DECLARE` of an existing variable is rejected.
  - Depends on: G4
  - Evidence: State-machine review item 3 (potential). `DECLARE @a INT = @b` stores the raw default (`src/transpiler/vars.rs:22-27`, `:117-129`). Re-`DECLARE` silently resets (`:22`).
  - Exit criteria: `DECLARE @b INT = 5; DECLARE @a INT = @b; SELECT @a` returns 5. An unbound variable is a rejection, never NULL. Re-`DECLARE` is rejected. Unit tests in `vars.rs`.

- **[G19]** Spatial metadata failure is scoped to spatial queries.
  - Depends on: G5
  - Evidence: State-machine review item 4. A malformed `gpkg_geometry_columns` makes every request fail in `spatial::initialize` (`src/spatial.rs:134-164`), so `SELECT 1` returns 500.
  - Exit criteria: With malformed metadata, `SELECT 1` returns 200; a spatial query returns the structured metadata error.

- **[G20]** The mukey TVF never uses another layer's geometry or an assumed SRID.
  - Depends on: G5, G6
  - Evidence: State-machine review item 5 (potential). Without `mupolygon` metadata the rewrite falls back to the first geometry row and SRID 4326 (`src/transpiler/spatial.rs`, `MukeyTarget` fallbacks, now bundled by plan 2).
  - Exit criteria: Without `mupolygon` metadata and without an explicit override, the query is rejected with a structured error. A GeoPackage with one non-`mupolygon` layer has a regression test. The SRID is the discovered SRS, not a default.

- **[G22]** The transpiler applies the `geometry::STGeomFromText` rewrite on the parsed AST, not as text, so it runs only where it is needed.
  - Depends on: G4
  - Evidence: Spec 1.4.1 (`planning/EXECUTION_PLAN_1.md:164`). `normalize_static_calls` rewrites text on every request, including non-spatial ones (`src/transpiler/spatial.rs:13-45`).
  - Exit criteria: Non-spatial queries do not pass through the text rewrite. The existing AOI tests still pass.

- **[G7]** The integration suite uses deterministic mock SSURGO tables and verifies the MVP endpoint, serializer, batch, temp-table, and spatial behavior.
  - Depends on: G3, G4, G6, G16, G17

- **[G6]** T-SQL variable folding, CLR spatial rewriting, and the WGS84 mukey intersection UDF execute end to end against seeded spatial fixtures.
  - Depends on: G4, G5, G18, G20

- **[G23]** Live SpatiaLite execution is exercised, not skipped.
  - Depends on: G6
  - Status note: `mod_spatialite` is installed locally (`/usr/lib/x86_64-linux-gnu/mod_spatialite.so`), and wave-boundary runs set `SDA_TEST_SPATIALITE`. Confirm the live test actually executes and does not early-return (`tests/sda_mvp.rs:115`); until confirmed, treat this as unverified.
  - Exit criteria: The conditional spatial test runs and passes with the extension configured, with no early return.

- **[G24]** Integration tests follow the in-memory DB rule.
  - Depends on: D1 (decided: shared-cache in-memory DBs)
  - Evidence: Standards hard breach. `planning/EXECUTION_PLAN_1.md:48` and `planning/INITIAL_PROMPT.md:62` require in-memory DBs. The current tests use temp-file SQLite (`src/routes.rs`, `tests/sda_mvp.rs:16`, `:117`).
  - Exit criteria: Tests use shared-cache in-memory DBs with a unique name per test.

## Independent Backlog

- **[G8]** The server provides CLI and environment configuration for backend URL, port, and geometry settings.
- **[G9]** Request handling includes CORS, tracing, and bounded payload behavior. Tracing is done (plan 2, `a51062b`); the payload-limit configuration is open.
- **[G25]** The transpiler's spatial module is split by reason to change: string normalization, derived-table expansion, and method/constructor rewriting.
  - Evidence: Divergent Change smell, `src/transpiler/spatial.rs`. `"mupolygon"` is hard-coded.
  - Depends on: G22
- **[G26]** The dependency list matches INITIAL_PROMPT, or the deviation is recorded.
  - Evidence: `planning/INITIAL_PROMPT.md` names `serde_json`, `quick-xml`, and `urlencoded`. None are in `Cargo.toml`; the serializer is hand-rolled. Only the JSON deviation is recorded in STATE_1.md.

## Deferred

- **[G10]** PostgreSQL/PostGIS backend executes the same supported query surface.
  - Trigger: SQLite/GeoPackage MVP passes its acceptance suite and backend interfaces are stable.
- **[G11]** The full documented `SDA_Get_*` function catalog is supported, including table-valued macros and table-input variants.
  - Trigger: The MVP WGS84 mukey intersection UDF is covered by passing integration tests.
- **[G12]** The broader CLR spatial method set—including `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—is supported with documented unit/SRID semantics.
  - Trigger: AOI construction and intersection behavior is stable on supported GeoPackage schemas.
- **[G13]** SQLite compatibility includes general `CROSS APPLY` and `OUTER APPLY` rewrites.
  - Trigger: A concrete supported workload demonstrates the need beyond the MVP query set.
- **[G14]** The service implements SDA modes beyond tabular query, including AOI, interpretation, and report services, plus Shape commands.
  - Trigger: A user requirement identifies one of these modes as a priority.
- **[G15]** Wire behavior is verified against live SDA captures or client implementations for undocumented wrapper, empty-set, XML, and metadata edge cases.
  - Trigger: Local structural tests pass and an authoritative comparison source is available.

## Completed

- **[C1]** Review cleanup: dead code, duplication, error classification, security denylist, tracing. Plan 2, commits `a51062b`, `5193bd1`, `07150f4`, `fe72f81`.
- **[G21]** Serializer XML element validation and result-set key numbering (D4). Plan 2, commit `5193bd1`.

## Blocked / Pruned

- **[G27]** Rewrite the 14 non-compliant commit subjects that contain wave/phase numbers (`planning/PROMPT_TEMPLATE_1.md:23`).
  - Status: Pruned
  - Reason: Rewriting history on a feature branch is destructive, and the rule can be enforced from here on. Revisit only if the branch is unpushed and the user asks.

## Decisions (2026-11-17)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, and the `SpatialCapability.geometry` field with `geometry_metadata()`. Done.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering. Done.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
