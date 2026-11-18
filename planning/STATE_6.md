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
| 6 | 6.6.1 implementation; 6.6.2 diagnosis; 6.6.3 fixture correction; 6.6.4 format and boundary verification | done | `9c23a78` |
| 7 | 6.7.1 One-key WKT TVFs; 6.7.2 compile recovery; 6.7.3 HTTP 500 diagnostic; 6.7.4 generated SQLite diagnosis / evidence-led fix | blocked on configured contract failure; 6.7.4 pending approval | — |
| 8 | 6.8.1 Table-input WKT TVFs/macros | blocked on Wave 7 | — |
| 9 | 6.9.1 soilDB/Iowa integration harness | blocked on Waves 7–8 | — |

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
- **Status:** implementation completed and included in Wave 6 commit `9c23a78`.
- **Scope:** attempted generic key intersections for mupoint, muline, featline, and featpoint in WGS84/Web-Mercator.
- **Acceptance:** configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` stopped at the new Mupoint WGS84 request: expected key `401`, endpoint returned `{}`. No format checks or commit followed.
- **Schema evidence:** the local Iowa GeoPackage has `featpoint` but lacks required `featpointkey`; it is not evidence of support for that catalog function. No Iowa data was modified.
- **Initial acceptance:** failed at Mupoint WGS84; see 6.6.2 diagnostic outcome below.

## 6.6.2
- **Status:** diagnostic completed; reported WGS84 failure did not reproduce.
- **Scope:** diagnose the failed Mupoint WGS84 case and correct its established cause while preserving the 6.6.1 diff.
- **Prior diagnostic attempt:** an initial invocation failed during compilation before reaching the test because temporary instrumentation borrowed `extension` after it was moved; no evidence was captured.
- **Approved GPT-6.2 Sol diagnostic:** corrected one-time serial invocation compiled and reached the test. The Mupoint WGS84 request returned HTTP 200 with key `401`; generated SQL used `mupoint.mp_shape`, SRID 4326, and the request-local predicate returned 1. Thus the reported WGS84 failure did not reproduce.
- **Additional observation:** the following Mupoint Web-Mercator assertion returned `{}`. Request-local replay showed the stored point transformed to `55659.745396636783,55660.451865421521`, versus rounded fixture WKT `55659.745,55660.451`; `ST_Intersects` returned 0.
- **Disposition:** probes removed and all four pre-existing implementation-file hashes verified unchanged. No production behavior change was needed for WGS84; the Web-Mercator fixture precision issue was addressed in 6.6.3.

## 6.6.3
- **Status:** fixture correction completed and included in `9c23a78`.
- **Change:** only four positive Web-Mercator query literals changed in `tests/sda_get_contract.rs`: full-precision projected points for Mupoint/Featpoint and a transverse segment crossing for Muline/Featline. Expected keys and production code unchanged.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` — passed (1 test; all contract assertions passed).
- **Checks:** `git diff --check` passed. `cargo fmt --check` initially failed on three formatting differences in the preserved 6.6.1 production diff; task 6.6.4 resolved those formatting-only differences.
- **Caveat:** the earlier Mupoint WGS84 `{}` result did not reproduce in the approved diagnostic and remains unexplained. Iowa has `featpoint` but lacks required `featpointkey`.
- **Disposition:** expected keys and production behavior unchanged; four Web-Mercator fixture queries now exercise true intersections.

## 6.6.4
- **Status:** complete; included in Wave 6 implementation commit `9c23a78`.
- **Change:** applied only rustfmt's three requested line wraps in `intersection_spec`; reviewed the full four-file Wave 6 diff. Documentation count corrected to fourteen additional variants.
- **Checks:** `cargo fmt --check` passed; `git diff --check` passed; configured full `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (61 unit tests, 20 integration tests, 0 failures; configured spatial contract executed).
- **Wave 6 commit:** `9c23a78` (`feat(spatial): add feature-layer intersection TVFs`).
- **Caveat:** the historical Mupoint WGS84 `{}` observation remains unexplained; current diagnostic passed. Iowa has `featpoint` but lacks required `featpointkey`.
- **Next:** Wave 6 is verified; Wave 7 may proceed.

## 6.7.1
- **Status:** blocked on compile failure; implementation diff uncommitted.
- **Contract evidence:** `planning/sda-rs-scoping.md` and the USDA Advanced Queries guide specify one-key functions as `FROM` TVF sources with WKT output aliases. The soilDB vignette query adds a separate `mukey` expression; the function output remains the WKT column.
- **Attempted changes:** added eight scalar/one-key WKT function specifications, metadata-aware source rewrites, fixture `mukey` columns for point/line layers, exact WKT/alias and empty-result assertions, and support-matrix statuses. No commit.
- **Acceptance:** configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` failed to compile before test execution. The rewrite has unreachable-expression warnings in the source matcher and an alias type mismatch (`Option<Identifier>` vs `Option<String>`, and `String` assigned where `Identifier` is expected).
- **Disposition:** stopped per approved plan; no formatting checks, correction, or retry. Preserve uncommitted scoped changes.
- **Next:** discovered task 6.7.2 takes over the preserved diff only after micro-plan approval; Wave 8 remains gated.

## 6.7.2
- **Status:** compile blocker cleared; configured contract failed at runtime. Approved plan: `3117b8b`.
- **Change:** used direct returns for both matcher fallbacks and passed `Option<Identifier>` through unchanged, preserving the parsed default alias when absent. The four-file Wave 7 diff remains uncommitted.
- **Acceptance:** ran `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract` once. Compilation succeeded; `active_intersection_tvfs_return_expected_keys_for_both_crs` failed at `tests/sda_get_contract.rs:34:5`: `assertion left == right failed`, left `500`, right `200` (exit 101).
- **Disposition:** stopped at the first failure. No optional compile check, retry, subsequent implementation edits, format or diff checks, or implementation commit.
- **Next:** 6.7.3 is planned to identify the failing request and expose its error body. Waves 7–9 stay blocked.

## 6.7.3
- **Status:** diagnostic complete; approved plan `86db12e`. No implementation commit.
- **Diagnostic:** ran `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test sda_get_contract -- --nocapture --test-threads=1` once. Compilation succeeded; exit 101 (0 passed, 1 failed). The first 24 requests passed their assertions.
- **First failure (request 25):** `SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey('inside')` returned `500 Internal Server Error` with full body `{"error":"SQLite execution failed: near \"MAX\": syntax error"}`. This identifies a SQLite syntax error; the emitted statement was not captured, so the origin of `MAX` is unconfirmed.
- **Not reached (13 requests):** Web-Mercator Mupolygon with `inside`; both CRS variants for Mupoint and Muline with `inside`; all eight missing-key/NULL one-key WKT cases.
- **Cleanup:** removed the temporary test-helper reporting; its tag is absent from the test source, all four Wave 7 SHA-256 hashes match their baselines, and `git diff --check` passed. The four-file diff remains unstaged and uncommitted.
- **Gate:** no retry, behavior change, or acceptance run. A behavior fix requires separate approval; Waves 7–9 remain blocked.

## 6.7.4
- **Status:** diagnostic run stopped as inconclusive; no implementation change.
- **Diagnostic:** ran the prescribed command once. The test completed, but all three `'inside'`, `11`, and `'11'` cases returned HTTP 400 with `{"error":"SQL rejected: spatial rewrite requires GeoPackage geometry metadata or a geometry column override"}`. Preparation failed before the callback could emit generated SQL, so no SQL/status/body evidence concerning `MAX` was obtained.
- **Cause:** fixture created `gpkg_geometry_columns` without the geometry extension metadata expected by `SpatialCapability`; the request-local route and callback therefore rejected before transpilation. The task requires stopping rather than retrying when the diagnostic lacks evidence.
- **Cleanup:** removed temporary diagnostic test; `[DEBUG-674]` and `debug_674_scalar_mukey` absent. All four baseline hashes match: `docs/sda-get-support.md` `e54c42b04df09620c57d6875099228d086475ab056325ad92e9a662c6a7a9ec3`; `src/spatial.rs` `a7ebc463b5534fde0cf58d38e77bab40a446bf0a194c958670e1797ecadb9b2d`; `src/transpiler/spatial.rs` `463c367b021b57e3d3cf25aba527cb5152e29de3c64198ca4a33bf6764e07307`; `tests/sda_get_contract.rs` `676cd9890525c6fa8c044c189f7636b73e8ee5456d951c834af6eba2e3f8d2ca`. `git diff --check` passed; original four-file Wave 7 diff preserved.
- **Disposition:** no cause established, no scalar rewrite fix, no focused regression, and no configured contract rerun. Waves 7–9 remain blocked pending further authorization.
