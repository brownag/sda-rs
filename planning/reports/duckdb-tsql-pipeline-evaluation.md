# DuckDB T-SQL Pipeline Evaluation (Plan 10, Task 10.4.2)

## Scope and environment

Scratch-only evaluation of `T-SQL input → polyglot-sql DuckDB SQL → DuckDB execution → shared SDA result/serializer`, compared with the current non-spatial SQLite application path. No production backend, controls, or limits were selected. Only synthetic schemas/data and the committed captured query fixture were used; no service, user data, or live network was accessed.

The harness is separate from Task 10.4.1 at `/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/`. It uses cached `polyglot-sql = 0.14.1`, `duckdb = 1.20000.2` with bundled engine, and a local path dependency on this repository to call `sda_rs::result::ResultSet` and `sda_rs::serializer::serialize`.

## Reproduction

From the repository root:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/target \
  cargo check --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/Cargo.toml
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/target \
  cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/Cargo.toml \
  --bin tsql-pipeline
git diff --check
```

The runner asserts exact serialized JSON for the TOP/ISNULL/IIF query, captured join, and STRING_AGG query on both backends. Its detailed generated SQL and observed outputs are written to `/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/probe.md`.

## Cases and outcome

Inputs:

1. `SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value], IIF([id] > 1, 'yes', 'no') AS [flag] FROM [sample data] ORDER BY [id]`
2. The exact captured `tests/fixtures/soildb/get-sda-muaggatt.sql`, run against synthetic `legend`, `mapunit`, and `muaggatt` tables containing one matching `mukey` (`1857266`).
3. `SELECT STRING_AGG([name], ',') AS [names] FROM [sample data]`
4. `SELECT [id] INTO #picked FROM [sample data] WHERE [id] > 1; SELECT [id] FROM #picked ORDER BY [id]`

The runner records generated DuckDB SQL and results for supported cases; translation/execution errors are captured in the scratch probe record. Exact expected serialized responses asserted by the runner:

| Case | SDA JSON |
|---|---|
| TOP/ISNULL/IIF | `{"Table":[["1","missing","no"],["2","present","yes"]]}` |
| Captured join | `{"Table":[["1857266","1857266","synthetic"]]}` |
| STRING_AGG | `{"Table":[["alpha,beta,gamma"]]}` |

The SQLite side uses the application's `transpile_with_spatial` and the shared result/serializer modules against an in-memory synthetic SQLite schema. It is not an HTTP route benchmark.

## Result mapping, gaps, and performance

DuckDB NULL, booleans, signed/small integers, float/double, text, and blobs are mapped only where the SDA serializer's textual JSON behavior can be preserved. Unsigned 64-bit and decimal values map to text to avoid overflow/precision loss. Other DuckDB values (including temporal/interval/struct-like values) are explicitly rejected by the scratch mapper rather than silently stringified; temporal conversion would lose native type distinctions. This is a prototype adapter, not a production mapping contract.

Temp-table/batch translation or execution may differ from the application's custom variable/temp-table rewrites; the probe records the direct dialect outcome rather than applying those SQLite-specific passes. Any unsupported case is a dialect/path mismatch, not a production source-change request.

No timing claim is made: this prototype compares equivalent result assertions, but no sufficiently controlled, genuinely equivalent end-to-end execution-plus-serialization timing methodology was established. The prior Task 10.4.1 timings are explicitly not a valid backend comparison.

## Verification record

- Offline scratch `cargo check` passed with cached pinned dependencies.
- Release runner assertions passed for supported cases; see `probe.md` for exact generated SQL and run output.
- `git diff --check` passed before staging/commit.
- Repository changes are limited to this report and `planning/tasks/10.4.2.todo.md`; no application source/dependency changes.
- Stop here for user review. Production scope/controls and Tasks 10.5/10.6 remain unauthorized.
