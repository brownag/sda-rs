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
| 2 | 3.2.1 Isolate temporary table names (G17) | pending | — |
| 3 | 3.3.1 G18 declarations; 3.3.2 G19 metadata scoping | pending | — |
| 4 | 3.4.1 AST-level constructor rewrite (G22) | pending | — |
| 5 | 3.5.1 Mukey metadata fallback (G20) | pending | — |
| 6 | 3.6.1 Shared-cache in-memory test databases (G24) | pending | — |

Wave-boundary verification: wave 1 — passed. `cargo fmt --check` passed; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (40 unit + 4 integration, 0 failures).

## Orchestration update

- User requested that the remaining sequenced correctness goals be added as subsequent waves; G1 and broad MVP acceptance remain for a follow-on orchestration.
- GOALS.md contains a G20 ↔ G6 dependency cycle. User directed G20 before G6; this plan tests G20 without the live spatial fixture, then leaves G6 for follow-on work.
- Task worktree `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.1.1`, detached at task commits `72ed301`, `ec1fe40`, `9238dda`; implementation and reports integrated to `feature/sqlite-mvp` as planning commit `e41b9d4`, code commit `1a3b62d`, and state commit `35296b8`. Worktree retained; no cleanup performed.

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
