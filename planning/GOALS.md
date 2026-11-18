# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-17
Source State: STATE_1.md; code review of `master...feature/sqlite-mvp` (2026-11-17, Standards / Spec / Subtractive / State-machine axes)

## Circled Goal (Active)

- **[C1]** Every small, disconnected, or dead-code defect from the 2026-11-17 review is resolved or explicitly decided, so the remaining work can run as large dependency-ordered plans without interruption.
  - Why now: These items are cheap and independent. Leaving them in place costs every later plan a context switch and a detour. Clearing them first reduces the scope of the big plans that follow.
  - Exit criteria:
    - Confirmed dead code removed: duplicate `varchar` arm (`src/serializer.rs:148`), `PhantomData` in `RequestVisitor` (`src/request.rs:15`).
    - Conditional deletions (D2) either removed or recorded as kept with a reason: `execute_spatial_blocking`, `SqliteExecutor`, `execute_blocking`, `SpatialCapability.geometry` field and `geometry_metadata()`.
    - Dead branch removed: `let _ = function;` and the unreachable `"geometry::STGeomFromText"` check (`src/transpiler/spatial.rs:216`, `:229`).
    - Duplicated pieces collapsed: `":memory:"` branch (`src/executor/sqlite.rs:109`, `:167`); STGeomFromText arity check (`src/transpiler/spatial.rs:233`, `:260`); mukey column/SRID fallbacks bundled once (`src/transpiler/spatial.rs:108-134`), with `mukey_srid` no longer an `Option`.
    - Error classification fixed: validation failures in `src/transpiler/spatial.rs:85`, `:88` return `Rejected`, not `Generate`.
    - Security guardrail denies `Expression::Pragma` and `Expression::Attach` (`src/transpiler/security.rs`), with tests.
    - Tracing subscriber installed in `src/main.rs`, so `TraceLayer` output is emitted.
    - Response-extension seam for `SpatialCapability` (`src/routes.rs:108`) removed or justified.
    - `cargo fmt --check` and `cargo test` pass after each change.
  - First action: Resolve decision D2 (keep or delete the Conditional items), then remove the Confirmed items in one commit.

## Sequenced Goals (Dependency DAG)

- **[G1]** A SQLite/GeoPackage-focused SDA tabular server executes a spatial MVP end to end: it accepts JSON and form-encoded POST requests, transpiles supported T-SQL, runs against SQLite with optional SpatiaLite support and GeoPackage geometry metadata discovery, and returns SDA-compatible output.
  - Depends on: C1 (cleanup first, so the MVP's remaining gaps are not mixed with dead code)
  - Why now: This proves the highest-risk end-to-end path—T-SQL compatibility, GeoPackage geometry handling, spatial execution, and SDA serialization—before investing in broader compatibility or another backend.
  - Exit criteria: Automated local tests cover request parsing, security rejection, basic query execution, `TOP`/`ISNULL`/`IIF`/`STRING_AGG` and bracketed identifiers, `DECLARE @aoi` with `STGeomFromText` and `STIntersects`, the `SDA_Get_Mukey_from_intersection_with_WktWgs84` function in supported query contexts, `#temp` batches, and exact `JSON+COLUMNNAME+METADATA` structure. `cargo fmt --check` and `cargo test` pass; spatial tests exercise SpatiaLite when available and otherwise verify the documented unavailable-capability behavior. Also satisfied by G16–G21 below.
  - First action: After C1, pick G16 (batch atomicity) or G17 (temp-name isolation), whichever decision D3 selects first.

- **[G2]** The service exposes `/Tabular/post.rest` and `/Tab/post.rest`, accepts JSON and URL-encoded requests with case-insensitive query/format fields, and executes safe basic T-SQL against SQLite.
  - Depends on: G1

- **[G3]** The service serializes zero or more result sets into SDA JSON, `JSON+COLUMNNAME`, `JSON+COLUMNNAME+METADATA`, and XML formats with verified structural tests.
  - Depends on: G2

- **[G4]** The SQLite execution path supports same-session multi-statement batches, `#temp` rewriting, and the MVP scalar/function compatibility set.
  - Depends on: G2

- **[G5]** The SQLite spatial path optionally loads SpatiaLite, discovers GeoPackage geometry columns and SRIDs from `gpkg_geometry_columns`, and clearly reports when requested spatial capability is unavailable.
  - Depends on: G2

- **[G6]** T-SQL variable folding, CLR spatial rewriting, and the WGS84 mukey intersection UDF execute end to end against seeded spatial fixtures.
  - Depends on: G4, G5

- **[G7]** The integration suite uses deterministic mock SSURGO tables and verifies the MVP endpoint, serializer, batch, temp-table, and spatial behavior.
  - Depends on: G3, G4, G6

- **[G16]** A failed batch commits nothing: multi-statement execution runs in one transaction and rolls back on any statement error.
  - Depends on: G4
  - Evidence: State-machine review item 1. `UPDATE t SET x=1; SELECT * FROM missing` currently commits the update and returns 500 (`src/executor/sqlite.rs:199-234`).
  - Exit criteria: A failing batch leaves rows unchanged and returns 500; a successful batch still commits. Decision D3 governs the rule.

- **[G17]** `#temp` names cannot shadow persistent tables.
  - Depends on: G4
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
  - Evidence: State-machine review item 5 (potential). Without `mupolygon` metadata the rewrite falls back to the first geometry row and SRID 4326 (`src/transpiler/spatial.rs:55-70`). Spec 1.5.1 says "rather than relying on the first `gpkg_geometry_columns` row."
  - Exit criteria: Without `mupolygon` metadata and without an explicit override, the query is rejected with a structured error. A GeoPackage with one non-`mupolygon` layer has a regression test. The SRID is the discovered SRS, not a default.

- **[G21]** The serializer emits valid XML for any column name, and the result-set key naming matches the spec.
  - Depends on: G3
  - Evidence: Spec review. Column names are used as XML element names without validation (`src/serializer.rs:169-185`). Keys are indexed by absolute position, so an empty first set yields `Table1` (`src/serializer.rs:46`). Decision D4 governs key naming.
  - Exit criteria: Names with spaces, leading digits, or empty strings produce well-formed XML, or the request is rejected with a clear error. Key naming matches D4.

- **[G22]** The transpiler applies the `geometry::STGeomFromText` rewrite on the parsed AST, not as text, so it runs only where it is needed.
  - Depends on: G4
  - Evidence: Spec 1.4.1 (`planning/EXECUTION_PLAN_1.md:164`). `normalize_static_calls` rewrites text on every request, including non-spatial ones (`src/transpiler/spatial.rs:13-45`).
  - Exit criteria: Non-spatial queries do not pass through the text rewrite. The existing AOI tests still pass.

- **[G23]** Live SpatiaLite execution is exercised, not skipped.
  - Depends on: G6
  - Status note: needs a loadable `mod_spatialite` on the test machine. Only `libspatialite.so` is present today, so this is partly external. The local part is installing the extension and un-skipping `tests/sda_mvp.rs:115`.
  - Exit criteria: The conditional spatial test runs and passes on a machine with the extension, with no early return.

- **[G24]** Integration tests follow the in-memory DB rule, or the rule is amended in the plan.
  - Depends on: D1 (user decision)
  - Evidence: Standards hard breach. `planning/EXECUTION_PLAN_1.md:48` and `planning/INITIAL_PROMPT.md:62` require in-memory DBs. The current tests use temp-file SQLite (`src/routes.rs:159`, `:257`, `tests/sda_mvp.rs:16`, `:117`).
  - Exit criteria: Per D1: either tests use shared-cache in-memory DBs, or the plan documents the file-based exception with a reason.

## Independent Backlog

- **[G8]** The server provides CLI and environment configuration for backend URL, port, and geometry settings.
- **[G9]** Request handling includes CORS, tracing, and bounded payload behavior. Tracing output is covered by C1; the remaining payload-limit configuration is open.
- **[G25]** The transpiler's spatial module is split by reason to change: string normalization, derived-table expansion, and method/constructor rewriting.
  - Evidence: Divergent Change smell, `src/transpiler/spatial.rs`. `"mupolygon"` is hard-coded at `:66`.
  - Depends on: G22 (the split is easier once the text pass is gone)
- **[G26]** The serializer and dependency list match INITIAL_PROMPT, or the deviation is recorded.
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

## Blocked / Pruned

- **[G27]** Rewrite the 14 non-compliant commit subjects that contain wave/phase numbers (`planning/PROMPT_TEMPLATE_1.md:23`).
  - Status: Pruned
  - Reason: Rewriting history on a feature branch is destructive, and the rule can be enforced from here on. Revisit only if the branch is unpushed and the user asks.

## Decisions (2026-11-17)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, and the `SpatialCapability.geometry` field with `geometry_metadata()`.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
