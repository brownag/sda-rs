# State 13 — DuckDB Qualification Through G42

Plan: `planning/EXECUTION_PLAN_13.md`

## Baseline

- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-feature1`; starting HEAD `bbf48f9`.
- Preserve the current uncommitted `planning/GOALS.md` updates. Plan 13 artifacts are approved for tracking; commit them separately from implementation.
- Separate worktree `/tmp/opencode/sda-rs-plan10-merge` on `feature/duckdb-production-backend` contains untracked Plan 10 draft files. Do not modify, stage, or remove them.
- Plan 12 default and configured-SpatiaLite suites passed (75 unit + 30 integration each); opt-in DuckDB and `--all-features` integration target each passed (12 tests); fmt and diff checks passed. No source changes since these checks.
- Baseline verification provenance is `STATE_12.md`, recorded at starting HEAD; no fresh full-suite run was required for this planning-only draft.
- Existing SQLite benchmark baseline: `planning/reports/duckdb-sqlite-benchmark-baseline.md`; it is a single-host microbenchmark, not production-load evidence.
- DuckDB production executor/security/deployment are not implemented in this branch; no runtime selector is authorized in Plan 13.
- Linux x86_64 only. No live data/network use for DuckDB tests.

## Decisions / Approval Gates

- G31/G8/G9 implementation and targeted tests are done; reconcile this branch's stale Plan 7 state without reimplementation.
- G37 evaluates transformations for all six geography methods; bounded exact subsets require runtime enforcement and compatibility tests. Otherwise retain explicit rejection.
- Candidate dependencies may be researched; request separate approval before adding one.
- D9 20% performance improvement is a non-blocking objective. Measure/report; missing it alone does not block.
- G42 may recommend candidate resource limits; final numeric values and G43 runtime enablement require later explicit approval.
- G43–G45 are out of scope. SQLite remains default.
- Maximum 2 workers; isolated worktrees for concurrent tasks; serial shared test suites. Worker model `opencode-go/deepseek-v4.1-flash`; one escalation max to `openai/gpt-6-luna`.
- Track planning artifacts separately from implementation commits.

## Wave Board

| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 13.1.1, 13.1.2 | pending | Plan 7 state reconciliation; ADR 0007 performance clarification. |
| 2 | 13.2.1 | pending | G37 contract and transformation evidence. |
| 3 | 13.3.1 | pending | Contains/Within. |
| 4 | 13.4.1 | pending | Intersection/Buffer. |
| 5 | 13.5.1 | pending | Area/AsText; close G37. |
| 6 | 13.6.1 | pending | G38 lifecycle. |
| 7 | 13.7.1 | pending | G39 controls. |
| 8 | 13.8.1 | pending | G40 compatibility. |
| 9 | 13.9.1 | pending | G41 provisioning. |
| 10 | 13.10.1 | pending | G42 benchmarking/recommendations. |

## Task Reports

<!-- Append concise reports under task IDs. -->
