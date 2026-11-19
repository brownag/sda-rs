# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-19
Source State: STATE_6.md (Plans 5 and 6 complete); PR #1 merged as `62a6b98`

## Circled Goal (Active)

- **[G31]** Spatial metadata is resolved for every referenced table and geometry column, and missing or ambiguous metadata is rejected explicitly.
  - Depends on: G29, G30.
  - Why now: Plans 5 and 6 established execution safeguards, the spatial contract, geometry decoding, and client-used SDA_Get coverage. A complete metadata audit is the remaining foundation for reliable broader spatial behavior.
  - Exit criteria: Tests demonstrate metadata resolution for each referenced table/geometry column and explicit rejection of missing or ambiguous metadata; existing Mukey validation and non-spatial queries remain intact.
  - First action: Audit `src/spatial.rs` and spatial-transpiler call sites against multi-table geometry queries; identify uncovered cases before planning implementation.

## Sequenced Goals (Dependency DAG)

- **[G33-1]** terrio's GeoPackage I/O, geometry decoding/operations, CRS transformations, and memory profile are evaluated on bounded representative fixtures without changing the service backend.
  - Depends on: G29, G30.
  - Exit criteria: A sourced spike note records API capabilities and gaps; a small, read-only fixture probe records geometry/CRS comparisons and elapsed time/peak RSS, or explicitly documents a blocker preventing safe measurement. No whole-layer load is attempted without a safety estimate; no SQL-engine adoption is inferred.
  - First action: Build an isolated probe using terrio's `sqlite` feature and a bounded sample, leaving Cargo dependencies and the Iowa source GeoPackage unchanged.
- **[G33-2]** DuckDB's Rust integration and Spatial extension are evaluated as an isolated candidate for reading the Iowa GeoPackage, with compatibility, deployment, and resource risks made explicit.
  - Depends on: G29, G30, G32.
  - Exit criteria: A sourced spike note records the Rust/Spatial/GDAL API surface and contract gaps; an isolated probe verifies extension availability and bounded GeoPackage reads, compares representative schema/geometry/CRS results with the current backend, and records elapsed time, peak RSS, and resource settings—or documents a concrete blocker before unsafe work. No dependency, wire-contract, or backend change is made by the spike.
  - First action: Create a disposable Rust probe pinned to a DuckDB version; verify whether Spatial can be loaded reproducibly and offline before querying the read-only fixture.
- **[G12]** The broader CLR spatial method set—including `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—is supported with documented unit/SRID semantics.
  - Depends on: G29; AOI construction/intersection MVP behavior is covered by Plan 4.

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
- **G31** is active because metadata correctness underpins spatial queries across layers and is a prerequisite for further spatial readiness work. **G33-1** and **G33-2** are bounded exploratory spikes, not backend-change authorization; G28 remains deferred until its correctness prerequisites are complete.
- **G34** remains deferred despite the local opt-in soilDB harness: hosted validation requires an identified deployment target and authorization. **G35** is a maintenance decision, not a reason to remove the current supplementary harness now.

## Decisions (2026-11-18–19)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete obsolete executor paths and the `SpatialCapability.geometry` field in favor of `geometry_metadata()`. Done.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering.
- **D5 (G35):** Keep `tests/soildb/` as optional supplementary coverage for now. Reassess after comparing its real-client behaviors with captured fixtures and deterministic replay; it is not a default test dependency.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
