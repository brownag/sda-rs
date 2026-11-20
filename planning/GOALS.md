# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-20
Source State: STATE_10.md (Plan 10 DuckDB evaluation complete; production tasks paused)

## Circled Goal (Active)

- **[G47]** A non-production DuckDB vertical slice accepts representative SDA T-SQL, translates it to DuckDB SQL, executes it, and returns the existing SDA response shape for a documented non-spatial subset.
  - Why now: The scratch evaluation established a narrow working subset and surfaced concrete mismatches (`IIF`, CTAS count result shaping, and typed result mapping). A repository-local spike will test the real integration seam before production architecture, controls, or scope are selected.
  - Evidence: `planning/reports/duckdb-tsql-pipeline-evaluation.md`; these findings are evaluation evidence, not production guarantees.
  - Exit criteria: An approved plan defines a strictly opt-in/non-production slice; representative T-SQL translation, DuckDB execution, and SDA serialization have contract tests; supported and explicitly rejected cases are recorded; SQLite behavior remains unchanged; no production enablement or controls are inferred.
  - First action: Create a scoped plan for the smallest vertical slice, starting with the `IIF` translation and non-row-producing batch result-shaping gaps, and identify the route/transpiler/executor/serializer seams.

## Sequenced Goals (Dependency DAG)

- **[G31]** Spatial metadata is resolved for every referenced table and geometry column, and missing or ambiguous metadata is rejected explicitly.
  - Depends on: G29, G30.
  - Exit criteria: Tests demonstrate metadata resolution for each referenced table/geometry column and explicit rejection of missing or ambiguous metadata; existing Mukey validation and non-spatial queries remain intact.
  - First action: Audit `src/spatial.rs` and spatial-transpiler call sites against multi-table geometry queries; identify uncovered cases before planning implementation.
- **[G36]** The production acceptance contract is reviewed and approved for DuckDB as an opt-in SDA query backend, with SQLite retained as the default and supported backend; the intended value is improved performance without requiring SpatiaLite.
  - Depends on: G47, G29, G30, G32, and completed G33-2 evaluation.
  - Exit criteria: User decisions resolve production role, supported query surface, Linux x86_64 initial target, security/resource controls, deployment behavior, and performance acceptance evidence. The existing SQLite contract remains the reference; the approved numerical performance gate is retained, but no performance result is claimed until a comparable end-to-end benchmark exists.
  - Gate: This is a decision goal, not implementation authorization. Production tasks remain paused until explicit user approval.
- **[G37]** DuckDB's production query surface has an explicit, tested support/rejection matrix, including T-SQL translation and per-method outcomes for the six G12 geography methods.
  - Depends on: G36, G47, G29.
  - Exit criteria: Included SQL constructs and spatial methods have defined semantics, units/CRS requirements, NULL/error behavior, and acceptance tests. `IIF`, batch result shaping, and DuckDB value mapping are resolved or explicitly rejected. The six CLR geography methods remain explicitly rejected absent an approved, runtime-enforceable SQL Server-compatible contract.
- **[G38]** DuckDB execution preserves the chosen request isolation, blocking-worker, batch, and failure semantics.
  - Depends on: G36, G37.
  - Exit criteria: The backend boundary and connection lifecycle are documented and integration tests show request state isolation, cancellation/deadline behavior, and the agreed batch failure/rollback contract.
- **[G39]** DuckDB query execution enforces reviewed file-access, write, extension-loading, and resource controls.
  - Depends on: G32, G36, G38.
  - Exit criteria: Adversarial tests demonstrate denied unauthorized reads/writes, extension changes, and resource-limit violations while valid read-only workloads continue to work.
- **[G40]** GeoPackage schema, geometry metadata/encoding, DuckDB result values, and SDA response serialization meet the selected contract.
  - Depends on: G31, G37, G38, G47.
  - Exit criteria: Fixtures cover relevant table metadata, geometry representations, CRS handling, NULL/empty/dimensional values, DuckDB type mapping, and serialized SDA result types.
- **[G41]** DuckDB and required extensions are reproducibly versioned and available for the selected deployment platform.
  - Depends on: G36.
  - Exit criteria: Linux x86_64 deployment provisions pinned DuckDB and official Spatial artifacts from approved sources, verifies versions and hashes, and records license, provenance, upgrade, and request-time offline behavior.
- **[G42]** DuckDB meets the approved memory, concurrency, latency, temporary-storage, and result-size targets on representative SDA workloads.
  - Depends on: G38, G40, G41.
  - Exit criteria: Reproducible workload tests compare DuckDB with the current backend against approved targets; peak resource use and failures are recorded. Current scratch timings are non-comparable and do not establish this goal.
- **[G43]** Users can explicitly select DuckDB while SQLite remains the default with a defined failure and fallback policy.
  - Depends on: G39, G40, G41, G42.
  - Exit criteria: Configuration and rollout tests prove default SQLite behavior is unchanged, opt-in selection is visible, non-spatial DuckDB queries remain available when Spatial is unavailable, spatial requests fail explicitly, and there is no silent SpatiaLite fallback.
- **[G44]** CI validates the DuckDB backend's supported behavior, security boundary, resource limits, and deployment artifacts.
  - Depends on: G37, G39, G40, G41, G42, G43.
  - Exit criteria: Required CI jobs run deterministic contract/security tests and platform-specific artifact checks without accessing user data or external sources.
- **[G45]** DuckDB backend setup, supported behavior, limitations, operations, and recovery are documented for users and maintainers.
  - Depends on: G43, G44.
  - Exit criteria: Documentation matches tested configuration and deployment behavior, identifies unsupported SQL/geography semantics, and explains rollback to SQLite.
- **[G33-1]** terrio's GeoPackage I/O, geometry decoding/operations, CRS transformations, and memory profile are evaluated on bounded representative fixtures without changing the service backend.
  - Depends on: G29, G30.
  - Exit criteria: A sourced spike note records API capabilities and gaps; a small, read-only fixture probe records geometry/CRS comparisons and elapsed time/peak RSS, or explicitly documents a blocker preventing safe measurement. No whole-layer load is attempted without a safety estimate; no SQL-engine adoption is inferred.
  - First action: Build an isolated probe using terrio's `sqlite` feature and a bounded sample, leaving Cargo dependencies and the Iowa source GeoPackage unchanged.
## Independent Backlog

- **[G9]** Request handling has an explicit, configurable payload limit with tested boundary behavior.
  - Evidence: Plan 4 records the current 2 MiB limit; configuration remains open in prior goal notes.
- **[G8]** CLI and environment configuration for backend URL, port, and geometry settings matches the documented operational contract.
  - First action: Audit existing configuration/tests and record any remaining gaps before implementation.
- **[G10]** PostgreSQL/PostGIS executes the same explicitly supported query surface as the SQLite/GeoPackage backend.
  - Trigger recorded earlier: G1 acceptance and backend interfaces are stable; architecture and backend scope still need planning.
- **[G13]** SQLite compatibility includes general `CROSS APPLY` and `OUTER APPLY` rewrites.
  - Trigger: A concrete supported workload demonstrates need beyond the MVP query set.
- **[G14]** The service implements SDA modes beyond tabular query, including AOI, interpretation, and report services, plus Shape commands.
  - Trigger: A user requirement identifies a mode as a priority.
- **[G25]** The spatial transpiler is split by reason to change: string normalization, derived-table expansion, and method/constructor rewriting.
  - Depends on: completed G22; lower leverage than correctness and security goals.
- **[G26]** The dependency list matches the initial project requirements, or remaining deviations are recorded with rationale.
  - First action: Compare `Cargo.toml` with `planning/INITIAL_PROMPT.md`; do not add dependencies without approval.

## Deferred

- **[G12]** The broader CLR spatial method set—including `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—is supported with documented unit/SRID semantics.
  - Depends on: G29; AOI construction/intersection MVP behavior is covered by Plan 4.
  - Trigger: Reconsider only when a runtime-enforceable implementation path can meet SQL Server geography semantics for a defined subset. Plan 9 found no safe implementation at the current SQLite/SpatiaLite seam; Plan 10 preserves those rejections unless new evidence changes the boundary.
- **[G46]** DuckDB's community `geography` extension provides production support for any member of the six-method G12 set.
  - Trigger: Reconsider only if a future maintained release supplies the missing/incorrect operations and passes the G37 semantic contract, including SQL Server boundary and SRID behavior.
- **[G34]** The server is hosted and validated against SDA applications using representative real client workflows.
  - Depends on: completed G32, G29, G30, and G11.
  - Trigger: Reconsider when a hosting/deployment target and authorization for hosted client trials are identified. The local opt-in soilDB/Iowa harness is not hosted validation.
  - Exit criteria: Run representative soilDB and soildb Python workflows against the hosted server; record pass/fail gaps and operational issues.
- **[G28]** Native SQLite spatial acceleration.
  - Depends on: G29, G30, G31.
  - Trigger: Reconsider only after those prerequisites validate the geometry representation, predicate semantics, exact evaluation, and R-tree assumptions.
  - Exit criteria: R-tree is used only for conservative candidate filtering followed by exact predicates; indexed and non-indexed results match, including when no usable index exists.
- **[G35]** Establish whether the opt-in `tests/soildb/` client harness remains worth maintaining after fixture coverage is established.
  - Trigger: Revisit when captured fixtures and deterministic replay have been compared against the client behaviors covered by the opt-in harness.
  - Exit criteria: Record which behaviors only the real soilDB/Iowa run covers; retain the harness as supplementary coverage or remove it only if those behaviors are covered elsewhere.

## Completed

- **[G32]** SQLite execution enforces an explicit, tested security boundary for submitted SQL, extension loading, registered UDFs, and resource use. Plan 5; `STATE_5.md` records all waves and acceptance checks complete.
- **[G29]** The supported spatial SQL contract documents method coverage, NULL/error behavior, CRS assumptions, and intentional semantic differences. Plan 6 Wave 1; `docs/spatial-sql-contract.md`.
- **[G30]** GeoPackageBinary decoding and validation cover headers, envelope flags, empty geometries, malformed blobs, and WKB extraction. Plan 6 Wave 2; `src/gpkg_geometry.rs` and its tests.
- **[G11]** The package-used bounded `SDA_Get_*` subset is implemented and tested. Plan 6 inventoried all 34 catalog functions and active soilDB/py-soildb call sites; active-client variants and additional feasible functions are documented and contract-tested where dataset prerequisites permit. See `docs/sda-get-support.md` and `STATE_6.md`.
- **[G33-2]** DuckDB Rust integration, Spatial, and community `geography` are evaluated as isolated candidates for G12 and bounded GeoPackage access. Plan 8; `STATE_8.md` and `planning/reports/G33-2-duckdb-spatial-followup.md`.
- **[C2]** PR #1 review findings were resolved and all 17 inline replies were posted with user approval; PR #1 was subsequently merged (`62a6b98`). The Cargo JSON reader drains through EOF and was smoke-tested with a non-default target directory.
- **[G1]** SQLite/GeoPackage-focused SDA tabular MVP acceptance: request encodings, supported T-SQL, AOI, mukey TVF contexts, `#temp` batches, exact metadata serialization, and unavailable spatial capability behavior. Plan 4; configured SpatiaLite run passed.
- **[G6]** Variable folding, CLR spatial rewriting, and WGS84 mukey intersection UDF execute end to end against seeded spatial fixtures. Plan 4.
- **[G7]** Deterministic in-process integration acceptance covers the G1 endpoint, serializer, batch, temp-table, and spatial behavior. Plan 4.
- **[G16]** Failed statement batches roll back fully; successful batches commit. Plan 3.
- **[G17]** `#temp` names cannot shadow persistent tables. Plan 3.
- **[G18]** Batch variable defaults fold at declaration time and redeclaration is rejected. Plan 3.
- **[G19]** Spatial metadata failures are scoped to spatial queries. Plan 3.
- **[G20]** Mukey rewriting does not use another layer's geometry or an assumed SRID when required metadata is absent. Plan 3.
- **[G21]** Serializer XML element validation and result-set key numbering. Plan 2.
- **[G22]** Spatial constructor compatibility rewriting is AST-based after a narrow parser shim. Plan 3.
- **[G23]** Configured live SpatiaLite acceptance executes and passes; an unset extension is an explicit visible skip. Plan 4.
- **[G24]** Integration tests use unique shared-cache in-memory databases. Plan 3.
- **[C1]** Review cleanup: dead code, duplication, error classification, security denylist, and tracing. Plan 2.

## Blocked / Pruned

- **[G15]** Authoritative wire-parity comparison.
  - Status: Blocked / External Watch.
  - Reason: No authoritative capture/client comparison source is currently recorded as available.
- **[G27]** Rewrite non-compliant historical commit subjects containing wave/phase numbers.
  - Status: Pruned.
  - Reason: Rewriting feature-branch history is destructive; revisit only if the branch is unpushed and the user requests it.

## Ranking Rationale

- The completed pre-hosting foundation is **G32 → G29 → G30**, followed by the bounded package-compatibility work in **G11** (Plans 5 and 6).
- **G47** is active because the new scratch evaluation demonstrates a narrow T-SQL → DuckDB → SDA-serializer path, while exposing translation, batch response-shaping, and typed-value gaps. A repository-local spike is the next evidence-producing step; it is explicitly non-production and does not select controls. **G36** follows as a user decision gate before production work. The approved 20% aggregate performance target remains unverified: current DuckDB timings are not comparable end-to-end. **G31** remains sequenced because metadata correctness underpins multi-table spatial behavior. **G33-1** remains a bounded exploratory spike; completed **G33-2** did not authorize backend adoption. **G12** is deferred after Plan 9 found no safe geography-compatible implementation at the current seam; preserve explicit rejections absent new evidence. G28 remains deferred until its correctness prerequisites are complete.
- **G34** remains deferred despite the local opt-in soilDB harness: hosted validation requires an identified deployment target and authorization. **G35** is a maintenance decision, not a reason to remove the current supplementary harness now.

## Decisions (2026-11-18–19)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete obsolete executor paths and the `SpatialCapability.geometry` field in favor of `geometry_metadata()`. Done.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering.
- **D5 (G35):** Keep `tests/soildb/` as optional supplementary coverage for now. Reassess after comparing its real-client behaviors with captured fixtures and deterministic replay; it is not a default test dependency.
- **D6 (G36):** DuckDB is an opt-in alternative query backend; SQLite remains default and supported. The first release targets the existing documented SQL contract, preserving explicit rejection where spatial semantics are unsupported. The DuckDB path must not require SpatiaLite.
- **D7 (G36/G41):** Qualify Linux x86_64 first; no cross-platform release promise is implied.
- **D8 (G39):** Mirror the SQLite security policy: deny persistent writes, arbitrary file access, and user-controlled extension loading while preserving request-local temporary-table behavior. Do not use an OS sandbox.
- **D9 (G42):** The production value sought is performance. Establish a baseline with the existing committed contract-suite workloads first, then obtain approval for a numeric performance target before backend implementation.
- **D10 (G41/G43):** Provision DuckDB Spatial during deployment from the official source; pin and verify the exact artifact/version. Requests do not use the network. If provisioning/verification fails, retain non-spatial capability and return a clear error for spatial requests; never fall back to SpatiaLite on DuckDB.
- **D11 (G38):** Plan around the pinned DuckDB Rust crate. This records a planning assumption only; adding the repository dependency still requires explicit implementation authorization.
- **D12 (G47, 2026-11-20):** User authorized the pinned `duckdb = "=1.20000.2"` crate and lockfile update as a dev-dependency for the test-only, non-production T-SQL → DuckDB → SDA-response spike. No DuckDB code is included in normal binaries; runtime selection, public routes, production controls, spatial support, and production enablement remain out of scope.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
