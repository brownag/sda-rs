# State 4 — G1 End-to-End MVP Acceptance

Plan: `planning/EXECUTION_PLAN_4.md`. Active goal: advance G1 by completing G6, G7, and G23 acceptance evidence.

## Baseline (recorded before wave 1)

- Branch: `feature/sqlite-mvp`; HEAD `bc97ca5`.
- Existing user-owned staged change: `planning/GOALS.md`. Preserve; do not stage or modify it as part of this plan.
- Prior Plan 3 worktrees remain retained; no cleanup performed.
- `cargo fmt --check`: passed.
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed (52 library tests + 5 integration tests; 0 failures).
- Live SpatiaLite extension path: `/usr/lib/x86_64-linux-gnu/mod_spatialite.so`.

## Wave board

| Wave | Task | Status | Code commit |
|---|---|---|---|
| 1 | 4.1.1 Request decoding; 4.1.2 metadata serialization | pending | — |
| 2 | 4.2.1 SQL/batch endpoint coverage; 4.2.2 AOI route coverage | pending | — |
| 3 | 4.3.1 Mukey TVF fixture coverage | pending | — |
| 4 | 4.4.1 Live SpatiaLite acceptance gate | pending | — |

## Orchestration decisions and hazards

- G1 contract only; G6 + G7 + G23 bundled. G29–G33 and G28 remain outside scope unless an explicit blocker is discovered and approved.
- Stay on `feature/sqlite-mvp`; maximum two workers. Tasks are currently serial due to shared integration-test file ownership.
- Planning artifacts are tracked and committed separately. Preserve staged `planning/GOALS.md`.
- Live-extension acceptance must set `SDA_TEST_SPATIALITE` and prove the test did not return early.
- Full configured suites run serially at each wave boundary. No remote Git actions.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->
