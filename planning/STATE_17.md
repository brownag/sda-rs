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
- No network endpoints or user data; external documentation only if already available, cited with guarantees distinguished from assumptions.
- Separate user approval is required for dependency addition, API/parser replacement, pin changes, and production authorization-gate work.

## Wave Board
| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 17.1.1 | pending | Freeze exact policy corpus and criteria. |
| 2 | 17.2.1 | pending | Verify exact release/source/API, offline; depends on Wave 1. |
| 3 | 17.3.1 | pending | Scratch visitor prototype and tests; depends on Wave 2. |
| 4 | 17.4.1 | pending | DuckDB 2.0.0 differential evidence and final recommendation; depends on Wave 3. |

## Task Reports

<!-- Append concise task reports after verified completion. -->
