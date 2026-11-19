# State 7 — Spatial Metadata, Geography Compatibility, and Readiness

Plan: `planning/EXECUTION_PLAN_7.md`

## Baseline

- Branch: `main`; baseline HEAD at planning start: `6b6ec34`; two commits ahead of `origin/main`.
- User-owned worktree changes exist in `planning/GOALS.md` and G33-1/G33-2 spike notes; older G33/G34 paths have staged-add/working-tree-delete changes. Preserve them. Plan 7 files use unique paths, and the DuckDB task is self-contained if those notes are absent from its worktree.
- Plans 1–6 exist. Planning artifacts are tracked and must be committed separately from implementation.
- Active ADRs 0001–0006 are Accepted; no supersession is authorized.
- `cargo test`: passed (70 unit tests; 23 integration tests; 0 failures).
- `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`: passed (70 unit tests; 23 integration tests; 0 failures; configured spatial tests executed).
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets`: passed with one existing `needless_question_mark` warning in `src/executor/sqlite.rs`.
- `git diff --check`: passed before Plan 7 files were created.
- `make test-soildb`: not run for this baseline; `STATE_6.md` records a previous successful opt-in run.
- Iowa GeoPackage: `/data/SSURGO/output/iowa_ssurgo.gpkg`; use read-only. Existing Plan 6 reports list stable area symbol `IA001` and mukey `1857266`.
- Targeted large files: `src/spatial.rs` (547 lines), `src/transpiler/spatial.rs` (1,330 lines); use grep anchors from task specs.
- No task worker has started. Do not infer task completion from baseline checks.
- No Plan 7 task worktrees exist yet. Before execution, choose/confirm a non-default task branch and verify user-owned changes are not lost or accidentally staged. Plan 3–5 detached worktrees are retained; four stale `.claude` worktree registrations are reported as prunable by Git. Do not reuse or remove any without inspecting and explicit authorization.

## Decisions and Hazards

- G31 first; G33-2 DuckDB probe precedes G12. G33-1/terrio is deferred.
- G12 means geography semantics; all six goal methods are in scope, instance-call syntax only, no new dependencies, and no silent planar fallback. Unsupported gaps must be evidenced and reported.
- G9: `--body-limit` and `SDA_BODY_LIMIT`; default 2 MiB; maximum 64 MiB; reject invalid, zero, or excessive values.
- Keep the soilDB harness optional; G35 is an evidence-based comparison only and does not authorize deletion.
- At most two concurrent workers, isolated worktrees for parallel tasks; configured full Rust suite runs serially at every wave boundary. Run `make test-soildb` at G35 and final acceptance.
- All external repositories and the Iowa GeoPackage remain read-only. DuckDB scratch is under `/tmp/opencode`; no service dependency or backend change.
- No remote Git actions.

## Wave Board

| Wave | Tasks | Status | Code commit |
|---|---|---|---|
| 1 | 7.1.1 G31 metadata; 7.1.2 G8/G9 configuration/body limit | pending | — |
| 2 | 7.2.1 G33-2 DuckDB isolated evaluation | pending | — |
| 3 | 7.3.1 G35 soilDB harness/fixture coverage comparison | pending | — |
| 4 | 7.4.1 G12 geography semantics/feasibility matrix | pending | — |
| 5 | 7.5.1 G12 predicates and intersection | pending | — |
| 6 | 7.6.1 G12 buffer, area, and text | pending | — |

## Worktree Registry

| Task | Repo/worktree | Branch | Created | Lifecycle |
|---|---|---|---|---|
| 3.1.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.1.1` | detached | Prior orchestration | Retained; do not clean |
| 3.2.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.2.1` | detached | Prior orchestration | Retained; do not clean |
| 3.3.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.3.1` | detached | Prior orchestration | Retained; do not clean |
| 3.3.2 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.3.2` | detached | Prior orchestration | Retained; do not clean |
| 3.4.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.4.1` | detached | Prior orchestration | Retained; do not clean |
| 3.5.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.5.1` | detached | Prior orchestration | Retained; do not clean |
| 3.6.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-3-task-3.6.1` | detached | Prior orchestration | Retained; do not clean |
| 4.1.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.1.1` | detached | Prior orchestration | Retained; do not clean |
| 4.1.2 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.1.2` | detached | Prior orchestration | Retained; do not clean |
| 4.2.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.2.1` | detached | Prior orchestration | Retained; do not clean |
| 4.2.2 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.2.2` | detached | Prior orchestration | Retained; do not clean |
| 4.3.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.3.1` | detached | Prior orchestration | Retained; do not clean |
| 4.4.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-4-task-4.4.1` | detached | Prior orchestration | Retained; do not clean |
| 5.2.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-5-task-5.2.1` | detached | Prior orchestration | Retained; do not clean |
| 5.2.2 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-5-task-5.2.2` | detached | Prior orchestration | Retained; do not clean |
| 5.3.1 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-5-task-5.3.1` | detached | Prior orchestration | Retained; do not clean |
| 5.3.2 | `/home/andrew/.local/share/opencode/worktree/6a1b92/plan-5-task-5.3.2` | detached | Prior orchestration | Retained; do not clean |
| User/unknown | `/home/andrew/workspace/sda-rs/.claude/worktrees/agent-a123dcd14d225c00d` | `worktree-agent-a123dcd14d225c00d` | Unknown | Git reports prunable; preserve pending user authorization |
| User/unknown | `/home/andrew/workspace/sda-rs/.claude/worktrees/agent-a476648278a9b3c37` | `worktree-agent-a476648278a9b3c37` | Unknown | Git reports prunable; preserve pending user authorization |
| User/unknown | `/home/andrew/workspace/sda-rs/.claude/worktrees/agent-a47dbb69b796fd546` | `worktree-agent-a47dbb69b796fd546` | Unknown | Git reports prunable; preserve pending user authorization |
| User/unknown | `/home/andrew/workspace/sda-rs/.claude/worktrees/agent-ad8993f104d84b2ac` | `worktree-agent-ad8993f104d84b2ac` | Unknown | Git reports prunable; preserve pending user authorization |
| Plan 7 | None created during planning | — | — | Before execution, create isolated task worktrees only after branch/user-change safety is resolved |

## Final Acceptance Gate (Orchestrator-Owned)

Pending after Wave 6: run and record `cargo fmt --check`, `git diff --check`, default `cargo test`, configured full `cargo test`, and `make test-soildb` serially. Preserve user edits when reconciling goals and state.

## Task Reports

<!-- Append each worker's report under its task ID, at most 15 lines. -->
