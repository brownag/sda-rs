# TODO

## Goal
Resolve the `main...HEAD` review findings for the DuckDB classifier, benchmark evidence, and Plan 14 scope compliance.

## Tasks

### 1. Review-finding disposition
- [x] Re-check the exact review findings against `planning/EXECUTION_PLAN_13.md` and `planning/EXECUTION_PLAN_14.md`; determine whether the benchmark gap is truly missing or already covered in evidence.
- [x] Define a narrow, fail-closed SQL classification change that closes demonstrated accepted bypasses without widening SQL support; add regression cases for the bypass forms identified in Plan 14 probes.
- [x] If benchmark evidence is missing, run or document the bounded spatial timing workload consistent with Plan 14.5.1, without selecting production caps.
- [x] Resolve Plan 14 scope mismatch by confirming the implementation changes are authorized by Plan 13 and documenting that cross-plan scope, or by proposing a minimal correction for user approval before changing implementation.

### 2. Validate and review
- [x] Run focused DuckDB security/execution tests, relevant full tests, formatting, and `git diff --check`.
- [x] Review the final diff against the review findings and both plans; record any residual limitation.

### 3. Git delivery
- [x] Commit this approved planning artifact separately with a `docs(planning):` message.
- [x] Commit verified implementation/evidence changes in focused local Conventional Commits; do not perform remote actions.

## Notes
- Branch: use existing `feature/duckdb-feature1`; do not switch branches.
- Planning artifacts are tracked.
- Preserve existing commits and unrelated work. No push, fetch, pull, or remote actions.
- If work outside these tasks is needed, stop and request approval before proceeding.
- Finding disposition: the existing Plan 14.5.1 report already includes a separate correctness-checked planar `ST_Area` timing (median/p95) and resource caveats; no benchmark rerun is needed. The reviewed executor and spatial rejection changes implement Plan 13 tasks 13.3–13.10, while Plan 14 explicitly bars implementation absent separate approval. Plan 13 authorizes those changes; do not edit them as part of this fix.
- Validation: `cargo fmt --check`, default `cargo test`, configured-SpatiaLite `cargo test`, focused offline DuckDB execution/security/result contracts, offline `cargo test --all-features`, and `git diff --check` passed. The all-features run includes the synthetic DuckDB geography contract (without a provisioned artifact environment variable, its artifact-dependent cases skip as designed).
