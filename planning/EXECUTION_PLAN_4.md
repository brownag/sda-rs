# Execution Plan 4 — G1 End-to-End MVP Acceptance

## Goal & Scope

### Goal

Move toward G1 by completing the explicitly scoped MVP acceptance evidence: deterministic router-level coverage for supported non-spatial and spatial SQL, seeded fixtures for the WGS84 mukey UDF and AOI path, and a configured SpatiaLite run that is proven to execute rather than skip. This plan validates and closes gaps in the existing implementation; it does not broaden the supported SQL surface.

### In scope

- G6: End-to-end execution of variable folding, CLR spatial rewriting, and the WGS84 mukey intersection UDF against seeded spatial fixtures.
- G7: Deterministic in-process integration acceptance for request encodings, supported T-SQL constructs, temp tables, spatial capability behavior, and exact serializer structure.
- G23: Require and prove live SpatiaLite execution in the acceptance environment.
- Fix implementation defects found by these acceptance tests only when they are necessary to meet the G1 contract; update task scope explicitly if a defect requires files outside the named task.

### Out of scope / deferred

- G29–G33 (spatial contract documentation, GeoPackageBinary decoder, per-table metadata resolution beyond the G1 acceptance cases, SQLite execution safeguards, and terrio evaluation) remain separate follow-on goals unless a concrete blocking defect is found; report such discoveries rather than silently expanding this plan.
- G28 native spatial acceleration, PostgreSQL/PostGIS, broader SDA UDFs, general APPLY rewrites, additional CLR methods, other SDA service modes, and live USDA parity.
- Expanding coverage to the wider benchmark/query set in `sda-rs-scoping.md`; acceptance is limited to the explicit G1 contract.
- Any remote Git operation.

### Verbatim constraints

- “Plan 3 is complete.”
- “All six waves passed serial full-suite verification: 52 unit and 5 integration tests passed, with SpatiaLite configured. cargo fmt --check and git diff --check passed.”
- “The work covered batch rollback (G16), temporary-table isolation (G17), variable declarations (G18), metadata error scoping (G19), AST-based constructor rewriting (G22), mukey fallback safety (G20), and shared-cache in-memory test databases (G24). Changes are committed locally on feature/sqlite-mvp; nothing was pushed.”
- “this seems good, let's bundle several of these together for a plan to get us towards G1.”
- User selected G6 + G7 + G23, the G1 contract only, up to two workers, required configured live-extension execution, and `feature/sqlite-mvp`.

## Reconnaissance log

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/sqlite-mvp`, HEAD `bc97ca5`. A staged user-owned modification exists in `planning/GOALS.md`; leave it untouched. Existing Plan 3 worktrees are retained; do not clean them up.
- Planning files through orchestration 3 exist. No repository-local `AGENTS.md`, `CONTRIBUTING.md`, or `docs/decisions/` were found. Planning artifacts are tracked by prior decisions and are committed separately from code.
- `cargo fmt --check` passes. `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passes: 52 library tests and 5 integration tests; the live test reports pass under this configured run.
- Integration tests use Axum `Router::oneshot` and unique shared-cache in-memory SQLite URIs in `tests/sda_mvp.rs`; no fixed TCP port is required. Full suites must run serially.
- Important anchors: `tests/sda_mvp.rs` has request/capability tests at `router_keeps_nonspatial_query_working_and_reports_spatial_unavailable`, mukey rewrite coverage at `mukey_rewrite_uses_mupolygon_metadata_not_first_geometry_row`, metadata rejection at `router_rejects_mukey_tvf_without_mupolygon_metadata_or_override`, and live execution at `spatial_tvf_executes_when_test_extension_is_configured`. `src/routes.rs` contains the in-process request tests. `src/transpiler/{mod,vars,spatial,temp_tables}.rs`, `src/serializer.rs`, and `src/spatial.rs` contain relevant implementation and unit coverage.
- `tests/sda_mvp.rs` live test currently returns early when `SDA_TEST_SPATIALITE` is unset. A configured acceptance command is therefore mandatory; workers must not treat a skipped live test as evidence.
- Existing G1 acceptance in `planning/GOALS.md` requires JSON and form POSTs, supported T-SQL (`TOP`, `ISNULL`, `IIF`, `STRING_AGG`, bracketed identifiers), `DECLARE @aoi` with `STGeomFromText`/`STIntersects`, mukey UDF contexts, `#temp` batches, exact `JSON+COLUMNNAME+METADATA`, and explicit unavailable-capability behavior when SpatiaLite is absent.
- No edits to `planning/GOALS.md` are in scope. No baseline failures observed.

## Dated decisions

- 2026-11-18 — Focus this plan on G6, G7, and G23 to produce G1 acceptance evidence; leave G29–G33 and G28 outside scope unless an acceptance blocker is discovered.
- 2026-11-18 — Limit integration coverage to the G1 contract; do not expand toward the full benchmark set.
- 2026-11-18 — Stay on `feature/sqlite-mvp`; allow at most two workers. Parallel tasks must have disjoint file scopes; run full suites serially at wave boundaries.
- 2026-11-18 — Live SpatiaLite must be configured and actually execute. An early-return skip does not pass G23.
- 2026-11-18 — Track planning artifacts and commit them separately after plan approval. Preserve the staged `planning/GOALS.md` change.
- 2026-11-18 — Use `openai/gpt-6-luna` for worker tasks and escalation, retaining wave-orchestration retry limits.

## DAG and wave table

Tasks are small testable deliverables. Waves group tasks by dependency and file conflict; a goal may span multiple tasks and waves.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 4.1.1, 4.1.2 | Add isolated request/encoding and serializer acceptance; separate test files allow parallel work. |
| 2 | 4.2.1, 4.2.2 | Add deterministic non-spatial SQL/batch and spatial rewrite acceptance; disjoint test files allow parallel work. |
| 3 | 4.3.1 | Exercise the spatial routes and mukey TVF against seeded live SpatiaLite data (G6). |
| 4 | 4.4.1 | Make the configured live-extension gate explicit and verify end-to-end suite evidence (G23/G1). |

Dependencies: `4.1.1 -> 4.2.1`; `4.1.2 -> 4.2.1`; `4.2.1 -> 4.3.1 -> 4.4.1`; `4.1.2 -> 4.4.1`. The G7 acceptance surface is assembled by tasks 4.1.1–4.2.2; G6 is completed by 4.2.2 and 4.3.1; G23 is completed by 4.4.1. G1 is the aggregate acceptance outcome, not a single task.

## Global hazards and execution rules

- Do not edit `planning/GOALS.md`; it contains staged user-owned changes.
- Keep the request-local SQLite connection, shared-cache in-memory database rule, transaction semantics, and `spawn_blocking` boundary unchanged.
- Use in-process router tests, unique shared-cache in-memory database names, and holder connections. Do not add fixed-port tests or temporary-file databases.
- A live-extension test may retain a documented conditional behavior for developer runs without the extension, but the acceptance command must set `SDA_TEST_SPATIALITE`; verification must demonstrate the test executed, not early-returned.
- Do not expand supported SQL behavior beyond G1. Record unexpected gaps under `## Discovered` and stop if fixing them requires out-of-scope files.
- Workers run only their targeted commands. The orchestrator runs `cargo fmt --check`, `git diff --check`, and the full configured test suite serially at each wave boundary.
- Use worktree isolation for any concurrent tasks (maximum two workers); no remote Git actions. Planning-only commits remain separate from implementation commits.

## Task specifications

### Task 4.1.1 — Cover request decoding and response routing

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/request_contract.rs`.
- **Can create:** `tests/request_contract.rs`.
- **Depends on:** none.
- **Spec:** Create focused in-process router tests for JSON and URL-encoded form requests, mixed-case request keys, query execution, and query-echo behavior. Reuse `sda_rs::app`, `Router::oneshot`, and unique shared-cache in-memory databases with a holder connection. Keep this file focused on request decoding/dispatch, not SQL feature coverage.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, Cargo manifests, serializer implementation.
- **Grep anchors:** `src/routes.rs` `app`, request parsing handler; existing `tests/sda_mvp.rs` `oneshot` pattern.
- **Acceptance:** `cargo test --test request_contract`; `cargo fmt --check`; `git diff --check`.

### Task 4.1.2 — Lock down exact metadata serialization contract

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/serializer_contract.rs`.
- **Can create:** `tests/serializer_contract.rs`.
- **Depends on:** none.
- **Spec:** Add exact serializer assertions for `JSON+COLUMNNAME+METADATA`: column-name row, metadata row, data rows, null handling, and top-level keys for multiple result sets, matching the existing documented contract. Keep this as serializer API coverage, not a router fixture.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, Cargo manifests.
- **Grep anchors:** `src/serializer.rs` `serialize`; serializer tests `names_metadata_multiple_sets_and_empty_sets`.
- **Acceptance:** `cargo test --test serializer_contract`; `cargo fmt --check`; `git diff --check`.

### Task 4.2.1 — Verify supported scalar SQL and batch behavior through the endpoint

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/sql_batch_contract.rs`.
- **Can create:** `tests/sql_batch_contract.rs`.
- **Depends on:** 4.1.1.
- **Spec:** Through the in-process router, test supported `TOP`, `ISNULL`, `IIF`, `STRING_AGG`, and bracketed identifiers against small seeded tables. Add one multi-statement `#temp` batch proving a later statement can read the created temporary table. Use a unique shared-cache in-memory URI and holder connection. Keep assertions on endpoint status and returned SDA JSON.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, request or serializer contract test files, Cargo manifests.
- **Grep anchors:** `src/transpiler/normalize.rs`; `src/transpiler/temp_tables.rs`; `src/routes.rs` `app`.
- **Acceptance:** `cargo test --test sql_batch_contract`; `cargo fmt --check`; `git diff --check`.

### Task 4.2.2 — Verify AOI variable and constructor lowering

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/aoi_contract.rs`.
- **Can create:** `tests/aoi_contract.rs`.
- **Depends on:** 4.1.1.
- **Spec:** Add a focused router integration test for `DECLARE @aoi`, `geometry::STGeomFromText`, and `.STIntersects` using a seeded spatial table. Assert behavior with the configured spatial extension and a structured unavailable-capability response without it. Keep the mukey TVF cases for task 4.3.1.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, other new contract test files, Cargo manifests.
- **Grep anchors:** `src/transpiler/vars.rs` `fold`; `src/transpiler/spatial.rs` constructor and method lowering; `src/routes.rs` `app`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test aoi_contract`; `cargo fmt --check`; `git diff --check`.

### Task 4.3.1 — Verify mukey TVF execution contexts on seeded fixtures

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/mukey_contract.rs`.
- **Can create:** `tests/mukey_contract.rs`.
- **Depends on:** 4.2.1, 4.2.2.
- **Spec:** Build a compact deterministic fixture with `mupolygon`, `mapunit`, and `gpkg_geometry_columns`. Verify `SDA_Get_Mukey_from_intersection_with_WktWgs84` executes in both `WHERE mukey IN (...)` and `FROM` table-valued contexts. Include intersecting, non-intersecting, and `areasymbol = 'US'` records to prove selection and STATSGO exclusion. Use the discovered geometry column/SRID and a unique shared-cache in-memory DB.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, other contract test files, Cargo manifests.
- **Grep anchors:** `src/transpiler/spatial.rs` `MukeyTarget`; existing `mukey_rewrite_uses_mupolygon_metadata_not_first_geometry_row`; existing live test fixture.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test mukey_contract`; `cargo fmt --check`; `git diff --check`.

### Task 4.4.1 — Enforce and record live SpatiaLite acceptance

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/sda_mvp.rs`.
- **Can create:** none.
- **Depends on:** 4.3.1.
- **Spec:** Replace silent early-return semantics for the required configured acceptance path with an explicit test contract: when `SDA_TEST_SPATIALITE` is set, extension initialization and at least one real spatial query must be asserted; when it is absent in ordinary developer runs, skip behavior may remain explicit and visible. Run the configured integration suite and confirm the live test is reported as executed. Do not duplicate the individual endpoint cases from tasks 4.1.1–4.3.1.
- **Do not touch:** `planning/GOALS.md`, production files, other contract test files, Cargo manifests.
- **Grep anchors:** `spatial_tvf_executes_when_test_extension_is_configured`; `SDA_TEST_SPATIALITE`; `LoadExtensionGuard`; `InitSpatialMetaData`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_mvp -- --nocapture`; `cargo fmt --check`; `git diff --check`. Orchestrator runs configured full suite serially.

## Wave-boundary verification (orchestrator)

- After each wave, run `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially.
- Before marking G23 done, inspect test output and confirm `spatial_tvf_executes_when_test_extension_is_configured` ran to completion with configured SpatiaLite.
- Final G1 evidence is the complete passing suite plus the mapped route and spatial cases above; this plan does not claim broader SDA parity.
