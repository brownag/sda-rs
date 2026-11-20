# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-20
Source State: STATE_13.md (DuckDB qualification plan through G42)

## Circled Goal (Active)

- **[G37]** DuckDB's production query surface has an explicit, tested support/rejection matrix, including T-SQL translation and evaluated transformations for the six G12 geography methods.
  - Why now: G36 is resolved: DuckDB is an opt-in alternative, SQLite remains default, Linux x86_64 is the first target, and current controls/deployment expectations stand. G37 determines what behavior subsequent lifecycle, security, compatibility, and runtime work must preserve.
  - Depends on: G36, G47, G29.
  - Exit criteria: Included SQL constructs and spatial methods have defined semantics, units/CRS requirements, NULL/error behavior, and acceptance tests. Evaluate transformations for all six geography methods; implement only paths whose tests establish the required SQL Server geography semantics. Explicitly reject methods for which no compatible path is established. `IIF`, batch result shaping, and DuckDB value mapping are resolved or explicitly rejected for production; Plan 11 test-only adaptations are not production fixes.
  - First action: Build a method-by-method contract matrix from SQL Server documentation, Plan 8 observations, and Plan 11 behavior; identify transformations that merit focused tests before deciding support.

## Sequenced Goals (Dependency DAG)
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
  - Depends on: G36, G48.
  - Exit criteria: Linux x86_64 deployment provisions pinned DuckDB and official Spatial artifacts from approved sources, verifies versions and hashes, and records license, provenance, upgrade, and request-time offline behavior.
- **[G42]** DuckDB's performance and resource behavior are measured on representative SDA workloads, with evidence-based candidate resource limits.
  - Depends on: G38, G39, G40, G41.
  - Exit criteria: Reproducible workload tests compare DuckDB with the current backend; peak resource use and failures are recorded; candidate resource limits are recommended but not finalized without approval. Report the 20% aggregate objective and whether any workload regresses by over 10%; neither performance result is a hard qualification blocker.
- **[G43]** Users can explicitly select DuckDB while SQLite remains the default with a defined failure and fallback policy.
  - Depends on: G39, G40, G41, G42, G48, G49.
  - Exit criteria: Configuration and rollout tests prove default SQLite behavior is unchanged, opt-in selection is visible, non-spatial DuckDB queries remain available when Spatial is unavailable, spatial requests fail explicitly, and there is no silent SpatiaLite fallback.
- **[G44]** CI validates the DuckDB backend's supported behavior, security boundary, resource limits, and deployment artifacts.
  - Depends on: G37, G39, G40, G41, G42, G43.
  - Exit criteria: Required CI jobs run deterministic contract/security tests and platform-specific artifact checks without accessing user data or external sources.
- **[G45]** DuckDB backend setup, supported behavior, limitations, operations, and recovery are documented for users and maintainers.
  - Depends on: G43, G44.
  - Exit criteria: Documentation matches tested configuration and deployment behavior, identifies unsupported SQL/geography semantics, and explains rollback to SQLite.
## Independent Backlog

- **[G33-1]** terrio's GeoPackage I/O, geometry decoding/operations, CRS transformations, and memory profile are evaluated on bounded representative fixtures without changing the service backend.
  - Depends on: G29, G30.
  - Exit criteria: A sourced spike note records API capabilities and gaps; a small, read-only fixture probe records geometry/CRS comparisons and elapsed time/peak RSS, or explicitly documents a blocker preventing safe measurement. No whole-layer load is attempted without a safety estimate; no SQL-engine adoption is inferred.
  - First action: Build an isolated probe using terrio's `sqlite` feature and a bounded sample, leaving Cargo dependencies and the Iowa source GeoPackage unchanged.

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
- **[G31]** Spatial metadata resolves by referenced relation and rejects missing, duplicate, or malformed metadata while preserving non-spatial and Mukey behavior. Plan 7 Task 7.1.1; integrated commit `257202a`; metadata contract tests passed in default/configured SpatiaLite runs and Mukey tests passed (4 unit, 1 contract, 2 integration). This goal outcome is complete; Plan 7's overall final gate is separately recorded complete in its reconciled state.
- **[G8]** CLI and environment configuration matches the documented operational contract. Plan 7 Task 7.1.2; integrated commit `e93eed0`; `cargo test config::tests` (3) and `cargo test --test body_limit_contract` (1) passed.
- **[G9]** Request body limit is configurable with a 2 MiB default and tested boundary behavior. Plan 7 Task 7.1.2; integrated commit `e93eed0`; `cargo test --test body_limit_contract` passed (1 test). This goal outcome is complete; Plan 7's overall final gate is separately recorded complete in its reconciled state.
- **[G11]** The package-used bounded `SDA_Get_*` subset is implemented and tested. Plan 6 inventoried all 34 catalog functions and active soilDB/py-soildb call sites; active-client variants and additional feasible functions are documented and contract-tested where dataset prerequisites permit. See `docs/sda-get-support.md` and `STATE_6.md`.
- **[G33-2]** DuckDB Rust integration, Spatial, and community `geography` are evaluated as isolated candidates for G12 and bounded GeoPackage access. Plan 8; `STATE_8.md` and `planning/reports/G33-2-duckdb-spatial-followup.md`.
- **[G47]** A repository-local, test-only vertical slice translates representative SDA T-SQL to DuckDB SQL, executes supported cases, and compares SDA JSON with SQLite; unsupported types are explicitly rejected and observed IIF/CTAS gaps are recorded. Plan 11 complete; `planning/STATE_11.md` and `planning/reports/duckdb-test-only-spike.md`. This is evidence only, not production enablement.
- **[G48]** DuckDB is available through an explicit optional Cargo dependency/build feature while default builds remain DuckDB-free; the feature alone does not select or enable a runtime backend. Plan 12; `planning/STATE_12.md` records default and opt-in build/dependency checks.
- **[G49]** The Plan 11 DuckDB pipeline integration test is explicitly opt-in and remains test-only. Plan 12; `planning/STATE_12.md` records the default suite excluding the target and 12 passing opt-in tests (including `--all-features`).
- **[G36]** DuckDB's production acceptance contract is resolved: it is an opt-in alternative, SQLite remains the default, Linux x86_64 is the initial target, and current security/deployment expectations stand. The recorded 20% performance target is an objective, not a hard qualification/release blocker; report measured results without making success a prerequisite.
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
- **G31, G8, and G9** are complete based on Plan 7 reports and implementation history. These goal outcomes are distinct from Plan 7's orchestrator-owned final gate, whose full checks are evidenced and recorded complete in reconciled `STATE_7.md`. **G36** and **G47–G49** are completed decision/build/evidence prerequisites, not production support. The 20% aggregate performance target is an objective, not a hard blocker; record comparable end-to-end results when available. **G37** is active because the supported query contract constrains lifecycle, security, compatibility, and runtime work. **G33-1** is independent and lower leverage; completed **G33-2** did not authorize backend adoption. **G12** remains broadly deferred, while Plan 13 evaluates only bounded methods that can meet SQL Server geography semantics.
- **G34** remains deferred despite the local opt-in soilDB harness: hosted validation requires an identified deployment target and authorization. **G35** is a maintenance decision, not a reason to remove the current supplementary harness now.

## Decisions (2026-11-18–20)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete obsolete executor paths and the `SpatialCapability.geometry` field in favor of `geometry_metadata()`. Done.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering.
- **D5 (G35):** Keep `tests/soildb/` as optional supplementary coverage for now. Reassess after comparing its real-client behaviors with captured fixtures and deterministic replay; it is not a default test dependency.
- **D6 (G36):** DuckDB is an opt-in alternative query backend; SQLite remains default and supported. The first release targets the existing documented SQL contract, preserving explicit rejection where spatial semantics are unsupported. The DuckDB path must not require SpatiaLite.
- **D7 (G36/G41):** Qualify Linux x86_64 first; no cross-platform release promise is implied.
- **D8 (G39):** Mirror the SQLite security policy: deny persistent writes, arbitrary file access, and user-controlled extension loading while preserving request-local temporary-table behavior. Do not use an OS sandbox.
- **D9 (G42, clarified 2026-11-20):** The 20% aggregate median-latency improvement target approved in Plan 10 is an objective, not a hard qualification or release blocker. Measure and report the result; missing the target alone does not block further qualification. Numeric resource limits remain separate and require measurement plus explicit approval.
- **D10 (G41/G43):** Provision DuckDB Spatial during deployment from the official source; pin and verify the exact artifact/version. Requests do not use the network. If provisioning/verification fails, retain non-spatial capability and return a clear error for spatial requests; never fall back to SpatiaLite on DuckDB.
- **D11 (G38):** Plan around the pinned DuckDB Rust crate. This records a planning assumption only; adding the repository dependency still requires explicit implementation authorization.
- **D12 (G47, 2026-11-20):** User authorized the pinned `duckdb = "=1.20000.2"` crate and lockfile update as a dev-dependency for the test-only, non-production T-SQL → DuckDB → SDA-response spike. No DuckDB code is included in normal binaries; runtime selection, public routes, production controls, spatial support, and production enablement remain out of scope.
- **D13 (G48, 2026-11-20):** User selected an optional compile-time Cargo dependency/build option as the next goal. This does not authorize runtime backend selection or production enablement; keep those as later goals and approval gates.
- **D14 (G37, 2026-11-20):** Explore mathematically valid transformations for the six SQL Server geography methods. A bounded, runtime-enforced subset is acceptable only where compatibility tests establish the SQL Server contract; reject inputs outside the proven domain and do not label approximations or planar results as compatible.
- **D15 (G37, 2026-11-20):** Candidate dependencies may be researched, but no dependency may be added without separate explicit user approval.
- **D16 (G42/G43, 2026-11-20):** Plan 13 may measure and recommend candidate DuckDB resource limits, but final numeric limits and runtime backend enablement remain gated for explicit approval. The 20% latency objective is not a blocker.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
