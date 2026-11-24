# State 16 — Verifiable DuckDB AST Traversal Guard

Plan: `planning/EXECUTION_PLAN_16.md`

## Baseline
- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-feature1`; baseline HEAD `b76f322`.
- Working tree clean at intake.
- Existing parser resolves to `polyglot-sql` 0.14.1; no dependency or pin change authorized without specific approval.
- Focused baseline: `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 5 passed.
- Plan 15 previously recorded full default, SpatiaLite, and all-features baselines; rerun full gates serially after implementation.
- Offline synthetic-only work; no request exposure. Planning artifacts are tracked and committed separately.

## Decisions
- Evaluate a local exhaustive visitor/guard first; implement it only if recursive coverage can be established.
- If existing API is insufficient, document parser/API alternatives and stop for specific user approval before dependency/pin changes.
- Preserve SQLite default, DuckDB internal-only boundary, and accepted ADR 0007.

## Wave Board
| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 16.1.1 | done | Feasibility report recommends STOP: enum exhaustiveness works, recursive child coverage is not verifiable downstream. |
| 2 | 16.2.1 | blocked | Pending user decision on parser/API or residual-risk approach; no implementation authorized. |
| 3 | 16.3.1 | blocked | Guard not implemented/verified; depends on resolution of Wave 2 blocker. |

## Task Reports

<!-- Append concise task reports after verified completion. -->

## 16.1.1
- Status: done; feasibility result blocks implementation pending user decision.
- Files: `planning/reports/duckdb-ast-guard-feasibility.md`; `planning/tasks/16.1.1.todo.md`; this state update.
- Evidence: offline isolated no-wildcard match probe failed with E0004 and 938+ uncovered variants; private traversal prevents downstream proof of recursive coverage.
- Verification: `git diff --check` — passed.
- Deviations: none; no source/dependency changes, implementation, or remote actions.
- Gotcha: an exhaustive enum match does not detect omitted child traversal; Wave 2/3 remain blocked.
