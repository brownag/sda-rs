# State 3 — G16: A failed batch commits nothing

Plan: `planning/EXECUTION_PLAN_3.md`. Goal: `planning/GOALS.md` [G16] (circled). Decision D3.

## Baseline (recorded before wave 1)

- Branch: `feature/sqlite-mvp`, HEAD `57c7a8f` (planning commit). Working tree clean except untracked `.claude/` (worktree directory).
- Last green (plan 2, wave 2 boundary): `cargo fmt --check` passed; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (38 unit + 3 integration, 0 failures).
- SpatiaLite: `/usr/lib/x86_64-linux-gnu/mod_spatialite.so` installed; set `SDA_TEST_SPATIALITE` for full runs.

## Wave board

| Wave | Task | Status | Code commit |
|---|---|---|---|
| 1 | 3.1.1 Roll back failed batches | pending | — |

Wave-boundary verification: wave 1 — pending.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->
