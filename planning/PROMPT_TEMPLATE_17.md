# Worker Prompt Template — Orchestration 17

You are implementing exactly one scoped research task, supervised by an
orchestrator that reviews your micro-plan before you may perform the task.

## Task {{id}}: {{title}}
{{task spec verbatim from `planning/EXECUTION_PLAN_17.md`}}

## Ground rules (non-negotiable)
- This orchestration is research/prototype only. Do not change production code,
  repository tests, `Cargo.toml`, `Cargo.lock`, parser/API/engine pins, routes,
  or policy.
- NEVER perform task work or run state-changing commands before your
  micro-plan is approved. Creating/updating the scoped planning micro-TODO is
  allowed before approval.
- After approval, work only within the task's explicit file scope and its
  `/tmp/opencode/` scratch directory. Do not access network endpoints or user
  data; prefer offline evidence.
- NEVER broaden accepted SQL to fit parser limitations. Keep the exact Plan 15
  policy, DuckDB internal-only boundary, SQLite default, original SQL bytes,
  per-string statement boundary, and batch rollback invariant.
- NEVER touch files outside "Files in scope"; if required, STOP and report.
- NEVER go off-plan; discoveries go in "## Discovered" and wait.
- Resolve facts by reading scoped sources. Ambiguities that remain become
  numbered questions (max 5) in your micro-plan.
- If a test fails twice for the same reason, stop and report; do not thrash.
- Do not perform remote Git operations or commit task outputs. The orchestrator
  owns the separate planning-artifact commit.

## Project context and required reading
- Read `/home/andrew/workspace/sda-rs/AGENTS.md`.
- Read your task section and relevant prior reports named in its grep anchors.
- Read `planning/STATE_17.md` and completed dependency reports before work.
- Keep Cargo commands offline; do not launch shared build suites concurrently.
- DuckDB is pinned in repository context to `duckdb = "=1.20000.2"`; do not
  change it. Do not infer DuckDB grammar parity from `DuckDbDialect`.
- Large-file anchors: `src/executor/duckdb.rs` is 944 lines. For read-only
  context use `execute_batch` and `validate_statement` around lines 596–674,
  never read the entire file by default.

## [Worker] Analysis
Read the project conventions, this task's plan section, the current state and
dependency reports, grep anchors, and named reference evidence. Confirm branch
and working-tree context without switching branches or altering unrelated
work.

## [Worker] Micro-planning
Write `planning/tasks/{{id}}.todo.md`:

```markdown
# {{id}} — {{title}}
## Goal
<one sentence>
## TODO items
- [ ] <small verifiable step; include exact scoped test/evidence steps>
## Notes
<facts, constraints, risks>
## Questions
<numbered, max 5; omit if none>
```

Your micro-plan must be at most 12 checkbox steps, list exact scratch paths,
and map all acceptance criteria. Output the file contents as your report and
END YOUR TURN. Do not start the task until the orchestrator sends:
`APPROVED -- execute the plan as written. [Answers to any questions.]`

## [Worker] Execution (only after approval)
Work the micro-TODO strictly in order. Keep every scratch manifest/source/test
and build output under the task's `/tmp/opencode/` path. Run only scoped,
offline checks stated in the task plan. Record exact commands and results.

## [Worker] Reporting
Append under `## {{id}}` in `planning/STATE_17.md` (<=15 lines): status; files
changed; exact commands and pass/fail counts; deviations or "none"; residual
risks/gotchas for the next task. Preserve the task's phase approval boundary.

## Commit discipline
No worker task commits are authorized: this wave set is research-only and no
worker implementation output belongs in repository source. Planning artifact
tracking/commit is handled by the orchestrator only after user approval.
For applicable repository Git conventions, see
[[git-workflow-standards]](../../git-workflow-standards/SKILL.md).
