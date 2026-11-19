# DuckDB Capability Evaluation (Plan 10, Task 10.4.1)

## Outcome and scope

This is an exploratory local prototype only; it does not enable a production backend or select production controls/limits. It uses the Rust client `duckdb = "=1.20000.2"` with bundled DuckDB engine `v2.0.0`. All database, CSV, and other generated files are under `/tmp/opencode/sda-rs-duckdb-evaluation/disposable/`. No external data, live network request, application code, or untrusted SQL was used. All SQL was authored for the probe.

## Environment and reproduction

- Linux x86_64, Ubuntu 24.04 userland, kernel `7.0.0-43-generic`; AMD Ryzen 9 9950X, 32 logical CPUs; 109 GiB RAM, 511 MiB swap.
- `rustc 1.99.1 (58cf8f9c2 2026-10-13)`, Cargo `1.99.0 (5f94df478 2026-08-27)`.
- Release benchmark; 10 warmups and 100 timed samples per workload. Each measured execution asserts expected output. Timings are local microbenchmarks and include statement preparation/query execution/row collection; batch includes temp-table create/select/drop. They are not HTTP or end-to-end measurements.

From any directory, using the scratch harness:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/target \
  cargo check --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/Cargo.toml
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/target \
  cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/Cargo.toml \
  --bin sda-rs-duckdb-evaluation
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/target \
  cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/Cargo.toml --bin benchmark
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-duckdb-evaluation/target \
  cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-duckdb-evaluation/Cargo.toml --bin rss_probe
```

The prototype initializes two threads and a 1 GiB DuckDB memory setting for the primary database. A second connection sets `enable_external_access=false`; extension autoload is disabled. These are probe settings, not proposed limits.

For the RSS/thread snapshot, remove any prior ready marker, then run the built `rss_probe` binary and inspect it during its 20-second sample window:

```sh
rm -f /tmp/opencode/sda-rs-duckdb-evaluation/disposable/ready
/tmp/opencode/sda-rs-duckdb-evaluation/target/release/rss_probe &
pid=$!
while [ ! -f /tmp/opencode/sda-rs-duckdb-evaluation/disposable/ready ]; do sleep 0.1; done
ps -o pid,nlwp,rss,vsz -p "$pid"
wait "$pid"
```

On orchestrator rerun, both the primary runner and benchmark binary passed all assertions. Timing distributions varied from the recorded run (for example, scalar median was 1.234 ms in the primary runner and 0.998 ms in the benchmark binary versus 1.058 ms in the recorded primary run), reinforcing that this is a directional single-host probe, not a production capacity result.

## Workload timings

The SQLite values are from `planning/reports/duckdb-sqlite-benchmark-baseline.md`. SQLite timing used its real application executor and, for serialization, the application's serializer. DuckDB does not use the application executor/serializer; SQL was authored to match the scalar/join/batch fixtures, while the join is equivalent to (not the literal captured T-SQL fixture). Thus these are directional comparisons, not a controlled backend A/B benchmark.

| Workload | DuckDB median ms | DuckDB p95 ms | DuckDB ops/s | SQLite median ms | SQLite p95 ms | SQLite ops/s | Comparison |
|---|---:|---:|---:|---:|---:|---:|---|
| Scalar/select | 1.058428 | 1.132310 | 937.82 | 0.017080 | 0.017640 | 58,066.55 | DuckDB prototype slower; differing connection/executor path |
| Captured join equivalent | 1.082809 | 1.238372 | 905.11 | 0.033100 | 0.036769 | 30,160.87 | DuckDB prototype slower; equivalent synthetic SQL, not captured T-SQL |
| Temp-table batch | 0.873014 | 1.036208 | 1,123.42 | 0.055860 | 0.062859 | 17,539.55 | DuckDB prototype slower; batch semantics differ |
| Serialization / row materialization | 0.142473 | 0.154804 | 7,005.88 | 0.012640 | 0.013030 | 77,458.97 | Not comparable: DuckDB measurement only fetched 100 synthetic rows; SQLite serialized JSON+COLUMNNAME+METADATA |

DuckDB benchmark outputs passed exact synthetic-fixture assertions. The separate primary runner also checks its typed result values. These measurements do not meet the approved target on their face, but the workloads are not sufficiently equivalent to make a production performance decision. No geometric-mean claim is made.

### Spatial

Spatial was not run. The scratch evaluation did not load or provision a verified local DuckDB Spatial extension; no network or extension installation was attempted. The SQLite baseline's spatial result (median 1.858080 ms, p95 1.910820 ms) is recorded in the baseline report only and has no DuckDB comparison here.

## Capability observations

| Probe | Observed result |
|---|---|
| Scalar/select, joins, temp tables | Synthetic select and three-table join returned the asserted rows. A temporary table could be created, queried, and dropped on the same connection. |
| Typed results | Rust client exposed distinct `Null`, `Boolean`, `Int`, `BigInt`, `Double`, `Text`, `Date32`, `Timestamp(Microsecond, …)`, and `Blob` values for the authored projection. |
| Batch failure | `execute_batch` ran `INSERT 10` before a later missing-table error. After the error the row 10 remained alongside row 9. Batch atomicity is not implicit; explicit transaction behavior was not separately tested. |
| Persistent write | Creating a file-backed database and table under the disposable directory succeeded (274,432-byte file). |
| Local file access | `read_csv_auto` read the probe-authored CSV with default external access enabled. |
| External access setting | With `enable_external_access=false`, local CSV reading failed with a permission error (“file system operations are disabled by configuration”). `INSTALL spatial` also failed while accessing the extension cache directory. No network attempt was made. |
| Extension behavior | Extension autoload was disabled. Loading a pre-existing extension was not tested; `INSTALL spatial` denial under the external-access setting is the only extension probe. |
| Threads/memory/temp | Configuration accepted `threads(2)` and `max_memory("1GB")`. No temp-storage quota was configured or tested. In a small in-memory probe (20,000 rows with 1 KiB strings), an initial process snapshot showed RSS 58,752 KiB, VSZ 11,809,828 KiB, and 160 threads. A later direct `rss_probe` rerun showed RSS 20,324 KiB, VSZ 5,451,972 KiB, and 74 threads. These are isolated snapshots, not peak RSS; the thread count varied despite the requested setting and may include runtime/engine workers. No concurrency/load scaling was measured. |
| Network | Not probed; explicitly avoided. |
| Spatial SQL | Not probed. |

The scratch harness source and Cargo manifest are in `/tmp/opencode/sda-rs-duckdb-evaluation/`. The disposable files are not repository content.

## Interpretation and limitations

These observations establish that the tested client/engine can execute local SQL, produce typed results, use connection-local temporary tables, access local files and persist writes under permissive settings, and reject file operations with external access disabled. They do not establish that these settings form a complete security boundary, enforce a service-level resource ceiling, cancel work reliably, isolate all filesystem/network paths, or provide extension policy. Nor do they prove request-local lifecycle, transaction rollback, parallel behavior, row/response limits, temp-storage containment, or SQL contract parity. No production numeric resource limits are recommended here.

The approved performance target is a 20% improvement in geometric-mean median latency across scalar/select, captured join, batch/temp-table, and serialization, with no workload regressing more than 10%; spatial is separate. The present DuckDB harness does not implement the shared serializer or the same execution path and therefore cannot validly calculate acceptance against that target. A later, explicitly scoped implementation evaluation would need an equivalent contract workload and serializer before deciding performance acceptance.

## Verification record

- Scratch `cargo check`: passed.
- Scratch release capability runner: passed; result and batch assertions passed.
- Scratch release benchmark: passed; all three SQL workload assertions passed. Serialization-shaped fetch is explicitly not serializer-equivalent.
- Repository source, manifests, application configuration/routes/executors, deployment, CI, and wire format were not changed.
- Repository `git diff --check` is required after adding this report and task completion update.
