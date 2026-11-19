# G34 — DuckDB backend spike

**Status:** Source/API reconnaissance complete; runtime fixture measurements not performed.  
**Date:** 2026-11-19  
**Scope:** Evaluate DuckDB as a possible execution backend against the existing Iowa SSURGO GeoPackage. No backend conversion or dependency change is authorized.

## Summary

DuckDB is a plausible analytical engine for reading GeoPackage data: its Spatial extension exposes GDAL-backed `ST_Read`, documents GeoPackage among recognized formats, and provides geometry operations and CRS support. The Rust client offers in-process connections and bundled builds. These facts establish a candidate for a controlled prototype, not compatibility with sda-rs.

The current service contract includes T-SQL-style parsing/transpilation, SQLite execution, optional SpatiaLite behavior, request-local state, statement-batch rollback, metadata discovery, result shaping, and resource limits. DuckDB SQL and spatial functionality do not establish parity with that contract. In particular, GeoPackage loading behavior, CRS/SRS propagation, memory use, extension deployment, interruption behavior, and fixture performance must be tested before any adoption decision.

## Supported surface established from first-party docs

| Area | Public API/evidence | Assessment for sda-rs |
|---|---|---|
| Rust integration | `duckdb-rs` provides in-memory and file-backed `Connection`s, and documents a bundled build option. The API is modeled after rusqlite. | Candidate for an isolated Rust probe. Bundling is not evidence that the Spatial extension is bundled or available offline; verify build and extension distribution. |
| GeoPackage I/O | Spatial integrates GDAL; `ST_Read` reads supported vector formats, and the Spatial docs list `.gpkg`. `ST_Read` has layer, scan, filter, driver, and batch-related parameters. | Strong file-reading candidate. Confirm actual layer selection, column typing, geometry decoding, metadata, and behavior on this fixture. |
| Spatial and CRS | Spatial provides geometry functions; DuckDB documents a `GEOMETRY` type and CRS-related functions/registration. | Useful capability, but exact CRS assignment/transformation and SQL Server/SpatiaLite semantic equivalence are unestablished. |
| SQL and transactions | DuckDB has its own SQL dialect and documents ACID transactions. | Does not demonstrate T-SQL compatibility, existing transpiler compatibility, SQLite syntax/pragma behavior, SDA TVFs, temporary-table semantics, batch error handling, or response parity. |
| Memory and execution | DuckDB documents `memory_limit`, `operator_memory_limit`, thread settings, profiling, and interruption APIs/controls. | Useful controls for a prototype; limits do not yet establish a process-wide RSS ceiling or safe execution of arbitrary service queries. Measure actual process memory and cancellation behavior. |
| Concurrency | In-process concurrency is documented around a single writer process; read-only access has separate documented behavior. | Must be checked against the service's request-local connection model and deployment expectations. Do not assume SQLite concurrency behavior transfers. |

## Fixture and measurement plan

The representative fixture is `/data/SSURGO/output/iowa_ssurgo.gpkg` (approximately 6.4 GB). Read-only reconnaissance recorded `mupolygon.geom`, `sapolygon.geom`, and `featpoint.geom` with EPSG:4326 metadata; `IA001` and mukey `1857266` are stable identifiers. Do not modify or copy the source file into the repository.

No DuckDB runtime probe was run for this note. An authorized follow-up should:

1. Build a minimal standalone Rust probe using a pinned DuckDB/`duckdb-rs` version; do not change the service dependencies.
2. Verify how the Spatial extension is obtained, loaded, and run offline, including whether the selected Rust build has a compatible extension binary.
3. Inspect GeoPackage layers and metadata read-only; query bounded samples from polygon and point layers using `ST_Read` layer/filter options.
4. Compare row counts, field types/nulls, geometry types, SRIDs/CRS, and representative results for `IA001` and mukey `1857266` against read-only SQLite/SpatiaLite observations.
5. Exercise selected spatial predicates, area/distance behavior, and EPSG:4326↔3857 transformations, documenting units, axis order, null/invalid geometry behavior, and tolerances.
6. Measure elapsed time and peak RSS for bounded reads, then (only if demonstrably safe) a larger representative query. Record DuckDB memory/thread settings and distinguish database/operator memory limits from observed process RSS.
7. Test statement-batch rollback, query interruption/deadline handling, concurrent requests, and output type conversion in a disposable in-memory DuckDB database—not against the source GeoPackage.
8. Verify source file hash and modification time before/after all fixture access.

## Gaps and cautions

- No runtime compatibility or performance results are claimed by this source review.
- GDAL-backed `ST_Read` support for `.gpkg` does not by itself establish efficient bounded access, preservation of GeoPackage metadata, or behavior on this specific Iowa file.
- DuckDB Spatial extension installation/loading and distribution are deployment concerns; confirm reproducible and offline-capable operation before considering a service dependency.
- DuckDB's geometry/CRS and SQL semantics must be compared explicitly with `docs/spatial-sql-contract.md`; function-name overlap is not semantic parity.
- DuckDB's memory controls and profiling are useful but do not replace measuring peak process RSS under representative workloads.
- The current SQLite executor has explicit SQL/value limits, a 60-second deadline, a returned-row cap, and batch rollback behavior (`src/executor/sqlite.rs`). Any backend proposal must preserve or deliberately redesign and validate these guarantees.
- Accepted ADR 0001 selects SQLite/GeoPackage for the MVP. This spike does not propose superseding that decision.

## Sources (first-party)

- Rust client overview: <https://duckdb.org/docs/2.0/clients/rust/overview.html>
- Rust connections and configuration: <https://duckdb.org/docs/2.0/clients/rust/connecting.html>
- Rust client troubleshooting and bundled builds: <https://duckdb.org/docs/2.0/clients/rust/troubleshoot.html>
- Spatial extension overview: <https://duckdb.org/docs/2.0/core_extensions/spatial/overview.html>
- Spatial functions, including `ST_Read`: <https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html>
- GDAL integration: <https://duckdb.org/docs/2.0/core_extensions/spatial/gdal.html>
- Geometry data type: <https://duckdb.org/docs/2.0/sql/data_types/geometry.html>
- Configuration and resource settings: <https://duckdb.org/docs/2.0/configuration/pragmas.html>
- Rust profiling: <https://duckdb.org/docs/2.0/clients/rust/profiling.html>
- Transactions: <https://duckdb.org/docs/2.0/sql/statements/transactions.html>
- Concurrency: <https://duckdb.org/docs/2.0/connect/concurrency.html>
- Existing backend decision and contract: `docs/decisions/0001-sqlite-geopackage-mvp-backend.md`, `docs/spatial-sql-contract.md`.
- Fixture identity/metadata reconnaissance: `planning/EXECUTION_PLAN_6.md` lines 61, 94.
