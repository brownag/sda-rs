# PR #1 Review Response Drafts

Drafts only; do not post without explicit instruction. Commit references are local commits on `feature/sda-readiness`.

| # | Review item | Commit | Draft reply |
|---|---|---|---|
| 1 | Batch execution deadline | `7484f8e` | Fixed: the deadline now starts at batch execution, is checked before each statement and returned row, and remains enforced inside long-running SQLite statements by the progress handler. |
| 2 | Temporary index authorization | `7484f8e` | Fixed: temp index create/drop actions are allowed only for the temp database; persistent index creation remains denied. Added coverage for constrained temp tables and persistent-index denial. |
| 3 | Deadline test must prove interruption | `7484f8e` | Fixed: replaced the recursive CTE (which the authorizer rejected) with an authorized workload. The test asserts `OperationInterrupted` for an expired deadline and success with sufficient time. |
| 4 | Clap help/version exit status | `8f12bd2` | Fixed: `main` now uses Clap's `parse_from` exit handling, so `--help` and `--version` exit successfully while normal arguments still use the filtered argument list. |
| 5 | Spatial detection must ignore literals/comments | `8f12bd2` | Fixed: capability detection tokenizes executable SQL while skipping strings, comments, and delimited identifiers; actual spatial calls remain detected. |
| 6 | EPSG:3857 absent from read-only GeoPackage metadata | `8f12bd2` | Fixed: request-local SpatiaLite metadata now includes an EPSG:3857 definition when the GeoPackage lacks one; the source GeoPackage is not modified. Added a transform test. |
| 7 | SQL preprocessing must preserve comments and delimited identifiers | `8f12bd2` | Fixed: both preprocessing passes now copy strings, comments, and delimited identifiers intact before macro/variable replacement. Added a regression test covering an apostrophe in a comment. |
| 8 | Reject unsupported WKT TVF column alias lists | `8f12bd2` | Fixed: scalar and table-input WKT source arms now reject non-empty column alias lists, with transpiler regression coverage. |
| 9 | Transform WKT operand to metadata SRID | `8f12bd2` | Fixed: the WKT operand is transformed from its input SRID to the geometry SRID before intersection. Added an executed non-origin point case. |
| 10 | Require `ST_Intersects = 1` for NULL handling | `8f12bd2` | Fixed: generated predicates explicitly require a result of `1`, excluding false and NULL outcomes. |
| 11 | Avoid lossy integer casts in table-input key joins | `8f12bd2` | Fixed: table-input joins compare keys directly, preventing non-numeric text from casting to and matching zero. Added a no-match input-zero regression while retaining numeric-key coverage. |
| 12 | Configured extension test should skip without SpatiaLite | `8f12bd2` | Fixed: configured spatial integration tests return early when `SDA_TEST_SPATIALITE` is unset and still run when configured. I extended this consistently to the AOI, Mukey, and SDA_Get contracts after the default suite exposed the same issue there. |
| 13 | soilDB tests must prove local endpoint routing | `60d9b0c` | Fixed: the harness issues a `soilDB::SDA_query` probe and verifies the local GeoPackage response before running integration tests; a routing mismatch now fails with a clear diagnostic. The configured soilDB suite passes. |
| 14 | Launch Cargo's actual executable path | `60d9b0c` | Fixed: the test script selects the `sda-rs` binary executable from Cargo JSON build output, supporting configured target directories. |
| 15 | Remove prospective status for table-input WKT support | `7edf016` | Fixed: the support document now records the eight feasible table-input WKT variants as implemented, subject to their documented runtime prerequisites. |
| 16 | Document the full supported SDA_Get contract | `7edf016` | Fixed: the spatial SQL contract now covers supported intersection, scalar WKT, and table-input WKT families and describes their argument/prerequisite limits with the support matrix as the detailed index. |
| 17 | Drain Cargo JSON output before selecting the executable | `d2cad15` | Fixed: the parser now records the first matching binary artifact but continues reading Cargo output through EOF before printing the path. The soilDB harness passed with a non-default `CARGO_TARGET_DIR`. |
