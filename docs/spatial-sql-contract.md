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

Geometry column names and SRIDs are discovered from
`gpkg_geometry_columns`; names are not assumed to be `geom` or `mupolygongeo`.
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
  exactly one argument. Recognized GeoPackage geometry columns are wrapped
  with `GeomFromGPB`; expressions that are not recognized geometry columns are
  passed through.
- `SDA_Get_Mukey_from_intersection_with_WktWgs84('literal WKT')` in the
  supported FROM/JOIN and IN-subquery shapes. It selects distinct `mukey`
  values from `mupolygon`, excludes `areasymbol = 'US'`, converts the stored
  GeoPackage geometry, and intersects it with the supplied WKT.

The mukey TVF accepts one string literal only. Zero arguments, column or other
expression arguments, qualified/method-call syntax, and unsupported AST
contexts are rejected rather than being treated as a valid result. The
function rewrite is intentionally narrower than the official SDA function
surface; other `SDA_Get_*` functions are not promised by this contract.

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

The AOI and mukey tests verify successful intersections and selected returned
keys. They do not define results for NULL WKT, NULL geometry columns, empty or
invalid geometries, malformed GeoPackageBinary, or arbitrary invalid WKT.
Those cases are delegated to SQLite/SpatiaLite behavior and may result in SQL
NULLs or execution errors; this project currently makes no stronger
compatibility guarantee.

Known validation failures include wrong constructor/intersection arity,
unsupported mukey TVF arguments or contexts, and missing required `mupolygon`
metadata without an override. Missing SpatiaLite produces the structured
capability error described above. A malformed `gpkg_geometry_columns` table
is reported as a metadata error when spatial capability is required.

## Evidence and intentional differences

Implementation and test anchors:

- `src/transpiler/spatial.rs`: `rewrite`, `expand_source`, and `rewrite_node`.
- `src/spatial.rs`: `geometry_metadata_for_table`, `is_geometry_column`,
  `require_spatial`, and `initialize`.
- `tests/aoi_contract.rs`: configured SpatiaLite AOI execution and the
  no-SpatiaLite structured error.
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
