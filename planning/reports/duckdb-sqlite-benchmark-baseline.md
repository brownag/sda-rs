# SQLite Contract-Workload Benchmark Baseline

## Scope and protocol

This is a SQLite-only baseline for Plan 10 Task 10.1.1; it makes no DuckDB performance claim and selects no performance target. The scratch harness is `/tmp/opencode/sda-rs-plan10-benchmark/` and is intentionally outside the repository. It uses deterministic, in-memory SQLite fixtures, the public `sda_rs::executor` APIs, and the committed captured query fixtures. It adds no repository dependency and accesses no external data or service.

The harness checks expected rows before timing and repeats the checks inside every measured iteration. Each workload has 10 warmups followed by 100 measured repetitions. Fixture construction and setup are excluded. Execution workloads time executor connection/open/initialization and query execution/result collection; serialization times serialization only. Reported median and p95 are per-operation elapsed times; throughput is 100 divided by the sum of measured elapsed times. This is a microbenchmark on one host, not a production-load or end-to-end HTTP measurement.

Host: Linux x86_64, kernel `7.0.0-43-generic`, Ubuntu 24.04 userland; AMD Ryzen 9 9950X (16 cores/32 threads); 109 GiB RAM, 511 MiB swap. Rust `rustc 1.99.1 (58cf8f9c2 2026-10-13)`, Cargo `1.99.0 (5f94df478 2026-08-27)`. Cargo release profile. No CPU pinning or isolation was applied.

## Workloads and results

All reported latency values are milliseconds; throughput is operations/second. Measurements below are from the main-workspace branch after correcting the scratch-only runner. Run command: `cargo run --release --manifest-path /tmp/opencode/sda-rs-plan10-benchmark/Cargo.toml`.

| Workload | Input and fixture | Correctness check | n | Median | p95 | Throughput |
|---|---|---|---:|---:|---:|---:|
| Scalar/select | Three-row in-memory `sample` table; select two rows with `COALESCE`, `CASE`, ordering and limit | Exact SQLite values checked before and during timing | 100 | 0.017080 | 0.017640 | 58,066.55 |
| Captured non-spatial join | `tests/fixtures/soildb/get-sda-muaggatt.sql`; in-memory `legend`, `mapunit`, `muaggatt` tables and one fixture row | Exact captured-join result checked before and during timing | 100 | 0.033100 | 0.036769 | 30,160.87 |
| Batch/temp-table | Three-row in-memory `[sample data]`; create request-local temp table, select, then drop | Exact two returned rows checked before and during timing | 100 | 0.055860 | 0.062859 | 17,539.55 |
| Serialization | 100 deterministic rows, three columns; JSON+COLUMNNAME+METADATA | Exact serialized output stable across timed samples | 100 | 0.012640 | 0.013030 | 77,458.97 |
| Spatial intersection | `tests/fixtures/soildb/sda-spatial-mukey.sql` with deterministic polygon/mapunit fixture and local SpatiaLite extension | Exact selected mukey/muname checked before and during timing | 100 | 1.858080 | 1.910820 | 569.41 |

The first runner invocations exposed scratch-only compile/test harness issues: an invalid async entry point, `ResultSet.columns` populated with a value instead of a string, an incorrect assumption that executor results include result sets for non-row-producing statements, and a mismatch in the expected row nesting. These were corrected in `/tmp/opencode/sda-rs-plan10-benchmark/src/main.rs`. All five workloads, including configured SpatiaLite, then passed expected-output assertions and produced the measurements above.

## Resource observations and limitations

Peak RSS: not measured. `/usr/bin/time -v` around Cargo would include compilation and the Cargo parent rather than isolate the benchmark process; no peak-RSS figure is claimed.

The benchmark exercises the executor API rather than the HTTP request path, avoiding a test-only `tower` dependency or repository manifest changes. It omits HTTP parsing/body handling and route serialization overhead except for direct serializer timing. The captured SQL fixture is an input, while scalar and batch use direct SQL. These results are microbenchmarks on one host and do not represent concurrent throughput or production-sized data.

## Reproduction

From `/tmp/opencode/sda-rs-plan10-merge`, with the scratch harness at `/tmp/opencode/sda-rs-plan10-benchmark`:

```sh
cargo run --release --manifest-path /tmp/opencode/sda-rs-plan10-benchmark/Cargo.toml
SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so \
  cargo run --release --manifest-path /tmp/opencode/sda-rs-plan10-benchmark/Cargo.toml
```

These commands run only against local deterministic fixtures and need no network access. The scratch harness source and manifest are in `/tmp/opencode/sda-rs-plan10-benchmark/`. The configured command includes the spatial case using the installed extension path. No numeric DuckDB performance target is selected by this SQLite baseline.
