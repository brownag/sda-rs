# Worker Prompt Template — Orchestration 2

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_2.md`, supervised by an orchestrator that reviews your micro-plan before implementation.

## Model

Use `openai/gpt-6-luna` for every worker task and any escalation/retry. Do not substitute another provider or model.

## Task {{id}}: {{title}}

Paste the task specification verbatim from `planning/EXECUTION_PLAN_2.md`.

## Ground rules

- Do not make implementation changes or run implementation state-changing commands before your micro-plan is approved. You may create/update only the scoped `planning/tasks/{{id}}.todo.md` for review.
- After approval, execute only the approved plan and task files. Do not touch files outside the task's “Files in scope.”
- If a required shared file is outside scope, stop and report the conflict; do not widen scope yourself.
- Resolve facts by inspecting the named files and installed crate sources. Do not guess `polyglot-sql` AST fields or dialect behavior.
- Discovered work goes under `## Discovered` in your report and waits for the orchestrator.
- If the same test fails twice for the same reason, stop and report.
- Do not run full/shared test suites; run only task-targeted commands. The orchestrator runs full suites serially at wave boundaries.
- Local focused commits are expected after verified task completion. No push, pull request, issue, or other remote action is authorized.
- Follow the repository's commit workflow; see `[[git-workflow-standards]](../../git-workflow-standards/SKILL.md)` for Conventional Commit discipline (one sentence, no phase/wave numbers). Planning-only changes must not be included in implementation commits.

## Analysis

Read:

1. The task spec in `planning/EXECUTION_PLAN_2.md`.
2. `planning/STATE_2.md`, especially dependency-task reports.
3. The anchors listed in the task spec. Do not read whole files over 150 lines unless the anchor is ambiguous.
4. Existing project conventions visible in neighboring code.

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
<numbered questions, max 5; omit if none>
```

Keep the micro-plan to at most 12 steps, scoped to the task's allowed files, and include exact acceptance commands. Output its full contents and END YOUR TURN. Do not implement until the orchestrator sends:

> APPROVED -- execute the plan as written. [Answers to any questions from their plan.]

## Execution

After approval, perform TODO items in order and mark them complete. Run only targeted tests named in the task spec. Commit each verified, reviewable logical task locally using a focused Conventional Commit. Keep planning-state changes out of code commits.

## Reporting

Append under `## {{id}}` in `planning/STATE_2.md` (15 lines maximum): status; changed files/line ranges; code commit ID; exact test commands and pass/fail; deviations or “none”; dependency-relevant gotchas.
