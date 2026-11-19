# State 9 — G12 T-SQL Spatial Methods

Plan: `planning/EXECUTION_PLAN_9.md`

## Baseline

- Repository: `/home/andrew/workspace/sda-rs`; branch `docs/backend-spike-research`; HEAD `a04e12c`.
- Preserve pre-existing user changes, especially `planning/GOALS.md`. Confirm branch/worktree safety before execution; no branch change is authorized by the plan.
- Planning artifacts are tracked; keep their commit separate from implementation.
- `cargo test`: passed (75 unit tests; 26 integration tests; 0 failures).
- `cargo test --test spatial_methods_contract`: passed (1 test).
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Configured SpatiaLite full-suite baseline: passed before Wave 1 (75 unit tests; 26 integration tests; 0 failures).
- `src/transpiler/spatial.rs` is 1,519 lines; use task grep anchors.
- Plan 7's G12 report classifies five methods unsupported and `STAsText` conditionally feasible only under restrictions not enforceable by the current rewrite seam. Current contract and tests explicitly reject all six.
- ADRs 0001–0006 are accepted; no supersession authorized.

## Decisions and Hazards

- G12 only; six listed CLR instance methods; bounded feasibility gate.
- No dependencies or architecture changes; never substitute planar semantics as geography.
- Serial workers; full Cargo suites run serially.
- Track planning files separately from implementation.
- Confirm branch/user-change safety before execution. No remote Git actions.

## Wave Board

| Wave | Tasks | Status | Code commit |
|---|---|---|---|
| 1 | 9.1.1 G12 feasibility revalidation | done | — (report-only) |
| 2 | 9.2.1 evidence-backed method behavior | done | — (verification-only) |

## Final Acceptance Gate (Orchestrator-Owned)

Completed after Wave 2: `cargo fmt --check`, `git diff --check`, `cargo test`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` all passed serially.

## Task Reports

<!-- Append each worker's report under its task ID, at most 15 lines. -->

## 9.1.1

- Status: done; all six methods no-go at current seam; report now explains why no safe runtime-enforceable input bound exists, including for `STAsText`.
- Files: `planning/reports/G12-geography-method-feasibility.md` (added Plan 9 assessment and explicit matrix outcomes); `planning/tasks/9.1.1.todo.md` (completed checklist).
- Commit: none yet; report and tracked planning state are isolated in the Plan 9 worktree.
- Tests: report-only task; `git diff --check` passed.
- Wave boundary: `cargo fmt --check`, `git diff --check`, configured full `cargo test` passed (75 unit, 26 integration); configured `cargo test --test spatial_methods_contract` passed (1).
- Deviations: none. No probes repeated because existing documented probe evidence covered the identified questions.
- Handoff: Task 9.2.1 may verify and clarify existing explicit rejections; do not implement planar substitutes.

## 9.2.1

- Status: done; current contract and tests already cover all six no-go outcomes, so no implementation or clarification was needed.
- Files: only `planning/tasks/9.2.1.todo.md` and this state record changed; service source, contract, and tests unchanged.
- Code commit: none (verification-only task).
- Tests: configured `cargo test --test spatial_methods_contract` passed (1); `cargo fmt --check` and `git diff --check` passed.
- Final gates: default and configured-SpatiaLite `cargo test` both passed (75 unit; 26 integration; 0 failures each).
- Deviations: none. No planar fallback and no dependencies added.
