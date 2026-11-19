# Execution Plan 9 — G12 T-SQL Spatial Methods

## Goal & Scope

### Goal

Advance G12 for T-SQL-to-SQLite by resolving whether the six named CLR spatial instance methods can be implemented safely under the project's existing SQLite/optional-SpatiaLite contract. Keep SQL Server `geography` semantics distinct from planar geometry behavior, implement only methods with a defensible bounded contract, and otherwise preserve explicit unsupported behavior with evidence.

### In scope

- G12: `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`, using CLR instance-method syntax.
- Reassess Plan 7's feasibility findings only where new evidence can materially establish safe behavior.
- Update contract tests and documentation for any bounded, verified behavior and for methods that remain unsupported.

### Out of scope / deferred

- General SQL Server geography parity, arbitrary geodetic operations, silent planar substitutes, and unsupported method expansion beyond the six named methods.
- New dependencies, a new spatial backend, or spatial architecture changes.
- Changes to the HTTP wire protocol, SQLite service backend, request lifecycle, security/resource controls, or accepted ADRs.
- G12 does not authorize unrelated goals in `GOALS.md`.

### Verbatim user direction

- “@plan-wave plan 9 for G12 for T-SQL->SQLite in GOALS.md”
- “Stay within G12”
- “Feasibility gate”
- “No new deps”
- “Confirm before execution”
- “Track”
- “Serial”
- “Recommended gate”

## Dated Decisions

- **2026-11-19 — Scope:** Restrict Plan 9 to G12 and its six named CLR spatial instance methods; do not pull in other goals.
- **2026-11-19 — Feasibility:** Use an implementation attempt with an early evidence gate. Implement only if safe semantics are demonstrated; otherwise retain explicit unsupported behavior.
- **2026-11-19 — Technology boundary:** No dependencies or spatial architecture change. Unsupported behavior remains explicit.
- **2026-11-19 — Execution branch:** Confirm branch and user-change safety before execution; no branch switch or creation is authorized by this plan.
- **2026-11-19 — Artifacts:** Track the Plan 9 planning artifacts and keep their commit separate from implementation.
- **2026-11-19 — Parallelism:** Serial workers only.
- **2026-11-19 — Acceptance:** Target spatial-method contract tests per implementation task; final gate runs formatting, diff check, default full tests, and configured-SpatiaLite full tests serially.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`; current branch `docs/backend-spike-research`, HEAD `a04e12c`. Preserve existing user changes, especially `planning/GOALS.md`; do not switch branches or stage broad paths.
- Planning files are tracked. Plans 1–7 exist; Plan 9 is the requested orchestration number (do not create Plan 8).
- `planning/EXECUTION_PLAN_7.md` and `planning/reports/G12-geography-method-feasibility.md` record prior scope and evidence. Plan 7 found five methods unsupported for geography compatibility and `STAsText` conditionally feasible only for bounded ordinary 2D geometries, but no current rewrite seam can enforce that bound.
- `docs/spatial-sql-contract.md` explicitly rejects all six methods for geography compatibility. `tests/spatial_methods_contract.rs` checks explicit rejection and existing `.STIntersects` behavior; current `src/transpiler/spatial.rs` is 1,519 lines and should be navigated by grep anchors.
- Baseline: `cargo test` passed (75 unit tests; 26 integration tests; 0 failures); `cargo test --test spatial_methods_contract` passed (1 test); `cargo fmt --check` and `git diff --check` passed.
- Configured SpatiaLite final gate: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`. Spatial contract tests skip configured operations when the environment variable is unset.
- Active ADRs 0001–0006 remain accepted. Preserve SQLite/GeoPackage, optional SpatiaLite, request-local connections, rollback, result numbering, and test database conventions; no ADR supersession is authorized.
- Conventions: root `AGENTS.md`; `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets`; do not add dependencies without approval.

## Dependency Graph and Waves

All tasks target `/home/andrew/workspace/sda-rs`. Serial tasks intentionally share spatial method files and must not run concurrently.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 9.1.1 | Revalidate the six-method feasibility boundary and make an explicit go/no-go decision from existing evidence. |
| 2 | 9.2.1 | Implement only any approved bounded feasible behavior, retaining explicit unsupported outcomes for the rest. |

Task 9.2.1 depends on 9.1.1. If the feasibility gate finds no newly justified safe behavior, task 9.2.1 is documentation/test-only: preserve rejection and make the evidence-backed boundary unambiguous. No implementation may proceed on planar substitution.

At the wave boundary, run `cargo fmt --check`, `git diff --check`, and configured full `cargo test` serially. Do not run Cargo suites concurrently.

## Global Hazards and Rules

- Preserve all pre-existing staged, unstaged, and untracked user work, especially `planning/GOALS.md`. Never stage broad paths or use destructive Git commands.
- Before execution, confirm the intended implementation branch and that its worktree safely includes required user changes; this plan authorizes no branch creation/switch.
- Every worker must stop after writing its micro-TODO until the orchestrator approves it.
- Geography semantics are not planar geometry semantics. Never lower a method to a planar function and call it geography-compatible.
- No new dependencies, backend changes, response-shape changes, or accepted ADR changes.
- If safe behavior cannot be enforced at the transpiler/runtime boundary, explicitly reject it and document why; do not infer constraints from AST shape that runtime values can violate.
- No remote Git actions.

## Task Specifications

### Task 9.1.1 — Revalidate G12 feasibility boundary

- **ID:** 9.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `planning/reports/G12-geography-method-feasibility.md`.
- **Can create:** disposable probes only under `/tmp/opencode/sda-rs-plan9-geography/`.
- **Depends on:** Plan 7 findings and Plan 9 baseline.
- **Spec:** Reassess the existing six-row evidence matrix for `STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText` against authoritative SQL Server geography documentation and the configured SpatiaLite 5.1 runtime. Focus on whether any new, reproducible evidence changes a prior classification and whether a runtime-enforceable input bound exists at the existing transpiler seam. Do not repeat probes without a concrete unresolved question. Record sources, exact bounded probe commands/results, and a per-method go/no-go decision. If no classification changes, state that the current explicit rejections are the correct G12 outcome under the no-new-dependency constraint.
- **Do not touch:** service source, tests, `docs/spatial-sql-contract.md`, dependencies, user data, `planning/GOALS.md`, external repositories, or existing reports other than the named output.
- **Grep anchors:** six-method evidence matrix in `planning/reports/G12-geography-method-feasibility.md`; `rewrite_node` and `unsupported` handling in `src/transpiler/spatial.rs`; rejection assertions in `tests/spatial_methods_contract.rs`; active method contract in `docs/spatial-sql-contract.md`.
- **Acceptance:** report has an explicit go/no-go for each of the six methods and cites primary sources; any probes are bounded and reproducible; `git diff --check`.

### Task 9.2.1 — Apply evidence-backed G12 behavior

- **ID:** 9.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`.
- **Can create:** `tests/spatial_methods_contract.rs`.
- **Depends on:** task 9.1.1.
- **Spec:** Use the reviewed feasibility report as a hard gate. For each method whose report establishes a safe, runtime-enforceable bounded contract with existing dependencies, implement precisely that instance-method behavior and add configured SpatiaLite end-to-end tests for its boundary, result, NULL/invalid handling, and output type as applicable. For every other method, retain explicit unsupported rejection and document its reason. If no method passes the gate, make only necessary contract/test clarifications; do not manufacture implementation work. Do not claim G12 complete unless all six methods are either verified against their approved bounded contract or explicitly classified unsupported with evidence.
- **Do not touch:** `Cargo.toml`, `Cargo.lock`, free-function spellings, unrelated TVFs, security/rollback, HTTP response shape, `planning/GOALS.md`, or external data.
- **Grep anchors:** `Expression::MethodCall`, `rewrite_node`, and current six-method rejection helper in `src/transpiler/spatial.rs`; configured test setup and rejection assertions in `tests/spatial_methods_contract.rs`; method contract sections in `docs/spatial-sql-contract.md`; go/no-go matrix from task 9.1.1.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test spatial_methods_contract`; `cargo fmt --check`; `git diff --check`. Tests must demonstrate configured execution for any newly supported behavior and explicit rejection for unsupported methods.

## Final Acceptance Gate (Orchestrator-Owned)

After Wave 2, run serially and record in `STATE_9.md`: `cargo fmt --check`; `git diff --check`; `cargo test`; and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`. Preserve user edits when updating state or reconciling G12 in `planning/GOALS.md`; no GOALS edit is authorized by the tasks.
