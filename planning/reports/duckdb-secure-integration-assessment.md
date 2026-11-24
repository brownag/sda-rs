# DuckDB secure-integration assessment

**Task:** Plan 14, task 14.6.1
**Date:** 2026-11-22
**Scope:** Synthesis of Plan 14 evidence for a possible optional DuckDB backend. This report proposes a qualification and integration direction; it does not change policy or authorize implementation, request-path exposure, dependency updates, deployment, or production use.

## Executive assessment

The evidence supports retaining DuckDB as an **internal qualification backend**, not exposing the current design to request-controlled SQL. On the pinned Rust client `duckdb = "=1.20000.2"` / bundled engine 2.0.0, configured connections deny selected filesystem and extension operations. Those controls are useful defense in depth, but direct-connection probes show that SQL the executor classifier rejects—including persistent in-memory DDL, compound statements, comments, quoting, and settings changes—can be accepted by DuckDB. The classifier is string-based, not a parser-backed authorization boundary. The probes are finite and do not establish exhaustive coverage.

The lifecycle work establishes fresh in-memory state per execution, transaction rollback for tested failures, and deadline-triggered interruption. It does **not** cancel the blocking task when the async future is dropped. The private GeoPackage loader has meaningful synthetic structural checks and read-only SQLite access, but no production caller/trust-provisioning lifecycle or materialization bounds. The benchmark is small, synthetic, single-host evidence and does not justify numeric limits or a production performance conclusion. It does include a separately reported planar `ST_Area` workload (median/p95); this measures the pinned Spatial backend only, not SQL Server geography parity.

The executor and spatial-rejection implementation changes visible in the reviewed branch were authorized under Plan 13 tasks 13.3–13.10. Plan 14's prohibition applies to fixes discovered during its evidence-only tasks unless separately approved; this assessment does not authorize further implementation.

**Recommendation:** Keep SQLite as the supported default and DuckDB qualification-only. Do not expose the current in-process DuckDB executor to request-controlled SQL. A later, explicitly approved integration plan may evaluate a narrower, parser/authorizer-backed query interface, a separately qualified and patched client/engine/extension tuple, a trusted import lifecycle, calibrated resource controls, and an appropriately isolated execution boundary. These are recommendations and gates, not approved changes. No OS sandbox necessity or sufficiency verdict is made.

## Decision context and alternatives

Accepted ADR 0007 keeps SQLite the default, permits qualification of an additive opt-in DuckDB backend, rejects silent fallback, preserves unsupported geography-method rejection, and requires request-local state and no request-time extension downloads. It records performance as an objective, not a security waiver or release gate; numeric resource limits require separate measurement and approval. This assessment does not supersede that decision.

| Alternative | Assessment from current evidence |
|---|---|
| Continue SQLite-only service operation; keep DuckDB internal for qualification | **Supported now.** Preserves the established service boundary while allowing bounded synthetic work. |
| Expose current DuckDB executor to request-controlled SQL in-process | **No-go on current evidence.** The classifier is not a parser/authorizer; direct API probes demonstrate material bypass differences, and resource, cancellation, and trust-lifecycle gaps remain. |
| Later add a narrow, explicitly selected DuckDB path while retaining SQLite | **Conditional future option.** Consistent with ADR 0007 only after separate authorization and closure or explicit acceptance of material risks through evidence-backed controls. No selector, fallback, or route is authorized here. |
| Replace SQLite, silently switch backends, or substitute SpatiaLite for DuckDB | **Not recommended / inconsistent with ADR 0007.** SQLite remains the supported default; backend and spatial behavior must not change silently. |

## Trust boundaries and proposed architecture

The following is a design proposal, not a description of an implemented request path:

1. **Input and SQL authorization.** Treat request SQL as hostile if a future route accepts it. Admit only a deliberately small query language, preferably using parser-backed statement classification and explicit relation/function authorization. Bind values where the application owns query structure; binding does not make arbitrary user-authored SQL safe. Keep persistent writes, settings changes, extension control, unrestricted table functions, comments/compound syntax, and unreviewed functions outside the admitted surface. Require adversarial grammar/version tests and direct-API defense-in-depth evidence; finite tests alone cannot prove completeness.
2. **Execution lifecycle.** Preserve fresh per-request in-memory connections, transactionally atomic batches, and explicit cleanup. Use blocking execution and an application deadline with interruption. Treat future-drop cancellation as unresolved: the blocking worker may continue until completion or deadline. Define and verify behavior for interrupted preparation/execution, rollback, cleanup, and connection disposal before exposure.
3. **Data import.** Continue to avoid DuckDB `ST_Read` or request-selected paths as a trust mechanism. The qualified loader canonicalizes an internally supplied path, opens SQLite read-only, validates selected GeoPackage metadata/SRID/dimensions and WKB structure, then copies bound values into a fresh in-memory DuckDB table. A future integration needs an explicitly authorized deployment provisioning/capability lifecycle, race/symlink policy, and tests for malformed/ambiguous inputs and limits. Its current private status does not establish who may provision or invoke it.
4. **Engine and extension.** Provision an exact client → bundled engine → platform extension tuple outside request handling. Verify provenance and hash before startup; load only from a deployment-controlled path, with no request-time download/autoinstall and no silent fallback. Pin and test the initialization order and exact settings on the selected tuple. The currently recorded Spatial hash is for engine 2.0.0 and is not evidence of compatibility with another engine.
5. **Resource and operations.** Before any service exposure, select and test application/engine bounds for deadline/cancellation, memory, threads, admission/concurrency, temporary storage, input/query size, rows, individual values, and serialized output. Monitor denials, failures, resource consumption, and extension identity with privacy-conscious logging. Monitoring detects incidents; it is not an authorization or containment control. The benchmark supplies no calibrated production values.
6. **Process privilege and isolation.** A fresh database connection does not isolate the process's filesystem, network, native extensions, CPU, or memory privileges. Evaluate a least-privilege service identity and process/container/OS/network controls against the intended threat model. DuckDB's cited security guidance recommends additional sandboxing for untrusted SQL, but this evidence does not establish that a particular sandbox is necessary or sufficient for this application.

## Evidence-backed safeguards and their limits

| Area | Evidence-backed observation | What it does not establish |
|---|---|---|
| Connection/configuration | Executor creates fresh in-memory connections with external access and known-extension autoload disabled. Direct 2.0.0 probes denied tested local file reads/writes, import, extension installation/loading, and attempts to re-enable external access in tested forms. | Exhaustive filesystem/network denial, immutable settings, protection against arbitrary SQL/API callers, or security of every extension/function. Direct probes could mutate autoload/autoinstall settings; downstream effects were not tested. |
| SQL gate | Executor rejects tested non-SELECT and disallowed forms and retains a narrow temporary-table workflow. | Parser completeness or authorizer-equivalent enforcement. Direct DuckDB accepted in-memory DDL, `ATTACH ':memory:'`, compound SQL, comments, quoted syntax, and setting changes that the executor path rejected. |
| I/O and secrets | Selected filesystem paths were denied by configuration. `CREATE SECRET` attempts hit the configured secrets-directory permission denial. | Universal secret denial or tested remote URL/path-traversal denial. The remote Parquet case was inconclusive because the extension was unavailable; no endpoint was contacted. |
| Lifecycle | G38 evidence records request-local state, batch transaction/rollback, cleanup, and an interrupt handle observed interrupting a synthetic long query; the deadline contract passed. | Cancellation on future drop, bounded cleanup after drop, or all interruption/reuse/error paths. The blocking task can continue until completion/deadline. |
| GeoPackage | Synthetic tests cover read-only opening, selected metadata/SRID/dimensional/WKB checks, NULL/empty handling, scalar copying, and an exact serialized example. Generic result BLOB/GEOMETRY values fail closed. | Production trust provisioning, race resistance, complete GeoPackage/WKB validity, topology validation, broad copy equivalence, or bounded row/byte/memory use. |
| Spatial semantics | Pinned artifact identity/hash is recorded. Six SQL Server geography methods remain rejected; probe evidence and Microsoft/DuckDB source review distinguish incompatible and unresolved semantics. | SQL Server parity. Four methods are no-go; area and text remain bounded candidates/inconclusive, not approvals. |
| Benchmark | Correctness-asserted synthetic serial and four-worker observations, including one process RSS sample, are recorded on one Linux host. | Production load/capacity, controlled SQLite-vs-DuckDB comparison, scaling, or numeric limits. Timing boundaries differ; no caps are recommended. |

## Threat model and residual risks

Assume a future caller may supply adversarial SQL or data and may attempt to read or write files, reach network resources, create secrets, change settings, load native extensions, persist state, exploit parser ambiguity, or exhaust memory/CPU/temp storage/output. Also consider malformed GeoPackage metadata/payloads, interruption races, and compromise impact under the service process identity. These are design threats; this report does not assert that a request route currently exposes DuckDB.

Material residuals in the current evidence:

- **Authorization bypass / overbroad SQL:** direct engine acceptance differs from classifier denials; grammar and reachable function surfaces are not exhaustively mapped.
- **Version-specific security defects:** the current engine is 2.0.0. Official 2.0.1 release notes identify fixes for `CREATE SECRET` with external access disabled and remote path traversal under that setting, plus a Brotli decoder CVE fix. Version/configuration applicability is supported by sources; application exploitability is not demonstrated.
- **Network and extension consequences:** remote URL denial was not isolated; mutable autoload/autoinstall settings were observed, but successful downstream extension/I/O consequences were not tested.
- **Cancellation and resource exhaustion:** future-drop cancellation is absent; the loader and execution path lack calibrated bounds for several dimensions.
- **Import trust/lifecycle:** private capability construction and read-only parsing do not define deployment authority, filesystem race behavior, or production lifecycle.
- **Native-code blast radius:** extensions share process privileges. No deployment identity, filesystem policy, network isolation, or sandbox has been tested.
- **Geography behavior:** unsupported methods remain rejected; candidate probes do not prove documented SQL Server parity.

## Upgrade position

The pinned crate `1.20000.2` bundles engine 2.0.0; it is not an engine 1.2.0. The separately provisioned Spatial artifact is v2.0.0 with SHA-256 `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc` and recorded extension identifier `a3dbcd1781`. The pin assessment recommends evaluating DuckDB 2.0.1 promptly, but the exact released Rust crate → engine 2.0.1 → compatible Spatial artifact/hash tuple was not established.

Therefore: retain the current tuple only as the reproducible qualification baseline; prioritize a separately approved upgrade qualification. Do not infer that a crate version implies a particular engine patch, reuse the 2.0.0 Spatial artifact with a different engine by assumption, or claim the 2.0.1 fixes have been runtime-verified. A candidate must pass exact identity/provenance checks and offline security, loader/result, and geography regression qualification before any adoption decision.

## Resource evidence and recommendation boundaries

Plan 14's synthetic benchmark reports a four-worker sample of about 267–268 operations/s and a process maximum RSS sample of 65,772 KiB; the latter includes runtime/library overhead and short-lived connections. It records engine default `threads=32` and `.tmp` as the temporary directory setting, without measuring worker-pool behavior or spill. Inputs were tiny; there was no scaling curve, sustained load, or deadline test. The scalar/join/batch timings are not comparable to the SQLite executor baseline, and serialization/spatial cases also differ in implementation or semantics. The ADR's 20% objective and >10% regression reporting criterion remain non-gating observations, not passed/failed evidence for production.

No numeric limits are recommended here. Candidate controls include configured thread ceilings, bounded admission, memory/temp-storage controls, SQL/input and output/row/value bounds, and a deadline; each requires representative calibration, enforcement tests, and explicit approval. The private loader's unbounded materialization is a separate concern.

## Go/no-go matrix and later gates

| Decision | Current verdict | Evidence or gate |
|---|---|---|
| Expose current in-process executor to request-controlled SQL | **NO-GO** | String classifier is not a parser/authorizer; direct bypasses are observed; remote/security surface and resource/lifecycle gaps remain. |
| Continue internal synthetic qualification | **GO, within existing scope** | Offline/synthetic probes and tests are evidence-building only; no service exposure or production claim. |
| Adopt current 2.0.0 pin for future service exposure | **NO-GO pending upgrade assessment** | 2.0.1 fixes overlap the current version/configuration; exploitability is not demonstrated, but retaining 2.0.0 requires explicit risk acceptance after candidate evaluation. |
| Select a newer client/engine/Spatial tuple | **NOT YET DECIDED** | Establish exact released pairing, ABI/load compatibility, provenance/hash, and rerun relevant offline contracts. Requires separate approval for any pin change. |
| Add request-path/backend selection | **NOT AUTHORIZED** | Later plan and explicit approval must preserve SQLite default, explicit selection, no silent fallback, and wire/API contract. |
| Permit geography methods | **NO-GO** | Four no-go findings; area/text evidence remains bounded/inconclusive. Keep explicit rejection absent method-specific contract evidence and approval. |
| Set production numeric resource caps or claim readiness | **NO-GO** | Benchmark does not calibrate capacities; lifecycle, loader, isolation, operations, and enforcement require qualification and explicit decisions. |

Before a later enablement proposal, require at minimum: (1) an approved threat model and accepted residual-risk statement; (2) exact patched client/engine/extension identity and compatibility evidence; (3) a narrowly specified parser/authorizer surface with adversarial testing and direct-engine controls; (4) trusted data provisioning and import lifecycle evidence; (5) cancellation, cleanup, and resource-bound tests with workload-based values; (6) a least-privilege/process-boundary and monitoring design; and (7) regression/compatibility evidence plus explicit approval of backend selection, deployment, and any policy changes. These are decision gates for a future plan, not blanket demands that every theoretical risk disappear; each material residual must be bounded, mitigated, or explicitly accepted by the authorized decision-maker.

## Evidence references

- [Plan 14 pin and upgrade assessment](duckdb-pin-and-upgrade-assessment.md)
- [Plan 14 security and loader inventory](duckdb-security-and-loader-evidence.md)
- [Plan 14 direct security probes and version-matched control analysis](duckdb-security-probe-results.md)
- [Plan 14 private GeoPackage loader audit](duckdb-loader-audit.md)
- [G37 geography contract and pinned probes](G37-geography-transformations.md)
- [Plan 14 DuckDB workload/resource benchmark](duckdb-qualification-benchmark.md)
- [SQLite benchmark baseline](duckdb-sqlite-benchmark-baseline.md)
- [Accepted ADR 0007](../../docs/decisions/0007-duckdb-opt-in-query-backend.md)
- [DuckDB execution policy](../../docs/security/duckdb-execution-policy.md)
- [G38 lifecycle evidence](../STATE_13.md#1361--implement-the-internal-duckdb-execution-lifecycle-g38)

The cited project reports include reproduction commands, test outcomes, and primary-source links. This synthesis adds no runtime evidence and makes no production-readiness claim.
