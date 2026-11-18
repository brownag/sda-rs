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
| 3 | 6.3.1 Active-client intersection TVFs | pending | — |
| 4 | 6.4.1 Area-symbol/survey-polygon intersection TVFs | pending | — |
| 5 | 6.5.1 Mapunit-polygon intersection/key TVFs | pending | — |
| 6 | 6.6.1 Other feature-layer intersection TVFs | pending | — |
| 7 | 6.7.1 Scalar key-to-WKT TVFs | pending | — |
| 8 | 6.8.1 Table-input WKT TVFs/macros | pending | — |
| 9 | 6.9.1 soilDB/Iowa integration harness | pending | — |

## Orchestration decisions and hazards

- Plan 5/G32 completion was verified before Plan 6 Wave 1 started.
- Wave 1 boundary: `cargo fmt --check` passed; `git diff --check` passed;
  `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`
  passed (57 unit + 18 integration tests; 0 failures, configured spatial queries executed).
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
