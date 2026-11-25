# State 17 — Qualify `sqlparser` for DuckDB Policy Prototyping

Plan: `planning/EXECUTION_PLAN_17.md`

## Baseline
- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-feature1`; baseline HEAD `44aa8ab`.
- Main worktree clean at planning intake; linked worktrees exist and are not to be modified.
- Existing pinned DuckDB crate: `duckdb = "=1.20000.2"` (engine v2.0.0); do not alter repository Cargo files or pins.
- Prior focused security baseline evidence: Plan 16 `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 5 passed. No fresh implementation baseline required for this research plan.
- Prior Plan 15 scratch DuckDB rebuild exhausted `/tmp`; favor cached/offline evidence and existing targets.
- Planning files tracked separately; scratch only under `/tmp/opencode/`.
- Accepted ADR 0007 remains in force: SQLite default; DuckDB internal-only/opt-in; request-local state and rollback preserved.

## Decisions
- Evaluate exact `sqlparser` release/API and visitor only as a research candidate.
- Preserve exact Plan 15 default-deny policy; do not expand policy to fit parser.
- No production code, tests, Cargo dependency changes, parser/API replacement, engine pin changes, route or policy changes.
- No live SDA/API/service endpoints, user data, or external datasets. Exact package/source/documentation retrieval from authoritative sources is allowed; cite sources and distinguish documented guarantees from tested evidence.
- Separate user approval is required for dependency addition, API/parser replacement, pin changes, and production authorization-gate work.

## Wave Board
| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 17.1.1 | done | Criteria report verified; recursive coverage and runtime contingencies recorded. |
| 2 | 17.2.1 | done | Exact crate source/API, docs.rs 0.64.0 docs, checksum, and scratch `cargo check` verified; an earlier supplemental `cargo run` hit the then-full `/tmp`. |
| 3 | 17.3.1 | done | Scratch visitor/tests pass; research-only, important coverage gaps remain. |
| 4 | 17.4.1 | pending | Depends on completed visitor prototype from Wave 3. |

## Task Reports

<!-- Append concise task reports after verified completion. -->

## 17.3.1
- Status: complete as bounded research prototype; not suitable as an authorization gate.
- Files changed: `planning/reports/duckdb-sqlparser-visitor-prototype.md`, `planning/tasks/17.3.1.todo.md`, `planning/STATE_17.md`; scratch only under `/tmp/opencode/sda-rs-plan17-visitor/`.
- Verification: scoped offline `cargo test` passed (2 integration tests; orchestrator rerun passed); scoped offline `cargo clippy --all-targets -- -D warnings` passed (orchestrator rerun passed); `git diff --check` passed.
- Deviations: fail-closed defaults retained; CTAS, INSERT, DROP, joins, unknown relations and external functions not authorized. CTE test uses a narrow test-only exception, not general scope logic.
- Residual risks: incomplete per-field visitor/source audit, C01 source correspondence, generic CTE/relation resolution, exhaustive future-node handling, batch lifecycle/rollback and DuckDB runtime compatibility remain unproven. No production or dependency authorization.

## 17.1.1
- Status: complete; froze Plan 15 policy corpus and prototype acceptance criteria.
- Files changed: `planning/reports/duckdb-sqlparser-prototype-criteria.md`, `planning/STATE_17.md`.
- Verification: policy-row/case coverage and required-criteria check passed (17 uniquely traceable rows/cases); `git diff --check -- planning/reports/duckdb-sqlparser-prototype-criteria.md` passed.
- Deviations: none. No code, tests, dependencies, scratch harness, or policy changed.
- Residual risks: visitor completeness and ordinary relation resolution remain unproven; INSERT VALUES compatibility remains limited to the tested form.

## 17.2.1
- Status: complete with probe-run limitation; exact `sqlparser` 0.64.0 source/API, versioned docs, provenance, and compile check recorded.
- Files changed: `planning/reports/duckdb-sqlparser-release-evidence.md`, `planning/STATE_17.md`; scratch only under `/tmp/opencode/sda-rs-plan17-release/`.
- Verification: crate SHA-256 recorded; exact-version `cargo check` passed; docs.rs 0.64.0 visitor and dialect pages retrieved and cited. `cargo run --offline` could not link because `/tmp` was full (55G used, under 1 MiB free).
- Deviations: probe execution remains incomplete; crates.io metadata endpoint returned HTTP 403, but docs.rs supplied exact-version docs. No repository Cargo/dependency/code/test changes.
- Residual risks: prototype syntax/traversal behavior and DuckDB compatibility remain untested; production authorization suitability is not established.
- **Historical execution blocker:** Waves 3–4 needed scratch build/test output under `/tmp/opencode/`; at the time, `/tmp` was full. No unrelated `/tmp` contents were deleted.
- **2026-11-24 resumption:** User freed old `target/` directories; `/tmp` now has 29 GiB free. Wave 3 may proceed. Recheck capacity before the DuckDB differential task.
