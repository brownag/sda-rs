# TODO

## Goal
Resolve the `main...HEAD` review findings for the DuckDB classifier, benchmark evidence, and Plan 14 scope compliance.

## Tasks

### 1. Review-finding disposition
- [ ] Re-check the exact review findings against `planning/EXECUTION_PLAN_13.md` and `planning/EXECUTION_PLAN_14.md`; determine whether the benchmark gap is truly missing or already covered in evidence.
- [ ] Define a narrow, fail-closed SQL classification change that closes demonstrated accepted bypasses without widening SQL support; add regression cases for the bypass forms identified in Plan 14 probes.
- [ ] If benchmark evidence is missing, run or document the bounded spatial timing workload consistent with Plan 14.5.1, without selecting production caps.
- [ ] Resolve Plan 14 scope mismatch by confirming the implementation changes are authorized by Plan 13 and documenting that cross-plan scope, or by proposing a minimal correction for user approval before changing implementation.

### 2. Validate and review
- [ ] Run focused DuckDB security/execution tests, relevant full tests, formatting, and `git diff --check`.
- [ ] Review the final diff against the review findings and both plans; record any residual limitation.

### 3. Git delivery
- [ ] Commit this approved planning artifact separately with a `docs(planning):` message.
- [ ] Commit verified implementation/evidence changes in focused local Conventional Commits; do not perform remote actions.

## Notes
- Branch: use existing `feature/duckdb-feature1`; do not switch branches.
- Planning artifacts are tracked.
- Preserve existing commits and unrelated work. No push, fetch, pull, or remote actions.
- If work outside these tasks is needed, stop and request approval before proceeding.
