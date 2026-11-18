# State 3 — SQLite MVP correctness gaps

Plan: `planning/EXECUTION_PLAN_3.md`. Initial goal: `planning/GOALS.md` [G16] (circled), expanded to G17–G20, G22, and G24 in subsequent waves. Decision D3.

## Baseline (recorded before wave 1)

- Branch: `feature/sqlite-mvp`, HEAD `57c7a8f` (planning commit). Working tree clean except untracked `.claude/` (worktree directory).
- Last green (plan 2, wave 2 boundary): `cargo fmt --check` passed; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (38 unit + 3 integration, 0 failures).
- SpatiaLite: `/usr/lib/x86_64-linux-gnu/mod_spatialite.so` installed; set `SDA_TEST_SPATIALITE` for full runs.

## Wave board

| Wave | Task | Status | Code commit |
|---|---|---|---|
| 1 | 3.1.1 Roll back failed batches | done | `1a3b62d` |
| 2 | 3.2.1 Isolate temporary table names (G17) | done | `a7b0849`, `790ccc5` |
| 3 | 3.3.1 G18 declarations; 3.3.2 G19 metadata scoping | pending | — |
| 4 | 3.4.1 AST-level constructor rewrite (G22) | pending | — |
| 5 | 3.5.1 Mukey metadata fallback (G20) | pending | — |
| 6 | 3.6.1 Shared-cache in-memory test databases (G24) | pending | — |

Wave-boundary verification: waves 1–2 — passed. Latest serial run: `cargo fmt --check` passed; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (42 unit + 4 integration, 0 failures).

## Orchestration update

- User requested that the remaining sequenced correctness goals be added as subsequent waves; G1 and broad MVP acceptance remain for a follow-on orchestration.
- GOALS.md contains a G20 ↔ G6 dependency cycle. User directed G20 before G6; this plan tests G20 without the live spatial fixture, then leaves G6 for follow-on work.
- Task worktree `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.1.1`, detached at task commits `72ed301`, `ec1fe40`, `9238dda`; implementation and reports integrated to `feature/sqlite-mvp` as planning commit `e41b9d4`, code commit `1a3b62d`, and state commit `35296b8`. Worktree retained; no cleanup performed.
- Task worktree `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.2.1`, detached at task commits `3b7c001`, `73a7415`, `693b674`, `7356810`, `adb7e06`; integrated as planning commits `5e7b710`, `ee546bb`, `4344ea6`, implementation/test commits `a7b0849`, `790ccc5`. Worktree retained; no cleanup performed.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->

## 3.1.1
- Status: complete; implementation commit `ec1fe40` (`fix(sqlite): roll back failed statement batches`).
- Planning TODO commit: `72ed301` (`docs(planning): add batch rollback micro-plan`).
- Changed `src/executor/sqlite.rs:132-169,171-261`: transaction wraps batch; rollback and successful-commit unit tests added.
- Changed `tests/sda_mvp.rs:85-122`: router rollback test returns 500 and confirms unchanged row.
- Red run: `cargo test executor::` before transaction change — 6 passed, 1 failed; rollback assertion read `2`, expected `1`.
- `cargo test executor::` after change: 7 passed, 0 failed (33 filtered).
- `cargo test --test sda_mvp`: 4 passed, 0 failed.
- `cargo fmt --check`: passed after applying its indicated formatting.
- `git diff --check`: passed.
- Existing temp-table session test passed in the executor run. Deviations: none; full suite not run (orchestrator-owned).
- Wave-boundary integration commit on `feature/sqlite-mvp`: `1a3b62d`; serial full-suite verification passed.

## 3.2.1
- Status: complete; implementation commit `73a7415` (`fix(transpiler): isolate temporary table names`).
- Planning TODO commit: `3b7c001` (`docs(planning): add temp table isolation micro-plan`).
- Changed `src/transpiler/temp_tables.rs`: per-batch `#name` mapping to reserved `__sda_temp_<pid>_<counter>` names; rewrites SELECT INTO, CREATE TABLE, table references, and INSERT targets. Added CREATE/INSERT and updated mapping unit tests.
- Changed `src/executor/sqlite.rs`: added persistent-`sample` regression proving unprefixed lookup remains persistent and `#sample` reads the temp table.
- `cargo test temp_tables::`: 3 passed, 0 failed (39 filtered).
- `cargo test executor::`: 8 passed, 0 failed (34 filtered); includes existing temp table session test and new collision regression.
- `cargo fmt --check`: passed. `git diff --check`: passed.
- Red run: initial `cargo test temp_tables::` failed because INSERT target is represented outside the AST traversal's table nodes; added explicit `Expression::Insert` target rewrite, then targeted tests passed.
- Deviations: none. Full suite not run (orchestrator-owned).
- Addendum: commit `7356810` updates only the existing generated-SQL assertion in `src/transpiler/mod.rs` to check generated-name form and same-name reference.
- The first full-suite wave check caught this stale literal-name assertion after generated names were introduced.
- `cargo test transpiler::folds_variables_and_select_into_batch` exited 0 but matched 0 tests; corrected command `cargo test folds_variables_and_select_into_batch` ran the focused test: 1 passed, 0 failed.
- `cargo fmt --check` and `git diff --check` passed for the addendum change.
- Wave-boundary serial full suite after the assertion fix: 42 unit + 4 integration passed, 0 failed; `cargo fmt --check` passed. Integrated implementation commits: `a7b0849`, `790ccc5`.

## 3.3.1
- Status: complete; implementation commit `23cacc1` (`fix(transpiler): fold declaration defaults`); planning TODO commit `8969f04` (`docs(planning): add variable declaration micro-plan`).
- Changed `src/transpiler/vars.rs`: declaration defaults now substitute earlier values; unbound defaults and case-insensitive redeclarations are rejected; added three focused tests.
- `cargo test transpiler::vars::`: passed, 5 passed, 0 failed (40 filtered); binary and integration test targets each ran 0 matching tests.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Deviations: none. Full suite not run as instructed.

## 3.3.2
- Status: complete; implementation commit `3f9d0b2` (`fix(spatial): defer malformed metadata errors`).
- Planning TODO commit: `706ab37` (`docs(planning): record spatial metadata task plan`).
- Changed `src/spatial.rs:13-18,45-54,72-98,136-183,201-220`: defer metadata discovery errors; spatial requirement prefers metadata error to unavailable capability.
- Changed `src/routes.rs:301-341`: malformed-metadata route regression verifies `SELECT 1` succeeds and spatial query returns structured metadata error.
- `cargo test spatial::`: 13 passed, 0 failed (31 filtered); `cargo test routes::`: 4 passed, 0 failed (40 filtered).
- `cargo fmt --check`: passed; `git diff --check`: passed. Full suite not run.
- Initial spatial test run failed once because SQLite reported “Query is not read-only” rather than the assumed missing-column text; assertion was made error-text-agnostic, then it passed.
- Deviations: none. Valid/absent metadata with unavailable extension continues to return the existing capability error.
