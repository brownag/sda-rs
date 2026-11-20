# State 12 — Optional DuckDB Cargo Build Feature

Plan: `planning/EXECUTION_PLAN_12.md`

## Baseline

- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `main`; starting HEAD `88b5fb6`.
- Preserve pre-existing modified `planning/GOALS.md`; do not switch branches or stage broad paths.
- `cargo fmt --check`: passed on 2026-11-20.
- `cargo test`: passed on 2026-11-20 (75 unit + 38 integration; 0 failures).
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed on 2026-11-20 (75 unit + 38 integration; 0 failures).
- Current manifest has pinned bundled DuckDB in `[dev-dependencies]`; Plan 11 test source has 12 passing tests.
- Planning artifacts are approved for tracking; planning-only commits remain separate from implementation.

## Decisions / Approval Gates

- **2026-11-20 — Goal:** Implement G48 optional compile-time inclusion and its direct G49 test-target gate only.
- **2026-11-20 — Feature:** Feature name is `duckdb`, disabled by default; `--all-features` includes DuckDB.
- **2026-11-20 — Bundling:** Retain the pinned crate's `bundled` feature for now; revisit if it proves problematic.
- **2026-11-20 — Platform:** Verify only on Linux x86_64; no other platform support promise.
- **2026-11-20 — Runtime boundary:** Feature inclusion does not select a runtime backend or authorize production enablement.
- **2026-11-20 — Execution:** Serial task; worker policy carried forward from Plan 11.
- **2026-11-20 — Branch:** Planning commit and follow-on Plan 12 work use `feature/duckdb-feature1`, created from the inspected `main` checkout.
- **2026-11-20 — Artifact tracking:** Track Plan 12 artifacts, with planning-only commits separate.

## Wave Board

| Wave | Task | Status | Notes |
|---|---|---|---|
| 1 | 12.1.1 | pending | Optional bundled DuckDB dependency and disabled-by-default feature. |
| 2 | 12.2.1 | pending | Gate the integration-test target and verify default/opt-in tests. |

## Task Reports

<!-- Append concise worker reports below. -->
