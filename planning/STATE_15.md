# State 15 — Parser-Backed DuckDB SQL Authorization

Plan: `planning/EXECUTION_PLAN_15.md`

## Baseline

- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-feature1`; baseline HEAD `9c68514`.
- Working tree was clean at plan intake. Existing worktrees are user/workflow-owned; do not modify or remove them.
- Existing pin: `polyglot-sql = 0.14`; DuckDB `=1.20000.2`, bundled engine v2.0.0.
- No new dependencies, pin changes, request routes, or runtime backend selection are authorized.
- Prior verified baseline: `cargo fmt --check`; `cargo test` (75 unit + 30 integration); configured-SpatiaLite `cargo test` (75 + 30); `CARGO_NET_OFFLINE=true cargo test --offline --all-features` (79 unit + all integration targets); focused DuckDB security (5), execution (4), and result (4) contracts passed.
- All test/probe work is local, offline, and synthetic. Shared Cargo suites run serially.

## Decisions

- Existing `polyglot-sql` parser is evaluated first; any new parser dependency requires separate explicit user approval.
- Policy allows ordinary table reads and selected reviewed scalar/table functions; unknown functions and external-I/O sources are denied.
- DuckDB remains internal-only and SQLite remains default.
- Tasks execute serially; targeted validation per task, full default/SpatiaLite/all-features gates at the end.
- Planning artifacts are tracked; use separate `docs(planning):` commits.
- Worker model: `opencode-go/deepseek-v4.1-flash`; at most one escalation to `openai/gpt-6-luna`.

## Wave Board

| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 15.1.1 | done | Feasibility report completed; bounded parser evidence supports policy design only. |
| 2 | 15.2.1 | done | Policy matrix drafted; AST completeness blocker identified. |
| 3 | 15.3.1 | blocked | Policy report states public traversal does not establish exhaustive AST child coverage; implementation gate unresolved. |
| 4 | 15.4.1 | blocked | Depends on parser-backed gate; no implementation authorized. |

## Task Reports

<!-- Append concise task reports after verified completion. -->

## 15.1.1
- Status: done; report `planning/reports/duckdb-parser-feasibility.md`.
- Files: feasibility report and `planning/tasks/15.1.1.todo.md`.
- Tests: offline parser probe passed; `cargo test --offline --features duckdb --test duckdb_security_contract -- --nocapture` (5 passed); `git diff --check` passed.
- Deviations: disposable DuckDB rebuild could not complete because `/tmp` ran out of space; existing pinned contract/probe evidence used and limitation documented.
- Gate: conditional parser reuse candidate; next task must inspect exact AST classification; no implementation authorization implied.

## 15.2.1
- Status: done; policy proposal at `planning/reports/duckdb-parser-policy.md`.
- Files: policy report and `planning/tasks/15.2.1.todo.md`.
- Tests: documentation review; `git diff --check` passed.
- Deviations: none.
- Gate: AST completeness is unresolved because public DFS does not prove exhaustive child coverage; tasks 15.3.1 and 15.4.1 blocked pending explicit decision on verifiable traversal/upgrade guard or parser/API/scope change.
