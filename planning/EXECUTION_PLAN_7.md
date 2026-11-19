# Execution Plan 7 — Spatial Metadata, Geography Compatibility, and Readiness

## Goal & Scope

### Goal

Complete the active spatial-metadata goal (G31), then mature the DuckDB GeoPackage candidate before investing in broader spatial methods. Add the requested bounded configuration work, compare the optional soilDB harness with captured fixture coverage, and investigate/implement the six named spatial methods only where existing dependencies can demonstrate SQL Server `geography` semantics. Preserve the SQLite/GeoPackage service contract and make unsupported behavior explicit.

### In scope

- G31: resolve spatial metadata per referenced relation/geometry column and explicitly reject missing or ambiguous metadata without changing non-spatial behavior or existing Mukey override rules.
- G8/G9: audit/document the existing CLI/environment configuration; make the request-body limit configurable by CLI and environment, with a 2 MiB default, a 64 MiB hard maximum, and rejection of zero/invalid/out-of-range values.
- G33-2: complete an isolated DuckDB/Spatial evaluation, including bounded read-only GeoPackage inspection, extension/build availability, CRS/schema/operation comparisons, resource measurements, and disposable-database execution-semantics probes. This is research only, not backend adoption.
- G35: compare the opt-in soilDB/Iowa harness with captured-query transpilation and deterministic replay; document unique coverage and a keep/remove recommendation. Do not remove the harness in this plan.
- G12: evaluate and support, where demonstrably feasible, the six listed instance methods (`STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, `STAsText`) under SQL Server `geography` semantics. Keep unsupported methods explicit; do not silently substitute planar behavior.
- Produce concise, tracked spike/coverage reports under `planning/reports/`.

### Out of scope / deferred

- G33-1 / terrio: skipped for now. Reconsider only after the DuckDB evaluation is mature enough to identify a concrete gap.
- DuckDB service/backend integration, dependency changes to this repository, migration away from SQLite, or supersession of ADR 0001.
- New production dependencies. If an operation cannot meet the selected geography contract with the existing stack, document it as unsupported rather than adding a crate.
- Hosting, deployment, or live production SDA trials (G34).
- Removing the soilDB integration harness, changing the SDA wire protocol, weakening security/rollback/resource controls, or changing default bind/database settings.
- Editing or staging the user's current planning changes or any external soilDB/GeoPackage data.

### Verbatim user direction

- “@plan-wave from GOALS.md circled goal and any other follow on or related or easy goals”
- “actually we should just skip terrio for now and make duckdb more mature first”

## Dated Decisions

- **2026-11-18 — Scope:** Include G31 and G12, plus the related/easy G8, G9, G35, and G33-2 follow-ups. G31 is first; DuckDB evaluation precedes G12.
- **2026-11-18 — G12 meaning:** Interpret G12 as SQL Server `geography`-semantic compatibility, not planar `geometry` semantics. Cover all six methods named in `GOALS.md`, using CLR instance-method syntax only. Support only behaviors the current stack can evidence; document gaps. No new dependencies and no silent planar fallback.
- **2026-11-18 — Body cap:** Add CLI `--body-limit` and environment `SDA_BODY_LIMIT`; preserve the 2 MiB default, allow configured values up to 64 MiB, and reject zero, invalid, and greater-than-64-MiB values.
- **2026-11-18 — DuckDB/terrio:** Run G33-2 before G12. Do not run the terrio spike in this plan; keep G33-1 deferred pending DuckDB results and a concrete reason to revisit it.
- **2026-11-18 — Harness:** Keep `tests/soildb/` as opt-in supplementary coverage. G35 records unique behaviors and a recommendation; no harness removal is authorized here.
- **2026-11-18 — Execution:** Reuse Plan 6's maximum of two concurrent workers, isolated worktrees for parallel tasks, and `openai/gpt-6-luna` for workers/escalation.
- **2026-11-18 — Acceptance:** Reuse Plan 6's gates: `cargo fmt --check`, `git diff --check`, and configured full Rust tests serially at each wave boundary; run `make test-soildb` for G35 and again at final acceptance. Run the DuckDB probe only in its isolated task.
- **2026-11-18 — Artifacts and branch:** User explicitly chose to track the plan files and commit them separately. Prepare planning artifacts on the current `main` branch only. Before implementation starts, obtain/confirm a non-default task branch and ensure existing user changes are safely available to execution worktrees. Plan approval alone does not authorize branch creation or implementation. No remote Git actions.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`, Git branch `main`, HEAD `6b6ec34`, two commits ahead of `origin/main`. Preserve existing staged/unstaged/untracked user changes, including `planning/GOALS.md`, staged-add/working-tree-delete changes for the older G33/G34 spike paths, and untracked replacement G33-1/G33-2 notes. Do not stage or alter them.
- Planning files are tracked; Plans 1–6 exist, so this is Plan 7. Existing task IDs use `N.W.T.todo.md`.
- `STATE_5.md` and `STATE_6.md` record G32, G29, G30, and the package-used bounded G11 work complete. G31 remains active. G12 remains a goal; G33-1/G33-2 source notes say runtime measurements are outstanding.
- Entry sizes: `src/spatial.rs` 547 lines; `src/transpiler/spatial.rs` 1,330; `src/config.rs` 124; `src/lib.rs` 35; `tests/sda_get_contract.rs` 505. Use grep anchors in the large spatial modules.
- Current spatial metadata is represented by `GeometryMetadata` and `SpatialCapability` in `src/spatial.rs`; `geometry_metadata_for_table()` returns the first case-insensitive match, and `is_geometry_column()` checks geometry-column names globally. Initialization reads `gpkg_geometry_columns`; the transpiler has per-TVF target metadata. G31 must test multi-table/alias handling and malformed or duplicate metadata, while preserving the explicit Mukey metadata/override behavior.
- Current spatial method support is narrower than G12: the transpiler handles `STGeomFromText` and `.STIntersects`; the spatial contract says arbitrary operand SRIDs are not reconciled. The contract must distinguish geography behavior from geometry behavior.
- Existing configuration uses Clap `env` annotations for backend URL, port, SpatiaLite, and geometry override; bind address is extracted separately. The body limit is currently `2 * 1024 * 1024` in `src/lib.rs`. README currently lists only a subset of configuration.
- Active ADRs 0001–0006 are accepted. Preserve SQLite/GeoPackage for the service, request-local blocking SQLite connections, optional SpatiaLite, metadata-based geometry resolution, batch rollback, result-set numbering, and shared-cache in-memory tests. No ADR supersession is authorized.
- Current green baseline:
  - `cargo test`: passed, 70 unit tests and 23 integration tests; no configured extension required.
  - `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed, 70 unit and 23 integration tests; configured spatial contracts executed.
  - `cargo fmt --check`: passed.
  - `cargo clippy --all-targets`: passed with the existing `needless_question_mark` warning in `src/executor/sqlite.rs`.
  - `git diff --check`: passed before Plan 7 files were drafted.
  - `make test-soildb`: not run in this planning session; prior `STATE_6.md` records a successful opt-in run.
- `Makefile` targets: `make test`, `make lint`, `make check`, and opt-in `make test-soildb`. SoilDB and the Iowa GeoPackage are external, read-only inputs. Prior harness rules require a temporary R library and loopback-owned server process.
- The untracked G33-2 source/API note describes a standalone pinned DuckDB probe. It is user-owned input and may not be present in isolated task worktrees; Task 7.2.1 is self-contained and must not rely on or edit that note. Keep all probe dependencies and scratch files outside this repository (scratch under `/tmp/opencode`). Never copy or modify `/data/SSURGO/output/iowa_ssurgo.gpkg`.

## Dependency Graph and Waves

All task specs target `/home/andrew/workspace/sda-rs`. Wave tasks have disjoint file scopes.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 7.1.1, 7.1.2 | Complete G31 metadata resolution and G8/G9 configuration work in separate files. |
| 2 | 7.2.1 | Mature G33-2 DuckDB evaluation before G12; isolated probe only. |
| 3 | 7.3.1 | Compare opt-in soilDB harness coverage with captured fixture tiers (G35). |
| 4 | 7.4.1 | Establish the evidence-based geography semantics/feasibility matrix for all six G12 methods. |
| 5 | 7.5.1 | Implement or explicitly classify the geography predicate/overlay methods: Contains, Within, Intersection. |
| 6 | 7.6.1 | Implement or explicitly classify geography measurement/text methods: Buffer, Area, AsText. |

Dependencies: 7.2.1 and 7.3.1 depend on the Wave 1 boundary. 7.4.1 depends on 7.2.1. 7.5.1 depends on 7.4.1. 7.6.1 depends on 7.5.1 because both G12 tasks touch the same transpiler, contract-test, and contract-document paths. The G35 and DuckDB tasks read the same large Iowa GeoPackage and are deliberately sequential.

At each wave boundary, the orchestrator runs `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially. The configured suite is a gate even for report-only waves; do not run shared Cargo suites concurrently.

## Global Hazards and Rules

- Preserve every pre-existing staged, unstaged, and untracked user change. Do not stage broad paths or use destructive Git commands.
- Existing G33-1/G33-2 spike notes are user-owned, currently untracked replacement drafts (with older path changes also in the index). Treat them as read-only inputs if available; do not rely on their presence in isolated worktrees or edit them.
- Do not edit `planning/GOALS.md` from worker tasks. The active G31/G12 direction and dated decisions in this plan govern this orchestration.
- Never alter, stage, copy, or write to the external soilDB/py-soildb repositories or the Iowa GeoPackage. Read the GeoPackage only; verify its hash/mtime around DuckDB reads.
- G31 must preserve `SDA_GEOM_COLUMN`, the Mukey `mupolygon` metadata requirement, and ordinary SQLite behavior without SpatiaLite. Do not weaken metadata validation or invent fallback geometry columns.
- Geography compatibility is not planar geometry compatibility. No method may silently call a planar operation and claim geography parity. If no supported implementation exists with current dependencies, record the evidence and stop that method's code work.
- Do not alter HTTP response shape, transaction rollback, request-local connection lifecycle, security/resource safeguards, or accepted ADRs.
- DuckDB is a candidate-only isolated probe. Do not edit `Cargo.toml`, `Cargo.lock`, service source, Makefile, or runtime configuration for DuckDB.
- G33-1/terrio is explicitly deferred. Do not install, prototype, benchmark, or add terrio as part of Plan 7.
- Body limit default remains 2 MiB. The configured maximum is 64 MiB; zero, invalid, and over-maximum configuration must fail clearly. Do not change the response shape for oversized requests.
- A worker may not implement before its micro-TODO is reviewed and approved. If a task's semantics or safe scope cannot be established, stop and report; discoveries wait for a plan amendment.
- No pushes, fetches, PRs, or remote comments.

## Task Specifications

### Task 7.1.1 — Resolve geometry metadata for referenced tables

- **ID:** 7.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/spatial.rs`, `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`.
- **Can create:** `tests/spatial_metadata_contract.rs`.
- **Depends on:** completed G29/G30 and Plan 5/G32; Plan 7 baseline.
- **Spec:** Complete G31. Resolve geometry metadata against the referenced relation/table and alias rather than a global geometry-column-name match. Reject missing metadata and ambiguous/malformed metadata for referenced spatial tables explicitly. Cover at least two joined geometry tables, aliases, repeated geometry-column names across relations, missing metadata, and duplicate/ambiguous metadata. Keep non-spatial queries usable without SpatiaLite, preserve the `mupolygon` TVF rule and `SDA_GEOM_COLUMN` override behavior, and document the verified contract. Do not infer schema or SRID from unrelated metadata rows.
- **Do not touch:** `Cargo.toml`, `Cargo.lock`, executor security/rollback, serializer, route/wire behavior, user data, `planning/GOALS.md`, or external repositories.
- **Grep anchors:** `SpatialCapability`, `geometry_metadata_for_table`, `is_geometry_column`, `initialize` in `src/spatial.rs`; `rewrite`, `rewrite_node`, `expand_source`, and metadata lookup in `src/transpiler/spatial.rs`; existing cases in `tests/mukey_contract.rs` and `tests/sda_mvp.rs`; ADR 0003.
- **Acceptance:** `cargo test --test spatial_metadata_contract`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test spatial_metadata_contract`; `cargo test mukey`; `cargo fmt --check`; `git diff --check`. The contract test must show new resolution/rejection cases, configured spatial queries, and existing Mukey/non-spatial behavior.

### Task 7.1.2 — Audit configuration and configure request-body limit

- **ID:** 7.1.2
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/config.rs`, `src/main.rs`, `src/lib.rs`, `README.md`.
- **Can create:** `tests/body_limit_contract.rs`.
- **Depends on:** Plan 7 baseline.
- **Spec:** Verify the existing backend URL, port, SpatiaLite, geometry-column, and bind-address CLI/environment contract against the README; correct any documented mismatch within scope. Add `--body-limit` and `SDA_BODY_LIMIT`, preserving a 2 MiB default and enforcing a 64 MiB maximum. Reject zero, malformed, and over-maximum values during startup configuration. Preserve the public `Config` struct-literal interface: follow the existing `extract_bind_address` pattern for this separate runtime option, pass the accepted limit from `main` to an app constructor, and keep `app(config)` at the unchanged default for existing callers. Test default, CLI and environment configuration, exact-limit acceptance, and over-limit rejection without changing the HTTP response shape.
- **Do not touch:** response serialization, route behavior other than configured request-body rejection, `AGENTS.md`, dependencies, or default bind/database settings.
- **Grep anchors:** `Config` and Clap attributes in `src/config.rs`; `extract_bind_address` and `main` in `src/main.rs`; `app` and `RequestBodyLimitLayer` in `src/lib.rs`; configuration table in `README.md`; existing `tests/request_contract.rs`.
- **Acceptance:** `cargo test config::tests`; `cargo test --test body_limit_contract`; `cargo fmt --check`; `git diff --check`.

### Task 7.2.1 — Mature the isolated DuckDB GeoPackage evaluation

- **ID:** 7.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spike-results.md`.
- **Can create:** scratch project and probe only under `/tmp/opencode/sda-rs-plan7-duckdb/`.
- **Depends on:** completed Wave 1.
- **Spec:** Complete G33-2 as an isolated evaluation, not a service change. In a standalone Rust project under `/tmp/opencode`, pin the DuckDB client version and verify Spatial extension acquisition/loading (including offline/reproducible availability). Open the Iowa GeoPackage read-only and query bounded samples only; compare selected layer/schema/field types/nulls, geometry types, CRS/SRID, stable `IA001`/mukey `1857266` results, selected spatial operations, and EPSG:4326↔3857 transforms against read-only SQLite/SpatiaLite observations. Record elapsed time, peak RSS, and relevant DuckDB memory/thread settings. In a disposable in-memory DuckDB database, probe batch rollback, interruption/deadline, concurrency, and output type conversion. Do not attempt an unbounded layer read; stop if extension/build or bounded-read safety cannot be established and report the blocker. Verify source GeoPackage hash and modification time before/after. The report must distinguish observed behavior from docs and state explicitly that DuckDB is not adopted.
- **Do not touch:** service `Cargo.toml`/`Cargo.lock`, Rust sources, Makefile, runtime config, other repositories, existing user-owned spike notes, or the source GeoPackage.
- **Grep anchors:** `planning/STATE_6.md` Iowa path and stable identifiers; `docs/spatial-sql-contract.md`; `docs/decisions/0001-sqlite-geopackage-mvp-backend.md`; DuckDB official Rust, Spatial/GDAL, profiling, transactions, and concurrency documentation.
- **Acceptance:** record and run exact standalone build/probe commands (including `cargo run --manifest-path /tmp/opencode/sda-rs-plan7-duckdb/Cargo.toml --release -- ...`) in the result report; verify GeoPackage hash/mtime unchanged; `git diff --check`. If a safe runtime probe is blocked, a sourced report of the exact blocker and completed checks is acceptable; do not claim compatibility or performance.

### Task 7.3.1 — Compare soilDB harness and captured-fixture coverage

- **ID:** 7.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `planning/reports/G35-soildb-coverage.md`.
- **Can create:** none.
- **Depends on:** completed Wave 1 and DuckDB task 7.2.1.
- **Spec:** Compare opt-in soilDB/Iowa tests with `tests/fixtures/soildb/`, `tests/soildb/`, and deterministic fixture replay. Produce a behavior matrix identifying coverage unique to the real client/harness, coverage already represented by captured SQL/replay, environmental cost, and maintenance risk. Run `make test-soildb` and record the result and input paths when prerequisites are available. Recommend retain or reconsider based on evidence. Preserve the existing decision to keep this as optional supplementary coverage; do not delete or weaken the harness or fixture tests.
- **Do not touch:** source, fixture data, soilDB/py-soildb repositories, Iowa GeoPackage, Makefile, or test behavior.
- **Grep anchors:** `tests/soildb/README.md`, `tests/soildb/testthat.R`, `tests/soildb/testthat/`; `tests/fixtures/soildb/README.md`; `tests/soildb_query_fixtures.rs`; `tests/soildb_fixture_transpile.rs`; `scripts/test-soildb.sh`.
- **Acceptance:** `make test-soildb`; inspect the named tests and record exact coverage mapping; `git diff --check`. If prerequisites are unavailable, report the diagnostic and retain the harness without claiming a pass.

### Task 7.4.1 — Establish geography-method semantics and feasibility

- **ID:** 7.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `planning/reports/G12-geography-method-feasibility.md`.
- **Can create:** disposable probes only under `/tmp/opencode/sda-rs-plan7-geography/`.
- **Depends on:** completed DuckDB task 7.2.1.
- **Spec:** For each instance method `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`, compare authoritative SQL Server `geography` semantics with the existing SpatiaLite capabilities and the project's accepted input/data contract. Record argument/result types, SRID restrictions, geographic units, NULL/invalid behavior, operation availability, and concrete test cases. Probe the configured SpatiaLite build where useful. Classify each method as implementable with existing dependencies, conditionally implementable with a precise bounded contract, or unsupported; do not equate planar geometry operations with geography. No production code changes or new dependencies in this task.
- **Do not touch:** service code, dependencies, existing spatial contract, source data, user-owned spike notes, or external repositories.
- **Grep anchors:** `docs/spatial-sql-contract.md`; `src/transpiler/spatial.rs` `rewrite_node` and `STIntersects`; `src/spatial.rs` `initialize`; Microsoft Learn geography/geometry method references; SpatiaLite 5.1 SQL function reference; accepted ADRs 0001 and 0003.
- **Acceptance:** complete six-row evidence matrix in the report with cited primary sources and reproducible probe commands/results; `git diff --check`. If the supported SRID/input domain cannot be bounded, stop and ask rather than inventing a contract.

### Task 7.5.1 — Support feasible geography predicates and intersection

- **ID:** 7.5.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`.
- **Can create:** `tests/spatial_methods_contract.rs`.
- **Depends on:** task 7.4.1.
- **Spec:** Implement only the evidence-supported `STContains`, `STWithin`, and `STIntersection` behaviors from the approved feasibility matrix, using CLR instance-method syntax. Validate arity, geometry operands, SRID compatibility, NULL behavior, and output type. Add configured end-to-end tests for positive/negative predicates and intersection output (including a supported text conversion path if required). For any infeasible method, leave it explicitly rejected/unsupported and document the evidence; never emit a planar substitute labeled geography-compatible.
- **Do not touch:** free-function spellings, `Cargo.toml`/`Cargo.lock`, unrelated SDA TVFs, rollback/security, wire response shape, external data, or other methods assigned to 7.6.1.
- **Grep anchors:** `Expression::MethodCall`, `rewrite_node`, and current `STIntersects` handling in `src/transpiler/spatial.rs`; test patterns in `tests/aoi_contract.rs` and `tests/mukey_contract.rs`; feasibility matrix from 7.4.1.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test spatial_methods_contract`; `cargo fmt --check`; `git diff --check`. The tests must initialize configured SpatiaLite and execute spatial queries. If the acceptance contract requires a dependency or unsupported semantics, stop and report.

### Task 7.6.1 — Support feasible geography buffer, area, and text methods

- **ID:** 7.6.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`.
- **Can create:** none; continue `tests/spatial_methods_contract.rs` from 7.5.1.
- **Depends on:** task 7.5.1.
- **Spec:** Implement only the evidence-supported `STBuffer`, `STArea`, and `STAsText` behaviors from the approved feasibility matrix, using CLR instance-method syntax. Geography buffer distance must use the contracted geographic units (normally meters); area must use the contracted geodetic units (normally square meters); text output must be valid WKT for the returned spatial value. Test supported SRIDs, NULL/empty values, output types, and explicit unsupported/invalid-SRID handling. Do not invoke planar `ST_Buffer`/`ST_Area` and claim geography semantics. Update the spatial contract with only behaviors exercised by tests. Mark infeasible methods unsupported with cited evidence.
- **Do not touch:** free-function spellings, dependencies, HTTP response shape, unrelated TVFs, or accepted ADRs.
- **Grep anchors:** feasibility matrix from 7.4.1; `rewrite_node` and current AsText SQL generation in `src/transpiler/spatial.rs`; `tests/spatial_methods_contract.rs`; Microsoft Learn and SpatiaLite primary references cited by 7.4.1.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test spatial_methods_contract`; `cargo fmt --check`; `git diff --check`. Do not mark G12 complete unless all six methods have either passed their geography-compatibility tests or are explicitly recorded as unsupported with reasons, as authorized by the dated G12 decision.

## Final Acceptance Gate (Orchestrator-Owned)

After Wave 6, run serially and record in `STATE_7.md`: `cargo fmt --check`; `git diff --check`; default `cargo test`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; and `make test-soildb`. Confirm the opt-in harness owns/stops its server, external soilDB and Iowa inputs remain unchanged, and all G12 method outcomes are explicitly documented. The orchestrator, not task workers, updates `STATE_7.md`; reconcile goal status only after reviewing and preserving the user's current `planning/GOALS.md` edits.
