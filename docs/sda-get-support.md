# SDA_Get compatibility

This matrix inventories the USDA-NRCS SDA tabular `SDA_Get_*` catalog and the
current soilDB and py-soildb client call sites. It is a support plan for the
SQLite/GeoPackage backend, not a claim that the listed functions already work.

## Evidence and package revisions

Catalog authority: [USDA-NRCS Advanced Queries](https://sdmdataaccess.nrcs.usda.gov/documents/AdvancedQueries.html),
Tabular Functions. The NRCS page groups WGS84 and Web Mercator spellings
together. The table below expands those pairs into all 34 catalogued function
names. The earlier scoping inventory is at
[`planning/sda-rs-scoping.md`](../planning/sda-rs-scoping.md), “Complete
SDA_Get_* Table-Valued Function Surface Area.”

| Package | Published release evidence | Local source used for call-site inspection |
|---|---|---|
| R `soilDB` | CRAN 2.9.2; [CRAN DESCRIPTION](https://cran.r-project.org/web/packages/soilDB/DESCRIPTION) | 2.9.4, branch `feat/custom-sda1`, commit `1eaa07178311a4ad3b80842e1ff4f10149733544`; `/home/andrew/workspace/soilDB` (dirty, read-only input) |
| Python `soildb` (`py-soildb`) | PyPI 0.8.0; [PyPI project JSON](https://pypi.org/pypi/soildb/json) | 0.9.0, branch/tag `v090`, commit `f070c0a410c871f8d2fde42d992d625e9e8d085a`; `/home/andrew/workspace/py-soildb` (read-only input) |

Published metadata and local checkout versions are intentionally distinguished:
the local source may contain unreleased changes and is the evidence used below
for current call sites. Package paths and line numbers refer to those local
checkouts. No package source was changed.

## Status definitions and scope

- **Already supported** means the server currently rewrites and executes the
  specific catalog function. The WGS84 mukey-intersection behavior is covered
  by [`tests/mukey_contract.rs`](../tests/mukey_contract.rs); the four mukey
  and sapolygonkey WGS84/Web-Mercator variants are covered by
  [`tests/sda_get_contract.rs`](../tests/sda_get_contract.rs).
- **Implementable** means the catalog operation can be represented using
  SQLite plus the optional SpatiaLite extension and the required SSURGO table
  and geometry layer. It is not yet implemented unless marked already
  supported. Geometry functions require configured SpatiaLite. A missing
  layer, table, required key, or status-map relation makes an entry unavailable
  for that dataset, not a fabricated empty success.
- **Unsupported** means there is evidence the requested semantics cannot be
  safely promised for this backend/data model. No catalog entry is classified
  unsupported solely because it is not implemented yet.

WKT suffixes identify the input CRS: `WktWgs84` takes WKT in EPSG:4326;
`WktWm` takes WKT in Web Mercator (EPSG:3857). WKT output suffixes identify
the requested output CRS. Table inputs require the SDA pre-parse macros:
`DeclareVarcharTable` for area symbols and `DeclareIntTable` for mukeys.
The official catalog describes the return fields below. Key spelling and WKT
aliases are retained as published (including `AreasymbolWkt84` and
`AreasymbolWktWm`).

## Catalog matrix

| Catalog function | Input | Return column(s) | Required relation / geometry layer | Client use and priority | sda-rs status |
|---|---|---|---|---|---|
| `SDA_Get_Areasymbol_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `areasymbol` | `sapolygon` geometry | No active direct call found in named client anchors | Supported; configured contract test |
| `SDA_Get_Areasymbol_from_intersection_with_WktWm` | WKT, EPSG:3857 | `areasymbol` | `sapolygon` geometry | No active direct call found | Supported; configured contract test |
| `SDA_Get_AreasymbolWKTWgs84_from_Areasymbol` | One area symbol | `AreasymbolWkt84` | `sapolygon` geometry | No active direct call found | Supported when SpatiaLite, `sapolygon.areasymbol`, and geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_AreasymbolWKTWm_from_Areasymbol` | One area symbol | `AreasymbolWktWm` | `sapolygon` geometry | No active direct call found | Supported when SpatiaLite, `sapolygon.areasymbol`, and geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_AreasymbolWktWgs84_from_AreasymbolTable` | `DeclareVarcharTable`; area symbols | `areasymbol`, `AreasymbolWkt84` | `sapolygon` geometry; macro-expanded input table | No active direct call found | Implementable |
| `SDA_Get_AreasymbolWktWm_from_AreasymbolTable` | `DeclareVarcharTable`; area symbols | `areasymbol`, `AreasymbolWktWm` | `sapolygon` geometry; macro-expanded input table | No active direct call found | Implementable |
| `SDA_Get_Mukey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `mukey` | `mupolygon` geometry | **First priority:** soilDB `R/SDA-spatial.R:469-476, 488-495`; `R/SDA_query.R:94-101` (documented query example); py-soildb `src/soildb/spatial.py:322-334` | **Already supported; first priority** |
| `SDA_Get_Mukey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `mukey` | `mupolygon` geometry | Web-Mercator sibling of the active WGS84 client path | **Supported** |
| `SDA_Get_MupolygonWktWgs84_from_Mukey` | One mukey | `MupolygonWktWgs84` | `mupolygon` geometry | soilDB vignette example only: `vignettes/sda.Rmd:1178-1212`; not counted as active package call | Supported when SpatiaLite, `mupolygon.mukey`, and geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_MupolygonWktWm_from_Mukey` | One mukey | `MupolygonWktWm` | `mupolygon` geometry | No active direct call found | Supported when SpatiaLite, `mupolygon.mukey`, and geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_MupolygonWktWgs84_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MupolygonWktWgs84` | `mupolygon` geometry; macro-expanded input table | No active direct call found | Implementable |
| `SDA_Get_MupolygonWktWm_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MupolygonWktWm` | `mupolygon` geometry; macro-expanded input table | No active direct call found | Implementable |
| `SDA_Get_MupointWktWgs84_from_Mukey` | One mukey | `MupointWktWgs84` | `mupoint` geometry | No active direct call found | Supported when SpatiaLite, `mupoint.mukey`, and the geometry layer/metadata are present; configured in-memory contract test |
| `SDA_Get_MupointWktWm_from_Mukey` | One mukey | `MupointWktWm` | `mupoint` geometry | No active direct call found | Supported when SpatiaLite, `mupoint.mukey`, and the geometry layer/metadata are present; configured in-memory contract test |
| `SDA_Get_MupointWktWgs84_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MupointWktWgs84` | `mupoint` geometry; macro-expanded input table | No active direct call found | Implementable when layer exists |
| `SDA_Get_MupointWktWm_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MupointWktWm` | `mupoint` geometry; macro-expanded input table | No active direct call found | Implementable when layer exists |
| `SDA_Get_MulineWktWgs84_from_Mukey` | One mukey | `MulineWktWgs84` | `muline` geometry | No active direct call found | Supported when SpatiaLite, `muline.mukey`, and the geometry layer/metadata are present; configured in-memory contract test |
| `SDA_Get_MulineWktWm_from_Mukey` | One mukey | `MulineWktWm` | `muline` geometry | No active direct call found | Supported when SpatiaLite, `muline.mukey`, and the geometry layer/metadata are present; configured in-memory contract test |
| `SDA_Get_MulineWktWgs84_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MulineWktWgs84` | `muline` geometry; macro-expanded input table | No active direct call found | Implementable when layer exists |
| `SDA_Get_MulineWktWm_from_MukeyTable` | `DeclareIntTable`; mukeys | `mukey`, `MulineWktWm` | `muline` geometry; macro-expanded input table | No active direct call found | Implementable when layer exists |
| `SDA_Get_Sapolygonkey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `sapolygonkey` | `sapolygon` geometry | **First priority:** soilDB `R/SDA-spatial.R:488-495`; py-soildb `src/soildb/spatial.py:336-348` | **Supported; first priority** |
| `SDA_Get_Sapolygonkey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `sapolygonkey` | `sapolygon` geometry | Web-Mercator sibling of the active WGS84 client path | **Supported** |
| `SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWgs84` | WKT, EPSG:4326 | `sapolygonkey` | `sapolygon` geometry joined to `sastatusmap` by `areasymbol` | No active direct call found | Supported when both relations and join key exist; configured contract test |
| `SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWm` | WKT, EPSG:3857 | `sapolygonkey` | `sapolygon` geometry joined to `sastatusmap` by `areasymbol` | No active direct call found | Supported when both relations and join key exist; configured contract test |
| `SDA_Get_Mupolygonkey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `mupolygonkey` | `mupolygon` geometry and key | No active direct call found | Supported when SpatiaLite, `mupolygonkey`, and `mupolygon` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Mupolygonkey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `mupolygonkey` | `mupolygon` geometry and key | No active direct call found | Supported when SpatiaLite, `mupolygonkey`, and `mupolygon` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Mupointkey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `mupointkey` | `mupoint` geometry and key | No active direct call found | Supported when SpatiaLite, `mupointkey`, and `mupoint` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Mupointkey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `mupointkey` | `mupoint` geometry and key | No active direct call found | Supported when SpatiaLite, `mupointkey`, and `mupoint` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Mulinekey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `mulinekey` | `muline` geometry and key | No active direct call found | Supported when SpatiaLite, `mulinekey`, and `muline` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Mulinekey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `mulinekey` | `muline` geometry and key | No active direct call found | Supported when SpatiaLite, `mulinekey`, and `muline` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Featlinekey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `featlinekey` | `featline` geometry and key | No active direct call found | Supported when SpatiaLite, `featlinekey`, and `featline` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Featlinekey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `featlinekey` | `featline` geometry and key | No active direct call found | Supported when SpatiaLite, `featlinekey`, and `featline` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Featpointkey_from_intersection_with_WktWgs84` | WKT, EPSG:4326 | `featpointkey` | `featpoint` geometry and key | No active direct call found | Supported when SpatiaLite, `featpointkey`, and `featpoint` geometry metadata or a supported override are present; configured contract test |
| `SDA_Get_Featpointkey_from_intersection_with_WktWm` | WKT, EPSG:3857 | `featpointkey` | `featpoint` geometry and key | No active direct call found | Supported when SpatiaLite, `featpointkey`, and `featpoint` geometry metadata or a supported override are present; configured contract test |

## Active-use evidence and implementation order

The first implementation target is the WGS84 mukey-intersection TVF, followed
by WGS84 sapolygonkey intersection, matching both clients' optimized query
paths. soilDB's active query construction is in
`R/SDA-spatial.R:469-495`; py-soildb's is in
`src/soildb/spatial.py:320-348`. The soilDB `SDA_query.R:94-101` call and
vignette calls at `sda.Rmd:1178-1186` are usage examples/documented patterns,
not additional functions called by package implementation code.

The server implements all four active-client intersection TVFs: mukey and
sapolygonkey, each with WGS84 and Web-Mercator WKT inputs. It also implements
the two area-symbol and two status-map sapolygonkey intersection variants.
`tests/sda_get_contract.rs` executes configured SpatiaLite queries for all four
active-client variants and all fourteen additional intersection variants; assertions check exact
result column names, SQLite key types for integer keys, and intersecting
results. It also exercises the eight one-key WKT TVFs using deterministic
in-memory geometry tables and exact aliases/transformed WKT. Status-map variants require `sastatusmap.areasymbol` and join it to
`sapolygon.areasymbol`; duplicate status-map matches are collapsed by the
distinct key result. The eight feature-layer variants use deterministic
in-memory schemas for their catalog relations and keys.
`tests/mukey_contract.rs:43-101` retains coverage of the original WGS84 mukey
query shapes. Geometry layer and CRS metadata are discovered through
`src/spatial.rs` and the active intersection dispatch in
`src/transpiler/spatial.rs`. Table-input matrix rows remain prospective Wave 8
targets, not an implicit support promise.

The source-data model bounds implementation. `sapolygon` and `mupolygon` are
the primary layers; `mupoint`, `muline`, `featline`, `featpoint`, and
`sastatusmap` are conditional on their presence and required key/columns in a
given GeoPackage. The local Iowa fixture has `featpoint`, but not all catalog
layers; its `featpoint` schema exposes `featkey`, not the catalog-required
`featpointkey`, so it does not satisfy that TVF's key prerequisite. A missing
layer cannot return meaningful geometry results. Intersection
entries need the matching feature geometry and key; status-map variants also
need the catalog join relation. WKT-output entries require geometry conversion
and transformation to the requested CRS. Table-input entries additionally
require supported declaration/population of the catalog macro input shape.

No function is classified unsupported: all 34 catalog entries have operations
that can in principle be expressed with SQLite/SpatiaLite and appropriate
source relations. This classification is conditional on optional SpatiaLite
being configured for spatial operations and on the dataset having the named
tables, geometry metadata, and columns. `SDA_Get_Chorizonkey_*` is not in the
official catalog and is not counted as a catalog entry.

## Verification boundaries

- Catalog count: 17 WGS84/Web-Mercator pairs = 34 named TVFs.
- Package release versions and local source revisions are recorded separately
  above; active-use rows cite package source files and lines.
- The full matrix is an inventory and feasibility assessment. Future support
  changes must be backed by execution tests and update the status here.
