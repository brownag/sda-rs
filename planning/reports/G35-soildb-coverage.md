# G35 — soilDB harness and captured-fixture coverage

## Result

The opt-in real-client harness passed using the local soilDB checkout and Iowa
GeoPackage. Keep it as optional supplementary coverage: the deterministic
fixtures cover captured request SQL and backend replay, while the harness adds
client integration and real-data evidence that those fixtures do not provide.
Neither test layer replaces the other, and neither should be weakened or
removed.

Exact invocation:

```sh
SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so \
SDA_TEST_GEOPACKAGE=/data/SSURGO/output/iowa_ssurgo.gpkg \
SOILDB_PATH=/home/andrew/workspace/soilDB \
make test-soildb
```

Result: exit status 0; `test-sda-rs.R` completed with `sda-rs: ............` and
`══ DONE ══`; the script reported `test-soildb: integration tests passed`.
The server was stopped (reported PID 4078383; `/proc/4078383` was absent on
post-run inspection), and no `/tmp/sda-rs-soildb.*` temporary directory
remained.

Inputs: soilDB source `/home/andrew/workspace/soilDB`; GeoPackage
`/data/SSURGO/output/iowa_ssurgo.gpkg`; SpatiaLite
`/usr/lib/x86_64-linux-gnu/mod_spatialite.so`. Preflight found Cargo, curl,
Python 3, R, Rscript, and all required R packages (`data.table`, `DBI`, `curl`,
`jsonlite`, `testthat`, `sf`, `wk`); all three input paths were readable.

The GeoPackage remained unchanged: SHA-256
`7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`, size
6,406,705,152 bytes, mtime `2026-11-18 07:31:13.192297150 -0800`, both before
and after the harness. The repository fixture and test paths had no changes.
The soilDB checkout was not edited by intent. Its post-run Git status shows
modifications of unknown origin (`.Rbuildignore`, `R/SDA_query.R`,
`R/createSSURGO.R`, and untracked `createSSURGO_patch.diff`); because its
pre-run status was not recorded, this run cannot establish whether those
external changes predated the harness. They were not altered intentionally.

## Behavior and coverage mapping

| Behavior | Live soilDB harness (`tests/soildb/testthat/test-sda-rs.R`) | Captured SQL and deterministic replay |
|---|---|---|
| Direct tabular client request | `SDA_query` reads `legend.areasymbol = 'IA001'` from the real Iowa dataset and checks the returned data frame/column/value. | `get-sda-muaggatt.sql` represents a soilDB query-string call; transpile snapshot checks its SQL rewrite. Replay executes a deterministic `legend`/`mapunit`/`muaggatt` join and asserts columns and fixture row `1857266`. Does not specifically capture the simple `IA001` query. |
| Spatial Mukey lookup | Queries `SDA_Get_MupolygonWktWgs84_from_Mukey(1857266)`, parses the returned WKT as EPSG:4326 with `sf`, then passes the polygon through `SDA_spatialQuery(..., what = "mukey")` and checks for mukey `1857266`. | `sda-spatial-mukey.sql` captures `SDA_spatialQuery(point, what="mukey", query_string=TRUE)` and the Mukey intersection TVF. Transpile test checks its SQL snapshot; replay executes against deterministic geometry and mapunit tables, asserting `1857266` and `Inside map unit`. Does not exercise R client geometry acquisition/parsing or real Iowa geometry. |
| Spatial areasymbol lookup | No corresponding `SDA_spatialQuery(..., what="areasymbol")` case. | `sda-spatial-areasymbol.sql` captures the areasymbol query-string request; transpile test checks its snapshot; replay uses a deterministic `sapolygon` and asserts `IA001`. |
| Aggregate attribute client call | `get_SDA_muaggatt(mukeys=1857266)` runs through installed soilDB and local endpoint against Iowa data; asserts a nonempty data frame containing that mukey. | `get-sda-muaggatt.sql` captures request SQL; transpile snapshot checks the SQLite output; replay asserts exact deterministic columns and `fixture value` row without soilDB. |
| Client configuration and live endpoint | Harness installs the local checkout in a temporary R library, configures `soilDB.sda.tabular.post.rest.url`, verifies endpoint routing with `SDA_query` against `gpkg_contents`, starts/stops the server, and exercises client return types. | No R client installation, private endpoint option, client/server lifecycle, or client result-conversion coverage. Replay invokes the Rust router directly. |
| Spatial runtime/data prerequisites | Uses actual Iowa GeoPackage and configured SpatiaLite; tests client-side WKT-to-`sf` flow. | Spatial replay requires SpatiaLite but builds deterministic in-memory tables/geometries; aggregate replay needs neither SpatiaLite nor soilDB. All fixture tests avoid SDA network access and Iowa data. |

The fixture README and manifest trace the captures to soilDB 2.9.4 commit
`1eaa07178311a4ad3b80842e1ff4f10149733544`, record source calls, formats,
expected columns/rows, and prerequisites. `tests/soildb_fixture_transpile.rs`
protects the generated SQLite SQL snapshots. `tests/soildb_query_fixtures.rs`
replays the aggregate query without SpatiaLite and the two spatial queries when
`SDA_TEST_SPATIALITE` is configured. Together these are deterministic
backend-owned compatibility checks, not substitutes for the live-client path.

## Cost and maintenance

The live test requires Cargo, curl, Python 3, R/Rscript, seven R packages, a
readable local soilDB checkout, SpatiaLite, and a sufficiently complete Iowa
GeoPackage with the expected stable rows/geometries. It builds the server,
installs soilDB into a temporary library, starts a loopback-only HTTP server,
and cleans up server and temporary files. It is opt-in and outside ordinary
`make test`, `cargo test`, and CI, so it does not burden routine test runs.

Maintenance risks include external soilDB API/option drift, local R package
availability, Iowa data/version differences, port/server lifecycle issues, and
the gap between stable real-data assertions and fully controlled fixture
expectations. Fixture captures also need intentional provenance and snapshot
review when soilDB query generation changes; refreshing them requires a local
soilDB checkout and R tooling. Retaining both layers gives a useful boundary:
fixtures localize deterministic transpiler/replay regressions, while the
opt-in harness checks actual client compatibility and selected real-data
behaviors.

## Recommendation

Retain the harness as optional supplementary coverage and keep the captured
fixture/transpile/replay tests. The successful run provides evidence that the
current local client works for the tested Iowa rows and flows; it does not
replace deterministic contract tests or establish broad SDA parity.
