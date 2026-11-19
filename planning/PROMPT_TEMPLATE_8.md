# Worker Prompt Template — Orchestration 8

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_8.md`, supervised by an orchestrator that reviews your micro-plan before implementation.

## Model

Use `openai/gpt-6-luna` unless the orchestrator explicitly provides an approved escalation model. Follow wave-orchestration retry limits.

## Task {{id}}: {{title}}

Paste the task specification verbatim from `planning/EXECUTION_PLAN_8.md`.

## Ground rules

- Do not make implementation/probe changes or run state-changing commands before your micro-plan is reviewed and approved by the orchestrator.
- After approval, execute only the approved plan. Do not touch files outside the task’s explicit files-in-scope list.
- Preserve all pre-existing changes. The current branch is `docs/backend-spike-research`; Plan 8 artifacts and report remain on this branch only. No `main` integration or remote action is authorized.
- `planning/EXECUTION_PLAN_8.md` and `planning/STATE_8.md` are the orchestration contract. Do not edit `planning/GOALS.md` or the existing G33 notes.
- Plan 8 is an isolated evaluation only. Do not modify production source, tests, Cargo manifests/lockfiles, service configuration, or accepted ADRs. Do not add dependencies to this repository.
- Keep all new probe files, Cargo targets, logs, and disposable databases under `/tmp/opencode/sda-rs-plan8-duckdb/`. Use `cargo --offline`; if cached Rust dependencies are insufficient, stop and report rather than fetching.
- First load the exact cached Spatial extension while external access is enabled. If that fails, the sole network exception is one `INSTALL spatial; LOAD spatial;` from DuckDB's official extension repository. Then set the exact source allowlist and disable external access. Do not use any other network access or alter the cached extension.
- Start with synthetic in-memory geometries. Iowa feature rows are allowed only if an OS sandbox is verified to mount the Iowa GeoPackage read-only and a preflight demonstrates indexed/bounded access. DuckDB `allowed_paths` allows writes; it is not a substitute for the OS read-only sandbox. If either Iowa gate fails, stop before reading feature rows.
- Never run a full scan, use `LIMIT` as a substitute for bounded access, modify/copy the Iowa GeoPackage, or stage external data. Record its hash, size, and mtime before and after any authorized feature access.
- Compare all six methods against SQL Server `geography` contracts; function availability or one matching sample is not parity. No SQL Server engine is available, so distinguish docs from observations.
- Run only the targeted commands in the task specification. Do not run full repository suites from the worker task.
- Keep the approved Plan 8 artifact commit separate from all task/report commits. Within each task, commit its TODO and report together using a focused Conventional Commit subject with no wave/task numbers or attribution trailers. No pushes, PRs, comments, or other remote actions.

## Analysis

Read:

1. The assigned task specification in `planning/EXECUTION_PLAN_8.md`.
2. `planning/STATE_8.md`.
3. `planning/reports/G33-2-duckdb-spike-results.md` and `planning/reports/G12-geography-method-feasibility.md`.
4. DuckDB 2.0 Spatial functions and Securing DuckDB documentation cited by the plan.
5. `AGENTS.md`, `CONTRIBUTING.md`, accepted ADRs 0001 and 0003, and the prior standalone probe under `/tmp/opencode/sda-rs-plan7-duckdb/` as a read-only reference.

## Micro-planning — stop after report

Create `planning/tasks/{{id}}.todo.md` with:

```markdown
# {{id}} — {{title}}
## Goal
<one sentence>
## TODO items
- [ ] <small, verifiable step, including exact probe/check>
## Notes
<facts, safety gates, limitations>
## Questions
<numbered, max 5; omit if none>
```

Use at most 12 steps. Map every acceptance criterion to a probe or check. For Task 8.2.1, do not perform any Iowa feature read while micro-planning; the separate orchestrator gate approval remains mandatory. Output the complete micro-TODO and stop. Do not implement until the orchestrator sends:

> APPROVED -- execute the plan as written. [Answers to any questions from their plan.]

## Execution

After approval, perform TODO items in order and mark them complete. Do not go beyond the exact synthetic/Iowa gates. Commit the planning TODO and report together in one focused research/report commit; do not create a separate TODO-only commit. For Task 8.2.1, the Wave 1 preflight must be independently approved before any feature read. No implementation commit is expected because no production files are in scope.

## Reporting

Return a concise report to the orchestrator: status; files changed; commit IDs; exact commands/results; versions and extension hash; method-by-method observed outcomes; whether Iowa feature access occurred and its sandbox/index evidence; source fingerprint before/after if accessed; deviations and blockers. The orchestrator alone updates `planning/STATE_8.md`.
