# Role & Objective
You are a Senior Rust Systems Engineer. Build `sda-rs`, a high-performance, standalone Rust HTTP server that emulates the USDA-NRCS Soil Data Access (SDA) `/Tabular/post.rest` endpoint. It accepts T-SQL queries (including multi-statement batches, `@variables`, `#temp` tables, CLR `.ST*()` spatial methods, and `SDA_Get_*` UDFs), transforms the AST using `polyglot-sql`, executes against either a **PostgreSQL/PostGIS** or **SQLite/SpatiaLite (GeoPackage)** backend, and returns exact SDA-compatible wire formats.

# Crate Stack & Workspace Structure
Create a clean Cargo project (`sda-rs`) using:
- **HTTP Server:** `axum`, `tokio`, `tower-http` (CORS, tracing, payload limits).
- **SQL Parser/Transpiler:** `polyglot-sql` (use the Rust crate directly for `Dialect::Tsql` parsing, AST manipulation, and `Dialect::Postgres` / `Dialect::Sqlite` generation).
- **Database Drivers:** `sqlx` (or `tokio-postgres` + `deadpool-postgres` for dynamic untyped queries and `rusqlite` with `load_extension` enabled for SpatiaLite/GPKG). *Note: Use low-level untyped row cursors so arbitrary user-submitted SQL queries with dynamic schemas and multi-statement batches can be executed and inspected at runtime.*
- **Serialization:** `serde`, `serde_json`, `quick-xml`, `urlencoded`.
- **CLI / Config:** `clap` (derive) + environment variables (`SDA_BACKEND_URL`, `SDA_PORT`, `SDA_GEOM_COLUMN`).

Organize `src/` into:
- `src/main.rs` — CLI args, DB pool/connection initialization, Axum router setup.
- `src/routes.rs` — `POST /Tabular/post.rest` and `POST /Tab/post.rest` handlers accepting both `application/json` and `application/x-www-form-urlencoded` (case-insensitive keys: `query`/`QUERY`, `format`/`FORMAT`).
- `src/transpiler/mod.rs` — Orchestrates the T-SQL -> Target Dialect pipeline.
- `src/transpiler/vars.rs` — Evaluates and folds `DECLARE @var ...; SET/SELECT @var = ...` across multi-statement AST batches.
- `src/transpiler/spatial.rs` — Rewrites T-SQL CLR static/instance methods (`geometry::STGeomFromText`, `.STIntersects()`, `.STArea()`, `.STAsText()`, `.STIntersection()`, `.STBuffer()`) and `SDA_Get_*` table/scalar UDFs into PostGIS or SpatiaLite AST nodes.
- `src/transpiler/apply.rs` — Rewrites `CROSS APPLY` / `OUTER APPLY` for SQLite targets (using window functions / correlated subqueries where applicable) while preserving `LATERAL` for Postgres.
- `src/executor/mod.rs` — Trait `SdaBackend` with implementations in `postgres.rs` and `sqlite.rs` supporting multi-statement execution within a single session (to support `#temp` tables mapped to `TEMP` tables) and returning `Vec<ResultSet>`.
- `src/serializer.rs` — Formats `Vec<ResultSet>` into SDA `JSON`, `JSON+COLUMNNAME`, `JSON+COLUMNNAME+METADATA`, and `XML`.

# Core Technical Requirements

## 1. AST Transformation Pipeline (`src/transpiler/`)
1. **Security Guardrail Pass:** Reject ASTs containing DDL/administrative statements outside temp tables (`DROP`, `ALTER`, `TRUNCATE`, `GRANT`, `EXEC`, `WAITFOR`) with an SDA-formatted error response.
2. **Procedural Variable Folding (`vars.rs`):**
   - Walk the `Vec<Expr>` statements parsed with `Dialect::Tsql`.
   - Track `DECLARE @name <type>` and capture assignments from `SET @name = <expr>` or `SELECT @name = <expr>`.
   - Strip variable declaration/assignment-only statements from the output statement list and substitute `@name` references in subsequent statements with the captured `<expr>` AST subtree.
3. **Temp Table Rewriting:**
   - Rewrite `SELECT <cols> INTO #<name> FROM ...` into `CREATE TEMP TABLE <name> AS SELECT <cols> FROM ...` and replace all `#<name>` table identifiers in subsequent statements with `<name>`.
4. **CLR Spatial & SDA UDF Pass (`spatial.rs`):**
   - Transform `geometry::STGeomFromText(wkt, srid)` and `geography::STGeomFromText(wkt, srid)` -> `ST_GeomFromText(wkt, srid)`.
   - Transform chained CLR instance methods bottom-up on any expression `<geom>`:
     - `<geom>.STIntersects(<arg>) = 1` (and standalone `<geom>.STIntersects(<arg>)`) -> `ST_Intersects(<geom>, <arg>)`
     - `<geom>.STContains(<arg>) = 1` -> `ST_Contains(<geom>, <arg>)`
     - `<geom>.STWithin(<arg>) = 1` -> `ST_Within(<geom>, <arg>)`
     - `<geom>.STIntersection(<arg>)` -> `ST_Intersection(<geom>, <arg>)`
     - `<geom>.STBuffer(<dist>)` -> `ST_Buffer(<geom>, <dist>)`
     - `<geom>.STArea()` -> `ST_Area(<geom>)`
     - `<geom>.STAsText()` -> `ST_AsText(<geom>)`
   - Transform SDA UDFs into subqueries against `mupolygon` (using configurable geometry column name, defaulting to `mupolygongeo` or `geom`):
     - `SDA_Get_Mukey_from_intersection_with_WktWgs84(wkt)` -> `(SELECT DISTINCT mukey FROM mupolygon WHERE ST_Intersects(mupolygongeo, ST_GeomFromText(wkt, 4326)))`
     - `SDA_Get_Mupolygonkey_from_intersection_with_WktWgs84(wkt)` -> `(SELECT DISTINCT mupolygonkey FROM mupolygon WHERE ST_Intersects(mupolygongeo, ST_GeomFromText(wkt, 4326)))`
     - `SDA_Get_MupolygonWktWgs84_from_Mukey(mukey_arg)` -> `(SELECT ST_AsText(mupolygongeo) FROM mupolygon WHERE mukey = mukey_arg)`
5. **Codegen:** Generate SQL string(s) for `Dialect::Postgres` or `Dialect::Sqlite`.

## 2. Wire Format Serialization (`src/serializer.rs`)
- **All non-null values in JSON rows must be serialized as JSON Strings (`"123"`, `"45.6"`, `"Sacramento"`), and SQL `NULL` as JSON `null`.**
- Multiple result sets map to top-level object keys: `"Table"`, `"Table1"`, `"Table2"`, etc.
- If a query returns 0 rows across all result sets, return `{}` with HTTP 200 OK (matching SDA's empty response behavior).
- Support all 4 formats:
  - `JSON`: `{"Table": [["v1", "v2"], ...]}`
  - `JSON+COLUMNNAME`: Row 0 is `["col1", "col2"]`, Row 1+ are data rows.
  - `JSON+COLUMNNAME+METADATA`:
    - Row 0: `["col1", "col2"]`
    - Row 1: Metadata strings per column, e.g., `"ColumnOrdinal=0,ColumnSize=...,DataTypeName=int,ProviderType=Int,..."`. Map backend column types accurately to `int`, `bigint`, `real`, `float`, `decimal`, `varchar`, `bit`, `datetime` so `soilDB` type conversion works seamlessly.
    - Row 2+: Data rows.
  - `XML`: Standard SDA `<NewDataSet><Table><col1>val</col1>...</Table></NewDataSet>` format.

## 3. Verification & Integration Tests (`tests/`)
- Build an automated integration test suite that spins up the Axum router backed by an in-memory SQLite/SpatiaLite database seeded with mock SSURGO tables (`legend`, `mapunit`, `component`, `chorizon`, `muaggatt`, `mupolygon`).
- Test:
  1. `TOP`, `ISNULL`, `IIF`, `STRING_AGG`, and bracketed identifiers `[mukey]`.
  2. Multi-statement batch with `DECLARE @aoi geometry; SELECT @aoi = geometry::STGeomFromText(...); SELECT ... WHERE mupolygongeo.STIntersects(@aoi) = 1`.
  3. `SDA_Get_Mukey_from_intersection_with_WktWgs84('POLYGON((...))')` in a `WHERE mukey IN (...)` and `FROM` join clause.
  4. `#temp` table creation and subsequent join in a single POST request.
  5. Exact structural verification of `JSON+COLUMNNAME+METADATA` output.
