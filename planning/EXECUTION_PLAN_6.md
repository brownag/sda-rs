# Execution Plan 6 — Spatial Readiness and SDA Client Compatibility

## Goal & Scope

### Goal

After Plan 5 completes G32, establish a documented spatial SQL contract, validate GeoPackageBinary decoding, support as much of the SDA `SDA_Get_*` catalog as the SQLite/GeoPackage backend can safely implement, and exercise the server through an opt-in integration harness using the R soilDB package and the local Iowa GeoPackage. Prioritize the smaller set of functions actively used by soilDB and py-soildb, while recording the full catalog and unsupported cases.

### In scope

- G29: document supported spatial methods, NULL/error behavior, CRS assumptions, and intentional differences from SDA.
- G30: implement and test GeoPackageBinary header, envelope, empty geometry, malformed input, and WKB decoding/validation.
- G11: inventory the official `SDA_Get_*` surface and package call sites; implement package-used functions first, then all additional functions that are feasible within this backend and available data model; document unsupported cases with reasons.
- Add an opt-in `make test-soildb` integration harness in this repository. It starts/stops the server, installs the local soilDB source into a temporary R library, sets soilDB's configurable SDA endpoint option, and runs testthat-style `SDA_query`, `SDA_spatialQuery`, and selected `get_SDA_*` cases against the Iowa GeoPackage.
- Use `SDA_TEST_GEOPACKAGE` with local default `/data/SSURGO/output/iowa_ssurgo.gpkg`; access the database read-only and never copy it into the repository.
- Use the local soilDB checkout at `~/workspace/soilDB` for integration tests. Its `feat/custom-sda1` working tree currently contains user changes, including the endpoint option in `R/SDA_query.R`; use but do not edit, stage, commit, stash, or clean that checkout. The `createSSURGO` changes are unrelated to this plan.
- Inspect current py-soildb sources at `~/workspace/py-soildb` when inventorying package usage; record package versions and source revisions in the compatibility matrix.

### Out of scope / deferred

- G32 implementation: this is already specified in Plan 5 and must be completed before any Plan 6 task starts.
- Hosting/deployment or pointing production SDA applications at the service; this remains gated on G32, G29, and G30, with package compatibility as a further readiness check.
- Changes to the soilDB or py-soildb repositories, including committing or otherwise resolving their current branch/worktree state.
- Changing the SDA wire protocol, replacing SQLite/GeoPackage, adding dependencies, or changing accepted architecture decisions.
- Implementing SDA functions that cannot be represented safely or meaningfully on the SQLite/GeoPackage backend; record them and why they are unsupported.
- Running the soilDB integration harness as part of ordinary `cargo test` or CI by default. It is opt-in and requires local R packages, SpatiaLite, the source checkout, and the large Iowa dataset.

### Verbatim constraints and user decisions

- “we want to support all the SDA_Get functions we can but the short set are the ones that are actuvely used”
- “as part of our testing, I'd like to set up an integration test using the R soilDB package in `~/workspace/soilDB` SDA_query/SDA_spatialQuery/get_SDA* style tests.”
- “we will need to create our own fixtures that point at the iowa geopackage `/data/SSURGO/output/iowa_ssurgo.gpkg`”
- The integration test starts and stops the service itself.
- The Iowa GeoPackage is read-only, referenced at its existing path, and configurable through an environment-variable override.
- The current soilDB checkout's endpoint-option patch is user-owned and is used by the harness without modification.
- Plan 5/G32 runs first. Plan 6 may be drafted now but cannot execute until G32 is complete.
- Use no more than two workers, isolated worktrees for parallel work, and `openai/gpt-6-luna` for workers/escalation.
- Planning artifacts are tracked and committed separately. No remote Git operations.
- At each wave boundary run `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially. Run the Iowa/soilDB integration target at its milestone and final acceptance, not every wave.

## Dated decisions

- 2026-11-18 — Plan 5 remains the implementation plan for G32. Plan 6 is gated on its completion; do not duplicate or silently replace G32 work.
- 2026-11-18 — Complete the pre-hosting path in order: G32, then G29, then G30. Keep hosting and live SDA-app trials deferred until all three pass.
- 2026-11-18 — G11 means support all catalog functions that can be implemented safely; inventory the full surface and prioritize the functions actively used by soilDB and py-soildb.
- 2026-11-18 — Use the local soilDB source tree's endpoint option, but do not alter its current dirty state. Add the client harness only in sda-rs.
- 2026-11-18 — Keep the Iowa GeoPackage outside the repository, open it read-only, and allow its path to be overridden with `SDA_TEST_GEOPACKAGE`.
- 2026-11-18 — The R integration run is an opt-in Make target; it starts/stops the server and runs only at the specified milestone/final gate.
- 2026-11-18 — Current workspace package references: soilDB 2.9.4 (`feat/custom-sda1`, dirty user-owned tree); py-soildb 0.9.0 (`v090`, clean tree). Inventory must record exact revisions and distinguish package source from released versions where they differ.
- 2026-11-18 — Preserve accepted ADRs 0001–0006. No supersession is authorized.
- 2026-11-18 — Plan 6 artifacts are tracked and committed separately from implementation.

## Reconnaissance log

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/sda-readiness`, clean after planning commit `4198dc8`. No remote action is authorized.
- Plan 5 (`EXECUTION_PLAN_5.md`) already specifies G32. `STATE_5.md` still marks all three waves pending; Plan 6 cannot start until Plan 5 completion is verified and recorded.
- Configured Rust baseline passed: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` (52 unit + 11 integration tests); `cargo fmt --check` and `git diff --check` passed.
- R environment: R 4.6.2; soilDB 2.9.4, testthat 3.3.2, sf 1.1.2, wk 0.9.5, DBI 1.3.0, and RSQLite 3.53.3 are installed.
- The soilDB source tree is `/home/andrew/workspace/soilDB`, branch `feat/custom-sda1`, HEAD `1eaa0717`; pre-existing user changes are `.Rbuildignore`, `R/SDA_query.R`, `R/createSSURGO.R`, and untracked `createSSURGO_patch.diff`. The `R/SDA_query.R` diff adds option `soilDB.sda.tabular.post.rest.url`. Never alter or stage these changes.
- py-soildb source tree is `/home/andrew/workspace/py-soildb`, branch `v090`, HEAD `f070c0a`, clean. Its package metadata reports version 0.9.0.
- `/data/SSURGO/output/iowa_ssurgo.gpkg` exists and is approximately 6.4 GB. Read-only metadata inspection found `mupolygon.geom`, `sapolygon.geom`, and `featpoint.geom`, all EPSG:4326. `IA001` exists; `mupolygon` includes mukey `1857266`.
- Current SDA TVF rewrite anchor is `src/transpiler/spatial.rs` (`MUKEY_FUNCTION`, `rewrite`, `rewrite_node`); geometry capability/metadata anchor is `src/spatial.rs`; configured spatial contract examples are in `tests/aoi_contract.rs` and `tests/mukey_contract.rs`.
- `src/transpiler/spatial.rs` is 503 lines; use grep anchors, not whole-file reads. `src/spatial.rs` is 333 lines. The repo is a Rust Cargo project with a Makefile; there is no existing R integration harness.
- soilDB's existing query/client patterns are in `tests/testthat/test-SDA_query.R`, `R/SDA-spatial.R`, and `R/get_SDA_muaggatt.R`. In the checked-out soilDB source, `SDA_spatialQuery()` uses mukey and sapolygonkey intersection TVFs; the local Iowa fixture can exercise these with real data.
- `planning/sda-rs-scoping.md` documents the broader TVF catalog but notes that historical package-source inspection was incomplete. Plan 6 must ground its matrix in the named current package sources and authoritative catalog evidence.
- Accepted ADRs 0001–0006 are active. Preserve SQLite/GeoPackage, request-local blocking connections, optional SpatiaLite, rollback, response numbering, and shared-cache in-memory test semantics.

## Dependency graph and waves

External prerequisite: completed Plan 5/G32.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 6.1.1, 6.1.2 | Define the spatial compatibility contract and inventory the complete SDA_Get catalog/package call sites. |
| 2 | 6.2.1 | Decode and validate GeoPackageBinary geometry values. |
| 3 | 6.3.1 | Support the SDA_Get intersection functions used by soilDB's spatial client path first. |
| 4 | 6.4.1–6.4.3 (blocked/diagnostic); 6.4.4; 6.4.5 | Add area-symbol and soil-survey-polygon intersection/key TVFs; correct the Web-Mercator mismatch and restore Mukey metadata-SRID behavior. |
| 5 | 6.5.1 | Add mapunit-polygon intersection/key TVFs. |
| 6 | 6.6.1 blocked; 6.6.2 diagnostic outcome; 6.6.3 fixture correction; 6.6.4 formatting and boundary verification | Add other feasible feature-layer intersection/key TVFs, correct Web-Mercator fixtures, and verify the preserved implementation. |
| 7 | 6.7.1–6.7.4 complete; boundary review pending | Implement feasible scalar key-to-WKT TVFs; trace and fix the SQLite `MAX` syntax error. |
| 8 | 6.8.1 | Complete (`f5e9541`); feasible table-input WKT TVFs and input-table macros are verified. |
| 9 | 6.9.1 | Ready for task planning; add and run the opt-in soilDB/Iowa integration harness. |

Dependencies: `6.2.1` depends on `6.1.1`; `6.3.1` depends on `6.1.2` and `6.2.1`; `6.4.1` depends on `6.3.1`; recovery task `6.4.2` also depends on `6.3.1` and takes over the preserved, uncommitted in-scope work from blocked task 6.4.1; diagnostic task `6.4.3` depends on 6.3.1 and the failure evidence from 6.4.1/6.4.2, and is diagnostic-only; implementation task `6.4.4` depends on 6.3.1 and the preserved partial work/diagnostic findings from 6.4.1–6.4.3; regression correction `6.4.5` depends on 6.4.4 and the Wave 4 boundary-test evidence. `6.5.1` depends on completed Wave 4 via 6.4.5; `6.6.1` depends on 6.5.1 and is blocked by its failing configured acceptance; diagnosis/fix task `6.6.2` depends on 6.6.1 and its failure evidence. Fixture correction `6.6.3` depends on the preserved 6.6.1 diff and the recorded 6.6.2 diagnostic evidence, not resolution of the historical WGS84 failure. `6.6.4` depends on the preserved 6.6.1 diff and the fixture changes and check results from 6.6.3. `6.7.1` depends on completed Wave 6 via 6.6.4; `6.7.2` depends on the preserved 6.7.1 diff and its compile failure evidence. `6.7.3` depends on the preserved Wave 7 diff and recorded 6.7.2 HTTP 500, not a successful 6.7.2 run. `6.7.4` depends on the preserved Wave 7 diff and 6.7.3 evidence. `6.8.1` and `6.9.1` remain blocked pending verified Wave 7 implementation and its boundary gate; neither diagnostic alone satisfies those gates. Every task depends on completed Plan 5/G32. Wave 1 tasks have disjoint output files; all same-module SDA_Get implementation tasks are intentionally sequential.

## Global hazards and execution rules

- Do not start any Plan 6 task until Plan 5/G32 is complete and its state reports the required security, resource, and configured-suite acceptance evidence.
- Do not edit Plan 5 artifacts or rewrite its history as part of Plan 6.
- Do not touch or stage any files in the soilDB or py-soildb repositories. Their source trees are inputs only; preserve soilDB's dirty working tree.
- Never copy, update, or write to `/data/SSURGO/output/iowa_ssurgo.gpkg`; use read-only inspection and requests only.
- The integration harness must use a temporary R library and temporary logs, start the service bound to loopback, wait for readiness, and stop its child process even on test failure. Do not use a fixed shared port; allow explicit overrides.
- Keep the R harness opt-in and separate from `cargo test`; fail with actionable diagnostics when its explicitly requested prerequisites are missing rather than reporting a false pass.
- Test fixtures should use stable Iowa data such as area symbol `IA001` and mukey `1857266`; use exact expected values only after verifying them with read-only queries.
- Keep spatial extension handling optional for ordinary SQLite. Configured live-SpatiaLite acceptance must prove the extension initialized and a real spatial query executed.
- Preserve the exact SDA response contract, `#temp` behavior, transaction rollback, request-local connections, and all accepted ADRs.
- Do not add crates, change dependency versions, alter the HTTP wire protocol, or deploy/host the service in this plan.
- At each wave boundary, run the configured Rust suite serially. Run `make test-soildb` after its harness task and again at final acceptance; report external package versions and the GeoPackage path.

## Task specifications

### Task 6.1.1 — Document the supported spatial SQL contract

- **ID:** 6.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `docs/spatial-sql-contract.md`.
- **Can create:** the named contract document only.
- **Depends on:** completed Plan 5/G32.
- **Spec:** Document current supported spatial constructors/methods and SDA query patterns; specify input/output SRID expectations, units where relevant, NULL behavior, invalid geometry/error behavior, optional SpatiaLite behavior, GeoPackageBinary assumptions, and intentional differences or unsupported cases. Cross-reference the scoped code/test anchors. Do not promise behavior that the implementation and tests do not demonstrate.
- **Do not touch:** source code, `planning/GOALS.md`, Plan 5 files, or other repositories.
- **Grep anchors:** `src/transpiler/spatial.rs` `rewrite`, `rewrite_node`; `src/spatial.rs` `geometry_metadata_for_table`, `prepare_spatial_capability`; `tests/aoi_contract.rs`; `tests/mukey_contract.rs`; `planning/sda-rs-scoping.md` spatial method table.
- **Acceptance:** document review against the named implementation/tests; `cargo fmt --check`; `git diff --check`.

### Task 6.1.2 — Inventory SDA_Get catalog and client usage

- **ID:** 6.1.2
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `docs/sda-get-support.md`.
- **Can create:** the named compatibility matrix only.
- **Depends on:** completed Plan 5/G32.
- **Spec:** Inventory the authoritative USDA-NRCS SDA_Get catalog and current soilDB/py-soildb package call sites. Compare current published release metadata/source with the local worktrees; record package versions, source revisions, source URLs/references, each function's arguments/return columns, WGS84/Web-Mercator variants, table-input requirements, required data tables/geometry layers, and whether it is actively called. Classify every function as already supported, implementable in this backend, or unsupported with an evidence-based reason. Mark soilDB's and py-soildb's actively used functions as first-priority implementation targets. Do not treat every function as feasible merely because it appears in the catalog.
- **Do not touch:** package source trees, production Rust code, Plan 5 files, or any dependency.
- **Grep anchors:** `/home/andrew/workspace/soilDB/R/SDA-spatial.R`, `/home/andrew/workspace/soilDB/R/SDA_query.R`, `/home/andrew/workspace/soilDB/vignettes/sda.Rmd`, `/home/andrew/workspace/py-soildb/src/soildb/spatial.py`, `planning/sda-rs-scoping.md` “Complete SDA_Get Table-Valued Function Surface”.
- **Acceptance:** matrix accounts for every catalog entry and cites package source locations for active usage; verify package versions/revisions and no external repo changed; `git diff --check`.

### Task 6.2.1 — Decode and validate GeoPackageBinary

- **ID:** 6.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/gpkg_geometry.rs`, `src/lib.rs`.
- **Can create:** unit tests within `src/gpkg_geometry.rs`.
- **Depends on:** 6.1.1.
- **Spec:** Add a small, testable decoder for GeoPackageBinary headers and WKB payload boundaries. Cover magic/version, flags and byte order, optional envelope lengths, empty geometries, truncated/invalid headers, and WKB extraction. Reject malformed values with explicit errors; do not silently reinterpret arbitrary blobs. Keep the parser independent of SpatiaLite and do not change query semantics unless the named contract requires it.
- **Do not touch:** `src/spatial.rs`, `src/serializer.rs`, the Iowa GeoPackage, or wire response code.
- **Grep anchors:** `src/lib.rs` module declarations; `src/spatial.rs` geometry metadata helpers; `planning/sda-rs-scoping.md` GeoPackage geometry references.
- **Acceptance:** `cargo test gpkg_geometry`; `cargo fmt --check`; `git diff --check`.

### Task 6.3.1 — Support active client intersection TVFs

- **ID:** 6.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `docs/sda-get-support.md`.
- **Can create:** `tests/sda_get_contract.rs`.
- **Depends on:** 6.1.2, 6.2.1.
- **Spec:** Implement and test all safely feasible WGS84/Web-Mercator variants of the spatial-intersection SDA_Get functions actively called by soilDB and py-soildb (including the mukey and sapolygonkey paths where catalog evidence confirms usage). Reuse the existing AST rewrite and spatial preparation architecture; validate returned key column names/types and real intersections on deterministic in-memory fixtures. Preserve existing `SDA_Get_Mukey_from_intersection_with_WktWgs84` behavior. Update the support matrix only if implementation evidence changes its classifications.
- **Do not touch:** external package repositories, `planning/GOALS.md`, the GeoPackage decoder, serializer, or wire protocol.
- **Grep anchors:** `src/transpiler/spatial.rs` `MUKEY_FUNCTION`, `rewrite`, `rewrite_node`; `src/spatial.rs` `SPATIAL_FUNCTIONS`, `prepare_spatial_capability`; `tests/mukey_contract.rs`; package call sites from task 6.1.2.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`; configured test must execute real spatial queries; `cargo fmt --check`; `git diff --check`.

### Task 6.4.1 — Add area-symbol and survey-polygon intersection TVFs

- **ID:** 6.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `docs/sda-get-support.md`.
- **Can create:** none.
- **Must edit:** `tests/sda_get_contract.rs`.
- **Depends on:** 6.3.1.
- **Spec:** Implement catalog-classified-feasible `areasymbol` intersection functions and the remaining `sapolygonkey` status-map intersection functions, including WGS84/Web-Mercator forms where supported by the source data model. The basic WGS84/Web-Mercator `sapolygonkey` intersection variants are owned by completed task 6.3.1 and must not be repeated. Use the task 6.1.2 matrix as the bounded list; every feasible entry added must have an execution test and exact result-column assertion. If a supposedly feasible entry requires a schema/semantic capability not present, stop and report rather than widening scope silently.
- **Do not touch:** external package repositories, geometry decoder, serializer, or route/wire behavior.
- **Grep anchors:** task 6.1.2 support matrix; `src/transpiler/spatial.rs` `rewrite_node`; `src/spatial.rs` spatial function dispatch; `tests/sda_get_contract.rs`.
- **Acceptance:** configured `cargo test --test sda_get_contract` passed; configured full `cargo test` passed (64 unit tests and all integration/doc targets); `cargo fmt --check` and `git diff --check` passed. `cargo clippy --all-targets` exited successfully with two warnings. Implementation commit `f5e9541`.

### Task 6.4.2 — Recover and complete Wave 4 intersections

- **ID:** 6.4.2
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `docs/sda-get-support.md`, `tests/sda_get_contract.rs`.
- **Can create:** none.
- **Depends on:** 6.3.1; takes over the preserved uncommitted task-scoped partial changes from blocked task 6.4.1.
- **Spec:** With user authorization, diagnose why the configured contract test previously returned `{}` for the first Mukey WGS84 request even though a separate diagnostic path returned the expected row. Preserve the current diff and capture the actual response/results for the same request path; establish a falsifiable cause before changing behavior. Correct the confirmed Web-Mercator mixed-SRID predicate by comparing both operands in a common CRS. Complete the remaining two area-symbol and two sapolygonkey-with-sastatusmap intersection variants, both WGS84 and Web-Mercator, without duplicating 6.3.1's four basic Mukey/Sapolygonkey variants. Keep exact result columns and test status-map filtering/DISTINCT with unmatched intersecting and duplicate matched rows. If the WGS84 discrepancy remains unexplained or a safe in-scope fix is not established, stop and report; do not guess.
- **Do not touch:** external package repositories, planning/GOALS.md, GeoPackage decoder, serializer, route/wire behavior, dependencies, or any files outside the four named paths. Preserve all current uncommitted changes.
- **Grep anchors:** `tests/sda_get_contract.rs` `query`, test loop and first Mukey case; `src/transpiler/spatial.rs` `intersection_spec`, `expand_source`; `src/spatial.rs` `requires_spatial`; `src/serializer.rs` `serialize_json` for why empty result sets appear as `{}`; baseline implementation in commit `4d4aa0a`.
- **Acceptance:** after evidence-led fix, at most one `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract -- --nocapture`; require all eight exact JSON cases to pass with configured SpatiaLite and real spatial queries. If it fails or root cause remains unresolved, stop immediately. If it passes, run `cargo fmt --check` and `git diff --check`.

### Task 6.4.3 — Diagnose inconsistent WGS84 test observations

- **ID:** 6.4.3
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/sda_get_contract.rs` (temporary, narrowly tagged diagnostic output only; remove it before stopping).
- **Can create:** none.
- **Read-only references:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `src/routes.rs`, `src/serializer.rs`, and `src/executor/sqlite.rs`.
- **Depends on:** 6.3.1 and recorded failure evidence from blocked tasks 6.4.1/6.4.2.
- **Spec:** Diagnose why the same first Mukey WGS84 request returned `{}` in three acceptance runs but returned the expected row in a later diagnostic request. Add temporary test-only reporting of SQL, HTTP status, and exact response body for each query in the current contract test, with no expectation changes. Run the diagnostic once serially with output visible. Compare the exact first request path and test fixture with prior observations. Do not alter production source, query behavior, expected WGS84 results, external repositories, or user data. Remove temporary reporting before stopping. If no specific cause is established from this run, stop and report; do not repeat the diagnostic.
- **Do not touch:** any production code, docs, fixtures beyond temporary reporting, planning/GOALS.md, dependencies, external repositories, or the Iowa GeoPackage.
- **Acceptance:** one diagnostic-only command: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract -- --nocapture --test-threads=1`; record all emitted SQL/status/body pairs. This is not an implementation acceptance pass. Remove temporary instrumentation and run `git diff --check`. No retry.

### Task 6.4.4 — Complete Wave 4 intersection TVFs

- **ID:** 6.4.4
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `docs/sda-get-support.md`, `tests/sda_get_contract.rs`.
- **Can create:** none; take over and preserve the existing uncommitted Wave 4 changes.
- **Depends on:** 6.3.1 and the recorded evidence from 6.4.1–6.4.3.
- **Spec:** With user authorization, finish the two `Areasymbol` intersection TVFs and two `Sapolygonkey ... with_sastatusmap` intersection TVFs in WGS84/Web-Mercator. Fix the confirmed Web-Mercator mixed-SRID predicate so both operands use a common CRS. Keep the first WGS84 mukey assertion unchanged and passing; prior `{}` observations remain unexplained and must not be represented as resolved. Preserve the four basic Mukey/Sapolygonkey variants from 6.3.1. Add an intersecting sapolygon omitted from `sastatusmap` and duplicate matched status rows, proving status-map filtering and distinct key output. Require exact result columns, key types, and real configured spatial execution for all eight contract cases.
- **Do not touch:** external repositories, planning/GOALS.md, geometry decoder, serializer, route/wire behavior, dependencies, Iowa data, or files outside the four in-scope paths.
- **Grep anchors:** `src/transpiler/spatial.rs` `intersection_spec`, `expand_source`; `src/spatial.rs` `requires_spatial`; `tests/sda_get_contract.rs` query list and status-map fixture; `docs/sda-get-support.md` corresponding four catalog rows; Web-Mercator mismatch evidence in `planning/STATE_6.md`.
- **Acceptance:** after the focused fix, at most one `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract -- --nocapture`; require all eight exact cases to pass with configured SpatiaLite and real queries. If it fails, stop and report. If it passes, run `cargo fmt --check` and `git diff --check`; orchestrator runs the full configured suite at the Wave 4 boundary.

### Task 6.4.5 — Restore Mukey metadata-SRID behavior

- **ID:** 6.4.5
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`.
- **Can create:** none.
- **Depends on:** 6.4.4; regression found by the configured Wave 4 full-suite check.
- **Spec:** Restore the pre-existing `SDA_Get_Mukey_from_intersection_with_WktWgs84` behavior: transform the source geometry to its `mupolygon` metadata SRID while keeping the WKT constructor at SRID 4326. Do not regress the new Web-Mercator variants: both geometry and WKT must use a common 3857 CRS there. Preserve optional metadata/override validation and all four basic Mukey/Sapolygonkey TVFs. Keep or strengthen the existing unit assertion for metadata SRID; do not weaken it.
- **Do not touch:** external repositories, planning/GOALS.md, spatial decoder, other spatial function support, serializer, routes, dependencies, or files outside `src/transpiler/spatial.rs`.
- **Grep anchors:** `src/transpiler/spatial.rs` `MUKEY_FUNCTION`, `expand_source`, and `mukey_override_column_uses_mupolygon_metadata_srid`; failing output recorded in `planning/STATE_6.md`.
- **Acceptance:** `cargo test mukey_override_column_uses_mupolygon_metadata_srid`; then `cargo fmt --check` and `git diff --check`. The orchestrator reruns the configured full suite at the Wave 4 boundary.

### Task 6.5.1 — Add mapunit-polygon intersection/key TVFs

- **ID:** 6.5.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `docs/sda-get-support.md`.
- **Can create:** none.
- **Must edit:** `tests/sda_get_contract.rs`.
- **Depends on:** 6.4.1.
- **Spec:** Implement catalog-classified-feasible mapunit-polygon intersection/key functions, including WGS84/Web-Mercator variants. Resolve the `mupolygon` geometry column and SRID through existing metadata; preserve the existing `mukey` intersection behavior and test exact return columns, key coercion, NULL/no-match behavior, and real intersections. Use the task 6.1.2 support matrix as the boundary.
- **Do not touch:** external package repositories, serializer, or wire protocol.
- **Grep anchors:** task 6.1.2 support matrix; `src/spatial.rs` `geometry_metadata_for_table`; `src/transpiler/spatial.rs` `rewrite_node`; `tests/sda_get_contract.rs`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`; `cargo fmt --check`; `git diff --check`.

### Task 6.6.1 — Add other feasible feature-layer intersection TVFs

- **ID:** 6.6.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, `docs/sda-get-support.md`.
- **Can create:** none.
- **Depends on:** 6.5.1.
- **Spec:** Implement catalog-classified-feasible intersection/key functions for `mupoint`, `muline`, `featpoint`, and `featline`, including WGS84/Web-Mercator variants. The Iowa fixture only has `featpoint` among these layers; use deterministic in-memory fixtures for generic behavior. Keep functions whose required geometry layer is absent from a particular GeoPackage classified as conditionally supported, not as an unconditional error or fabricated result. Update unsupported classifications only with evidence.
- **Do not touch:** external package repositories, geometry decoder, serializer, or route/wire behavior.
- **Grep anchors:** task 6.1.2 support matrix; `src/transpiler/spatial.rs` `rewrite_node`; `src/spatial.rs` spatial function dispatch; `tests/sda_get_contract.rs`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`; `cargo fmt --check`; `git diff --check`.

### Task 6.6.2 — Diagnose and correct feature-layer intersection failure

- **ID:** 6.6.2
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, `docs/sda-get-support.md`.
- **Can create:** none; preserve and take over the uncommitted 6.6.1 diff.
- **Depends on:** 6.5.1 and failure evidence from 6.6.1.
- **Spec:** Diagnose why `SDA_Get_Mupointkey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')` returned `{}` for a fixture point at `(0.5, 0.5)`, then correct the evidence-established cause. Trace request status/body, generated SQL, chosen geometry column/metadata/SRID, decoded geometry, and predicate result. Preserve existing Mukey, Mupolygonkey, and Wave 4 behavior. Record accurately that Iowa has `featpoint` but lacks catalog-required `featpointkey`; do not claim Iowa support for that variant. Retain only catalog-feasible additions, conditionally supported when required relation, key, geometry metadata, and spatial extension are present. If no cause is established or correction requires out-of-scope changes, stop and report.
- **Do not touch:** external repositories, geometry decoder, serializer, route/wire behavior, dependencies, Iowa user data, or files outside the four 6.6.1 scope paths.
- **Grep anchors:** failing assertion in `tests/sda_get_contract.rs`; `IntersectionTarget`, `intersection_spec`, `expand_source` in `src/transpiler/spatial.rs`; table metadata lookup in `src/spatial.rs`; recorded failure in `planning/STATE_6.md`.
- **Acceptance:** after establishing and correcting the cause, run at most once: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`. On failure stop without retry. If it passes, run `cargo fmt --check` and `git diff --check`, report causal evidence and scoped diff before committing.

### Task 6.6.3 — Correct Wave 6 Web-Mercator fixture coordinates

- **ID:** 6.6.3
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `tests/sda_get_contract.rs` only, after micro-plan approval.
- **Can create:** none for implementation.
- **Depends on:** preserved 6.6.1 diff and 6.6.2 diagnostic evidence.
- **Spec:** Correct the positive Wave 6 `WktWm` queries only. Use the observed full-precision EPSG:3857 point for Mupoint and Featpoint; use a transverse crossing for Muline and Featline rather than assuming the projected midpoint lies exactly on the transformed segment. Preserve all exact expected keys/types and unchanged WGS84, earlier-wave, and no-match cases. The prior Mupoint WGS84 failure did not reproduce in the diagnostic and remains unexplained. Preserve the Iowa caveat: `featpoint` exists but lacks `featpointkey`.
- **Do not touch:** production code, support matrix, stored fixture geometries/metadata, dependencies, external repositories, or Iowa data.
- **Grep anchors:** `active_intersection_tvfs_return_expected_keys_for_both_crs` in `tests/sda_get_contract.rs`; 6.6.2 evidence in `planning/STATE_6.md`; `planning/tasks/6.6.3.todo.md`.
- **Acceptance:** after edits, run at most once: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`. On failure stop and report without retry. If it passes, run `cargo fmt --check` and `git diff --check`, then review the scoped delta. The orchestrator owns the separate Wave 6 boundary gate.

### Task 6.6.4 — Format the feature-layer intersection specs and verify Wave 6

- **ID:** 6.6.4
- **Repo:** `/home/andrew/workspace/sda-rs`, branch `feature/sda-readiness`.
- **Files in scope — must edit:** `src/transpiler/spatial.rs` only, after micro-plan approval.
- **Can create:** none for implementation.
- **Depends on:** preserved 6.6.1 implementation and 6.6.3 fixture correction/check results.
- **Spec:** Format only the three reported `intersection_spec` entries using the crate's Rust 2021 conventions. Preserve the full existing four-file diff and all fixture assertions; do not apply repository-wide formatting writes. Keep the historical WGS84 observation unexplained and the Iowa `featpointkey` caveat intact.
- **Do not touch:** other implementation files, dependencies, protocol, external repositories, or Iowa data.
- **Grep anchors:** `intersection_spec` in `src/transpiler/spatial.rs`; 6.6.3 check outcome in `planning/STATE_6.md`; `planning/tasks/6.6.4.todo.md`.
- **Acceptance:** run serially `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` once, with configured spatial tests executing. Stop and report at the first failure without retry or implementation commit. On pass, report results and commit only the verified Wave 6 implementation after reviewing its full diff; record acceptance separately in planning.

### Task 6.7.1 — Implement scalar key-to-WKT TVFs

- **ID:** 6.7.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, `docs/sda-get-support.md`.
- **Can create:** none.
- **Depends on:** completed Wave 6 via 6.6.4.
- **Spec:** Implement catalog-classified-feasible scalar SDA_Get functions that return geometry WKT by a single key (for example, area-symbol or mukey inputs), including documented WGS84/Web-Mercator variants. Resolve geometry columns/SRIDs through existing metadata, produce the catalog's exact aliases, and test NULL/no-match behavior and CRS transformations. Use the support matrix as the task boundary.
- **Do not touch:** external package repositories, serializer, or wire protocol.
- **Grep anchors:** task 6.1.2 support matrix; `src/spatial.rs` `geometry_metadata_for_table`; `src/transpiler/spatial.rs` `rewrite_node`; `tests/sda_get_contract.rs`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`; `cargo fmt --check`; `git diff --check`.

### Task 6.7.2 — Fix the scalar key-to-WKT TVF compile blocker

- **ID:** 6.7.2
- **Repo:** `/home/andrew/workspace/sda-rs`, branch `feature/sda-readiness`.
- **Implementation scope:** take over and preserve `docs/sda-get-support.md`, `src/spatial.rs`, `src/transpiler/spatial.rs`, and `tests/sda_get_contract.rs`; correct the compiler blocker in `src/transpiler/spatial.rs` only. No new implementation files.
- **Depends on:** completed Wave 6 via 6.6.4 and the preserved 6.7.1 diff and compile failure evidence, not a successful 6.7.1 run.
- **Spec:** Fix unreachable tuple expressions in the scalar source matcher and carry the original `Option<Identifier>` through to the derived-table alias, preserving quoting and the default alias. Keep the eight one-key `FROM` TVFs WKT-only with exact catalog aliases; the separate `mukey` in the soilDB example is caller-supplied. Preserve exact WKT/CRS and NULL/no-match expectations and prior intersection behavior. Do not widen scope to table-input TVFs.
- **Grep anchors:** `expand_source`, `expand_intersection_source`, `expand_scalar_wkt_source`; `planning/tasks/6.7.1.todo.md` and the 6.7.1 report in `planning/STATE_6.md`.
- **Acceptance:** after micro-plan approval and correction, optionally run `cargo check --test sda_get_contract` once and run `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` once with real spatial assertions. Stop on first failure without retry or implementation commit. On pass, run `cargo fmt --check` and `git diff --check`, then review and locally commit only the verified four-file Wave 7 implementation if both checks pass. Keep planning artifacts separate. The orchestrator owns the Wave 7 boundary gate.

### Task 6.7.3 — Identify the Wave 7 contract request returning HTTP 500

- **ID:** 6.7.3
- **Repo:** `/home/andrew/workspace/sda-rs`, branch `feature/sda-readiness`.
- **Depends on:** the preserved four-file Wave 7 diff and the HTTP 500 recorded in 6.7.2.
- **Scope:** after approval, temporarily instrument only `tests/sda_get_contract.rs::query` with `[DEBUG-673]` reporting. Capture the exact SQL, HTTP status, and full response body for every reached request before its existing assertions; retain the HTTP 200 assertion and return the same body to callers. Do not change requests, fixtures, expectations, production code, or other files in the Wave 7 diff.
- **Diagnostic:** run exactly once, serially: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract -- --nocapture --test-threads=1`. Report the first failing request, its status and full body, the command outcome, and any cases not reached. This is not acceptance; do not retry, including if the 500 does not reproduce.
- **Cleanup and stop:** remove only the tagged instrumentation and its temporary helper changes, verify all four files match their pre-instrumentation contents, and run `git diff --check`. If the response identifies a specific fixable cause, report it and stop for a separately approved behavior-fix task. If the cause remains unclear, report the evidence and stop. No implementation commit or remote actions. Waves 7–9 remain blocked.

### Task 6.7.4 — Diagnose `MAX` in generated scalar mukey WKT SQL

- **ID:** 6.7.4
- **Repo:** `/home/andrew/workspace/sda-rs`, branch `feature/sda-readiness`.
- **Depends on:** the preserved four-file Wave 7 diff and 6.7.3 request 25 evidence. Code tracing found that T-SQL parsing converts `TEXT` casts into invalid SQLite `VARCHAR(MAX)`.
- **Fixture correction:** keep `InitSpatialMetaData(1)` and create the fixture's GeoPackage `gpkg_geometry_columns` table once; in this environment SpatiaLite initialization does not create that table.
- **Fix and acceptance:** remove the T-SQL `TEXT` cast from scalar Mukey equality, accept `Expression::Null`, and verify literal substitution for `'inside'`, `11`, and `'11'`. Preserve all eight WKT-only aliases, exact WKT/CRS, NULL/no-match and prior intersection behavior. Configured contract and full suite passed; `cargo fmt --check` and `git diff --check` passed; `cargo clippy --all-targets` exited successfully with two warnings. Implementation commit: `23dea36`. Wave 7 boundary review remains required before Wave 8.
- **Do not touch:** dependencies, route/wire behavior, shared normalization/executor code, table-input TVFs, Iowa/external data or repositories, or remotes. No new implementation files.

### Task 6.8.1 — Implement feasible table-input WKT TVFs

- **ID:** 6.8.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `src/transpiler/temp_tables.rs`, `tests/sda_get_contract.rs`, `docs/sda-get-support.md`.
- **Can create:** none.
- **Depends on:** verified Wave 7 implementation and a passing Wave 7 boundary gate. The Wave 7 diagnostics alone do not unblock this task.
- **Spec:** Implement the feasible table-input SDA_Get variants and the `DeclareIntTable`, `DeclareVarcharTable`, and `DeclareVarchar255Table` forms required by supported catalog entries. Preserve request-local temporary-table isolation and batch rollback; test declaration, population, joins, aliases, WGS84/Web-Mercator variants, and malformed or absent inputs. Follow the task 6.1.2 matrix and document any remaining unsupported variants there.
- **Do not touch:** executor transaction policy, external package repositories, serializer, or wire protocol.
- **Grep anchors:** task 6.1.2 support matrix; `src/transpiler/temp_tables.rs` `rewrite`, `#temp` mapping; `src/transpiler/spatial.rs` `rewrite_node`; `tests/sql_batch_contract.rs`; `tests/sda_get_contract.rs`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract`; `cargo fmt --check`; `git diff --check`.

### Task 6.9.1 — Add the opt-in soilDB/Iowa integration harness

- **ID:** 6.9.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `Makefile`.
- **Can create:** `scripts/test-soildb.sh`, `tests/soildb/testthat.R`, `tests/soildb/testthat/test-sda-rs.R`, `tests/soildb/README.md`.
- **Depends on:** 6.3.1, 6.4.1, 6.5.1, completed Wave 6 via 6.6.4, verified Wave 7 implementation and its passing boundary gate, and 6.8.1.
- **Spec:** Add opt-in `make test-soildb` that builds/starts the server on loopback, waits for readiness, and always stops the child process. Use configurable `SOILDB_PATH`, `SDA_TEST_GEOPACKAGE`, `SDA_TEST_SPATIALITE`, and port settings with actionable prerequisite errors; pass the extension path to the server as `SDA_SPATIAL_EXTENSION` and the GeoPackage as its backend. Install the local soilDB source into a temporary R library; set the soilDB endpoint option `soilDB.sda.tabular.post.rest.url` to the child server URL ending in `/Tabular/post.rest`. Tests must be testthat-style and cover `SDA_query`, `SDA_spatialQuery`, and selected `get_SDA_*` package functions with stable Iowa fixtures (including verified `IA001`/mukey examples). Assert useful results rather than only HTTP success. Never write to or copy the Iowa GeoPackage, edit the soilDB checkout, use a fixed shared port, or add this external harness to ordinary `cargo test`.
- **Do not touch:** any file in `/home/andrew/workspace/soilDB` or `/home/andrew/workspace/py-soildb`; Rust application code except building/running it; CI defaults.
- **Grep anchors:** soilDB `R/SDA_query.R` endpoint option and `tests/testthat/test-SDA_query.R`; `R/SDA-spatial.R` `SDA_spatialQuery`; `R/get_SDA_muaggatt.R`; `src/main.rs`; `src/config.rs`; `Makefile`.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so SDA_TEST_GEOPACKAGE=/data/SSURGO/output/iowa_ssurgo.gpkg SOILDB_PATH=/home/andrew/workspace/soilDB make test-soildb`; confirm all named testthat cases executed without skips; `git diff --check`. Then run the configured full Rust suite serially.

## Wave-boundary verification

- After each wave: `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` (serial).
- After task 6.9.1 and at final acceptance: run the configured `make test-soildb` command above, and verify the Iowa database and both external package repositories remain unchanged.
- No task may run before Plan 5/G32 is complete. No remote Git operations.
