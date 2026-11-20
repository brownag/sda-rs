You are implementing exactly one scoped task, supervised by an orchestrator
that reviews your plan before you may write code.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules (non-negotiable)
- NEVER make implementation changes or run implementation state-changing commands before your micro-plan is approved. Creating/updating the scoped micro-TODO is allowed before approval.
- After approval, make a focused local Conventional Commit for verified implementation work. No pushes, pull requests, comments, or other remote actions.
- NEVER touch files outside "Files in scope"; if required, STOP and report.
- NEVER go off-plan; discovered work goes in "## Discovered" and waits.
- Resolve unknowns by reading code in Analysis; remaining ambiguities become numbered questions (max 5).
- If a test fails twice for the same reason, stop and report.
- Worker runs only targeted commands named in the task. Orchestrator runs shared/global Cargo suites serially at the wave boundary.
- Plan 12 runs serially on `feature/duckdb-feature1`. Preserve any unrelated changes; do not switch branches or stage broad paths.
- DuckDB is authorized only as the pinned optional bundled dependency/build feature and feature-gated Plan 11 integration target. No DuckDB runtime selector, route, executor, production controls, or production enablement.
- Preserve test-only IIF lowering, CTAS suppression, and value mapping. Do not modify the DuckDB test source.
- Validate opt-in behavior on Linux x86_64 only. `--all-features` includes DuckDB; bundling is intended for now.
- Required small-worker model: `opencode-go/deepseek-v4.1-flash`; at most one escalation to `openai/gpt-6-luna`. Record usage in `STATE_12.md`.
- Read `AGENTS.md`, `CONTRIBUTING.md`, the assigned task section, `STATE_12.md`, ADR 0007, `docs/security/duckdb-execution-policy.md`, and the named references.
- For commit discipline, follow [git-workflow-standards](../../git-workflow-standards/SKILL.md): focused Conventional Commit, no attribution trailer, no remote action.

## [Worker] Analysis
Read the named files and grep anchors in the task spec. Inspect Git status/diff before edits. Resolve codebase facts from the repository; put remaining decisions as numbered questions (max 5) in your micro-plan.

## [Worker] Micro-planning
Write `planning/tasks/{{id}}.todo.md`:

```markdown
# {{id}} — {{title}}
## Goal
<one sentence>
## TODO items
- [ ] Small, verifiable steps including the exact targeted tests
## Notes
<facts, boundaries, and risks>
## Questions
<numbered, max 5; omit if none>
```

Output the TODO contents and stop. Wait for explicit orchestrator approval before implementation.

## [Worker] Execution
Execute the approved TODO in order and update its checkboxes. Run only the targeted commands specified by the task, including default dependency/build checks and the offline feature-enabled DuckDB integration test. Commit verified implementation changes locally in a focused commit; keep planning/state artifact changes out of that commit.

## [Worker] Reporting
Append a concise report under `## {{id}}` in `planning/STATE_12.md` (<=15 lines): status; files changed; implementation commit ID; exact targeted commands and results; deviations or "none"; known limitations; worker model/usage.

## Project references
- `AGENTS.md`
- `CONTRIBUTING.md`
- `planning/GOALS.md` (G48–G49)
- `planning/EXECUTION_PLAN_12.md`
- `planning/STATE_12.md`
- `planning/EXECUTION_PLAN_11.md` and `planning/STATE_11.md`
- `planning/EXECUTION_PLAN_10.md` and `planning/STATE_10.md`
- `docs/decisions/README.md` and `docs/decisions/0007-duckdb-opt-in-query-backend.md`
- `docs/security/duckdb-execution-policy.md`
- `tests/duckdb_pipeline_contract.rs` (read-only reference)
