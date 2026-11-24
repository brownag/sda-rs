# Worker Prompt Template — Plan 15

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_15.md`, supervised by an orchestrator that reviews your micro-plan before execution.

## Ground rules

- Read `AGENTS.md`, `CONTRIBUTING.md`, the exact task spec, and relevant prior reports/state before planning.
- Do not make implementation changes or run state-changing commands before the orchestrator approves your micro-TODO.
- After approval, work only within the task's listed files and run only its targeted commands.
- Do not add dependencies, change pins, expose DuckDB to requests, or perform remote actions.
- Stop and report if parser feasibility requires an unapproved dependency or any out-of-scope source/policy change.
- Keep probes offline and synthetic. Do not use user/Iowa data or contact network endpoints.
- If a test fails twice for the same reason, stop and report.
- Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md): make focused local Conventional Commits for verified implementation units; planning changes use separate `docs(planning):` commits. No pushes, fetches, pulls, PRs, or attribution trailers.

## Analysis

Use grep anchors in `planning/EXECUTION_PLAN_15.md`; do not read oversized source files wholesale. Identify existing parser and DuckDB contract patterns before planning.

## Micro-planning

Create `planning/tasks/15.W.ID.todo.md` with:

```markdown
# 15.W.ID — <task title>
## Goal
<one sentence>
## TODO items
- [ ] <small verifiable step>
## Notes
<constraints and facts>
## Questions
<numbered questions, or None>
```

Use no more than 12 checklist steps. Report the full micro-TODO and stop for orchestrator approval.

## Execution

After receiving the explicit message `APPROVED -- execute the plan as written`, execute the micro-TODO in order. Keep planning artifacts out of implementation commits. Do not begin another task.

## Reporting

Append at most 15 lines to `planning/STATE_15.md` under the task ID: status, files changed, commit ID(s), exact tests and counts, deviations, and dependent-task notes.
