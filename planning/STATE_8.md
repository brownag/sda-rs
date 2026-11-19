# State 8 — DuckDB Spatial Follow-up for Geography Methods

Plan: `planning/EXECUTION_PLAN_8.md`

## Baseline

- Planning/execution branch: `docs/backend-spike-research`; current HEAD at plan start `a04e12c`. `main` points to the same commit but is not an integration target for this plan.
- Current checkout was clean at reconnaissance. Existing G33 notes/GOALS updates are committed at `a04e12c`; do not rewrite them.
- Planning artifacts are untracked drafts pending approval. Commit the three Plan 8 artifacts together, separately from later task/report commits.
- The cached DuckDB Spatial binary is present at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension` (about 104 MiB). The previous probe used DuckDB Rust crate `1.20000.2` / engine 2.0.0.
- Baseline commands/results: `cargo fmt --check` passed; `git diff --check` passed; `cargo test` passed (75 unit, 30 integration); `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (75 unit, 30 integration). Baseline commit was `a04e12c` on `docs/backend-spike-research`; checkout was clean at reconnaissance.
- Relevant reports: `planning/reports/G33-2-duckdb-spike-results.md` and `planning/reports/G12-geography-method-feasibility.md`.
- Iowa source: `/data/SSURGO/output/iowa_ssurgo.gpkg`; previous SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`; 6,406,705,152 bytes. Prior unbounded/full-scan query plans blocked feature reads.
- OS sandbox tool `bwrap` is installed, but whether its namespace and read-only mount setup works is unknown. No Iowa row reads are authorized unless the gate passes.
- Accepted ADRs 0001–0006 remain active. DuckDB remains evaluation-only; no dependencies or production changes.

## Decisions and Hazards

- Re-evaluate G33-2 because DuckDB Spatial may help address G12; compare all six method contracts, not names alone.
- Try the cached extension first; if it fails, one `INSTALL spatial; LOAD spatial;` from DuckDB's official extension repository is allowed. No other network access or extension source is permitted.
- Exact-path allowlisting does not enforce read-only. Iowa features require an OS read-only sandbox plus preflight-proven indexed/bounded access; otherwise stop without feature rows.
- Maximum concurrent tasks: one; the two research tasks are sequential, and Wave 2 is conditional. Worker model: `openai/gpt-6-luna`, following Plan 7's recorded convention.
- No backend adoption, production prototype, dependency change, ADR supersession, `main` integration, or remote action.

## Wave Board

| Wave | Tasks | Status | Commit |
|---|---|---|---|
| 1 | 8.1.1 Synthetic evaluation and no-feature-row Iowa preflight | pending | — |
| 2 | 8.2.1 Conditional bounded Iowa comparison | blocked pending explicit orchestrator approval of Wave 1 preflight | — |

## Task Reports

<!-- Append the worker report under its task ID, at most 15 lines. -->

## Final Acceptance Gate

- Run the synthetic/security command and `git diff --check`.
- Wave 1 must not read feature-table rows. Permit the Wave 2 Iowa command only after review of the no-row sandbox/index evidence and explicit orchestrator approval; otherwise record the blocker and do not execute it.
- Do not run full Rust suites for this research-only plan; no production files are in scope.
