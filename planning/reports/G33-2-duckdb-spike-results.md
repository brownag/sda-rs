# G33-2 DuckDB GeoPackage evaluation

## Outcome

**Blocked; no DuckDB compatibility or performance conclusion. DuckDB is not adopted.** The standalone Rust client built and the Spatial extension installed and loaded online. The offline/reproducible extension check failed: this environment disallows network namespaces, while DuckDB refuses to load external extensions when external access is disabled. Per the task stop condition, no further runtime probing or GeoPackage feature-row reads were attempted.

## Environment and source integrity

- Source: `/data/SSURGO/output/iowa_ssurgo.gpkg`; size **6,406,705,152 bytes**; mode `-rw-r--r--` (not filesystem read-only; all opened SQLite connections used `mode=ro`).
- Before and after source fingerprint: SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`; mtime `2026-11-18 07:31:13.192297150 -0800`; size unchanged.
- `/tmp/opencode` had 16,660,054,016 bytes free before dependency fetch/build; free space remained above the required 4 GiB during observed work. Cargo target, DuckDB home/cache, and scratch project were under `/tmp/opencode/sda-rs-plan7-duckdb/`. The cached Spatial extension occupied 108,189,118 bytes.
- Rust/Cargo: `rustc 1.99.1`, `cargo 1.99.1`; client pinned to `duckdb = "=1.20000.2"` (crates.io’s current published version; DuckDB docs label that release line 2.0). DuckDB CLI was unavailable. Python DuckDB 1.5.4 and `psutil` 5.9.8 were present but not used for engine probes.

## Commands and observations

Commands run (with the scratch target and DuckDB home environment variables pointing under `/tmp/opencode/sda-rs-plan7-duckdb/`):

```sh
cargo fetch --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml
CARGO_BUILD_JOBS=2 cargo build --release --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml
cargo run --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml --release -- install
CARGO_NET_OFFLINE=true cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml --release -- offline
unshare -n cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml --release -- offline
```

- `cargo fetch` succeeded. Release build of the bundled DuckDB Rust client and probe succeeded after correcting an ownership/`Send` compile error in the disposable interrupt harness.
- `install` succeeded: `INSTALL spatial; LOAD spatial;` completed, and DuckDB cached the extension at `duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension`.
- Offline run with `enable_external_access=false` failed with `Permission Error: Loading external extensions is disabled through configuration`. This establishes that a cached extension did not load under the tested offline-restriction setting; it does not establish that DuckDB Spatial cannot be used offline through another supported packaging configuration.
- A stricter no-network attempt failed before running the probe: `unshare: unshare failed: Operation not permitted`. Thus actual network-isolated extension loading could not be verified. No alternative mechanism was tried, consistent with the stop instruction.
- A read-only SQLite metadata inspection returned:

  | Layer | Geometry column/type | SRS |
  |---|---|---:|
  | `mupolygon` | `geom` / POLYGON | 4326 |
  | `sapolygon` | `geom` / POLYGON | 4326 |
  | `featpoint` | `geom` / POINT | 4326 |

- Before feature-row queries, read-only `EXPLAIN QUERY PLAN` checks for `mupolygon.mukey='1857266'` and `legend.areasymbol='IA001'` reported `SCAN mupolygon` and `SCAN legend`, respectively. Those full scans were not executed. DuckDB’s documented `ST_Read` spatial filter is pushed down only for some drivers and may otherwise be applied by GDAL after reading, so it did not establish a safe bound for this file. No Iowa feature rows, schema field/null samples, key results, spatial-operation comparisons against Iowa data, or transform comparisons were run.
- No disposable transaction rollback, deadline/interruption, concurrency, or Rust output-conversion probes were run because the offline extension gate stopped execution. No elapsed-time/peak-RSS workload measurements were collected. Build was restricted to two jobs; DuckDB probe configuration requested 1 GiB memory and 2 threads, but those settings were not verified at runtime.

## Documentation evidence (documented behavior, not observations)

- Rust client overview and connecting: <https://duckdb.org/docs/2.0/clients/rust/overview.html>, <https://duckdb.org/docs/2.0/clients/rust/connecting.html> (bundled client, read-only connection configuration, connection thread-safety).
- Extension installation and local cache: <https://duckdb.org/docs/lts/extensions/installing_extensions>; Spatial requires `INSTALL spatial` then `LOAD spatial` and is not autoloadable: <https://duckdb.org/docs/2.0/core_extensions/spatial/overview>.
- GDAL integration and `ST_Read` behavior: <https://duckdb.org/docs/2.0/core_extensions/spatial/gdal>, <https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html#st_read>. The latter says `spatial_filter` pushdown is driver-dependent; otherwise GDAL filters and may be slower.
- Profiling and query interruption: <https://duckdb.org/docs/2.0/clients/rust/profiling.html>; settings including memory, threads, and maximum execution time: <https://duckdb.org/docs/2.0/configuration/overview.html>.
- Transaction rollback and batch atomicity requirements: <https://duckdb.org/docs/2.0/sql/statements/transactions.html>; concurrency model: <https://duckdb.org/docs/2.0/connect/concurrency.html>.
- Published Rust crate version/features: <https://crates.io/api/v1/crates/duckdb> (version `1.20000.2`, bundled feature; Spatial is not a bundled Rust-client feature).

## Reproducibility and acceptance status

The exact standalone release invocation shape is:

```sh
CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan7-duckdb/target \
DUCKDB_HOME=/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home \
cargo run --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml --release -- offline
```

The probe built and ran the invocation but exited nonzero at extension loading. Hash/mtime integrity checks passed; `git diff --check` is required after this report is written. Iowa comparisons, resource measurements, and disposable execution-semantics probes remain unverified. Do not infer compatibility or performance from this partial evaluation.
