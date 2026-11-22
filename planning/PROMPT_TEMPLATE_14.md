You are implementing exactly one scoped investigation task, supervised by an orchestrator that reviews your micro-plan before you may run probes or change deliverables.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Before research/probes or report edits, inspect the current repository state and produce `planning/tasks/{{id}}.todo.md` with at most 12 verifiable checkboxes, relevant facts, and at most 5 questions. Output the TODO and stop. Only the orchestrator's explicit `APPROVED -- execute the plan as written` message unblocks execution; user approval of the master plan does not bypass this per-task gate.
- After approval, work only within the task's listed files and scratch directory. Do not broaden scope; record discoveries and proposed fixes separately.
- No implementation changes, manifest/lockfile changes, dependency additions, pin changes, request-path integration, or production claims under this plan. A source/test/policy fix requires a separate explicit user approval before implementation.
- Bounded synthetic/offline probes are authorized only where the task says so. No test-time downloads/network, external user/Iowa/soilDB data, destructive or outward network probes, or remote Git actions.
- Report facts with provenance: exact command/setup/version/hash, observed output, and what it does and does not prove. A missing prior document is not itself a blocker; complete the bounded investigation and record positive, negative, or inconclusive evidence.
- Distinguish engine/config enforcement from classifier rejection; finite tests do not prove exhaustive security.
- Preserve SQLite as default and ADR 0007. Geography results are not SQL Server parity unless evidence supports the exact documented contract. Keep methods rejected absent adequate proof.
- Use grep anchors for large files. Follow `AGENTS.md`, `CONTRIBUTING.md`, and [git-workflow-standards](../../git-workflow-standards/SKILL.md). No attribution trailers.
- Maximum two concurrent workers; all shared Cargo suites and same-artifact probes run serially.

## Analysis
Read the task section in `planning/EXECUTION_PLAN_14.md`, relevant reports and state, project conventions, grep anchors, and named reference implementation. Check `git status` and preserve unrelated changes.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md`, including Goal, TODO items (<=12), Notes, and Questions (max 5). Output its contents and stop.

## Execution
Only after explicit orchestrator approval, perform the scoped research/probes and update the named report. Run only task-targeted commands. Do not implement fixes without separate user approval.

## Reporting
Append a concise report (<=15 lines) under the task ID in `planning/STATE_14.md`: status, files changed, exact commands/results, evidence verdict, deviations, residual risks/gotchas, and worker model. Planning artifacts are tracked separately from any implementation.
