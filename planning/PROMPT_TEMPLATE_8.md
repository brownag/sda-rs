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
- Follow the extension acquisition rules for the assigned task: Task 8.1.1 is complete; Task 8.2.1's one official install attempt is complete and must not be repeated; Task 8.3.1 may make exactly one corrected official `INSTALL geography FROM community; LOAD geography;` attempt after setting and verifying `extension_directories` to scratch. Do not alter pre-existing cached binaries or source-build.
- Use synthetic in-memory geometries for Tasks 8.1.1–8.3.1. Task 8.4.1 is EXPLAIN-only and must not execute an Iowa feature-table query. Iowa feature rows are in scope only for Task 8.5.1, after explicit orchestrator approval and only if indexed/bounded access is proven. Do not use an OS sandbox. DuckDB `allowed_paths` allows writes; use read-only access procedures, prohibit writes, cap results at 100, and record source fingerprints before/after. If access cannot be demonstrated bounded and read-only, stop before reading feature rows.
- Task 8.4.1 may use read-only official DuckDB 2.0 and GDAL documentation/source access to resolve the `ST_Read` API mismatch and may iteratively correct the scratch EXPLAIN-only mode. Do not treat one invalid parameter or inconclusive plan as terminal; do not download/install packages or extensions. Never execute the Iowa spatial-filter query in Task 8.4.1.
- Task 8.6.1 may diagnose and correct the scratch DuckDB execution path after Task 8.5.1's internal transaction error. Charge each uncertain invocation by the current RTree candidate count against the 100-feature total budget, including the prior attempt; iterate while budget remains rather than stopping arbitrarily. Use only the exact approved query; no alternate query or feature source.
- Never run a full scan, use `LIMIT` as a substitute for bounded access, modify/copy the Iowa GeoPackage, or stage external data. Record its hash, size, and mtime before and after any authorized feature access.
- Compare all six methods against SQL Server `geography` contracts; function availability or one matching sample is not parity. No SQL Server engine is available, so distinguish docs from observations.
- Run only the targeted commands in the task specification. Do not run full repository suites from the worker task.
- Keep planning artifacts separate from task/report commits: commit each tracked micro-TODO as a planning-only `docs(planning):` change before execution; commit task research/report changes separately using a focused Conventional Commit subject with no wave/task numbers or attribution trailers. No pushes, PRs, comments, or other remote actions.

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

Use at most 12 steps. Map every acceptance criterion to a probe or check. For Task 8.2.1, do not repeat its completed install attempt. For Task 8.3.1, make at most one corrected official community-registry install attempt and do not perform any Iowa feature read. Task 8.4.1 is iterative, official-doc-guided, and EXPLAIN-only; it must stop before feature rows. Task 8.5.1 requires explicit orchestrator gate approval; Task 8.6.1 requires separate approval and must account uncertain reads against the 100-feature budget. Output the complete micro-TODO and stop. Do not implement until the orchestrator sends:

> APPROVED -- execute the plan as written. [Answers to any questions from their plan.]

## Execution

After approval, perform TODO items in order and mark them complete. Do not go beyond the exact task and safety gates. Tracked planning TODOs are committed separately with `docs(planning):`; task research/report changes use a separate focused `docs(research):` commit. Task 8.5.1 requires orchestrator approval of successful indexed/bounded-access evidence before any feature read. Task 8.6.1 has a separate approval and may only retry the exact query within the remaining cumulative feature budget. No implementation commit is expected because no production files are in scope.

## Reporting

Return a concise report to the orchestrator: status; files changed; commit IDs; exact commands/results; versions and extension hash; method-by-method observed outcomes; whether Iowa feature access occurred and its indexed/bounded-access evidence; source fingerprint before/after if accessed; deviations and blockers. The orchestrator alone updates `planning/STATE_8.md`.
