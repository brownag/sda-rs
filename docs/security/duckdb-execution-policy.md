# DuckDB execution policy (Plan 10)

This policy specifies the security boundary for the opt-in DuckDB backend. It
does not claim that DuckDB provides a SQLite-equivalent authorizer. Controls
whose enforcement API has not been established are marked **BLOCKER** and must
be resolved before production execution is enabled. Persistent database writes
are outside the accepted query surface. Request-local state, including
temporary-table workflows, is retained as required by ADR 0007.

## Threat controls and adversarial acceptance cases

| Threat / behavior | Required control | Adversarial acceptance case |
| --- | --- | --- |
| Persistent writes or schema changes | Reject user SQL that writes persistent tables or changes persistent schema. Permit only specifically supported request-local temporary-table operations. The exact DuckDB enforcement seam is **BLOCKER** until an API is verified; do not assume SQLite authorizer parity. | Persistent `INSERT`, `UPDATE`, `DELETE`, `CREATE TABLE`, and schema-changing statements are denied. A supported temporary-table create/insert/read/drop sequence succeeds within one request; a second request cannot observe its state. |
| Arbitrary file reads/writes | Disable DuckDB external access and use the narrowest verified path controls for any explicitly required source. The G33-2 probe observed `enable_external_access=false` and rejection of an attempt to set it back to true; exact production path policy and all file-capable SQL surfaces still require adversarial verification. | Queries attempting to read or write an unapproved local path fail. If a deployment explicitly permits a data path, access outside that path fails. |
| Network access | Disable external access before processing user SQL. No request-time extension installation or other network acquisition is allowed. Validate all supported network-capable functions and extension paths; the setting alone is not treated as proof until tested on the pinned production build. | A query attempting network access fails, and the failure does not trigger installation, download, or fallback. |
| Extension autoload/autoinstall or user-selected extension | Disable `autoload_known_extensions` and `autoinstall_known_extensions` before user SQL. Load only the deployment-provisioned official Spatial artifact by its verified path, before external access is disabled if the pinned build requires that order. User SQL cannot load, install, or select another extension. | With a clean extension cache and networking unavailable, startup loads only the provisioned artifact; SQL attempting `INSTALL`/`LOAD` or changing extension settings fails. Repeat with Spatial absent: non-spatial SQL remains available and spatial SQL fails explicitly. |
| Cross-request state or partial batch effects | Use request-local execution state and preserve all-or-nothing failure semantics in multi-statement requests, per ADR 0007. The production lifecycle and transaction behavior require implementation tests. | A temporary table created in one request is absent in another. A failing multi-statement batch leaves no partial persistent or request-local effects visible after request completion. |
| Resource exhaustion | Apply separately verified limits for execution duration/cancellation, memory, threads, concurrent work, temporary storage, input size, rows, value size, and serialized response. Required values and enforcement are described below; unresolved limits block production enablement. | For each enforceable limit, exercise below-limit success and above-limit rejection/cancellation, including cleanup and connection reuse/closure behavior. |

The transpiler's SQL validation remains required defense in depth. It must reject
unsupported and transaction-control SQL before execution; it is not a substitute
for the DuckDB runtime controls above.

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

**BLOCKER:** Verify on the exact production pin that settings cannot be
re-enabled or bypassed by user SQL, that extension loading is restricted to the
provisioned artifact, and that external access is disabled before untrusted SQL
can run. If this cannot be enforced, do not enable the DuckDB backend.

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
| Execution deadline and cancellation | **BLOCKER:** establish a DuckDB/client cancellation mechanism that reliably interrupts running work and bounds cleanup. The Plan 8 process timeout and SQLite progress handler do not establish in-process DuckDB cancellation. |
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
