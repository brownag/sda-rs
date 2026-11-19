# State 8 — DuckDB Spatial Follow-up for Geography Methods

Plan: `planning/EXECUTION_PLAN_8.md`

## Baseline

- Planning/execution branch: `docs/backend-spike-research`; current HEAD at plan start `a04e12c`. `main` points to the same commit but is not an integration target for this plan.
- Current checkout was clean at reconnaissance. Existing G33 notes/GOALS updates are committed at `a04e12c`; do not rewrite them.
- Planning artifacts were initially committed at `01ded44` (`docs(planning): add DuckDB Spatial evaluation plan`); the community-geography scope amendment is pending its separate planning-only commit.
- The cached DuckDB Spatial binary is present at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension` (about 104 MiB). The previous probe used DuckDB Rust crate `1.20000.2` / engine 2.0.0.
- Baseline commands/results: `cargo fmt --check` passed; `git diff --check` passed; `cargo test` passed (75 unit, 30 integration); `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (75 unit, 30 integration). Baseline commit was `a04e12c` on `docs/backend-spike-research`; checkout was clean at reconnaissance.
- Relevant reports: `planning/reports/G33-2-duckdb-spike-results.md` and `planning/reports/G12-geography-method-feasibility.md`.
- Iowa source: `/data/SSURGO/output/iowa_ssurgo.gpkg`; previous SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`; 6,406,705,152 bytes. Prior unbounded/full-scan query plans blocked feature reads.
- OS sandbox tool `bwrap` is installed, but whether its namespace and read-only mount setup works is unknown. No Iowa row reads are authorized unless the gate passes.
- Accepted ADRs 0001–0006 remain active. DuckDB remains evaluation-only; no dependencies or production changes.

## Decisions and Hazards

- Re-evaluate G33-2 because DuckDB Spatial may help address G12; compare all six method contracts, not names alone.
- Task 8.1.1 used the cached official Spatial extension; its one official fallback was allowed but unused. Task 8.2.1 may make one official community-registry install attempt even if the compatibility matrix is blank, and may research official DuckDB/maintainer sources if it fails; no source build. Task 8.3.1 must use cached extension files only and no network.
- Exact-path allowlisting does not enforce read-only. Iowa features require an OS read-only sandbox plus preflight-proven indexed/bounded access; otherwise stop without feature rows.
- The community `geography` extension is S2 spherical, not assumed equivalent to SQL Server's ellipsoidal geography. Official catalog compatibility for DuckDB 2.0.0/Linux AMD64 was not listed at reconnaissance; this does not prevent the single user-authorized official-registry install attempt.
- Maximum concurrent tasks: one; the three research tasks are sequential, and Iowa Wave 3 is conditional. Worker model: `openai/gpt-6-luna`, following Plan 7's recorded convention.
- No backend adoption, production prototype, dependency change, ADR supersession, `main` integration, or remote action.

## Wave Board

| Wave | Tasks | Status | Commit |
|---|---|---|---|
| 1 | 8.1.1 Official Spatial synthetic evaluation and no-feature-row Iowa preflight | done | `cb45a3c`, `87c2deb` |
| 2 | 8.2.1 Community geography compatibility and synthetic evaluation | pending | — |
| 3 | 8.3.1 Conditional bounded Iowa comparison | blocked pending explicit orchestrator approval of Wave 1/2 evidence | — |

## Task Reports

<!-- Append the worker report under its task ID, at most 15 lines. -->

### 8.1.1 — Complete

- Report: `planning/reports/G33-2-duckdb-spatial-followup.md`; task TODO: `planning/tasks/8.1.1.todo.md`.
- Commits: `cb45a3c` records the evaluation; `87c2deb` clarifies native `GEOMETRY` / Rust `ValueRef::Geometry` results for Intersection and Buffer.
- Orchestrator re-ran offline release build, scratch rustfmt check, sandboxed synthetic probe, no-network Iowa metadata/RTree preflight, and `git diff --check`: all passed.
- DuckDB 2.0.0 / crate 1.20000.2; cached Spatial hash `8ec99d...a8d0bc`; PROJ 9.1.1.
- All six synthetic methods were recorded. Iowa preflight returned 29 RTree IDs and virtual-table index evidence; no feature-table rows were read. Wave 3 remains gated.

## Final Acceptance Gate

- Verify the community-extension compatibility outcome and synthetic security gate; run `git diff --check`.
- Wave 1 and 2 must not read feature-table rows. Permit the Wave 3 Iowa command only after review of the no-row sandbox/index evidence and explicit orchestrator approval; otherwise record the blocker and do not execute it.
- Do not run full Rust suites for this research-only plan; no production files are in scope.
