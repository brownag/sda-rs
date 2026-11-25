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
| 2 | 17.2.1 | pending | Resumed with scoped authoritative release/source/documentation retrieval allowed. |
| 3 | 17.3.1 | pending | Depends on verified exact release/API from Wave 2. |
| 4 | 17.4.1 | pending | Depends on completed visitor prototype from Wave 3. |

## Task Reports

<!-- Append concise task reports after verified completion. -->

## 17.1.1
- Status: complete; froze Plan 15 policy corpus and prototype acceptance criteria.
- Files changed: `planning/reports/duckdb-sqlparser-prototype-criteria.md`, `planning/STATE_17.md`.
- Verification: policy-row/case coverage and required-criteria check passed (17 uniquely traceable rows/cases); `git diff --check -- planning/reports/duckdb-sqlparser-prototype-criteria.md` passed.
- Deviations: none. No code, tests, dependencies, scratch harness, or policy changed.
- Residual risks: visitor completeness and ordinary relation resolution remain unproven; INSERT VALUES compatibility remains limited to the tested form.

## 17.2.1
- Status: blocked; no exact `sqlparser` source or crate archive was available in the local Cargo cache.
- Files changed: `planning/reports/duckdb-sqlparser-release-evidence.md`, `planning/STATE_17.md`.
- Verification: offline cache searches found no source/archive; `Cargo.lock` has no `sqlparser` entry. No release was selected or compiled.
- Deviations: stopped as required; no network access, repository Cargo changes, or scratch harness.
- Residual risks: public API, visitor behavior, AST coverage, exact release provenance, and compile compatibility remain unverified. This initial stop condition was superseded by the resumption decision below.
- **2026-11-24 resumption decision:** User rejected the offline-only evidence limitation. Plan 17 and its worker template now allow retrieval of exact crate/source/versioned docs from authoritative sources, while keeping live SDA/API/service access, user/external data, runtime SQL external I/O, and remote Git actions prohibited. Task 17.2.1 must redo its micro-plan against the revised scope before execution.
