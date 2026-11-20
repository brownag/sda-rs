# State 7 — Spatial Metadata, Geography Compatibility, and Readiness

Plan: `planning/EXECUTION_PLAN_7.md`

## Baseline

- Planning branch: `main`; implementation branch: `spatial-readiness` in `/tmp/opencode/sda-rs-spatial-readiness`, created from planning HEAD `b0dae6a`. Baseline HEAD at planning start: `6b6ec34`; current `main` is three commits ahead of `origin/main`.
- User-owned worktree changes remain on `main` in `planning/GOALS.md` and G33-1/G33-2 spike notes; older G33/G34 paths have working-tree deletions. Preserve them. Plan 7 files use unique paths, and the DuckDB task is self-contained if those notes are absent from its worktree.
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
- Plan 7 task worktree: `/tmp/opencode/sda-rs-spatial-readiness`, branch `spatial-readiness`, created from `b0dae6a`; active. It excludes the user-owned changes left on `main`. Plan 3–5 detached worktrees are retained; four stale `.claude` worktree registrations are reported as prunable by Git. Do not reuse or remove any without inspecting and explicit authorization.
- Clean implementation baseline on `spatial-readiness`: `cargo test` passed (70 unit tests; 23 integration tests; 0 failures); `cargo fmt --check` passed; `git diff --check` passed. Configured SpatiaLite suite not rerun yet.

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
| 1 | 7.1.1 G31 metadata; 7.1.2 G8/G9 configuration/body limit | done | `257202a`, `e93eed0` |
| 2 | 7.2.1 G33-2 DuckDB isolated evaluation | done (runtime follow-up blocked) | `0b55ffb` |
| 3 | 7.3.1 G35 soilDB harness/fixture coverage comparison | done | `e37910c` |
| 4 | 7.4.1 G12 geography semantics/feasibility matrix | done | `7fcb406`, `640af68` |
| 5 | 7.5.1 G12 predicates and intersection | done | `c28c322` |
| 6 | 7.6.1 G12 buffer, area, and text | done | `2a8d335` |

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

Complete per the recorded final gate below. Plan-level completion is distinct from completion of any individual goal.

## Task Reports

<!-- Append each worker's report under its task ID, at most 15 lines. -->

### 7.1.1 — G31
- Status: complete; worker commit `1c05bfaa7defda5b22362579f6f0289209f24dfb`, integrated as `65dabbe` on `spatial-readiness` and `257202a` on this branch.
- Tests: metadata contract default/configured SpatiaLite (5 each); `cargo test mukey` (4 unit, 1 contract, 2 integration); fmt and diff checks passed.

### 7.1.2 — G8/G9
- Status: complete; worker commit `6160c094d8ab02f8aeb150b438ca4f709bdd311e`, integrated as `08c212a` on `spatial-readiness` and `e93eed0` on this branch.
- Tests: `cargo test config::tests` (3 passed); `cargo test --test body_limit_contract` (1 passed); fmt and diff checks passed.

### Final acceptance gate
- `cargo fmt --check` and `git diff --check`: passed.
- Default `cargo test`: passed (75 unit, 30 integration); configured SpatiaLite `cargo test`: passed (75 unit, 30 integration).
- `make test-soildb`: passed; harness integration completed and server stopped.
- Source: authoritative Plan 7 state at `spatial-readiness` commit `60584ed`; its soilDB checkout caveat remains applicable.
