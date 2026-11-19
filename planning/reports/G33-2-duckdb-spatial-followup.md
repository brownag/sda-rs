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

## Wave 2 — Task 8.2.1 community geography

### Compatibility and official-source evidence

- Inspected DuckDB's official community page:
  <https://duckdb.org/community_extensions/extensions/geography>. Its
  compatibility matrix has columns `1.5.5`, `1.5.6`, and `2.0.0`; the Linux
  AMD64 row is blank in all three columns, including `2.0.0`. The page lists
  `geography` as a community extension, identifies paleolimbot as maintainer,
  and describes global spatial processing on the sphere using S2.
- DuckDB 2.0 extension-install docs identify `community` as the
  `http://community-extensions.duckdb.org` repository and say the default
  extension install directory is `~/.duckdb/extensions/...`. The single
  attempt below confirms this client tried `/home/andrew/.duckdb/extensions`,
  despite `DUCKDB_HOME` being set to the scratch directory. The docs say
  `extension_directories` controls the location; `home_directory` does not.
- The maintained repository README
  (<https://github.com/paleolimbot/duckdb-geography>) calls the extension
  preliminary and says it is not currently published as a community
  extension, while the live official catalog page lists it. This is conflicting
  source status; neither statement overrides the observed install error.
- The maintainer function reference
  (<https://raw.githubusercontent.com/paleolimbot/duckdb-geography/main/docs/function-reference.md>)
  documents `s2_contains`, `s2_intersection`, `s2_area` (square meters on a
  perfect sphere), and `s2_astext`. Its function list does not list an S2
  buffer function. These are docs, not runtime observations.

### Scratch build and sole runtime attempt

- The existing scratch manifest pins `duckdb = "=1.20000.2"` with `bundled`;
  source, manifest, lockfile, and target are under
  `/tmp/opencode/sda-rs-plan8-duckdb/`. The `geography-synthetic` mode executes
  one `INSTALL geography FROM community;`, then one `LOAD geography;`; it
  returns immediately on either error and, only after successful load, applies
  lockdown before SQL probes.
- `cargo fmt --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml
  --check` first reported formatting differences. Ran `cargo fmt` for that
  scratch manifest, then the same `--check` passed.
- Exact offline release build:

  ```sh
  CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target \
  DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home \
  cargo build --offline --release \
    --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml
  ```

  Result: success (also succeeded once before formatting).
- The single and only runtime invocation was:

  ```sh
  DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home bwrap \
    --die-with-parent --ro-bind / / --dev /dev --proc /proc \
    --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb \
    --chdir /tmp/opencode/sda-rs-plan8-duckdb \
    /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe \
    geography-synthetic
  ```

  Output:

  ```text
  engine_before_install=v2.0.0
  install_error=IO Error: Failed to create directory "/home/andrew/.duckdb/extensions/v2.0.0": Read-only file system
  Error: DuckDBFailure(Error { code: Unknown, extended_code: 1 }, Some("IO Error: Failed to create directory \"/home/andrew/.duckdb/extensions/v2.0.0\": Read-only file system"))
  ```

  Exit status: 1. The install stopped before `LOAD`; the probe returned
  immediately. No retry, extension load, lockdown, or synthetic SQL occurred.
  Extension version, resolved binary path, and binary SHA-256 are unavailable.
  No source build, mirror, or fork was used.

### Six-method result and boundaries

No method was runtime-probed; all six mappings are **unverified at runtime**
because the sole official install attempt failed before load. In particular,
no SQL or Rust result types, SRID behavior, boundary/antimeridian behavior,
WKT outputs, or NULL/empty/dimensional behavior were observed.

| SQL Server method | DuckDB community mapping under evaluation | Wave 2 outcome |
|---|---|---|
| `STContains` | `s2_contains` | Unverified at runtime |
| `STWithin` | reversed `s2_contains` (and documented `s2_within`) | Unverified at runtime |
| `STIntersection` | `s2_intersection` | Unverified at runtime |
| `STBuffer` | directly available S2 buffer equivalent, if any | Unverified; no buffer function appears in the inspected maintainer reference; no planar substitute |
| `STArea` | `s2_area` | Unverified at runtime; docs describe square meters on a perfect sphere |
| `STAsText` | `s2_astext` | Unverified at runtime |

The extension is documented as S2 spherical, not SQL Server's ellipsoidal
geography contract. No parity claim is made. No Iowa feature rows or source
data were accessed.

## Wave 3 — Task 8.3.1 corrected community geography attempt

### Corrected official install and lockdown

- The prior Wave 2 attempt remains historical and was not repeated. The
  standalone Rust probe uses the existing scratch manifest pinned to
  `duckdb = "=1.20000.2"` with `bundled`; source, manifest, lockfile, target,
  extension cache, and logs are under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- Exact offline build command:

  ```sh
  CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target \
    cargo build --offline --release \
    --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml
  ```

  Result: success. The initial correction to the scratch verifier needed a
  further adjustment because this DuckDB client does not implement Rust
  `FromSql` for `Vec<String>`; the final verifier checks list count and first
  element in SQL.
- The one corrected official install/load invocation was:

  ```sh
  /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe geography-synthetic
  ```

  Before install it set `extension_directories` and verified its effective
  value was exactly one entry, the scratch DuckDB home. Output:

  ```text
  engine_before_install=v2.0.0
  extension_directories=(1, "/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home")
  install=success
  load=success
  external_access=false
  external_access_reenable=rejected
  engine=v2.0.0
  extension_metadata=Err(DuckDBFailure(Error { code: Unknown, extended_code: 1 }, Some("Permission Error: Cannot access directory \"/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home/v2.0.0/linux_amd64\" - file system operations are disabled by configuration")))
  ```

  Thus the official install and load succeeded; after lockdown DuckDB's
  metadata query could not access the extension directory. A later load-only
  synthetic invocation (no second install) obtained metadata before lockdown:
  extension `geography`, version `f777314ad0`, path
  `/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home/v2.0.0/linux_amd64/geography.duckdb_extension`.
  SHA-256:
  `f0c5172314e86530d7680b34c62bc092acd64ae8440b43428a5be7ecbe1ad7ad`.
- Both invocations verified `enable_external_access=false` and that a
  re-enable attempt was rejected before synthetic SQL. No OS sandbox or
  Bubblewrap was used. Full logs are retained in the scratch directory.
  The probe disabled `autoload_known_extensions` and
  `autoinstall_known_extensions`, set `memory_limit='1GB'` and `threads=2`,
  then disabled external access before verifying lockdown and issuing probe
  SQL.

### Synthetic S2 probes and contract comparison

- A subsequent load-only invocation used the installed scratch extension and
  re-established lockdown before any probe SQL. No network/install command was
  run in this invocation. Its exact command was:

  ```sh
  /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe geography-probe
  ```

- The official community catalog describes the extension as global processing
  on the sphere using Google's S2 library, lists `GEOGRAPHY` as internally
  blob-backed, and shows a blank Linux AMD64 compatibility row including
  DuckDB 2.0.0:
  <https://duckdb.org/community_extensions/extensions/geography>.
  The maintained function reference describes `s2_area` as square meters on a
  perfect sphere and documents `s2_contains`, `s2_within`, `s2_intersection`,
  and `s2_astext`; its function list has no buffer function:
  <https://raw.githubusercontent.com/paleolimbot/duckdb-geography/main/docs/function-reference.md>.
- Observed DuckDB/Rust result representations: predicates have SQL `BOOLEAN`
  and Rust `ValueRef::Boolean`; `s2_intersection` has SQL `GEOGRAPHY` and is
  returned by this client as `ValueRef::Blob`; area has SQL `DOUBLE` and Rust
  `ValueRef::Double`; text has SQL `VARCHAR` and Rust `ValueRef::Text`.
  Casting ordinary WKT to `GEOGRAPHY` reports logical `GEOGRAPHY`, while the
  underlying Rust value is a blob.
- Results and comparison:

  | SQL Server method | Observed mapping/result | Contract assessment |
  |---|---|---|
  | `STContains` | `s2_contains` returned `true` for an interior point, `false` for one edge point, and `true` for a polygon vertex; the call's logical result was `BOOLEAN`/Rust Boolean. | Boundary behavior differs from the documented SQL Server `STContains` result for boundary points; S2 spherical semantics also do not establish SQL Server SRID-dependent ellipsoidal parity. Not compatible as a general mapping. |
  | `STWithin` | Reversed `s2_contains(point, polygon)` returned `false` for an interior point. A direct `s2_within(point, polygon)` query errored: `Catalog Error: Scalar Function with name s2_within does not exist! Did you mean "s2_dwithin"?` | The maintained reference lists `s2_within`, but this installed binary does not expose it. No defensible working mapping was observed; incompatible/unverified for SQL Server behavior. |
  | `STIntersection` | Result logical type `GEOGRAPHY`, Rust `ValueRef::Blob`; `s2_astext` rendered the polygon/line result as `LINESTRING (0 0.5001523435130507, 1 0.5001523435130508)`. Disjoint point intersection rendered `GEOMETRYCOLLECTION EMPTY`. | Spherical overlay is observed, but not SQL Server ellipsoidal/SRID parity. SQL Server returns geography and has additional geodetic domains/behaviors not established here. |
  | `STBuffer` | No `s2%buffer%` function was returned by `duckdb_functions()` (query returned no rows); maintained function reference also lists no buffer. | Unsupported: no defensible S2 buffer equivalent was identified. No planar substitute was used. |
  | `STArea` | `s2_area` SQL `DOUBLE`, Rust `ValueRef::Double`; 1-degree synthetic square `12364036567.076418`, empty polygon `0.0`, point `0.0`. | Documented units are square meters on a perfect sphere. SQL Server 4326 uses square meters but SRID-specific ellipsoidal semantics; numerical equality/parity is not established. |
  | `STAsText` | `s2_astext` SQL `VARCHAR`, Rust `ValueRef::Text`; point WKT `POINT (0.9999999999999998 2)`, disjoint intersection `GEOMETRYCOLLECTION EMPTY`. Z and M point inputs both rendered as 2D `POINT (0.9999999999999998 2)`. | Omitting Z/M matches the documented SQL Server WKT dimensional behavior for this example, but broader types and geography-specific cases remain unverified. |

- Inputs used ordinary WKT longitude/latitude coordinates; a synthetic
  `SRID=4326;POINT (1 2)` input was rejected with
  `Invalid Error: Expected geometry type at byte 0`. No SRID-setting or
  SRID-mismatch interface was established, so SQL Server's mismatch-to-NULL
  contract is not met/verified. The antimeridian-crossing polygon contained
  `POINT (180 0.5)` and its intersection WKT was that point. SQL `NULL`
  propagated to tested contains, area, and text outputs. Z/M ordinates were
  discarded; empty area was zero and disjoint intersection was an empty
  geometry collection. No separate empty-input predicate or dimensional
  overlay behavior was tested.
- No SQL Server runtime was available; these are DuckDB observations and
  documentation comparisons, not parity claims. No Iowa data or feature rows
  were read. No source build, mirror, fork, production change, repository Cargo
  change, or cached Spatial binary modification occurred.
