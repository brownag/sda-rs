# G33-2 DuckDB Spatial follow-up for G12 geography methods

## Outcome

DuckDB Spatial loaded from the cached absolute path and supported synthetic
evaluation of the six corresponding operations. These observations do not
establish SQL Server `geography` parity; DuckDB remains evaluation-only. The
no-feature-row Iowa preflight passed its sandbox and bounded-index gates, but
Wave 1 did not run any Iowa feature query. Wave 2 still requires separate
orchestrator approval.

## Reproducibility and extension/security

- Rust client: `duckdb = "=1.20000.2"`; DuckDB engine: `v2.0.0`.
- Spatial extension loaded first from
  `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension`.
- Cached extension SHA-256:
  `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`.
  Read-only extension metadata before lockdown reported
  `("spatial", "a3dbcd1781", "")`; the extension version identifier is
  `a3dbcd1781`, and the binary is in the `v2.0.0/linux_amd64` cache directory.
  No install fallback or network access was used.
- `DuckDB_PROJ_Compiled_Version()` returned
  `Rel. 9.1.1, December 1st, 2022`.
- After loading, the probe disabled autoload/autoinstall, set `memory_limit`
  to `1GB`, `threads` to 2, allow-listed the exact Iowa source path, then
  disabled external access. Runtime output was `external_access=false`;
  `SET enable_external_access=true` was rejected.
- The probe source and pinned manifest are under
  `/tmp/opencode/sda-rs-plan8-duckdb/`. Exact build:

  ```sh
  CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target \
  DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home \
  cargo build --offline --release \
    --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml
  ```

  Result: success. `cargo fmt --manifest-path
  /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml --check` passed after formatting.
- Exact synthetic/security command:

  ```sh
  bwrap --die-with-parent --ro-bind / / --dev /dev --proc /proc \
    --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb \
    --chdir /tmp/opencode/sda-rs-plan8-duckdb \
    /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe synthetic
  ```

  Result: success, cached-path load and lockdown verified.

## Synthetic observations and contract comparison

Probe output types below are DuckDB Rust client's observed `ValueRef` types.
The comparison baseline is
`planning/reports/G12-geography-method-feasibility.md` and its cited Microsoft
Learn method pages. There was no SQL Server runtime.

| Method / DuckDB operation | Observed synthetic result | SQL Server geography contract and assessment |
|---|---|---|
| `STContains` / `ST_Contains` | Interior point in polygon returned `Boolean(true)`. NULL argument returned `Null`. | SQL Server returns nullable `bit`/`SqlBoolean`, with mismatched SRIDs yielding NULL. The sample establishes only a planar containment result, not geodetic edge or SRID parity. |
| `STWithin` / `ST_Within` | Interior point within polygon returned `Boolean(true)`. NULL argument returned `Null`. | SQL Server returns nullable `bit`/`SqlBoolean`; mismatched SRIDs yield NULL. The sample does not establish geography semantics. |
| `STIntersection` / `ST_Intersection` | Polygon/line intersection WKT was `LINESTRING (0 0.5, 1 0.5)` (`Text`); disjoint points returned `POINT EMPTY` (`Text`); NULL input returned `Null`. A synthetic 4326/3857 CRS mismatch failed at bind time with a CRS incompatibility error. | SQL Server returns geography and NULL for mismatched SRIDs. The observed planar result and CRS error are not SQL Server geography parity. |
| `STBuffer` / `ST_Buffer` | A distance-1 buffer around `(0,0)` returned a polygon (`Text`) with coordinates extending to ±1; NULL input returned `Null`. | SQL Server distance is in meters and returns geography. DuckDB's ordinary buffer uses planar coordinate units; this output is not a 1-meter geography buffer. |
| `STArea` / `ST_Area` | A 1-degree square returned `Double(1.0)`; `ST_Area_Spheroid` on a synthetic north-of-equator polygon returned `Double(12305128751.042904)` with `geometry_always_xy=true`. Empty polygon returned `Double(0.0)`; NULL returned `Null`. | SQL Server geography area uses SRID-associated square units (square meters for 4326) and returns zero for empty or zero-/one-dimensional instances. Ordinary DuckDB area is planar coordinate units. The spheroid result is a distinct WGS84-oriented operation, not demonstrated SQL Server numeric parity. |
| `STAsText` / `ST_AsText` | WKT is returned as Rust `Text`; `POINT Z (1 2 3)` and `POINT M (1 2 3)` retained Z/M. | SQL Server returns WKT text and omits Z/M. This observed dimensional behavior differs; only a restricted ordinary 2D comparison may be useful. |

### Native spatial result types (supplemental probe)

The original `ST_Intersection` and `ST_Buffer` observations wrapped their
results in `ST_AsText`; therefore their recorded `Text` type was the wrapper's
output type, not the native spatial operation's type. A focused rerun queried
the functions directly, using the same in-memory synthetic geometries and
cached extension load followed by autoload/autoinstall lockdown and
`enable_external_access=false`. The direct Rust `ValueRef` observations were:

- `ST_Intersection(...)` returned `Geometry([ ... ])` (DuckDB Rust
  `ValueRef::Geometry`), for the polygon/line crossing whose wrapped WKT is
  `LINESTRING (0 0.5, 1 0.5)`.
- `ST_Buffer(..., 1)` returned `Geometry([ ... ])` (`ValueRef::Geometry`),
  for the planar point buffer. The `ST_AsText(ST_Buffer(...))` expression
  returned `Text(POLYGON (...))` (`ValueRef::Text`).

Thus the native results are DuckDB `GEOMETRY` values represented as
`ValueRef::Geometry`; only their `ST_AsText` wrappers are Rust `Text`. This is
an observed DuckDB/Rust representation, not a SQL Server geography type claim.

The DuckDB 2.0 Spatial reference lists the corresponding operations; it does
not make name similarity a geography contract. The probe also found that
`spatial_version()` and `ST_SRID` were unavailable, and `duckdb_extensions()`
after lockdown could not inspect the default extension directory. The loaded
extension's identifier was recorded before lockdown instead. CRS metadata was
tested with `ST_SetCRS`; unlike SQL Server's documented NULL result for
mismatched SRIDs, a mismatched-CRS intersection was rejected by DuckDB.

Documentation references:

- DuckDB 2.0 Spatial functions:
  <https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html>
- DuckDB 2.0 Spatial overview:
  <https://duckdb.org/docs/2.0/core_extensions/spatial/overview>
- DuckDB 2.0 Securing DuckDB:
  <https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/overview>
- Microsoft Learn method citations are listed in the G12 report, including
  `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and
  `STAsText` geography pages.

## Iowa no-feature-row sandbox/index preflight

Exact command:

```sh
bwrap --die-with-parent --unshare-net --ro-bind / / --dev /dev --proc /proc \
  --ro-bind /data/SSURGO/output/iowa_ssurgo.gpkg /data/SSURGO/output/iowa_ssurgo.gpkg \
  --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb \
  --chdir /tmp/opencode/sda-rs-plan8-duckdb \
  python3 /tmp/opencode/sda-rs-plan8-duckdb/preflight_iowa.py \
  --layer mupolygon --bbox=-93.70,41.50,-93.69,41.51 --candidate-limit 101
```

The actual invocation mounted host `/` read-only, explicitly mounted the
GeoPackage read-only, mounted scratch writable, and unshared the network.
Read-only metadata reported `mupolygon` as a feature layer, geometry column
`geom`, type `POLYGON`, SRS 4326. The RTree schema was
`CREATE VIRTUAL TABLE "rtree_mupolygon_geom" USING rtree(id, minx, maxx, miny, maxy)`.

Exact candidate SQL:

```sql
SELECT id FROM rtree_mupolygon_geom
WHERE minx <= ? AND maxx >= ? AND miny <= ? AND maxy >= ?
LIMIT 101
```

Parameters: `(-93.69, -93.7, 41.51, 41.5)`. `EXPLAIN QUERY PLAN` returned
`SCAN rtree_mupolygon_geom VIRTUAL TABLE INDEX 2:B0D1B2D3`, demonstrating an
RTree virtual-table index. It returned 29 RTree candidate IDs (below the 101
maximum). No Iowa feature-table rows were queried or read. The result is only
preflight evidence, not authorization for Wave 2.

## Acceptance and boundaries

- The cached exact-path load succeeded; the official INSTALL fallback was not
  used. No Cargo network access or other network operation occurred.
- Synthetic probes cover all six operations and record results, Rust types,
  engine/extension/PROJ identifiers, CRS/SRID behavior, units, and safely
  tested NULL/empty/Z/M cases.
- Sandbox and indexed bounded preflight succeeded without feature rows.
- No production files, manifests, Plan 7 report, or data source were modified.
- No Wave 2 work was started. DuckDB adoption and SQL Server parity remain
  unclaimed.
