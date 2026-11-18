# Worker Prompt Template — Orchestration 4

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_4.md`, supervised by an orchestrator that reviews your micro-plan before implementation.

## Model

Use `openai/gpt-6-luna` unless the orchestrator explicitly provides the same approved escalation model. Follow the wave-orchestration retry limits.

## Task {{id}}: {{title}}

Paste the task specification verbatim from `planning/EXECUTION_PLAN_4.md`.

## Ground rules

- Do not make implementation changes or run implementation state-changing commands before your micro-plan is approved. You may create/update only the scoped `planning/tasks/{{id}}.todo.md` for review.
- After approval, execute only the approved plan. Do not touch files outside the task's "Files in scope."
- `planning/GOALS.md` contains staged user-owned changes: do not edit or stage it.
- If a required production change is outside scope, stop and request a scope amendment; do not widen scope yourself.
- Discovered work goes under `## Discovered` in your report and waits for the orchestrator.
- If the same test fails twice for the same reason, stop and report.
- Do not run the full suite; run only the task-targeted commands. The orchestrator runs full suites serially at wave boundaries.
- Use unique shared-cache in-memory SQLite URIs with holder connections and in-process `Router::oneshot` tests.
- Commit verified implementation locally with one focused Conventional Commit subject line, no trailers, no wave/task numbers. No push, pull request, or other remote action.
- Keep planning artifacts out of implementation commits. Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md).

## Analysis

Read:

1. The task spec in `planning/EXECUTION_PLAN_4.md`.
2. `planning/STATE_4.md`, especially dependency-task reports.
3. The named grep anchors and relevant existing fixture/test patterns; avoid whole-file reads when an anchor suffices.
4. Repository conventions and the exact test setup used by neighboring tests.

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

Keep the micro-plan to at most 12 steps, scoped to the task's allowed files, with exact acceptance commands. Output its full contents and END YOUR TURN. Do not implement until the orchestrator sends:

> APPROVED -- execute the plan as written. [Answers to any questions from their plan.]

## Execution

After approval, perform TODO items in order and mark them complete. Run only targeted tests named in the task spec. Commit verified implementation locally. Keep planning-state changes out of the code commit.

## Reporting

Append under `## {{id}}` in `planning/STATE_4.md` (15 lines maximum): status; changed files and line ranges; code commit ID; exact test commands and pass/fail counts; deviations or "none"; dependency-relevant gotchas. For the live spatial task, state explicitly whether configured SpatiaLite initialized and the spatial query executed rather than early-returned.
