# G33 — terrio geometry spike

**Status:** Source/API reconnaissance complete; runtime fixture measurements not performed.  
**Date:** 2026-11-19  
**Scope:** Evaluate terrio as a geometry/data-I/O library against the existing Iowa SSURGO GeoPackage, not as a proposed replacement SQL engine.

## Summary

terrio 0.9.0 (published 2026-11-15) exposes useful first-party APIs for GeoPackage vector I/O, geometry operations, CRS parsing/transformation, and geometry types. Its documented vector open path reads layers/features into memory; this is the primary concern for the approximately 6.0 GiB Iowa fixture. The documentation reviewed here does not establish bounded, row-at-a-time access for local GeoPackage layers, nor a measured peak-memory bound. Therefore a runtime spike on a deliberately small representative subset is still needed before using it in a service path.

The crate also has an `ops::sql` read-only OGR SQL subset, but this is not evidence that it can replace SQLite or execute sda-rs's T-SQL/SQLite contract. The current project relies on SQLite, AST-based T-SQL rewrites, optional SpatiaLite, transaction/batch behavior, metadata discovery and request-local execution. No SQL-engine adoption is supported by this evaluation.

## Supported surface established from first-party docs

| Area | Public API/evidence | Assessment for sda-rs |
|---|---|---|
| GeoPackage vector open | `terrio::open_vector(path, &VectorOpenOptions)`; `VectorDriver::GPKG`; GeoPackage is behind the `sqlite` feature. | Candidate for reading layer/schema/geometry samples. API explicitly reads vector layers into memory. Verify feature filtering and actual Iowa schema before relying on it. |
| Geometry model and parsing | `Geometry` and `Feature` are public re-exports; vector types include coordinate/ring/schema helpers. GPKG driver covers GeoPackage, and the crate describes pure-Rust codecs. | Useful candidate for decoding geometry into typed objects, but reviewed docs do not independently establish behavior for every GeoPackageBinary extension/edge case. Keep `src/gpkg_geometry.rs` validation contract separate until tested. |
| Geometry operations | `ops::geometry::{centroids,singleparts,vertices}`; `ops::topology` documents buffer, overlay, dissolve, validity, make-valid, cleanup. `ops::sql::query` supports a limited set of planar spatial functions. | Candidate for offline geometry utilities. Semantics are explicitly planar; units are input CRS units. Not equivalent by default to SQL Server CLR geography or SpatiaLite behavior. |
| CRS | `crs::SpatialRef::parse` accepts EPSG/WKT/PROJ; `crs::Transformer` transforms coordinates. `vector::reproject_layer` requires known layer CRS and preserves Z/M for ordinary 2D reprojection. | Promising for focused CRS comparisons (4326/3857 and fixture metadata SRIDs). Compare output coordinates to SpatiaLite/GDAL with tolerances and verify axis order/datum-operation behavior on the actual fixture. |
| SQL | `ops::sql::query` is documented as a read-only OGR SQL subset with SELECT, WHERE, field selection, ORDER/LIMIT, limited aggregates/joins, and selected planar spatial functions. | Does not demonstrate T-SQL, SDA TVFs/macros, SQLite compatibility, mutation, batch transactions, request-local temp tables, or wire-protocol behavior. Do not adopt as SQL engine. |
| Memory | `open_vector` reads vector layers into memory. `stream` is a separate feature-reading API; docs also describe selected streaming paths for other formats. | No memory measurement or documented bounded local-GPKG layer iteration was established in this source review. Treat whole-layer open as unsafe for a 6 GiB fixture until measured. |

## Fixture and measurement plan

The representative local fixture is `/data/SSURGO/output/iowa_ssurgo.gpkg` (about 6.0 GiB). Earlier repository reconnaissance recorded `mupolygon.geom`, `sapolygon.geom`, and `featpoint.geom` with EPSG:4326 metadata; `IA001` and mukey `1857266` are known stable identifiers. The file must remain read-only and must not be copied into the repository.

No terrio runtime probe was run in this spike. A follow-up executable should:

1. Open the GeoPackage using terrio 0.9.0 with only the required `sqlite` feature.
2. First inspect/list layers and select a bounded small sample (e.g. a query-scoped temporary fixture derived from existing project test fixtures, or a small sample extracted under `/tmp/opencode`; never modify the source GeoPackage).
3. Test representative polygon, point, and multipart geometries; parse/round-trip geometry and compare exact topological predicates/area/buffer outputs with SpatiaLite for explicitly chosen fixtures.
4. Compare EPSG:4326↔3857 transformations against SpatiaLite/GDAL and document tolerance, axis order, and behavior for the Iowa layer's declared CRS.
5. Measure wall time and peak RSS for opening a small selected layer/sample and, only if safe, one representative larger layer. Record row/feature counts and geometry sizes. Abort before whole-layer loading if estimated memory use is unsafe.
6. Verify source file hashes/mtime before and after, and confirm it remains unchanged.

Until those measurements are recorded, the memory conclusion is **unknown in practice**, with an API-level risk: the documented `open_vector` path is eager. Do not infer bounded memory from the crate's pure-Rust implementation or from stream APIs without confirming that the local GPKG driver can use them.

## Gaps and cautions

- No actual parser/operation/CRS correctness run against the Iowa data was completed; this note records public API capabilities, not compatibility results.
- GeoPackage support is feature-gated (`sqlite`), so build configuration and transitive dependency footprint need evaluation if a prototype is authorized.
- `open_vector` documents whole-layer in-memory loading; this is a poor default for a large SSURGO layer in a long-running request process.
- `ops::topology` documents planar XY semantics. Buffer distance and grid spacing use CRS coordinate units; geographic CRS distances can therefore be degrees. Overlay requires matching CRSs and polygon inputs.
- terrio's OGR SQL subset is read-only and much narrower than the current sda-rs SQL/transpilation/transaction behavior. No SQL-engine adoption case is established.
- `src/gpkg_geometry.rs` currently validates GeoPackageBinary header bounds and extracts WKB; terrio should be compared against these explicit edge-case tests rather than silently replacing that contract.

## Sources (first-party)

- terrio 0.9.0 crate documentation and feature list: <https://docs.rs/terrio/0.9.0/terrio/> (includes public APIs, `sqlite` feature, vector in-memory model, operations, CRS, SQL subset, and Rust version).
- `open_vector` API: <https://docs.rs/terrio/0.9.0/terrio/fn.open_vector.html> (explicitly reads vector layers into memory).
- Vector helper APIs: <https://docs.rs/terrio/0.9.0/terrio/vector/> (`reproject_layer`, coordinate mapping, layer extent, schema/CRS types).
- Geometry utility semantics: <https://docs.rs/terrio/0.9.0/terrio/ops/geometry/>.
- Planar topology semantics: <https://docs.rs/terrio/0.9.0/terrio/ops/topology/>.
- OGR SQL subset: <https://docs.rs/terrio/0.9.0/terrio/ops/sql/>.
- GeoPackage driver: <https://docs.rs/terrio/0.9.0/terrio/formats/enum.VectorDriver.html>.
- Existing sda-rs geometry decoder and contract: `src/gpkg_geometry.rs`, `docs/spatial-sql-contract.md`.
- Fixture identity/metadata reconnaissance: `planning/EXECUTION_PLAN_6.md` lines 61, 94.
