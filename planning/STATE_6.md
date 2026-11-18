# State 6 — Spatial Readiness and SDA Client Compatibility

Plan: `planning/EXECUTION_PLAN_6.md`. This plan is gated on completion of Plan 5/G32.

## Baseline

- Branch: `feature/sda-readiness`; baseline HEAD `4198dc8` (`docs(planning): rerank pre-hosting goals`).
- Working tree was clean before Plan 6 artifacts.
- Plan 5/G32 is not complete: all waves remain pending in `planning/STATE_5.md`. No Plan 6 task may start until G32 acceptance is recorded.
- `cargo fmt --check`: passed at baseline.
- `git diff --check`: passed at baseline.
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed (52 unit + 11 integration tests; 0 failures). Configured live spatial tests executed.
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
| 1 | 6.1.1 Spatial SQL contract; 6.1.2 SDA_Get inventory | pending | — |
| 2 | 6.2.1 GeoPackageBinary decoder | pending | — |
| 3 | 6.3.1 Active-client intersection TVFs | pending | — |
| 4 | 6.4.1 Area-symbol/survey-polygon intersection TVFs | pending | — |
| 5 | 6.5.1 Mapunit-polygon intersection/key TVFs | pending | — |
| 6 | 6.6.1 Other feature-layer intersection TVFs | pending | — |
| 7 | 6.7.1 Scalar key-to-WKT TVFs | pending | — |
| 8 | 6.8.1 Table-input WKT TVFs/macros | pending | — |
| 9 | 6.9.1 soilDB/Iowa integration harness | pending | — |

## Orchestration decisions and hazards

- Plan 6 execution is blocked until Plan 5/G32 is complete.
- At most two workers; isolated worktrees for parallel tasks. Only Wave 1 is parallel.
- The soilDB checkout and Iowa GeoPackage are user data; inspect read-only and leave unchanged.
- Run configured Rust suites serially at wave boundaries; run `make test-soildb` at its milestone and final acceptance.
- The integration target is opt-in; it must own its server process and temporary R library, and must not be added to ordinary `cargo test`.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->
