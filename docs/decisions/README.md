# Architecture Decision Records

| ID | Title | Status | Date | Supersedes | Superseded By | Context |
|---|---|---|---|---|---|---|
| [0001](0001-sqlite-geopackage-mvp-backend.md) | Use SQLite/GeoPackage as the MVP Backend | Accepted | 2026-11-17 | - | - | Plan 1 |
| [0002](0002-request-local-sqlite-connections.md) | Open a Request-Local SQLite Connection on a Blocking Worker | Accepted | 2026-11-17 | - | - | Plan 1 |
| [0003](0003-optional-spatialite-and-geometry-resolution.md) | Load SpatiaLite Optionally and Resolve Geometry Columns from GeoPackage Metadata | Accepted | 2026-11-17 | - | - | Plans 1, 3 |
| [0004](0004-batch-rollback-on-failure.md) | Roll Back All Writes When a Multi-Statement Batch Fails | Accepted | 2026-11-17 | - | - | Plan 3 |
| [0005](0005-result-set-numbering.md) | Number Result Sets by Non-Empty Position | Accepted | 2026-11-17 | - | - | Plan 2 |
| [0006](0006-shared-cache-in-memory-test-databases.md) | Use Shared-Cache In-Memory SQLite for Integration Tests | Accepted | 2026-11-17 | - | - | Plans 2, 3 |
| [0007](0007-duckdb-opt-in-query-backend.md) | Add DuckDB as an Opt-In Query Backend | Accepted | 2026-11-19 | - | - | Plan 10 |
