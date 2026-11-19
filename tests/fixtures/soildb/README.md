# Captured soilDB query fixtures

These SQL examples are backend-owned compatibility fixtures, not copied soilDB
implementation code. They were captured from soilDB 2.9.4 at commit
`1eaa07178311a4ad3b80842e1ff4f10149733544` on branch `feat/custom-sda1`, using
the read-only `query_string = TRUE` paths in `R/SDA-spatial.R` and
`R/get_SDA_muaggatt.R`. `manifest.json` records each originating call, exact
request SQL, request format (`JSON+COLUMNNAME+METADATA`), expected columns and
key rows, and data prerequisites. The `.sqlite.sql` files are checked-in
transpiler output snapshots.

| Fixture | Source call | Expected deterministic result |
|---|---|---|
| `sda-spatial-mukey.sql` | `SDA_spatialQuery(point, what = "mukey", query_string = TRUE)` | `mukey=1857266`, `muname=Inside map unit` |
| `sda-spatial-areasymbol.sql` | `SDA_spatialQuery(point, what = "areasymbol", query_string = TRUE)` | `areasymbol=CA001` |
| `get-sda-muaggatt.sql` | `get_SDA_muaggatt(mukeys = 1857266, query_string = TRUE)` | `mukey=1857266`, aggregate value from the deterministic fixture row |

The point used for both spatial fixtures is `POINT (0.5 0.5)` in EPSG:4326.
Spatial fixture execution requires `SDA_TEST_SPATIALITE`; the ordinary
aggregate-attribute query does not. Tests use in-memory tables and do not
contact SDA, install/load soilDB, or require soilDB's private endpoint option.

To refresh request SQL and provenance, run
`Rscript scripts/capture-soildb-fixtures.R /path/to/soilDB` on a machine with
`pkgload`, `sf`, `wk`, and `jsonlite` installed. This script loads the local
checkout and invokes only query-string generation; it does not contact SDA or
modify the checkout. Review the resulting diffs, then regenerate the expected
`.sqlite.sql` snapshots from the Rust transpiler tests when transpilation
behavior intentionally changes. CI and ordinary tests do not invoke the R
capture script or load soilDB.

The query text and format in each manifest entry are sufficient to construct
the endpoint request for external deployment checks; authentication belongs to
the caller and is intentionally outside these fixtures and tests.
