# State 6 — Spatial Readiness and SDA Client Compatibility

Plan: `planning/EXECUTION_PLAN_6.md`. This plan is gated on completion of Plan 5/G32.

## Baseline

- Branch: `feature/sda-readiness`; Plan 6 began at HEAD `ea46fbe` after Plan 5 integration.
- Working tree was clean before Plan 6 task micro-plans.
- Plan 5/G32 is complete: all waves and tasks are marked done in `planning/STATE_5.md`; security, resource limits, and configured-suite acceptance are recorded.
- `cargo fmt --check`: passed before Wave 1.
- `git diff --check`: passed before Wave 1.
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed (57 unit + 18 integration tests; 0 failures). Configured live spatial tests executed.
- R: 4.6.2; soilDB 2.9.4 and required testthat/sf/wk/DBI/RSQLite packages are installed.
- Iowa GeoPackage: `/data/SSURGO/output/iowa_ssurgo.gpkg`, approximately 6.4 GB; existing read-only data includes `IA001` and mukey `1857266`.
- soilDB source input: `/home/andrew/workspace/soilDB`, branch `feat/custom-sda1`, HEAD `1eaa0717`; pre-existing dirty files `.Rbuildignore`, `R/SDA_query.R`, `R/createSSURGO.R`, and untracked `createSSURGO_patch.diff`. Do not alter or stage.
- py-soildb source input: `/home/andrew/workspace/py-soildb`, branch `v090`, HEAD `f070c0a`, clean.
- Historical Plan 3/4 worker worktrees and existing `.claude` agent worktrees are retained; do not clean them up as part of this plan.
- Accepted ADRs 0001–0006 remain in force; no supersession authorized.
- Planning artifacts are tracked and committed separately. No remote Git actions.

## Wave board

| Wave | Tasks | Status | Code commit |
|---|---|---|---|
| 1 | 6.1.1 Spatial SQL contract; 6.1.2 SDA_Get inventory | done | `b90bd8b`, `366a02a` |
| 2 | 6.2.1 GeoPackageBinary decoder | done | `e2eb5c2` |
| 3 | 6.3.1 Active-client intersection TVFs | done | `4d4aa0a` |
| 4 | 6.4.1–6.4.3 diagnostic attempts; 6.4.4 implementation; 6.4.5 regression correction | done | `8a54496`, `d4d867b` |
| 5 | 6.5.1 Mapunit-polygon intersection/key TVFs | done | `84e0138` |
| 6 | 6.6.1 blocked; 6.6.2 diagnosis/fix | in progress | — |
| 7 | 6.7.1 Scalar key-to-WKT TVFs | pending | — |
| 8 | 6.8.1 Table-input WKT TVFs/macros | pending | — |
| 9 | 6.9.1 soilDB/Iowa integration harness | pending | — |

## Orchestration decisions and hazards

- Plan 5/G32 completion was verified before Plan 6 Wave 1 started.
- Wave 1 boundary: `cargo fmt --check` passed; `git diff --check` passed;
  `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`
  passed (57 unit + 18 integration tests; 0 failures, configured spatial queries executed).
- Wave 2 boundary: `cargo fmt --check` passed; `git diff --check` passed;
  `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`
  passed (61 unit + 18 integration tests; 0 failures, configured spatial queries executed).
- Wave 3 boundary: `cargo fmt --check` passed; `git diff --check` passed;
  `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`
  passed (61 unit + 19 integration tests; 0 failures, configured spatial queries executed).
- At most two workers; isolated worktrees for parallel tasks. Only Wave 1 is parallel.
- The soilDB checkout and Iowa GeoPackage are user data; inspect read-only and leave unchanged.
- Run configured Rust suites serially at wave boundaries; run `make test-soildb` at its milestone and final acceptance.
- The integration target is opt-in; it must own its server process and temporary R library, and must not be added to ordinary `cargo test`.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->

## 6.1.1
- **Status:** complete.
- **Files:** `docs/spatial-sql-contract.md` (lines 1–100).
- **Code commit:** `b90bd8b` (`docs: define spatial SQL contract`).
- **Checks:** `cargo fmt --check` and `git diff --check` passed; orchestrator reviewed the document against its scoped implementation/test anchors.
- **Deviation/gotcha:** none; untested NULL/invalid geometry behavior is explicitly left unspecified.

## 6.1.2
- **Status:** complete.
- **Files:** `docs/sda-get-support.md` (lines 1–131).
- **Code commit:** `366a02a` (`docs: inventory SDA_Get compatibility`).
- **Checks:** `git diff --check` passed; matrix covers 34 catalog functions and cites active package call sites.
- **Evidence caveat:** official catalog page returned HTTP 502; entries were cross-checked against USDA-NRCS catalog search results and the existing scoping inventory.
- **External inputs:** soilDB remains at `1eaa07178311a4ad3b80842e1ff4f10149733544` with pre-existing dirty files; py-soildb remains clean at `f070c0a410c871f8d2fde42d992d625e9e8d085a`.
- **Deviation/gotcha:** none; all unimplemented functions are conditional on the required table/layer/columns and spatial configuration.

## 6.2.1
- **Status:** complete.
- **Files:** `src/gpkg_geometry.rs` (lines 1–184); `src/lib.rs` (module export).
- **Code commit:** `e2eb5c2` (`feat: decode GeoPackageBinary geometry headers`).
- **Targeted test:** `cargo test gpkg_geometry` — 4 passed; orchestrator rerun passed.
- **Checks:** `cargo fmt --check` and `git diff --check` passed.
- **Coverage:** magic/version/flags, byte order/SRS, all envelope sizes, empty flag, payload extraction, and malformed/truncated values.
- **Deviation/gotcha:** none; decoder is independent of SpatiaLite and does not alter query semantics.

## 6.3.1
- **Status:** complete.
- **Files:** `src/transpiler/spatial.rs` (lines 11–300); `src/spatial.rs` (lines 64–74); `tests/sda_get_contract.rs` (lines 1–118); `docs/sda-get-support.md` (updated support evidence).
- **Code commit:** `4d4aa0a` (`feat: support active SDA intersection TVFs`).
- **Targeted tests:** configured `cargo test --test sda_get_contract` — 1 passed; configured `cargo test --test mukey_contract` — 1 passed.
- **Checks:** `cargo fmt --check`, `git diff --check`, and configured full suite passed (61 unit + 19 integration; 0 failures).
- **Spatial evidence:** configured SpatiaLite initialized and real WGS84/Web-Mercator mukey and sapolygonkey intersections executed; key names/types and existing mukey behavior verified.
- **Deviation/gotcha:** none; task owns the four basic mukey/sapolygonkey CRS variants, so subsequent intersection work must avoid duplicating them.

## 6.4.1
- **Status:** blocked; no implementation commit.
- **Uncommitted task-scoped files:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, and `docs/sda-get-support.md`. Preserve; do not discard.
- **Acceptance:** configured `cargo test --test sda_get_contract` failed on the first Mukey WGS84 assertion with `{}` instead of the expected table result in the worker's two attempts and the orchestrator's reproduction (3 failures total). No final acceptance rerun was made during escalation.
- **Diagnostic:** configured `cargo test --test sda_get_contract debug_641_request_path -- --nocapture` — 1 passed; spatial intersection and generated SQL/executor returned `("inside", "text")`, so the original zero-row cause is unresolved.
- **Independent regression evidence:** Web-Mercator predicate returned 0 with mixed SRIDs; common-SRID comparison returned 1. Fix was not made because the approved plan required stopping while WGS84 root cause remained unresolved.
- **Checks:** `git diff --check` passed; temporary diagnostic test was removed. No implementation files were changed by the escalated worker.
- **Disposition:** superseded for execution by user-authorized recovery task 6.4.2.

## 6.4.2
- **Status:** blocked after approved diagnosis; no implementation commit.
- **Scope:** preserve and take over the four uncommitted 6.4.1 implementation files; diagnose the WGS84 request discrepancy, fix the evidenced Web-Mercator CRS mismatch, and complete remaining Wave 4 variants only after micro-plan approval.
- **Authorization:** user approved a new narrowly scoped diagnosis task after the mid-tier escalation could not establish the WGS84 cause.
- **Diagnostic:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract debug_641_request_path -- --nocapture` — 1 passed. The same fixture/request returned HTTP 200 with `("inside", "text")`; holder and request-local geometry/SRID/intersection results matched.
- **Unresolved:** earlier configured acceptance runs returned `{}` for the first Mukey WGS84 query, while the diagnostic path returned the expected row. The discrepancy is not explained, so acceptance was not rerun and no behavioral fix was made.
- **Known defect:** Web-Mercator comparison still has mixed SRIDs; a common-SRID probe returned an intersection. Not fixed because the approved stop condition was met.
- **Preservation:** the original uncommitted four-file task diff remains in the shared checkout; temporary diagnostic test was removed. Later waves remain gated on Wave 4 completion.

## 6.4.3
- **Status:** blocked after the authorized diagnostic-only run; no implementation commit.
- **Scope:** temporary test-only reporting to reconcile the prior `{}` response and later expected row; no production behavior changes.
- **Authorization:** user approved broader diagnosis without weakening the expected WGS84 contract.
- **Diagnostic:** one serial configured run returned HTTP 200 with the expected `inside` row for Mukey WGS84, then HTTP 200 with `{}` for Mukey Web-Mercator; the test stopped at the second query, leaving six queries unexecuted. Exit 101; 0 passed, 1 failed.
- **Outcome:** this run matches the later successful WGS84 observation but does not explain the earlier three WGS84 empty results. The Web-Mercator result is consistent with the known mixed-SRID defect. No behavior was changed.
- **Checks/preservation:** temporary reporting was removed; hashes of the four existing implementation files match their pre-instrumentation values; `git diff --check` passed. Existing four-file diff remains uncommitted.
- **Next action:** await user direction; all downstream waves remain gated on Wave 4 completion.

## 6.4.4
- **Status:** implementation committed; Wave 4 boundary uncovered a regression requiring follow-up.
- **Scope:** finish the four remaining area-symbol/status-map intersection variants, fix the confirmed Web-Mercator CRS mismatch, and retain the WGS84 expected result without claiming the earlier inconsistent responses are explained.
- **Authorization:** user approved proceeding with the confirmed Web-Mercator fix and remaining Wave 4 work while preserving the WGS84 assertion.
- **WGS84 caveat:** user confirmed that earlier `{}` responses may remain recorded as unexplained if the post-fix configured contract passes.
- **Files:** `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, `docs/sda-get-support.md`.
- **Code commit:** `8a54496` (`feat(spatial): complete SDA intersection TVFs`).
- **Targeted acceptance:** configured `cargo test --test sda_get_contract -- --nocapture` passed (1 integration test; all eight query cases executed with real spatial fixtures).
- **Checks:** `cargo fmt --check` and `git diff --check` passed.
- **Wave boundary:** configured full `cargo test` failed in unit test `transpiler::spatial::tests::mukey_override_column_uses_mupolygon_metadata_srid` (60 unit passed, 1 failed; integration tests were not reached). Generated SQL transformed geometry to SRID 4326 instead of expected metadata SRID 3857.
- **Next:** 6.4.5 restores the existing Mukey WGS84 metadata-SRID behavior while preserving the Web-Mercator common-CRS fix; Wave 5 remains gated.

## 6.4.5
- **Status:** implementation verified and committed.
- **Scope:** `src/transpiler/spatial.rs` only; preserve the WGS84 Mukey metadata-SRID contract and the new Web-Mercator correction.
- **Authorization:** existing Plan 6 scope requires preserving the previously tested Mukey behavior; regression surfaced at the Wave 4 boundary.
- **Change:** Mukey WGS84 source geometry uses `mupolygon` metadata SRID; its WKT remains SRID 4326. The Web-Mercator target retains common SRID 3857.
- **Focused test:** `cargo test mukey_override_column_uses_mupolygon_metadata_srid` — passed.
- **Checks:** `cargo fmt --check` and `git diff --check` — passed.
- **Wave boundary:** configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` — passed (61 unit tests, 20 integration tests, 0 failures; configured spatial tests executed).
- **WGS84 caveat:** earlier inconsistent `{}` observations remain unexplained as authorized; current contract passes.
- **Code commit:** `d4d867b` (`fix(spatial): preserve Mukey metadata SRID`).
- **Next:** Wave 4 is verified; Wave 5 may proceed.

## 6.5.1
- **Status:** implementation verified and committed.
- **Scope:** added the two Mupolygonkey WGS84/Web-Mercator intersection variants, exact fixture/type and no-match assertions, and conditional support-matrix entries.
- **Behavior:** no-match serializes as `{}` per `src/serializer.rs::column_names_format_and_all_empty`; test expectation reflects established response behavior.
- **Acceptance:** first configured contract attempt stopped at the no-match assertion because it expected a column header. Following evidence review and user authorization, expected no-match body was corrected to `{}`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` passed (1 test, both new CRS cases included).
- **Checks:** `cargo fmt --check` and `git diff --check` passed.
- **Code commit:** `84e0138` (`feat(spatial): add Mupolygonkey intersection TVFs`).
- **Boundary suite:** configured full `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (61 unit tests, 20 integration tests, 0 failures).
- **Next:** Wave 6 may begin.

## 6.6.1
- **Status:** blocked; implementation diff preserved uncommitted.
- **Scope:** attempted generic key intersections for mupoint, muline, featline, and featpoint in WGS84/Web-Mercator.
- **Acceptance:** configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` stopped at the new Mupoint WGS84 request: expected key `401`, endpoint returned `{}`. No format checks or commit followed.
- **Schema evidence:** the local Iowa GeoPackage has `featpoint` but lacks required `featpointkey`; it is not evidence of support for that catalog function. No Iowa data was modified.
- **Preservation:** uncommitted changes remain in `src/transpiler/spatial.rs`, `src/spatial.rs`, `tests/sda_get_contract.rs`, and `docs/sda-get-support.md`.
- **Next:** task 6.6.2 investigates the request/geometry/SRID/predicate path and only proceeds with an evidence-established correction; downstream waves remain gated.

## 6.6.2
- **Status:** discovered follow-up task; user requested investigation and fix.
- **Scope:** diagnose the failed Mupoint WGS84 case and correct its established cause while preserving the 6.6.1 diff.
