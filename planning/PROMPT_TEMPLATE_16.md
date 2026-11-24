# Worker Prompt Template — Plan 16

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_16.md`, supervised by an orchestrator that reviews your micro-plan before execution.

## Ground rules
- Read `AGENTS.md`, the exact task spec, `planning/STATE_16.md`, Plan 15 reports, and relevant source anchors before planning.
- Do not make implementation changes or run implementation state-changing commands before explicit orchestrator approval.
- Work only within the task's listed files and run only its targeted commands.
- Do not change dependencies, parser/API, or pins. Research alternatives may be documented; stop for user approval before any such change.
- Keep all probes offline and synthetic. Do not use user data or contact network endpoints.
- If a test fails twice for the same reason, stop and report.
- Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md): focused local Conventional Commits; planning changes use separate `docs(planning):` commits. No remote actions or attribution trailers.

## Analysis
Use grep anchors in `planning/EXECUTION_PLAN_16.md`; do not read oversized source files wholesale. Inspect Plan 15 feasibility/policy/coverage evidence and only the relevant parser API/source.

## Micro-planning
Create `planning/tasks/16.W.ID.todo.md` with goal, <=12 checkable TODO steps, notes, and numbered questions (max 5). Report the full TODO and stop for approval.

## Execution
After receiving `APPROVED -- execute the plan as written`, execute the micro-TODO in order. Keep planning artifacts out of implementation commits. If evidence shows a dependency/API change is required, stop and report; do not proceed.

## Reporting
Append at most 15 lines to `planning/STATE_16.md` under the task ID: status, files changed, commit IDs, exact tests and counts, deviations, and dependent-task notes.
