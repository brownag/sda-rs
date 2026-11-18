# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-18
Source State: STATE_4.md (Plan 4 complete); STATE_3.md (Plan 3 complete)

## Circled Goal (Active)

- **[G32]** SQLite execution enforces an explicit, tested security boundary for submitted SQL, extension loading, registered UDFs, and resource use.
  - Why now: G1's SQLite/GeoPackage MVP acceptance is complete. User-submitted SQL is the service boundary; explicit safeguards reduce system-wide risk before broadening query or spatial capability.
  - Exit criteria: Document the authorizer allow/deny policy, trusted optional extension loading, conservative UDF flags, and resource limits; test each policy while preserving G1 behavior. Initial limits: 2 MiB SQL text, 100,000 rows/request, 64 MiB serialized output, 60 seconds execution, and 64 MiB SQLite string/BLOB length.
  - First action: Finalize Plan 5 for the authorizer policy, extension/UDF tests, and resource caps; then execute its approved waves.

## Sequenced Goals (Dependency DAG)

- **[G29]** The supported spatial SQL contract documents method coverage, NULL/error behavior, CRS assumptions, and intentional semantic differences.
  - Depends on: completed G1; establishes the compatibility boundary for spatial expansion.
- **[G30]** GeoPackageBinary decoding and validation cover headers, envelope flags, empty geometries, malformed blobs, and WKB extraction.
  - Depends on: completed G1.
- **[G31]** Spatial metadata is resolved for every referenced table and geometry column, and missing or ambiguous metadata is rejected explicitly.
  - Depends on: completed G20; needed by G28.
- **[G33]** A focused spike evaluates terrio's geometry parsing, operations, CRS handling, and memory behavior against a representative GeoPackage fixture.
  - Depends on: G29, G30.
  - Exit criteria: Record supported APIs and gaps; do not adopt terrio as a SQL engine without evidence.
- **[G12]** The broader CLR spatial method set—including `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—is supported with documented unit/SRID semantics.
  - Depends on: G29; AOI construction/intersection MVP behavior is covered by Plan 4.
- **[G11]** The documented `SDA_Get_*` function catalog is supported, including table-valued macros and table-input variants.
  - Depends on: completed G1 and mukey intersection coverage; define a bounded function subset before implementation.

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

- **[G28]** Native SQLite spatial acceleration.
  - Depends on: G29, G30, G31, G33.
  - Trigger: Reconsider only after those prerequisites validate the geometry representation, predicate semantics, exact evaluation, and R-tree assumptions.
  - Exit criteria: R-tree is used only for conservative candidate filtering followed by exact predicates; indexed and non-indexed results match, including when no usable index exists.

## Completed

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

## Decisions (2026-11-18)

- **D1 (G24):** Integration tests use shared-cache in-memory DBs.
- **D2 (C1):** Delete obsolete executor paths and the `SpatialCapability.geometry` field in favor of `geometry_metadata()`. Done.
- **D3 (G16):** A failed multi-statement batch rolls back all of its writes.
- **D4 (G21):** The first non-empty result set is `Table`; later non-empty sets are `Table1`, `Table2`, and so on. Empty sets are skipped when numbering.

Cleanup plan: `planning/EXECUTION_PLAN_2.md`.
