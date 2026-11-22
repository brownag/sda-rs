# DuckDB execution policy (Plan 13 G39)

This policy describes tested mechanisms in the internal, feature-gated DuckDB
executor. DuckDB remains unreachable from HTTP requests. It does not claim
production readiness or SQLite-authorizer parity. Controls not demonstrated
below remain **BLOCKER**. Persistent writes are outside the accepted SQL
surface; request-local temporary-table workflows are retained.

## Threat controls and adversarial acceptance cases

| Threat / behavior | Required control | Adversarial acceptance case |
| --- | --- | --- |
| Persistent writes or schema changes | A conservative SQL gate admits SELECT and a narrow temp-table workflow only. No SQLite-equivalent authorizer is available in the pinned client. | Tested persistent CREATE/INSERT/UPDATE/DELETE/schema changes are rejected. TEMP create-as-select, insert to a tracked temp table, select, and drop pass; untracked targets fail. |
| Arbitrary file reads/writes | Connection is opened in-memory with `enable_external_access=false`; policy gate rejects non-read/temp SQL. | Tested `read_csv` and `COPY TO` attempts fail. All DuckDB file-capable surfaces and paths are not exhaustively proven. |
| Network access | External access is disabled before statements, and non-SELECT/temp statements are rejected. | Tested remote `read_parquet` SQL fails. This finite contract test is not proof against every extension/function network path. |
| Extension autoload/autoinstall or user-selected extension | `Config::enable_autoload_extension(false)` disables known-extension autoload and autoinstall; SQL gate rejects INSTALL/LOAD and setting changes. No user-selected extension load path exists. | Tested INSTALL, LOAD, and attempts to change autoload, autoinstall, and external-access settings fail. Spatial is not loaded in this executor. |
| Cross-request state or partial batch effects | Fresh in-memory connection per execution, executor-owned transaction and rollback. | Tested temp state is unavailable in a subsequent execution; lifecycle contract separately tests failed-batch cleanup. |
| Resource exhaustion | Apply separately verified limits for execution duration/cancellation, memory, threads, concurrent work, temporary storage, input size, rows, value size, and serialized response. Required values and enforcement are described below; unresolved limits block production enablement. | For each enforceable limit, exercise below-limit success and above-limit rejection/cancellation, including cleanup and connection reuse/closure behavior. |

The transpiler's SQL validation remains required defense in depth. The executor
gate is a deliberately restrictive interim classifier, not a general SQL
parser/authorizer: comments, quoted identifiers, compound statements, and SQL
outside the accepted forms are rejected. Its current implementation is
string-based; **BLOCKER:** establish parser-backed complete statement
classification and adversarially verify bypass resistance before production
execution. Do not broaden the accepted grammar without corresponding tests.

## Trusted Spatial artifact and initialization

ADR 0007 requires deployment provisioning and verification, no request-time
extension network access, no SpatiaLite requirement or silent fallback, and
continued non-spatial operation when Spatial is unavailable.

The G33-2 follow-up records a scratch DuckDB `v2.0.0` / Rust client
`duckdb = "=1.20000.2"` probe. It loaded the cached Spatial binary by absolute
path, recorded extension identifier `a3dbcd1781` and SHA-256
`8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`, then
disabled autoload/autoinstall and external access. The probe verified
`external_access=false` and that re-enabling it was rejected. Later probe
iterations also set an exact `allowed_paths` entry. These are observations for
that scratch build and are not, by themselves, production API guarantees.

Production provisioning must pin the engine/client and official Spatial
artifact, verify the artifact hash before startup, and load only that artifact
from its deployment-controlled path. The artifact must not be fetched or
updated while serving a request. The exact supported official artifact/version,
its compatibility with the chosen engine, and the initialization sequence must
be fixed and tested for the production pin. Do not treat the scratch hash above
as the production pin without explicit version selection and deployment
verification.

**BLOCKER:** The above settings and denials are verified only on the pinned
DuckDB Rust client `1.20000.2` through focused contract tests. They do not
establish a production artifact pin, trusted Spatial loading path, complete
external-I/O denial, or resistance to every SQL classifier bypass. Resolve
these blockers before any request-path enablement.

## Internal GeoPackage value import

The qualification executor includes a private Rust-side GeoPackage loader.
It accepts an internally constructed canonical-path capability, opens the
GeoPackage with SQLite read-only flags, validates the required geometry
metadata, CRS/SRID agreement, GeoPackageBinary header, and supported WKB
structure/dimensions, then copies validated values into a fresh in-memory
DuckDB connection using bound values. The file path is never placed in DuckDB
SQL; this path does not call `ST_Read`, load Spatial, or enable file-reading
extensions. SQL and HTTP requests cannot supply or select the path. This is
internal qualification code only; no route/configuration integration or
production-readiness claim is made.

This import boundary does not resolve G39's string-based SQL classifier and
its bypass-resistance blocker. It is not a general GeoPackage importer:
unsupported metadata, geometry types, malformed WKB, or dimensional
disagreement fail closed. SQLite remains the default backend.

## Resource limits and calibration

Task 10.1.1 measured five deterministic SQLite workloads serially on one host,
with 100 measured repetitions per workload. It did not measure peak RSS,
concurrent throughput, production-sized data, or end-to-end HTTP behavior.
Those results cannot calibrate DuckDB memory, concurrency, temporary-storage,
or capacity limits. No DuckDB production numeric caps are established by this
policy.

The SQLite policy's 2 MiB SQL text, 64 MiB SQLite value, 100,000 rows, 64 MiB
serialized output, and 60-second execution deadline are SQLite/G32 values, not
automatically portable DuckDB limits. The Plan 8 probe's `1GB` memory, two
threads, and 60-second process timeout were experimental probe settings, not
production caps. Keep these categories distinct.

| Resource | DuckDB policy / evidence status |
| --- | --- |
| Execution deadline and cancellation | Existing G38 executor has an interrupt mechanism, but future-drop cancellation and bounded cleanup remain **BLOCKER**. |
| Memory | DuckDB's `memory_limit` was set to `1GB` in Plan 8 probes only. **BLOCKER:** calibrate a production value and verify its scope, enforcement, and behavior under allocation failure; the SQLite benchmark did not measure memory. |
| Threads | `threads=2` was a Plan 8 probe setting only. **BLOCKER:** select and verify a production per-instance/per-query thread policy and its interaction with concurrent requests. |
| Concurrency | **BLOCKER:** establish a bounded admission/concurrency mechanism and production target. Task 10.1.1 was serial and measured no concurrent throughput. |
| Temporary storage | A scratch temp directory was used in Plan 8 probes. **BLOCKER:** establish and verify production temp-directory isolation, quota/size bounds, permissions, and cleanup. |
| SQL/input size | Preserve the HTTP request-body limit and establish a specific SQL text limit at the application boundary. No DuckDB-specific SQL cap is calibrated here. Verify oversized input denial before query execution. |
| Rows | Enforce a returned-row limit in result collection, independent of engine memory limits. No DuckDB row threshold is approved by the SQLite benchmark; calibrate and approve a value before production enablement. |
| Value size | **BLOCKER:** establish whether individual DuckDB values can be bounded before materialization/serialization; otherwise document the residual risk and do not claim a cap. |
| Serialized response | Enforce an application-layer serialized-byte limit before sending the response. No DuckDB-specific threshold is calibrated; choose and test a value separately from SQL engine settings. |

Numeric limits for rows, SQL/input, serialized output, value size, deadline,
memory, concurrency, and temporary storage require representative workload
measurements and explicit approval. The small SQLite baseline supplies workload
shapes and latency only; it does not justify copying its thresholds or
extrapolating DuckDB capacity. Resource calibration must remain separate from
the Plan 8 probe caps.

## Evidence and blockers

- DuckDB 2.0 securing guide:
  <https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/overview>
- DuckDB 2.0 Rust querying guide:
  <https://duckdb.org/docs/2.0/clients/rust/querying>
- Runtime probe observations and exact scratch artifact fingerprint:
  `planning/reports/G33-2-duckdb-spatial-followup.md`
- SQLite comparison policy (not a DuckDB API specification):
  `docs/security/sqlite-execution-policy.md`
- Accepted backend contract:
  `docs/decisions/0007-duckdb-opt-in-query-backend.md`
- SQLite executor comparison anchors: `ExecutionLimits`, authorizer, and
  progress handler in `src/executor/sqlite.rs`.

Before enabling production requests, resolve the blockers in this document:
prove write denial and temporary-table allowance on the chosen DuckDB API;
verify file/network/extension controls on the exact engine and extension pins;
prove cancellation and cleanup; bound memory, threads, concurrency, temporary
storage, values, rows, and output; and calibrate/approve numeric thresholds.
No OS sandbox is part of this policy.
