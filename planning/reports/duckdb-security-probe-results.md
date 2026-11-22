# DuckDB G39 direct security probe results

**Task:** Plan 14, task 14.2.1
**Date:** 2026-11-22
**Pin:** Rust `duckdb` 1.20000.2 (`Cargo.lock`); bundled engine v2.0.0 per `duckdb-pin-and-upgrade-assessment.md`.
**Setup:** In-memory direct `Connection`, `enable_autoload_extension(false)`, `enable_external_access(false)`. Paired executor results use public `sda_rs::executor::duckdb::execute`, which includes its validation/transpilation path; these are classifier/executor-path outcomes, not isolated calls to private `validate_statement`.

## Reproduction

Disposable source and build outputs are under `/tmp/opencode/sda-rs-plan14-security/`; no repository source/manifests/tests were changed. Commands were run offline, with no endpoint contacted:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-security/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-security/Cargo.toml -- security-matrix
CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract
```

The first command compiled the scratch runner and its locked local dependencies successfully, then ran the matrix. It printed crate `1.20000.2`, engine identity v2.0.0 (from repository pin assessment), and both configuration flags. The second command ran 4 tests: **4 passed, 0 failed**, in 0.25s. No dependencies were downloaded. The remote URL input used `example.invalid` and did not contact it.

## Paired observations

| Probe | Executor/classifier path | Direct configured connection | Attribution and limits |
|---|---|---|---|
| `SELECT * FROM read_csv('/etc/passwd')` | Rejected: `Query is not read-only` | Permission Error: file system operations are disabled by configuration | Classifier/executor rejects; direct outcome demonstrates config-level local-file denial for this function/path. |
| `SELECT * FROM read_parquet('https://example.invalid/data.parquet')` | Rejected: `Query is not read-only` | Catalog Error: `read_parquet` absent; it exists in parquet extension | Remote access was **not reached or tested** because extension was unavailable. No endpoint contacted. |
| `CREATE SECRET plan14_probe (TYPE memory, PROVIDER config)` | Rejected: `Query is not read-only` | Permission Error accessing `/home/andrew/.duckdb/stored_secrets`; filesystem operations disabled | Direct attempt failed before secret creation due configured filesystem denial. Does not establish universal `CREATE SECRET` denial or resolve the 2.0.0/2.0.1 release-note issue under other storage paths. |
| `SET enable_external_access = true` | Rejected: `Query is not read-only` | Invalid Input Error: cannot enable external access while database is running | Classifier rejects; engine also refused changing this setting in this active connection. |
| `SET autoload_known_extensions = true` | Rejected: `Query is not read-only` | `OK` | Demonstrates setting is mutable directly, despite initial autoload-disabled config. Does not show a successful extension load or file/network access. |
| `ATTACH ':memory:' AS probe` | Rejected: `Query is not read-only` | `OK` | Classifier-only restriction for this harmless in-memory attach. |
| `IMPORT DATABASE '/tmp/opencode/sda-rs-plan14-security/no-such-db'` | Rejected: `Query is not read-only` | Permission Error accessing scratch `schema.sql`; filesystem operations disabled | Config blocked file access before import. No file was imported. |
| `COPY (SELECT 1) TO '/tmp/opencode/sda-rs-plan14-security/copy.csv'` | Rejected: `Query is not read-only` | Permission Error: filesystem operations disabled | Config blocked persistent file output. No output file created. |
| `CREATE TABLE durable (id INTEGER)` | Rejected: `Query is not read-only` | `OK` | Direct connection permits persistent-schema DDL in its in-memory database; classifier prevents this executor-path statement. |
| `INSTALL spatial` | Rejected: `Query is not read-only` | Permission Error accessing extension directory under `/home/andrew/.duckdb/extensions/v2.0.0/linux_amd64` | Config blocked extension installation at filesystem access; no download attempted. |
| `LOAD spatial` | Rejected: `Query is not read-only` | Permission Error: loading external extensions disabled through configuration | Engine/config denial observed. |
| `SELECT 1; CREATE TABLE durable (id INTEGER)` | Rejected: `Query is not read-only` | `OK` | Direct `execute_batch` accepts both statements. This direct probe used a disposable in-memory connection. |
| `SELECT 1 -- comment` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts the harmless commented statement. |
| `SELECT 1 AS "quoted"` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts quoted identifier syntax. |
| `SELECT 'safe'` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts quoted string syntax. |
| Temp workflow: create temp table, insert, select, drop | Batch accepted; 4 result sets | All four statements returned `OK` on one direct connection | Supported compatibility path confirmed when statements share a connection/batch. Individual statement probes used fresh connections and therefore could not observe prior temp-table state; their isolated “table missing” outcomes are not workflow failures. |

## Evidence interpretation

The direct observations distinguish selective engine/config controls from executor-path restrictions. Filesystem operations were denied for local read, import, copy, secret-storage, and extension installation attempts; external extension loading was denied. In contrast, direct `SET autoload_known_extensions`, in-memory `ATTACH`, persistent in-memory DDL, compound statements, comments, and quoting succeeded. The executor rejected these non-SELECT operations/syntax forms. This is evidence that classifier enforcement is material and that the configured engine is not a complete substitute for it.

The remote URL case is inconclusive: `read_parquet` was unavailable because the parquet extension was not installed, so `enable_external_access=false` was not isolated for remote access. No attempt was made to install/load it or contact an endpoint. The direct `CREATE SECRET` attempt failed on the configured secrets-directory filesystem access; this bounded result does not establish the full applicability or exploitability of the documented v2.0.1 fix. Likewise, the single successful autoload-setting change did not test downstream extension autoload or external I/O.

These finite probes do not prove exhaustive SQL/API security, general filesystem containment, or request-path safety. DuckDB remains qualification-only and is not exposed through HTTP; no production-readiness conclusion follows.

## Regression and diff checks

`CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`: **4 passed, 0 failed**.
`git diff --check`: passed with no output.
