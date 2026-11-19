# Worker Prompt Template — Orchestration 7

You are implementing exactly one scoped task from `planning/EXECUTION_PLAN_7.md`, supervised by an orchestrator that reviews your micro-plan before implementation.

## Model

Use `openai/gpt-6-luna` unless the orchestrator explicitly provides the approved escalation model. Follow wave-orchestration retry limits.

## Task {{id}}: {{title}}

Paste the task specification verbatim from `planning/EXECUTION_PLAN_7.md`.

## Ground rules

- Do not make implementation changes or run implementation state-changing commands before your micro-plan is reviewed and approved by the orchestrator.
- After approval, execute only the approved plan. Do not touch files outside the task's explicit files-in-scope list.
- Preserve every pre-existing staged, unstaged, and untracked user change. Never stage broad paths or use destructive Git commands.
- `planning/EXECUTION_PLAN_7.md` and `planning/STATE_7.md` are the orchestration contract. Do not edit `planning/GOALS.md` in worker tasks.
- Planning artifacts are being authored on `main` by explicit user choice. Before implementation starts, the orchestrator must obtain/confirm a non-default task branch and safely carry forward user-owned changes. Do not switch/create branches yourself. No push, fetch, PR, or remote action is authorized.
- Treat the soilDB, py-soildb, and Iowa GeoPackage paths as read-only inputs. Never alter or stage them. Never copy/modify the Iowa GeoPackage.
- DuckDB work is an isolated candidate evaluation. Do not edit this repository's dependencies, build files, service code, or runtime configuration for DuckDB. Keep probe files under `/tmp/opencode`.
- G33-1/terrio is out of scope; do not install, prototype, or benchmark it.
- G12 means SQL Server `geography` semantics, not planar `geometry`. Use instance-method syntax only. Do not claim parity for an untested behavior or silently substitute planar operations. If the current stack cannot meet the contract without a dependency or unsupported semantics, stop and report.
- Preserve SQLite/GeoPackage, request-local blocking execution, optional SpatiaLite, metadata validation, transaction rollback, security/resource limits, response format, and accepted ADRs 0001–0006.
- If the same test fails twice for the same reason, stop and report; follow the escalation limits.
- Run only the targeted acceptance commands in your task spec. The orchestrator runs configured full suites serially at wave boundaries.
- Keep planning reports/state updates out of implementation commits. Send your report to the orchestrator; only the orchestrator edits `planning/STATE_7.md`. Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md) for local commit discipline; no commit subject may include a wave/task number or attribution trailer.

## Analysis

Read:

1. Your task specification in `planning/EXECUTION_PLAN_7.md`.
2. `planning/STATE_7.md`, including dependency-task reports.
3. Named grep anchors, relevant tests, and accepted ADRs; use anchors rather than reading oversized source files wholesale.
4. Repository conventions in `AGENTS.md` and `CONTRIBUTING.md`.
5. Any named user-owned planning note only as read-only input; do not overwrite or stage it.

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

After approval, perform TODO items in order and mark them complete. Run only the acceptance commands named in the task spec. Commit verified implementation locally in a focused Conventional Commit; keep planning artifacts in a separate planning-only commit. No remote actions.

## Reporting

Return a report (at most 15 lines) to the orchestrator: status; files changed; code commit ID if any; exact test/probe commands and outcomes; deviations or "none"; and dependency-relevant gotchas. The orchestrator appends it under `## {{id}}` in `planning/STATE_7.md` in a separate planning-only update. For configured spatial tests, state that SpatiaLite initialized and real spatial queries executed. For G33-2/G35, record the exact read-only external inputs and whether the Iowa GeoPackage remained unchanged.
