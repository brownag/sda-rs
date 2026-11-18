---
id: "0003"
title: "Load SpatiaLite Optionally and Resolve Geometry Columns from GeoPackage Metadata"
status: "Accepted"
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_1, EXECUTION_PLAN_3 (G20 decisions); commits 590a99f, 704d7ed"
---

# 3. Load SpatiaLite Optionally and Resolve Geometry Columns from GeoPackage Metadata

## Context and Problem Statement
Plain SQLite must remain usable without spatial extensions. Spatial queries need the geometry column for each table. OpenSSURGO stores geometry column names and SRIDs dynamically in `gpkg_geometry_columns`, not in a fixed column name.

## Decision Drivers
* Non-spatial queries work without SpatiaLite.
* No hard-coded geometry column name.

## Considered Options
1. Require SpatiaLite at startup. Rejected: plain SQLite must remain usable (plan 1 decision).
2. Load SpatiaLite optionally; return a clear error for unsupported spatial operations. **Chosen.**
3. For the `mukey` TVF, use the first `gpkg_geometry_columns` row. Rejected: plan 1 requires retaining and selecting by table, especially `mupolygon`.
4. For the `mukey` TVF, fall back to another table's geometry column when `mupolygon` metadata is absent. Rejected: plan 3 rejects the query instead.
5. Assume the `mupolygongeo` column name. Rejected: plan 1 says to use discovered metadata rather than assume it.

## Decision Outcome
Chosen option: SpatiaLite is loaded only when `SDA_SPATIAL_EXTENSION` is set. Unsupported spatial operations return a clear error. Geometry metadata is discovered from `gpkg_geometry_columns`.

For the `mukey` TVF:
* If `mupolygon` metadata exists, its discovered SRS is used.
* If it is absent and no explicit override is configured (`SDA_GEOM_COLUMN`), the request is rejected. The service does not fall back to another table's geometry column.
* If it is absent but an explicit override is configured, the override column is used with the existing SRID 4326 fallback. This exception was directed by the user.

### Consequences
* **Positive**: Spatial queries never silently use the wrong layer.
* **Negative**: Callers on GeoPackages without `mupolygon` metadata must set `SDA_GEOM_COLUMN`. The SRID 4326 fallback is a documented exception to strict metadata use.
