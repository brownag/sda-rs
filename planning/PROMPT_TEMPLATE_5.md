# Worker Prompt Template — Orchestration 5

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_5.md`, supervised by an orchestrator that reviews your micro-plan before implementation.

## Model

Use `openai/gpt-6-luna` unless the orchestrator explicitly provides the same approved escalation model. Follow wave-orchestration retry limits.

## Task {{id}}: {{title}}

Paste the task specification verbatim from `planning/EXECUTION_PLAN_5.md`.

## Ground rules

- Do not make implementation changes or run implementation state-changing commands before your micro-plan is approved. You may create/update only `planning/tasks/{{id}}.todo.md` for review.
- After approval, execute only the approved plan. Do not touch files outside the task's "Files in scope."
- `planning/GOALS.md` is the approved active-goal source; do not alter it as part of a task.
- If an out-of-scope production change is required, stop and request a scope amendment.
- Discovered work goes under `## Discovered` in your report and waits for the orchestrator.
- If the same test fails twice for the same reason, stop and report.
- Do not run the full suite; run the task's named acceptance commands, including its targeted tests, formatting check, and diff check. The orchestrator runs configured full suites serially at wave boundaries.
- Preserve request-local connections, `spawn_blocking`, batch rollback, optional spatial support, and the existing SDA wire contract.
- Commit verified implementation locally with one focused Conventional Commit subject line, no trailers, no phase/wave/task numbers. Do not push or perform remote actions.
- Keep planning artifacts out of implementation commits. Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md).

## Analysis

Read:

1. The task spec in `planning/EXECUTION_PLAN_5.md`.
2. `planning/STATE_5.md`, especially dependency-task reports.
3. The named grep anchors, accepted ADRs, and relevant reference tests; avoid whole-file reads when anchors suffice.
4. Repository conventions in `AGENTS.md`.

## Micro-planning — stop after report

Create `planning/tasks/{{id}}.todo.md`:

```markdown
# {{id}} — {{title}}
## Goal
<one sentence>
## TODO items
- [ ] <small, verifiable step, including targeted test>
## Notes
<facts, constraints, risks>
## Questions
<numbered, max 5; omit if none>
```

Keep the micro-plan to at most 12 steps, scoped to allowed files, with exact acceptance commands. Output its full contents and stop. Do not implement until the orchestrator sends:

> APPROVED -- execute the plan as written. [Answers to any questions from their plan.]

## Execution

After approval, perform TODO items in order and mark them complete. Run the acceptance commands named in the task spec. Commit verified implementation locally. Keep planning-state changes out of the code commit.

## Reporting

Append under `## {{id}}` in `planning/STATE_5.md` (15 lines maximum): status; changed files and line ranges; code commit ID; exact test commands and pass/fail counts; deviations or "none"; dependency-relevant gotchas. For spatial tests, explicitly report whether configured SpatiaLite initialized and real spatial queries executed.
