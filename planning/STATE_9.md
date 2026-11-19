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
- Configured SpatiaLite full-suite baseline not run during planning; run it serially before wave 1 or at its boundary: `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`.
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
| 1 | 9.1.1 G12 feasibility revalidation | pending | — |
| 2 | 9.2.1 evidence-backed method behavior | pending | — |

## Final Acceptance Gate (Orchestrator-Owned)

Pending after Wave 2: run serially `cargo fmt --check`, `git diff --check`, `cargo test`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`.

## Task Reports

<!-- Append each worker's report under its task ID, at most 15 lines. -->
