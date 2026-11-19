# G12 geography-method feasibility

## Conclusion

This service currently stores GeoPackage geometries and optionally executes
SpatiaLite geometry functions. That is not SQL Server CLR `geography`: SQL
Server models round-earth coordinates, assigns geodetic meaning and units from
the SRID, and returns `NULL` for mismatched geography SRIDs in the predicate
and overlay methods below. SpatiaLite's corresponding topology, intersection,
and buffer operations are GEOS planar operations. A successful sample on
longitude/latitude coordinates does not establish geography compatibility.

**No broad geography-compatible implementation is established for these six
methods.** `STAsText` is conditionally implementable for bounded ordinary
2D geometries (WKT text only; not FullGlobe, >hemisphere, curves, or Z/M
semantics). The other five are classified unsupported for SQL Server
geography compatibility with the current service contract. In particular,
SpatiaLite offers ellipsoidal area calculation for longitude/latitude
coordinates, but the evidence here does not establish numerical parity with
SQL Server's SRID-dependent `STArea`.

The current transpiler only rewrites `.STIntersects` and `.STGeomFromText`;
the constructor rewrite does not distinguish `geography` from `geometry`.
The existing contract explicitly makes no general geography promise, makes no
area/unit guarantee, and does not reconcile arbitrary `.STIntersects`
operand SRIDs. This report does not change that contract or authorize a planar
fallback.

### Plan 9 revalidation (2026-11-19)

The existing probe and source review were checked against the current
`rewrite_node` seam and six-method rejection helper. No concrete unresolved
runtime question justified repeating the SpatiaLite probes: the recorded
version, results, and warning already distinguish the relevant planar,
ellipsoidal-area, NULL, and WKT behaviors. The current seam sees expression
shape, not validated runtime geography values; it cannot establish per-value
SRID, dimensionality, extent, validity, or FullGlobe constraints. Existing
GeoPackage metadata describes declared column metadata, not actual content of
each geometry value. No safe input bound for any method is enforceable here.

**Plan 9 go/no-go:** no method is approved for implementation under the
existing backend and no-new-dependency constraint. Retain explicit rejection
for all six methods. This includes `STAsText`: although SQL Server documents
WKT output semantics that motivate a theoretical ordinary-2D subset, that
subset cannot be enforced for arbitrary receivers at the current seam.

## Evidence matrix

| Method | SQL Server `geography` contract | Existing SpatiaLite capability and probe | Feasibility |
|---|---|---|---|
| `STContains(other_geography)` | One geography argument; returns `bit`/`SqlBoolean` (1 or 0); returns `NULL` when SRIDs differ. | `ST_Contains(g1,g2)` is a GEOS planar predicate; SpatiaLite documents integer 1/0 and -1 for NULL arguments. Probe: polygon contains interior point → 1; boundary point → 0; deliberately mismatched SRIDs 4326/3857 with identical point coordinates returned 1. | **No-go — unsupported.** The SRID mismatch result is observably different, and straight planar edges/coordinates do not establish round-earth topology. No enforceable safe bound; do not lower to `ST_Contains` as geography behavior. |
| `STWithin(other_geography)` | One geography argument; returns `bit`/`SqlBoolean` (1 or 0); always returns `NULL` for mismatched SRIDs. | `ST_Within(g1,g2)` is a GEOS planar predicate; SpatiaLite documents integer 1/0 and -1 for NULL arguments. Probe: interior point within polygon → 1. | **No-go — unsupported.** The sample verifies only a local planar fact; geography SRID NULL behavior and geodetic edge semantics remain incompatible/unimplemented. No enforceable safe bound. |
| `STIntersection(other_geography)` | One geography argument; returns `geography`; always returns `NULL` for mismatched SRIDs. Geography can include >hemisphere and FullGlobe instances; circular arcs may occur in results when present in inputs. | `ST_Intersection(g1,g2)` is a GEOS planar geometry overlay. Probe: polygon crossed by line returned `LINESTRING(0 0.5, 1 0.5)`; disjoint points returned SQL NULL. | **No-go — unsupported.** Planar overlay is not a geodetic overlay; result type/domain includes cases not represented by the demonstrated path, and SRID mismatch behavior is not established as SQL Server-compatible. No enforceable safe bound. |
| `STBuffer(distance)` | One `float` distance argument, in meters; returns `geography`. SQL Server specifies a tolerance-based calculation, negative-buffer behavior, FullGlobe cases, and a maximum distance bound. | `ST_Buffer(geom,dist[,quadrantsegments])` is GEOS planar; distance uses the geometry SRS coordinate units. Probe at SRID 4326 with distance `1` produced a unit-radius planar-coordinate polygon (approximately 1 degree), not a 1-meter geography buffer. | **No-go — unsupported.** No general geodesic buffer operation is documented/probed in this existing stack. Reprojecting ad hoc does not establish one globally bounded geography contract; no enforceable safe bound. |
| `STArea()` | No arguments; returns `float`/`SqlDouble`. Units are the square unit associated with the geography SRID; for 4326, square meters. Returns zero for empty or only zero-/one-dimensional instances. | `ST_Area(surface)` is GEOS planar area in coordinate units. SpatiaLite also documents `ST_Area(surface, use_ellipsoid)` (RTTOPO; long/lat only; meters; ellipsoid when true). Probe on a 1° square at 4326: planar area `1.0`; ellipsoidal area `12308463893.975204`; an empty polygon returned NULL for the one-argument GEOS function, while ellipsoidal area of a line returned `0.0`. | **No-go — unsupported.** The optional ellipsoidal function does not establish numerical parity with SQL Server or all geography input/empty/invalid cases. Never use bare `ST_Area` as geography area; no enforceable safe bound. |
| `STAsText()` | No arguments; returns `nvarchar(max)`/`SqlChars`; OGC WKT omits Z and M. Geography method supports FullGlobe and >hemisphere values. | `AsText(geom)` / `ST_AsText(geom)` return WKT text. Probe showed `POINT Z(1 2 3)` and `POINT M(1 2 3)` retained dimensional ordinates, unlike SQL Server's documented output; SpatiaLite has no demonstrated FullGlobe handling. | **No-go — unsupported at the current seam.** Microsoft documents WKT output omitting Z/M and support for FullGlobe and >hemisphere values. Though ordinary 2D output remains theoretically conditionally feasible, receivers' actual dimensions/content cannot be bounded by the current transpiler. No conditional implementation is approved. |

### NULL and invalid-input evidence

Microsoft method pages explicitly document SRID-mismatch NULL behavior for
`STContains`, `STWithin`, and `STIntersection`; the `STBuffer` page documents
argument constraints and potential exceptions, rather than a general NULL
rule. The `STArea` and `STAsText` pages describe their result behavior and
types but do not establish a general invalid-input policy in the material
reviewed here. SQL Server geography may store invalid instances; its separate
`MakeValid` documentation says only validity/export-related methods can be
called on invalid instances.

SpatiaLite's function reference documents `ST_Contains`/`ST_Within` as
integer predicates returning -1 for NULL arguments, versus SQL Server's
nullable bit. The probe observed `ST_Contains(NULL, geom) = -1`; for tested
geometry-valued functions `ST_Intersection(NULL,geom)`, `ST_Buffer(NULL,1)`,
`ST_Area(NULL)`, and `AsText(NULL)` returned NULL. A bow-tie invalid polygon
predicate returned 0 in this single probe; this does not define general
invalid-geometry behavior. Do not turn these observations into broader
error-handling claims.

## Bounded synthetic probe

No Iowa, soilDB, GeoPackage, or other source data was accessed. Probe used
Python's standard `sqlite3`, an in-memory SQLite database, the explicitly
approved extension path, and synthetic WKT only. The reference metadata was
initialized in memory before the ellipsoidal-area case.

### Exact command

```sh
python3 /tmp/opencode/sda-rs-plan7-geography/probe.py
```

The disposable script loads `/usr/lib/x86_64-linux-gnu/mod_spatialite.so`
with extension loading enabled only for the load call, disables it afterward,
then runs bounded single-row `SELECT` statements for:

```sql
SELECT spatialite_version(), proj_version(), geos_version(), rttopo_version();
SELECT ST_Contains(GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326),
                   GeomFromText('POINT(0.5 0.5)',4326));
SELECT ST_Contains(GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326),
                   GeomFromText('POINT(0 0.5)',4326));
SELECT ST_Within(GeomFromText('POINT(0.5 0.5)',4326),
                 GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326));
SELECT AsText(ST_Intersection(
  GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326),
  GeomFromText('LINESTRING(-1 0.5,2 0.5)',4326)));
SELECT AsText(ST_Intersection(GeomFromText('POINT(0 0)',4326),
                             GeomFromText('POINT(2 2)',4326)));
SELECT ST_Area(ST_Buffer(GeomFromText('POINT(0 0)',4326),1)),
       AsText(ST_Buffer(GeomFromText('POINT(0 0)',4326),1));
SELECT ST_Area(GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326)),
       ST_Area(GeomFromText('POLYGON((0 0,0 1,1 1,1 0,0 0))',4326),1);
SELECT ST_Area(GeomFromText('POLYGON EMPTY',4326)),
       ST_Area(GeomFromText('LINESTRING(0 0,1 1)',4326),1);
SELECT AsText(GeomFromText('POINT Z (1 2 3)',4326)),
       AsText(GeomFromText('POINT M (1 2 3)',4326));
SELECT ST_Contains(GeomFromText('POINT(0 0)',4326),
                   GeomFromText('POINT(0 0)',3857));
SELECT ST_Contains(NULL, GeomFromText('POINT(0 0)',4326)),
       ST_Intersection(NULL, GeomFromText('POINT(0 0)',4326)),
       ST_Buffer(NULL,1), ST_Area(NULL), AsText(NULL);
SELECT ST_Contains(
  GeomFromText('POLYGON((0 0,1 1,0 1,1 0,0 0))',4326),
  GeomFromText('POINT(0.5 0.5)',4326));
```

### Observed results

| Probe | Result |
|---|---|
| Versions | SpatiaLite `5.1.0`; PROJ `Rel. 9.4.1, June 1st, 2024`; GEOS `3.12.2-CAPI-1.18.2`; RTTOPO `1.1.0`. |
| Contains interior / boundary | `1` / `0`. |
| Within interior point | `1`. |
| Polygon/line intersection | `LINESTRING(0 0.5, 1 0.5)`. |
| Disjoint intersection | SQL NULL. |
| One-unit buffer at SRID 4326 | Area `3.1401573745766296`; output coordinates range about -1 to 1, demonstrating coordinate-unit planar buffer rather than meters. |
| 1° square area | Bare planar `ST_Area`: `1.0`; ellipsoidal `ST_Area(...,1)`: `12308463893.975204`. |
| Empty polygon / line | `ST_Area(POLYGON EMPTY)`: NULL; ellipsoidal area of line: `0.0`. |
| WKT with Z/M | Output retained Z and M ordinates. |
| Mismatched SRID predicate | `ST_Contains` returned `1` for same-coordinate points assigned SRIDs 4326 and 3857. |
| NULL / invalid | NULL predicate returned `-1`; tested overlay, buffer, area, and text NULL cases returned NULL; bow-tie invalid polygon predicate returned `0`. |

The buffer output was long and sampled only to establish its numeric units;
it is not asserted to match SQL Server output. The runtime probe tests
SpatiaLite operations only; no SQL Server engine was available or queried.

### Ellipsoidal-area warning and follow-up

The original probe's stdout included the reported crossing-equator polygon
area `12308463893.975204`, but stderr also emitted this warning:

```text
RTTOPO error: ptarray_area_spheroid: cannot handle ptarray that crosses equator
```

Therefore that numeric result is **not clean evidence of error-free
ellipsoidal behavior**. The `ST_Area(...,1)` observation above must be read
with this material qualification; it is not evidence of a successful
cross-equator calculation.

One additional bounded in-memory probe used only a synthetic polygon wholly
north of the equator. Exact command:

```sh
python3 -c "import sqlite3; c=sqlite3.connect(':memory:'); c.enable_load_extension(True); c.load_extension('/usr/lib/x86_64-linux-gnu/mod_spatialite.so'); c.enable_load_extension(False); c.execute('SELECT InitSpatialMetadata(1)'); print('versions:', c.execute('SELECT spatialite_version(), proj_version(), geos_version(), rttopo_version()').fetchone()); print('north-of-equator ST_Area:', c.execute(\"SELECT ST_Area(GeomFromText('POLYGON((0 1,0 2,1 2,1 1,0 1))',4326),1)\").fetchone())"
```

Exact stdout:

```text
versions: ('5.1.0', 'Rel. 9.4.1, June 1st, 2024', '3.12.2-CAPI-1.18.2', '1.1.0')
north-of-equator ST_Area: (12304814950.073112,)
```

No stderr warning was emitted in this additional run. This establishes only
that this north-of-equator synthetic case returned a number without that
warning; it does not establish SQL Server numerical parity, error-free
ellipsoidal behavior for other inputs, or a geography-compatible area
contract.

## Project boundaries and accepted decisions

- `docs/spatial-sql-contract.md` documents only constructor and
  `.STIntersects` rewrites today, makes no area/unit guarantee, and says
  arbitrary `.STIntersects` operands are not automatically reconciled.
- `src/transpiler/spatial.rs::rewrite_node` currently lowers `.STIntersects`
  to `ST_Intersects` and geometry/geography `STGeomFromText` spellings to
  `GeomFromText`; there are no rewrites for the six methods in this task.
- `src/spatial.rs::initialize` loads SpatiaLite only when configured; plain
  SQLite remains available without it.
- Accepted ADR 0001 keeps SQLite/GeoPackage for the MVP and explicitly does
  not make PostgreSQL parity a release gate. Accepted ADR 0003 keeps
  SpatiaLite optional and requires explicit unsupported-capability behavior.
  This task neither supersedes those decisions nor adds dependencies.
- Task 7.2.1's DuckDB probe was blocked at offline Spatial extension loading.
  Its incomplete observations establish nothing about geography semantics
  or this SpatiaLite probe.

## Primary references

### Microsoft Learn — SQL Server `geography`

- [`STContains (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stcontains-geography-data-type?view=sql-server-ver17) — argument, bit/SqlBoolean result, 1/0 and mismatched-SRID NULL.
- [`STWithin (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stwithin-geography-data-type?view=sql-server-ver17) — argument, bit/SqlBoolean result, mismatched-SRID NULL.
- [`STIntersection (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stintersection-geography-data-type?view=sql-server-ver17) — geography result, mismatched-SRID NULL, FullGlobe/>hemisphere and curve-result notes.
- [`STBuffer (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stbuffer-geography-data-type?view=sql-server-ver17) — float distance in meters, geography result, tolerance, negative buffer, FullGlobe and distance bounds.
- [`STArea (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/starea-geography-data-type?view=sql-server-ver17) — float result, SRID-specific squared units, 4326 square meters, zero for empty/0D/1D.
- [`STAsText (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stastext-geography-data-type?view=sql-server-ver17) — WKT/nvarchar(max), omission of Z/M, FullGlobe and >hemisphere.
- [`geography (Transact-SQL)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/spatial-types-geography-transact-sql?view=sql-server-ver17) — CLR type and round-earth/ellipsoidal model.
- [`MakeValid (geography Data Type)`](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/makevalid-geography-data-type?view=sql-server-ver17) — invalid geography storage and method limitations.

### SpatiaLite 5.1.0 official SQL reference

- [SpatiaLite 5.1.0 SQL functions reference list](https://www.gaia-gis.it/gaia-sins/spatialite-sql-5.1.0.html) — the Geometry predicates section documents `ST_Contains` and `ST_Within` as GEOS operations returning integer 1/0/-1 for NULL arguments; the spatial-operators section documents GEOS `ST_Intersection` and `ST_Buffer`; the Surface section documents GEOS `ST_Area(surface)` in coordinate units and the distinct RTTOPO `ST_Area(surface,use_ellipsoid)` in meters for long/lat; the representation section documents `ST_AsText` as WKT. These are geometry functions, not SQL Server geography methods.

## Acceptance status

- Six methods: individually classified with source citations and actual
  configured-build probe evidence where applicable.
- Geography-versus-planar distinction: explicit; no planar substitution is
  called geography-compatible.
- Probe: bounded, in-memory, synthetic WKT only; exact commands, observed
  versions/results, and the crossing-equator stderr warning and qualification
  are recorded above.
- No source, dependency, existing contract, or data files changed.
