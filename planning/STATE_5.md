# State 5 — SQLite Execution Safeguards (G32)

Plan: `planning/EXECUTION_PLAN_5.md`. Active goal: explicit and tested SQLite security/resource safeguards.

## Baseline (recorded before wave 1)

- Branch: `feature/sda-readiness`; HEAD `4198dc8`.
- Intentional Magic Wand edit to `planning/GOALS.md` is present; preserve it and keep it in planning-only commits.
- Prior Plan 3/4 worktrees remain retained; no cleanup performed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed (52 unit + 11 integration tests; 0 failures). Live SpatiaLite tests executed.
- Live SpatiaLite extension path: `/usr/lib/x86_64-linux-gnu/mod_spatialite.so`; configured live tests executed.
- Accepted ADRs 0001–0006 remain in force; no supersession authorized.
- Plan 6 artifacts are present as untracked user work and are intentionally untouched.

## Wave board

| Wave | Tasks | Status | Code commit |
|---|---|---|---|
| 1 | 5.1.1 Persistent-state authorizer | pending | — |
| 2 | 5.2.1 Extension lifecycle; 5.2.2 test UDF flags | pending | — |
| 3 | 5.3.1 SQLite/row limits; 5.3.2 serialized-output cap | pending | — |

## Orchestration decisions and hazards

- Read-only persistent GeoPackage; retain request-local `#temp` workflows.
- Trusted configured extension loading only; submitted SQL cannot load extensions.
- Test-only UDF fixture; no production UDF.
- Initial resource caps: 2 MiB SQL text, 100,000 rows, 64 MiB serialized output, 60 seconds execution, 64 MiB SQLite string/BLOB.
- Feature-only rusqlite changes (`hooks`, `limits`, `functions`) are authorized if needed; no new crates/version changes.
- Maximum two workers; isolated worktrees for parallel tasks. Planning artifacts are tracked and committed separately.
- Run configured full suites serially at each wave boundary. No remote Git operations.

## Task reports

<!-- Workers append "## {{id}}" sections below, max 15 lines each. -->
