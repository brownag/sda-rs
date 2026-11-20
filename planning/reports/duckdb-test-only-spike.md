# Test-Only DuckDB T-SQL Pipeline Spike (Plan 11, Task 11.4.1)

## Scope and environment

Repository-local, test-only evidence for the G47 data path: representative SDA T-SQL is translated by `polyglot-sql` to DuckDB SQL, executed against deterministic in-memory synthetic data, mapped to the existing `sda_rs::result::ResultSet` value representation, and serialized with the existing SDA serializer. The same inputs and synthetic rows run through the existing SQLite transpiler/executor/serializer path and are compared as exact JSON.

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-production-backend-task`
- Date: 2026-11-20
- Dependencies: `duckdb = { version = "=1.20000.2", features = ["bundled"] }` (dev-dependency only), `polyglot-sql 0.14.1`
- Test file: `tests/duckdb_pipeline_contract.rs`
- Implementation commit: `0a98da1` (`test(duckdb): adapt IIF and batch gaps in test pipeline`)
- No runtime backend, route, configuration, executor, transpiler, or serializer change; no accepted ADR change.

## Reproduction

```sh
CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract
rustfmt --check tests/duckdb_pipeline_contract.rs
git diff --check
```

Targeted result: 12 passed, 0 failed. The orchestrator separately runs `cargo fmt --check`, `git diff --check`, default `cargo test`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially at the wave boundary.

## Supported cases and exact outputs

Synthetic DuckDB schema: `"sample data"(id, name, optional)` rows `(1,'alpha',NULL),(2,'beta','present'),(3,'gamma',NULL)`; `legend(1)`; `mapunit(1,'1857266')`; `muaggatt('1857266','synthetic')`. The SQLite schema carries the same rows with SQLite types. Inputs are probe-authored or the committed `tests/fixtures/soildb/get-sda-muaggatt.sql`; no production or user data is used.

| Case | Input (T-SQL) | Generated DuckDB SQL | SDA JSON (both engines) |
|---|---|---|---|
| TOP / ISNULL | `SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value] FROM [sample data] ORDER BY [id]` | `SELECT "id", COALESCE("optional", 'missing') AS "value" FROM "sample data" ORDER BY "id" NULLS FIRST LIMIT 2` | `{"Table":[["1","missing"],["2","present"]]}` |
| Captured join | `tests/fixtures/soildb/get-sda-muaggatt.sql` | `SELECT muaggatt.mukey, muaggatt.* FROM legend INNER JOIN mapunit ON mapunit.lkey = legend.lkey INNER JOIN muaggatt ON mapunit.mukey = muaggatt.mukey WHERE mapunit.mukey IN ('1857266')` | `{"Table":[["1857266","1857266","synthetic"]]}` |
| STRING_AGG | `SELECT STRING_AGG([name], ',') AS [names] FROM [sample data]` | `SELECT LISTAGG("name", ',') AS "names" FROM "sample data"` | `{"Table":[["alpha,beta,gamma"]]}` |
| Brackets / NULL | `SELECT [id] AS [row id], [optional] AS [maybe text] FROM [sample data] ORDER BY [id]` | `SELECT "id" AS "row id", "optional" AS "maybe text" FROM "sample data" ORDER BY "id" NULLS FIRST` | `{"Table":[["1",null],["2","present"],["3",null]]}` |

Column names, column order, and NULL-versus-text representation are asserted for each case.

## Observed gap 1: IIF is not translated

Raw dialect output for the combined probe input:

```sql
SELECT "id", COALESCE("optional", 'missing') AS "value", IIF("id" > 1, 'yes', 'no') AS "flag" FROM "sample data" ORDER BY "id" NULLS FIRST LIMIT 2
```

DuckDB rejects the raw statement; the error is raised while preparing it, not only at execution:

```text
Catalog Error: Scalar Function with name iif does not exist!
Did you mean "isinf"?
```

The test retains that raw assertion (generated SQL contains `IIF(` and no `CASE`; preparation fails with an `iif` error). A test-only `lower_iif` adapter then rewrites `IIF(cond, a, b)` to `CASE WHEN cond THEN a ELSE b END`, handling balanced parentheses, quoted literals, top-level argument splitting, and nested IIF calls. It is limited to these probe-authored inputs and does not cover general SQL or quoted identifiers.

Lowered SQL:

```sql
SELECT "id", COALESCE("optional", 'missing') AS "value", CASE WHEN "id" > 1 THEN 'yes' ELSE 'no' END AS "flag" FROM "sample data" ORDER BY "id" NULLS FIRST LIMIT 2
```

DuckDB then returns `{"Table":[["1","missing","no"],["2","present","yes"]]}`, identical to the SQLite path and to the existing `tests/sql_batch_contract.rs` expectation. No production transpiler fix was made.

## Observed gap 2: CTAS count appears as a result set

Input:

```sql
SELECT [id] INTO #picked FROM [sample data] WHERE [id] > 1; SELECT [id] FROM #picked ORDER BY [id]
```

Generated DuckDB statements:

```sql
CREATE TEMPORARY TABLE picked AS SELECT "id" FROM "sample data" WHERE "id" > 1;
SELECT "id" FROM picked ORDER BY "id" NULLS FIRST
```

Raw DuckDB collection (the `CREATE TABLE ... AS` count is the first non-empty set):

```json
{"Table":[["2"]],"Table1":[["2"],["3"]]}
```

The SQLite executor skips statements with no result columns, so its response is `{"Table":[["2"],["3"]]}`. The test records the raw DuckDB JSON, then a test-only collection filter suppresses sets whose statement begins with `CREATE TABLE` / `CREATE TEMPORARY TABLE`. The adapted response is `{"Table":[["2"],["3"]]}`, with column `id`, identical to SQLite. The shared serializer is unchanged; this filter is not a production batch or result-shaping contract.

## Value mapping and rejection

Mapped to the existing `ResultSet` values: NULL, booleans, signed integers, small unsigned integers, `Float`/`Double`, text, and blobs. `UBigInt` and `Decimal` map to text to avoid overflow/precision loss. All other DuckDB variants are rejected rather than stringified, including temporal, interval, huge-integer, list, struct, array, map, union, geometry, and enum values.

A focused rejection test executes `SELECT DATE '2024-01-02' AS date_value` and asserts the adapter returns an error containing `unsupported/lossy` and `Date32`. Temporal typing could not be preserved through `rusqlite::types::Value`, so it remains an explicit gap rather than a silent conversion.

## Support boundaries and remaining gaps

- This is integration evidence only: not a runtime backend, route, or opt-in selector, and not production enablement, security qualification, or a performance verdict. No timing claim is made.
- IIF lowering and CTAS-count suppression exist only in `tests/duckdb_pipeline_contract.rs`; they are explicitly test-only workarounds, not production fixes.
- Unsupported/lossy DuckDB types remain rejected; no typed-value mapping contract or metadata behavior is established.
- Spatial queries, DuckDB Spatial, CLR geography methods, batch rollback semantics, concurrency, cancellation, and resource limits were not exercised.
- The executed subset is limited to the selected cases above; the wider SQLite T-SQL surface and general batch wire behavior are not covered.
- Existing SQLite behavior, routes, defaults, response contract, and accepted ADRs 0001–0007 are unchanged.
