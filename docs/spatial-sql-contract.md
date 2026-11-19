# Spatial SQL contract

This document describes the spatial SQL behavior currently implemented and
demonstrated by the contract tests. The service uses SQLite/GeoPackage with
optional SpatiaLite; this is not a promise of complete SQL Server or upstream
SDA spatial parity.

## Availability and geometry metadata

Spatial operations require SpatiaLite. Configure its trusted extension path
with `SDA_SPATIAL_EXTENSION`. Without the extension, ordinary SQLite queries
remain available, while spatial requests fail with a structured error such as
`Spatial capability unavailable: SpatiaLite is not loaded`. An explicitly
configured extension that cannot be loaded is an initialization error.

Geometry column names and SRIDs are resolved per referenced table from
`gpkg_geometry_columns`; names are not assumed to be `geom` or `mupolygongeo`.
Spatial column references use their table qualifier or alias. An unqualified
geometry reference is wrapped only when exactly one referenced relation
declares that geometry-column name. Missing metadata for a referenced spatial
relation, multiple metadata rows for that table, and rows with an empty table
or geometry-column name or a non-positive SRID are rejected explicitly;
unrelated metadata rows do not supply a relation's column or SRID.
`SDA_GEOM_COLUMN` provides an explicit geometry-column override. For the
`SDA_Get_Mukey_from_intersection_with_WktWgs84` rewrite, metadata must identify
the `mupolygon` geometry unless an override is set. If metadata is absent but
an override is set, the source SRID falls back to 4326. If `mupolygon`
metadata exists, its SRID is used even when the column is overridden.

Rewritten geometry-column references in the covered paths are wrapped in
`GeomFromGPB(...)`, reflecting the GeoPackage geometry BLOB representation.
This contract does not specify a general-purpose GeoPackageBinary decoder or
guarantee acceptance of arbitrary geometry BLOBs.

## Supported SQL rewrites

The transpiler currently recognizes:

- `geometry::STGeomFromText(wkt, srid)`, lowered to SpatiaLite
  `GeomFromText(wkt, srid)`.
- The instance form `.STGeomFromText(wkt, srid)`, lowered to the same
  constructor. Both forms require exactly two arguments.
- `.STIntersects(other_geometry)`, lowered to `ST_Intersects`. It requires
  exactly one argument. The referenced relation's GeoPackage geometry column
  is wrapped with `GeomFromGPB`; metadata for joined geometry relations is
  resolved independently, including when the relations share a column name.
  This is an existing SpatiaLite/planar geometry operation and does not claim
  SQL Server `geography` semantics.
- `SDA_Get_*_from_intersection_with_WktWgs84/WktWm('literal WKT')` variants
  catalogued as supported in [the SDA_Get support matrix](sda-get-support.md).
  These return the documented distinct keys, use the relation's discovered
  geometry column and SRID metadata, exclude STATSGO mapunit rows, and compare
  spatial operands in a common CRS. `ST_Intersects` must evaluate to `1`.
- Scalar WKT functions such as
  `SDA_Get_MupolygonWktWgs84_from_Mukey(literal_key)` and
  `SDA_Get_AreasymbolWKTWgs84_from_Areasymbol(literal_key)`. Each accepts one
  literal key and returns the catalogued WKT column/alias, transforming to
  WGS84 or Web Mercator as named.
- Table-input WKT variants such as
  `SDA_Get_MupolygonWktWgs84_from_MukeyTable(@keys)` and
  `SDA_Get_AreasymbolWktWgs84_from_AreasymbolTable(@areas)`. These require the
  corresponding supported `DeclareIntTable` or `DeclareVarcharTable` macro,
  and return the catalogued key and WKT columns.

These TVFs require SpatiaLite and their source tables, keys, and geometry
metadata (or a supported geometry-column override). The complete supported
function list and per-function prerequisites are maintained in the support
matrix. Unsupported arities, correlated/nonliteral scalar arguments,
qualified/method-call syntax, unsupported AST contexts, and column alias lists
are rejected rather than silently treated as valid results.

The CLR instance methods `.STContains(...)`, `.STWithin(...)`, and
`.STIntersection(...)` are explicitly rejected as unsupported for SQL Server
`geography` compatibility. They are not lowered to SpatiaLite's GEOS planar
`ST_Contains`, `ST_Within`, or `ST_Intersection`: the configured feasibility
probe showed mismatched-SRID predicate behavior that differs from SQL Server,
and the planar operations do not establish geodetic predicate or overlay
semantics. The rejection applies regardless of apparent arity; no partial
geometry implementation is implied. Free-function spellings are not method
rewrites. See the
[`G12 geography feasibility report`](../planning/reports/G12-geography-method-feasibility.md)
for the evidence matrix and probe results.

The CLR instance methods `.STBuffer(...)` and `.STArea()` are also explicitly
rejected. SpatiaLite `ST_Buffer` is a planar operation whose distance follows
the geometry coordinate units (not geography meters), and bare `ST_Area` is
planar coordinate area (not SRID-dependent geography area). The optional
ellipsoidal SpatiaLite area operation has not been established as SQL Server
compatible across supported inputs. Neither method is rewritten to those
operations.

`.STAsText()` is explicitly rejected in this release as well. The feasibility
assessment finds text conversion conditionally feasible for bounded ordinary
2D values, but the transpiler cannot enforce that condition for all supported
receivers: geometry metadata does not validate each stored value's actual
dimensions or content, and constructor expressions may contain arbitrary WKT,
SRIDs, NULLs, or empty values. In particular, AST shape alone cannot exclude
Z/M, FullGlobe, or greater-than-hemisphere inputs, and SpatiaLite's demonstrated
Z/M serialization differs from SQL Server geography output. It is therefore
not safe to lower the method to `AsText`/`ST_AsText` or claim even a conditional
contract from the current rewrite seam. The feasibility report remains an
assessment of theoretical conditional feasibility; qualifying that report
requires a separately scoped planning update.

## Coordinate reference systems and units

The constructor passes its supplied SRID to SpatiaLite; the tests demonstrate
WKT constructed with SRID 4326. The mukey TVF is named for WGS84 and constructs
its input WKT with SRID 4326. Its stored geometry is transformed to the SRID
read from `mupolygon` metadata (or the 4326 override fallback) before
intersection. This is the behavior of the current rewrite, not generalized
input-SRID detection or automatic reconciliation for arbitrary
`.STIntersects` operands.

No area, distance, or length method is currently included in this rewrite
contract. In particular, no unit guarantee (such as square meters for area)
is made. Callers must not assume SQL Server CLR or PostGIS geography semantics.

## NULLs, malformed geometries, and errors

Intersection rewrites require the predicate to equal `1`, so false and NULL
intersection results do not produce keys. Scalar/table WKT tests include empty
and no-match inputs. Invalid geometries, malformed GeoPackageBinary, and
arbitrary invalid WKT are delegated to SQLite/SpatiaLite and may produce
execution errors.

Known validation failures include wrong constructor/intersection arity,
unsupported mukey TVF arguments or contexts, and missing required `mupolygon`
metadata without an override. Missing SpatiaLite produces the structured
capability error described above. A malformed `gpkg_geometry_columns` table
is reported as a metadata error when spatial capability is required.

## Evidence and intentional differences

Implementation and test anchors:

- `src/transpiler/spatial.rs`: `rewrite`, `expand_source`, and `rewrite_node`.
- `src/spatial.rs`: `geometry_metadata_for_table`,
  `resolve_geometry_metadata_for_table`,
  `require_spatial`, and `initialize`.
- `tests/aoi_contract.rs`: configured SpatiaLite AOI execution and the
  no-SpatiaLite structured error.
- `tests/spatial_methods_contract.rs`: configured execution of the existing
  `.STIntersects` operation and explicit rejection of the unsupported
  geography instance methods, including `.STBuffer`, `.STArea()`, and
  `.STAsText()`; `.STIntersects` results are not geography-parity evidence.
- `tests/mukey_contract.rs`: configured SpatiaLite mukey TVF result and
  supported query contexts.
- `src/transpiler/spatial.rs` unit tests: constructor/method rewrites,
  metadata/override behavior, and rejected input shapes.

Intentional limits relative to SDA/SQL Server include optional rather than
mandatory SpatiaLite, SQLite/GeoPackage rather than the upstream backend,
metadata-dependent geometry resolution, and the narrower implemented TVF and
method surface. Accepted architecture decisions are recorded in
[ADR 0001](decisions/0001-sqlite-geopackage-mvp-backend.md) and
[ADR 0003](decisions/0003-optional-spatialite-and-geometry-resolution.md).
