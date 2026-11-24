# Execution Plan 16 — Verifiable DuckDB AST Traversal Guard

## Goal & Scope

Resolve Plan 15's remaining AST-coverage blocker. First try to implement a verifiable exhaustive AST classification guard using the existing pinned `polyglot-sql` API. If this cannot establish recursive child coverage, investigate and present parser/API alternatives, but stop before any dependency or pin change and request a specific approval. The authorization gate itself remains contingent on the guard being demonstrably complete.

**In scope**
- Evaluate a local exhaustive visitor/guard using current `polyglot-sql` 0.14.1.
- Implement and test that guard if public AST structure allows reliable recursive classification and upgrade detection.
- If infeasible, document exact gap and viable parser/API alternatives for a follow-up user decision.
- Preserve Plan 15's fail-closed SQL policy and keep DuckDB internal-only.

**Out of scope**
- Any dependency addition, parser/API replacement, dependency or engine pin change without a separate specific approval.
- Request routes, backend selection, production enablement, SQLite behavior, response/wire changes, OS sandboxing, or broad SQL compatibility expansion.
- Claiming authorization completeness unless the implemented mechanism and adversarial evidence support it.

**Verbatim user constraints and decisions**
- “authorize a verifiable traversal mechanism or parser/API change”
- Chosen path: “Evaluate both” — first evaluate an existing-crate exhaustive visitor/guard; if inadequate, present parser/API alternatives for another approval.
- Chosen implementation scope: “Implement existing-crate guard” if feasible; stop and report if parser/API replacement is needed.

## Reconnaissance

- Repo `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-feature1`, HEAD `b76f322`; clean at intake. Planning artifacts are tracked; separate planning commits.
- Plan 15's feasibility, policy, and API-coverage reports are the inputs: `planning/reports/duckdb-parser-feasibility.md`, `duckdb-parser-policy.md`, and `duckdb-ast-coverage.md`.
- `polyglot-sql` resolves to 0.14.1; `Expression` is public and exhaustively matchable. Public DFS delegates to private generated child traversal, so exhaustive enum matching alone cannot verify recursive completeness.
- `src/executor/duckdb.rs` is 944 lines; use grep anchors. Security test target is `tests/duckdb_security_contract.rs`.
- Baseline: `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 5 passed. Plan 15 records prior full test baselines; rerun final gates only after an implementation change.
- Accepted ADR 0007 keeps SQLite default, DuckDB opt-in/internal until qualification; no ADR supersession.
- Keep probes/tests offline and synthetic. Do not run shared Cargo suites concurrently.

## Decisions

- **2026-11-23 — Existing API first:** Attempt a verifiable guard with current pinned parser before considering alternatives.
- **2026-11-23 — Dependency boundary:** Parser/API alternatives may be researched and documented, but no dependency/API replacement or pin change until the user selects and separately approves a concrete option.
- **2026-11-23 — Implementation gate:** Implement only if recursive child coverage can be established and future relevant changes fail closed or fail compilation. Otherwise stop with evidence; do not substitute a string heuristic.
- **2026-11-23 — Scope:** Keep DuckDB internal-only and SQLite default; preserve Plan 15 policy constraints and batch rollback.
- **2026-11-23 — Planning artifacts:** Tracked, separate `docs(planning):` commits, inherited from Plan 15.

## Dependency graph and wave table

| Wave | Task(s) | Purpose / dependency |
|---|---|---|
| 1 | 16.1.1 | Build a compile-time/manual visitor feasibility prototype and prove recursive coverage strategy using existing public AST; no production gate change. |
| 2 | 16.2.1 | Implement and test the exhaustive local guard only if Wave 1 demonstrates feasibility; otherwise stop the orchestration and return its alternatives for user selection. |
| 3 | 16.3.1 | Integrate the guard into the DuckDB executor and qualify focused security behavior only if Wave 2 succeeds and no external API change is needed. |

Tasks are sequential because each establishes whether the next is authorized. Any dependency or pin change is a user-approval boundary, not a task assumption.

## Task specifications

### Task 16.1.1 — Prove an existing-crate traversal guard is feasible
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** none.
- **Files in scope — must edit:** `planning/reports/duckdb-ast-guard-feasibility.md`.
- **Can create:** bounded scratch under `/tmp/opencode/sda-rs-plan16-ast/`.
- **Do not touch:** source, tests, Cargo files/lockfile, dependencies, policy docs, routes/configuration, ADRs, or other reports.
- **Spec:** Using the pinned public AST, determine whether a handwritten exhaustive visitor can recursively cover every syntax-bearing field relevant to the approved policy, including nested payload structs/containers, without relying on opaque private DFS. Establish a compile-time guard for enum and relevant payload changes. If that is infeasible or cannot be maintained honestly, identify exact limitations and candidate parser/API alternatives without changing dependencies.
- **Grep anchors:** `Expression` and public payload definitions in cached `polyglot-sql-0.14.1/src/expressions.rs`; private `ast_children`/derive visitor; `planning/reports/duckdb-ast-coverage.md`; policy matrix in `duckdb-parser-policy.md`.
- **Acceptance:** Report includes a construct/field coverage method, reproducible offline compile/check evidence or exact reason unavailable, upgrade-guard limits, and clear proceed/stop recommendation; `git diff --check`. No implementation changes.

### Task 16.2.1 — Implement the local exhaustive AST guard or stop with alternatives
- **Repo:** same; **Depends on:** 16.1.1 proving a viable existing-crate guard.
- **Files in scope — must edit:** `src/executor/duckdb.rs` (only if the current API supports the approved visitor).
- **Can edit:** `tests/duckdb_security_contract.rs`; scratch under `/tmp/opencode/sda-rs-plan16-ast/`.
- **Do not touch:** Cargo files/lockfile, dependencies, policy docs, routes/configuration, SQLite executor, response shape, or unlisted files.
- **Spec:** Implement a narrowly scoped exhaustive visitor/classification primitive that recursively checks policy-relevant nodes and fails closed for unreviewed variants/forms. Guard relevant payload field changes where feasible and explain residual guard limits. Preserve existing interim behavior unless the guard itself passes focused contracts. If the feasibility report does not establish viable complete traversal, this task is not authorized to run; stop the orchestration and present parser/API options for a concrete user decision.
- **Grep anchors:** `validate_statement`, `execute_batch` in `src/executor/duckdb.rs`; security assertions in `tests/duckdb_security_contract.rs`; approved policy matrix.
- **Acceptance:** Offline focused security contract passes, plus `cargo fmt --check` and `git diff --check`; tests cover nested children and unknown variants where constructible. No dependency changes.

### Task 16.3.1 — Integrate and adversarially qualify the guard
- **Repo:** same; **Depends on:** 16.2.1 implementation and successful review.
- **Files in scope — must edit:** `src/executor/duckdb.rs`; `tests/duckdb_security_contract.rs`; `docs/security/duckdb-execution-policy.md`.
- **Do not touch:** Cargo files/lockfile, dependencies/pins, routes/configuration, ADRs, response shape, or unlisted files.
- **Spec:** Replace the interim validator only if the implemented visitor has verified recursive coverage. Test accepted SELECT/temp lifecycle and rejected sensitive statements, nested dangerous functions/sources, compound inputs, raw/unknown forms, comments/quoted syntax per approved policy, and parser/runtime disagreement. Update security policy with exact guarantees and remaining risks. Keep request-path exposure prohibited.
- **Grep anchors:** guard and `validate_statement` in `src/executor/duckdb.rs`; `tests/duckdb_security_contract.rs`; policy proposal and coverage report.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; `cargo fmt --check`; `git diff --check`. Any unresolved traversal completeness blocks integration and returns to user.

## Orchestrator gates

- Each task begins with a <=12-step micro-TODO and stops for review; approval message required before execution.
- A failed feasibility result blocks dependent implementation; do not change dependencies or pins without specific user approval.
- Run tasks serially and never run shared Cargo suites concurrently.
- At completion of an implementation wave run full gates serially: `cargo fmt --check`; `git diff --check`; `cargo test`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; `CARGO_NET_OFFLINE=true cargo test --offline --all-features`.
- No remote Git operations. Keep planning-only commits separate from implementation commits.
