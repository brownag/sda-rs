# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-17
Source State: None

## Circled Goal (Active)

- **[G1]** A SQLite/GeoPackage-focused SDA tabular server executes a spatial MVP end to end: it accepts JSON and form-encoded POST requests, transpiles supported T-SQL, runs against SQLite with optional SpatiaLite support and GeoPackage geometry metadata discovery, and returns SDA-compatible output.
  - Why now: This proves the highest-risk end-to-end path—T-SQL compatibility, GeoPackage geometry handling, spatial execution, and SDA serialization—before investing in broader compatibility or another backend.
  - Exit criteria: Automated local tests cover request parsing, security rejection, basic query execution, `TOP`/`ISNULL`/`IIF`/`STRING_AGG` and bracketed identifiers, `DECLARE @aoi` with `STGeomFromText` and `STIntersects`, the `SDA_Get_Mukey_from_intersection_with_WktWgs84` function in supported query contexts, `#temp` batches, and exact `JSON+COLUMNNAME+METADATA` structure. `cargo fmt --check` and `cargo test` pass; spatial tests exercise SpatiaLite when available and otherwise verify the documented unavailable-capability behavior.
  - First action: Inspect the `polyglot-sql` crate API and supported T-SQL-to-SQLite normalization, then establish the Cargo project and a focused parser/codegen test for the first representative query.

## Sequenced Goals (Dependency DAG)

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

## Independent Backlog

- **[G8]** The server provides CLI and environment configuration for backend URL, port, and geometry settings.
- **[G9]** Request handling includes CORS, tracing, and bounded payload behavior.

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

- None.
