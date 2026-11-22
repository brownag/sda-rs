# DuckDB workload and resource qualification benchmark

**Task:** Plan 14, task 14.5.1
**Date:** 2026-11-22
**Scope:** Synthetic in-memory observations on the pinned DuckDB engine. This is not production-load, HTTP-path, security, or readiness evidence.

## Setup and reproduction

The disposable Rust harness, lockfile, and target output are confined to `/tmp/opencode/sda-rs-plan14-benchmark/`. It uses `duckdb = 1.20000.2` (bundled engine v2.0.0) and the repository serializer. The selected engine's reported defaults were `threads=32` and `temp_directory=".tmp"`. Host: Linux x86_64, kernel `7.0.0-43-generic`, Ubuntu 24.04 userland; AMD Ryzen 9 9950X, 16 cores/32 hardware threads, 109 GiB RAM. Rust `1.99.1`, Cargo `1.99.1`. No CPU pinning or isolation.

The serial harness ran 10 warmups and 100 measured repetitions per query workload; it asserts expected output before timing and on each timed execution. Scalar and join fixtures are seeded before timing. Batch creates a fresh connection and fixture, creates/queries/drops a temp table within each timed operation. Serialization constructs 100 deterministic rows before timing and compares exact serialized output each repetition. The join workload reproduces the captured `get-sda-muaggatt.sql` relational shape and expected output, but executes a direct DuckDB SQL equivalent, not the captured T-SQL through the application transpiler. The restricted executor's DDL policy prevents seeding an equivalent executor-path benchmark, so query timings below are direct configured-connection observations, not end-to-end executor timings.

The separate spatial case loads only the already-provisioned local Spatial artifact into a fresh in-memory connection, then disables external access before queries. Artifact SHA-256: `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`; observed extension metadata from prior qualification: `(spatial, a3dbcd1781)`. It measures planar `ST_Area` on one synthetic polygon, not SQL Server geography behavior.

Successful prescribed commands (offline; run serial first, then bounded-concurrent):

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-benchmark/target SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-plan14-benchmark/Cargo.toml -- --mode serial
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-benchmark/target cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-plan14-benchmark/Cargo.toml -- --mode bounded-concurrent
```

The successful serial output included `duckdb_settings=[[BigInt(32), Text(".tmp")]]` and all five workload rows below. Concurrent output included the same settings and `workers=4 operations=100 total_ms=373.286 throughput_ops_s=267.89 per_op_ms=3.733`. A process-only RSS observation (no Cargo parent/build) used:

```sh
CARGO_NET_OFFLINE=true /usr/bin/time -v /tmp/opencode/sda-rs-plan14-benchmark/target/release/sda-rs-plan14-benchmark --mode bounded-concurrent
```

It completed with status 0, 0.37 s wall time and maximum RSS 65,772 KiB. This includes runtime/library overhead and 100 short-lived in-memory connections under four workers; it is not a per-query or peak-under-load capacity estimate.

## Results

Times are milliseconds; throughput is operations/second. Serial samples are n=100. Exact result assertions passed before and during timing.

| Workload | Timed boundary | Median | p95 | Throughput |
|---|---|---:|---:|---:|
| Scalar/select | Query plus result collection on seeded persistent connection | 0.996575 | 1.087278 | 999.14 |
| Captured-join shape | Equivalent three-table join plus result collection on seeded connection | 1.041667 | 1.088089 | 956.56 |
| Batch/temp-table | Fresh connection, fixture setup, temp-table create/query/drop | 8.555302 | 9.740124 | 114.09 |
| Serialization | SDA JSON+COLUMNNAME+METADATA for 100 rows, serialization only | 0.011461 | 0.011591 | 85,526.23 |
| Spatial planar area | `ST_Area` plus result collection, loaded artifact and seeded polygon | 0.315674 | 0.346765 | 3,121.35 |

Bounded concurrent mode ran four host threads, each doing 25 independently seeded connection/query/assertion operations (100 total). It includes setup, had aggregate throughput 267.89 operations/s and mean wall time 3.733 ms per operation. The separate `/usr/bin/time -v` repeat measured 266.64 operations/s. This is one short bounded sample, not a concurrency scaling curve.

The SQLite-only baseline in `planning/reports/duckdb-sqlite-benchmark-baseline.md` reports (same host, 10 warmups/100 reps) scalar median/p95 0.017080/0.017640 ms, join 0.033100/0.036769 ms, batch 0.055860/0.062859 ms, serialization 0.012640/0.013030 ms, and spatial intersection 1.858080/1.910820 ms. The scalar/join/batch DuckDB timings are not comparable to those executor timings: this harness's direct DuckDB setup, connection lifecycle, SQL path, and timing boundaries differ; batch specifically includes new DuckDB startup and fixture setup. Serialization is the closest isolated comparison but still uses separate output implementations/representations and is not a controlled comparative benchmark. The spatial functions/semantics differ (planar area versus SpatiaLite intersection), so those numbers must not be ranked.

The approved 20% latency improvement objective and >10% individual-regression reporting criterion are non-gating observations. This qualification does not establish a pass/fail result against either target.

## Resource observations and candidate-control uncertainty

| Resource | Observation | Limitation / candidate control |
|---|---|---|
| Memory / RSS | One four-worker run: process maximum RSS 65,772 KiB. | Includes fixed runtime/library footprint; no baseline subtraction, allocation profile, large inputs, sustained load, or scaling. Observe per-process/container memory; no numeric limit justified. |
| Threads | DuckDB reports default `threads=32`; benchmark used 4 Rust worker threads. | Does not measure DuckDB worker-pool activity or oversubscription behavior. A configured thread ceiling is a candidate control, but its throughput/correctness tradeoff needs workload calibration. |
| Concurrency | Four workers, 100 total small operations, 266.64–267.89 ops/s across two short runs. | No repeated distributions, higher concurrency, fairness, cancellation, or saturation curve. Bound concurrent requests/workers as a candidate control; no safe value established. |
| Temporary storage | DuckDB setting reports `.tmp`; no spill was observed or specifically measured. | Small in-memory fixtures likely do not characterize spill. Restrict/monitor temp storage in deployment as a candidate; filesystem placement and quota require environment-specific evidence. |
| Input size / rows / values / output | Fixtures contain 3 sample rows, 1 join result, and serialization 100 rows × 3 scalar values. Assertions verify these exact outputs. | No size/row/value/output scaling or adversarially large result test. Request/input/output bounds are candidate controls, but this run cannot select values. |
| Deadline | No deadline or interruption was measured. | Benchmark operations are short and synchronous. Preserve an application deadline as a candidate control; calibrate separately against representative workloads and cancellation semantics. |

No numeric caps are recommended. The private GeoPackage loader also materializes results without row/byte bounds (see `duckdb-loader-audit.md`); this benchmark does not close that gap.

## Limits and deviations

No external user/Iowa/soilDB data, network access, or arbitrary file reads were used. Initial scratch-only attempts exposed a ResultSet value-type mismatch, an invalid temp-table timing sequence, and the need to load the explicitly trusted Spatial artifact before disabling external access; those were corrected before the successful run. An initial concurrent assertion expected `Int` rather than DuckDB's `BigInt`; corrected before successful concurrent mode. An additional settings-print formatting compile error was corrected before the final successful runs. No repository source, tests, manifests, dependencies, baseline, or state report were edited. The result does not qualify the executor's authorization path, HTTP behavior, production safeguards, or geography compatibility.
