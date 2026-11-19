# State 8 — DuckDB Spatial Follow-up for Geography Methods

Plan: `planning/EXECUTION_PLAN_8.md`

## Baseline

- Planning/execution branch: `docs/backend-spike-research`; current HEAD at plan start `a04e12c`. `main` points to the same commit but is not an integration target for this plan.
- Current checkout was clean at reconnaissance. Existing G33 notes/GOALS updates are committed at `a04e12c`; do not rewrite them.
- Planning artifacts were initially committed at `01ded44`; scope amendments were committed at `6a0ba73` and `f279d12`. Current approved direction removes OS sandboxing; indexed/bounded access, read-only query procedures, 100-feature cap, and fingerprints remain for any Iowa read.
- The cached DuckDB Spatial binary is present at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension` (about 104 MiB). The previous probe used DuckDB Rust crate `1.20000.2` / engine 2.0.0.
- Baseline commands/results: `cargo fmt --check` passed; `git diff --check` passed; `cargo test` passed (75 unit, 30 integration); `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (75 unit, 30 integration). Baseline commit was `a04e12c` on `docs/backend-spike-research`; checkout was clean at reconnaissance.
- Relevant reports: `planning/reports/G33-2-duckdb-spike-results.md` and `planning/reports/G12-geography-method-feasibility.md`.
- Iowa source: `/data/SSURGO/output/iowa_ssurgo.gpkg`; previous SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`; 6,406,705,152 bytes. Prior unbounded/full-scan query plans blocked feature reads.
- Do not use Bubblewrap or other OS-level sandboxing. No Iowa feature rows have been read; any future Iowa read remains gated on explicit orchestrator approval plus indexed/bounded access evidence and read-only procedures.
- Accepted ADRs 0001–0006 remain active. DuckDB remains evaluation-only; no dependencies or production changes.

## Decisions and Hazards

- Re-evaluate G33-2 because DuckDB Spatial may help address G12; compare all six method contracts, not names alone.
- Task 8.1.1 used the cached official Spatial extension; its official fallback was unused. Task 8.2.1 made the one authorized community install attempt but failed before network access because `DUCKDB_HOME` did not redirect DuckDB's extension directory. Task 8.3.1 is the corrected single attempt using `extension_directories`; no other source/build is permitted.
- Exact-path allowlisting does not enforce read-only. Do not use an OS sandbox. Any future Iowa feature access requires explicit approval, preflight-proven indexed/bounded access, read-only procedures, 100-feature cap, and before/after fingerprints.
- The community `geography` extension is S2 spherical, not assumed equivalent to SQL Server's ellipsoidal geography. Official catalog compatibility for DuckDB 2.0.0/Linux AMD64 was not listed at reconnaissance; this does not prevent the single user-authorized official-registry install attempt.
- Maximum concurrent tasks: one; research tasks are sequential, and Iowa comparison is conditional. Worker model: `openai/gpt-6-luna`, following Plan 7's recorded convention.
- No backend adoption, production prototype, dependency change, ADR supersession, `main` integration, or remote action.

## Wave Board

| Wave | Tasks | Status | Commit |
|---|---|---|---|
| 1 | 8.1.1 Official Spatial synthetic evaluation and no-feature-row Iowa preflight | done | `cb45a3c`, `87c2deb` |
| 2 | 8.2.1 Community geography initial attempt | done (setup blocker) | `31d06b2` |
| 3 | 8.3.1 Corrected community geography install and synthetic evaluation | done | `de80cd5` |
| 4 | 8.4.1 Iterative DuckDB spatial-filter preflight | done; corrected plan shows RTree-backed filter pushdown | `10202d4`, `548f50a`, `0714756` |
| 5 | 8.5.1 Conditional bounded Iowa comparison | done with DuckDB transaction error; possible reads unknown | `18cd367`, `3a85296` |
| 6 | 8.6.1 Repair and retry bounded DuckDB execution | complete; streaming retry returned 24 rows; conservative charge 58/100 | `ab37920`; planning commit pending |

## Task Reports

<!-- Append the worker report under its task ID, at most 15 lines. -->

### 8.1.1 — Complete

- Report: `planning/reports/G33-2-duckdb-spatial-followup.md`; task TODO: `planning/tasks/8.1.1.todo.md`.
- Commits: `cb45a3c` records the evaluation; `87c2deb` clarifies native `GEOMETRY` / Rust `ValueRef::Geometry` results for Intersection and Buffer.
- Orchestrator re-ran offline release build, scratch rustfmt check, synthetic probe, no-network Iowa metadata/RTree preflight, and `git diff --check`: all passed. The historical task used Bubblewrap; that is not a requirement going forward.
- DuckDB 2.0.0 / crate 1.20000.2; cached Spatial hash `8ec99d...a8d0bc`; PROJ 9.1.1.
- All six synthetic methods were recorded. Iowa preflight returned 29 RTree IDs and virtual-table index evidence; no feature-table rows were read. Wave 3 remains gated.

### 8.2.1 — Complete with setup blocker

- Report/TODO: `planning/reports/G33-2-duckdb-spatial-followup.md`, `planning/tasks/8.2.1.todo.md`; commit `31d06b2`.
- Offline scratch rustfmt/build passed; sole attempt failed before network access because DuckDB used `/home/andrew/.duckdb/extensions/v2.0.0`. The historical command used Bubblewrap; do not repeat it.
- No extension loaded, no synthetic methods ran, no Iowa feature rows were read; all six methods remain runtime-unverified for community geography.
- Follow-up 8.3.1 made one corrected official install/load attempt after setting `extension_directories` to scratch; do not repeat either install attempt.

### 8.3.1 — Complete

- Report/TODO: Wave 3 section in `planning/reports/G33-2-duckdb-spatial-followup.md`; `planning/tasks/8.3.1.todo.md`; commit `de80cd5`.
- Offline build passed; one corrected official community install/load succeeded after verifying `extension_directories` contained only the scratch home.
- Lockdown preceded synthetic probes: autoload/autoinstall disabled, 1 GB and 2-thread limits, external access false; re-enable rejected.
- Extension `geography` `f777314ad0`, SHA-256 `f0c5172314e86530d7680b34c62bc092acd64ae8440b43428a5be7ecbe1ad7ad`; six method observations and contract limits are in the report.
- No Iowa reads or OS sandbox; no production/Cargo/cached Spatial changes. `git diff --check` passed; working tree was clean after commit.

### 8.4.1 — Complete; bounded plan demonstrated

- Report appended in `planning/reports/G33-2-duckdb-spatial-followup.md`; preflight commits `10202d4` and `548f50a`; TODO: `planning/tasks/8.4.1.todo.md`.
- SQLite opened the exact GeoPackage with URI `mode=ro` and `PRAGMA query_only=ON`; metadata confirmed `mupolygon.geom`, POLYGON, SRS 4326.
- RTree candidate query used `LIMIT 101`, returned 29 IDs, and `EXPLAIN QUERY PLAN` showed `VIRTUAL TABLE INDEX 2:B0D1B2D3`.
- Source SHA-256/size/mtime matched before and after: `7b81951e...e90571a8`, 6,406,705,152 bytes, mtime `2026-11-18 07:31:13.192297150 -0800`.
- Scratch `cargo fmt --check` and offline build passed. The initial EXPLAIN-only invocation failed because DuckDB rejected `spatial_filter_box`; subsequent distinct queries resolved the mismatch.
- Source fingerprint immediately before/after that invocation matched: SHA-256 `7b81951e...e90571a8`, 6,406,705,152 bytes, mtime `2026-11-18 07:31:13.192297150 -0800`.
- Four distinct EXPLAIN attempts were recorded; the fourth shows the exact bbox filter pushed into `ST_Read`. DuckDB 2.0 docs/source and GDAL GeoPackage source confirm the filter API and RTree use.
- All per-attempt source fingerprints matched; no `EXPLAIN ANALYZE`, underlying `ST_Read`, or feature-table query ran. Zero Iowa feature rows were read/returned; no download, source-data write, OS sandbox, or repository code change.
- **Gate decision:** Based on the passing DuckDB plan, GDAL RTree implementation evidence, and matching 29-ID read-only SQLite preflight, the orchestrator approves Task 8.5.1 to read at most 100 features using only the exact planned filter query. Read-only procedures, no SQL writes, and before/after source fingerprints remain required.

### 8.5.1 — Complete with internal execution error

- Report/TODO: Wave 5 section in `planning/reports/G33-2-duckdb-spatial-followup.md`; `planning/tasks/8.5.1.todo.md`; commits `18cd367`, `3a85296`.
- Read-only SQLite preflight reverified `query_only=ON`, the exact GeoPackage metadata, and 29 RTree IDs for the approved envelope.
- Scratch fmt/offline build and lockdown checks passed; only the approved SQL was invoked once.
- DuckDB failed with `TransactionContext::ActiveTransaction called without active transaction`; no count/types/results returned.
- The query may have read rows before failing; exact number is unknown. Before/after source fingerprints matched; no other source query or write occurred.
- Conservatively reserve all 29 RTree candidates against the 100-feature cap. Task 8.6.1 may diagnose and correct the scratch execution path; recovery requires separate orchestrator approval.

### 8.6.1 — Complete

- TODO: `planning/tasks/8.6.1.todo.md`; report: Wave 6 in `planning/reports/G33-2-duckdb-spatial-followup.md`.
- Report commit: `ab37920`.
- Streaming Arrow scratch correction passed synthetic geometry validation, `cargo fmt --check`, and offline release build.
- Read-only RTree recheck returned 29 candidates. The exact approved query returned 24 GeoArrow-WKB Binary geometries, one batch, zero nulls.
- Source fingerprints matched before/after. Conservative cumulative charge: 29 prior uncertain + 29 retry = 58/100; no further attempt needed.
- No parity or performance conclusion is claimed; prior attempt's read count remains unknown.

## Final Acceptance Gate

- Verify corrected community-extension outcome and synthetic lockdown gate; run `git diff --check`.
- Waves 1–4 did not read Iowa feature-table rows. Wave 5's authorized query failed internally and its read count remains unknown. Wave 6's separately approved streaming correction succeeded with 24 returned rows; cumulative conservative candidate charge is 58/100 and source fingerprints match. No OS sandbox was used.
- Do not run full Rust suites for this research-only plan; no production files are in scope.
