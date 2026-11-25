# Execution Plan 17 — Qualify `sqlparser` for DuckDB Policy Prototyping

## Goal & Scope

Determine whether `sqlparser` with its public `DuckDbDialect` and AST visitor can safely support Plan 15's constrained DuckDB SQL policy. Evaluate the documented research assumptions, build an isolated prototype, and report whether it establishes adequate syntax compatibility and recursive policy coverage. This is scratch research only; the result does not authorize a production gate or any dependency/API/pin change.

**In scope**
- Convert the prior reports and approved policy into an exact, checkable parser and AST coverage corpus.
- Select and inspect an exact `sqlparser` release in isolated scratch space; test public API and actual release source.
- Build and exercise a scratch-only visitor, including recursive syntax-bearing nodes, statement payloads, raw/unsupported syntax, and fail-closed behavior for unhandled/new nodes.
- Differentially compare parser decisions with the repository's pinned DuckDB 2.0.0 runtime on existing local/synthetic fixtures, keeping exact original SQL bytes as execution input.
- Deliver an evidence-based pass/fail report, gaps, residual risks, and proceed/stop recommendation.

**Out of scope**
- Executing any production prototype, changing application/production code or tests, adding dependencies, or editing `Cargo.toml`/`Cargo.lock`.
- Replacing/changing parser APIs, changing dependency or DuckDB engine pins, changing routes, policy, batch behavior, or the DuckDB internal-only / SQLite-default boundary.
- Broadening the accepted SQL policy to accommodate a candidate parser.
- Network endpoints, user data, external data, remote Git actions, or scratch files outside `/tmp/opencode/`.
- Any production authorization-gate work. Dependency addition, parser/API replacement, pin change, and production gate work each require separate explicit approval.

**Verbatim user constraints**
- “Research and prototype only; no production code, repository dependency, parser/API, engine pin, route, or policy changes.”
- “No network endpoints or user data. Prefer cached/offline evidence; if external documentation must be consulted, cite sources and distinguish documented guarantees from assumptions.”
- “Use scratch space only under `/tmp/opencode/`.”
- “Preserve DuckDB’s internal-only boundary, SQLite as default, existing batch rollback, and the approved policy. Do not broaden allowed SQL to fit a parser.”
- “Follow wave-orchestration’s task sizing, explicit file scopes, disjoint wave files, micro-plan approval gates, test rules, and planning-state/commit conventions.”
- “Verify current Git/worktree state during reconnaissance; do not switch branches or perform remote actions.”
- “Do not execute the prototype or change implementation code/dependencies; stop after presenting the plan for my approval.”

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-feature1`, HEAD `44aa8ab` (`docs(planning): research DuckDB parser API alternatives`); working tree clean at planning intake.
- Existing linked worktrees were listed; do not modify, switch, or remove them. Main worktree remains the target.
- Plan 16 and its state show the `polyglot-sql` downstream recursive-coverage attempt stopped: public visitor callbacks alone did not prove every policy-relevant child is visited.
- Prior reports, all required inputs: `planning/reports/duckdb-parser-api-alternatives.md`, `duckdb-ast-guard-feasibility.md`, `duckdb-parser-policy.md`, `duckdb-parser-feasibility.md`, `duckdb-ast-coverage.md`, and `duckdb-tsql-pipeline-evaluation.md`.
- `Cargo.toml` has optional `duckdb = "=1.20000.2"` (bundled engine v2.0.0) and `polyglot-sql = "0.14"`; no `sqlparser` entry was observed in `Cargo.lock`. Never alter repository dependency files. `src/executor/duckdb.rs` is 944 lines; use grep anchors around `execute_batch` (596–638) and `validate_statement` (current entry around 641), not whole-file reads. Existing security contract has 5 focused tests in `tests/duckdb_security_contract.rs`.
- Accepted ADR 0007 preserves SQLite default, DuckDB opt-in/internal boundary, request-local state, and rollback; this plan does not supersede it.
- Planning artifacts are tracked and have been committed separately in prior plans. Plan 17 artifacts are to be tracked and committed separately after plan approval.
- No production baseline suite is required for this research-only plan. Record existing focused baseline as prior evidence, not a fresh run: Plan 16 reports `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 5 passed. Prototype-only commands and differential runtime checks belong to scratch; run serially if they share build resources.
- `/tmp` previously filled during a bundled DuckDB scratch rebuild (Plan 15); prefer reuse of existing local target/fixtures and avoid redundant builds. No repository writes from scratch tasks.
- `.gitignore` ignores only `/target/`; scratch is explicitly restricted to `/tmp/opencode/`.

## Decisions

- **2026-11-24 — Candidate:** Evaluate `sqlparser` public `DuckDbDialect` plus public AST visitor as a candidate, not as presumed parity/completeness.
- **2026-11-24 — Authorization boundary:** Keep the exact Plan 15 policy and source-byte correspondence rule; unsupported syntax remains rejected rather than broadening policy.
- **2026-11-24 — Approval boundary:** Research/prototype/report only. No dependency, API, engine pin, production implementation, or authorization gate change is authorized.
- **2026-11-24 — Environment:** Prefer cached/offline release source and runtime evidence; any consulted external docs must be cited and their claims separated from tested evidence. No network endpoints or user data.
- **2026-11-24 — Artifacts:** Tracked planning files with separate planning-only commit, inherited from Plans 15–16.
- **2026-11-24 — Parallelism:** Waves are sequential phases; no concurrent shared Cargo/DuckDB builds. Scratch paths are task-specific under `/tmp/opencode/`.

## Dependency graph and wave table

| Wave | Task(s) | Purpose / dependency |
|---|---|---|
| 1 | 17.1.1 | Freeze exact policy corpus and checkable acceptance/coverage criteria from prior evidence; no prototype yet. |
| 2 | 17.2.1 | Choose and verify an exact `sqlparser` release and inspect its actual public API/source in an isolated offline-capable harness; depends on 17.1.1. |
| 3 | 17.3.1 | Implement scratch-only visitor and test syntax, recursive coverage, payloads, raw/unknown behavior, and original bytes; depends on 17.2.1. |
| 4 | 17.4.1 | Differentially compare the prototype against pinned DuckDB 2.0.0 using only local/synthetic fixtures; report results and recommendation; depends on 17.3.1. |

Waves are intentionally serial: the corpus constrains release evaluation; verified release API constrains the prototype; prototype findings and runtime comparison jointly inform the final report. Within a wave there is one task, so file-scope disjointness holds trivially.

## Task specifications

### Task 17.1.1 — Freeze policy corpus and prototype criteria
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** none.
- **Files in scope — must edit:** `planning/reports/duckdb-sqlparser-prototype-criteria.md`.
- **Can create:** none; analysis only.
- **Do not touch:** source, tests, Cargo files/lockfile, policy, routes, ADRs, other reports, and scratch prototype files.
- **Spec:** Convert every row of the exact Plan 15 policy table into a finite, traceable corpus and visitor-coverage checklist. Include exactly-one parsed statement per SQL string, distinct vector-batch semantics/rollback, classification of the exact original SQL bytes that remain execution input, and recursive coverage requirements for statements and payloads, query/select, CTE/set operations, joins/relations/table factors, expressions, scalar and table functions, temp CREATE/INSERT/SELECT/DROP, and unknown/raw/unsupported syntax. Preserve distinctions among conditional, narrow, deny-by-default, and deny outcomes. Identify where policy text leaves runtime compatibility contingent (ordinary relation resolution, narrow VALUES form); do not invent allowances. Criteria must provide case IDs and expected allow/deny/conditional outcomes traceable to `duckdb-parser-policy.md`.
- **Grep anchors:** Policy matrix and batch/source rules in `planning/reports/duckdb-parser-policy.md`; recommendation and explicit criteria in `planning/reports/duckdb-parser-api-alternatives.md`; relevant compatibility evidence in `duckdb-parser-feasibility.md` and `duckdb-tsql-pipeline-evaluation.md`.
- **Acceptance:** Every policy row and every user-required criterion maps to at least one explicit case/check or clearly marked conditional check; exact byte/batch boundaries are explicit; `git diff --check`. No code or scratch harness.

### Task 17.2.1 — Verify exact release and public API
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** 17.1.1.
- **Files in scope — must edit:** `planning/reports/duckdb-sqlparser-release-evidence.md`.
- **Can create:** `/tmp/opencode/sda-rs-plan17-release/` including isolated manifest, source/API probe, and build output.
- **Do not touch:** repository source/tests/Cargo.toml/Cargo.lock, policy, routes, ADRs, criteria report, or other reports.
- **Spec:** Establish one exact `sqlparser` release supported by available cached sources (or document why offline evidence is unavailable and stop for user direction). Inspect that release's actual source/API and manifest, including `DuckDbDialect`, visitor feature/API, callback traversal claims, statement representation/count behavior, and relevant AST nodes. Build/check a minimal scratch consumer against an exact version without resolving or editing repository Cargo files. Record checksum/source provenance where available, exact commands, API facts, documented guarantees versus assumptions, and limitations for fields/new variants/raw SQL. No latest-doc substitution for release source. Do not fetch from network; if a source is unavailable offline, cite any already-available documentation and mark evidence incomplete rather than fetching.
- **Grep anchors:** Criteria report from 17.1.1; prior candidate evidence/assumptions in `planning/reports/duckdb-parser-api-alternatives.md`; `Cargo.toml` and lockfile only as read-only repository context.
- **Acceptance:** Exact release is named and source/API inspected, or task ends with a clear offline blocker; a scratch exact-version compile/check result and reproducible commands are recorded when possible; guarantees and assumptions are separated; repository dependency files remain unchanged; `git diff --check`.

### Task 17.3.1 — Build and test scratch-only recursive visitor
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** 17.2.1 with a verified usable release/API.
- **Files in scope — must edit:** `planning/reports/duckdb-sqlparser-visitor-prototype.md`.
- **Can create:** `/tmp/opencode/sda-rs-plan17-visitor/` only, containing isolated manifest, prototype source, corpus/tests, and build output.
- **Do not touch:** repository source/tests/Cargo files/lockfile, policy, routes, ADRs, criteria/release reports, or other reports.
- **Spec:** Implement a scratch visitor/classifier against the exact verified release and criteria corpus. Exercise accepted and rejected cases, nested queries/expressions/relations/functions, CTEs, set operations, joins, temp lifecycle and statement payloads; probe unsupported/raw syntax and unhandled/new node behavior. Demonstrate parser statement-count handling and retain the exact input `String`/bytes as the designated execution input (never execute generated SQL). Audit actual visitor traversal/source for policy-relevant fields and document exactly what the public visitor guarantees versus what is manually/procedurally verified. The prototype is an evaluation artifact, not production authorization; do not alter policy to fit parser behavior. If required syntax or recursive coverage cannot be established, record failures and stop rather than devising a permissive fallback.
- **Grep anchors:** Criteria report, release evidence report, exact source/API under scratch Cargo registry; policy rows in `planning/reports/duckdb-parser-policy.md`.
- **Acceptance:** Scratch tests assert each criterion's expected parser/classifier outcome, exactly-one statement behavior, exact-byte preservation, nested traversal and fail-closed unsupported/raw/unhandled handling to the extent constructible; report documents uncovered fields/variants and reproducible offline commands; no repository code/dependency changes; `git diff --check`.

### Task 17.4.1 — Compare with DuckDB 2.0.0 and report recommendation
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** 17.3.1.
- **Files in scope — must edit:** `planning/reports/duckdb-sqlparser-duckdb-differential.md`.
- **Can create:** `/tmp/opencode/sda-rs-plan17-differential/` only, reusing existing targets/fixtures where practical.
- **Do not touch:** repository source/tests/Cargo files/lockfile, policy, routes, ADRs, criteria/release/visitor reports, or other reports.
- **Spec:** Run the criteria corpus against the scratch parser and the repository's pinned DuckDB v2.0.0 runtime using only existing local/synthetic fixtures and offline resources. Compare required syntax and dangerous constructs, separating parse acceptance from runtime acceptance/binding and policy decision. Keep original SQL bytes as runtime input; do not compare execution of generated/reformatted SQL. Preserve per-string statement validation and do not weaken batch rollback; do not access network, real external files, or user data. Provide case-by-case pass/fail/unknown, visitor and syntax gaps, evidence commands, residual risks, and a clear proceed/stop recommendation. Proceed means only “candidate merits a separately approved next step,” never production authorization.
- **Grep anchors:** Criteria corpus; visitor prototype report and scratch harness; `src/executor/duckdb.rs` `execute_batch`/`validate_statement` around lines 596–674 (read-only); `tests/duckdb_security_contract.rs`; prior pinned runtime evidence in `planning/reports/duckdb-parser-feasibility.md` and `duckdb-security-probe-results.md`.
- **Acceptance:** Report contains full corpus outcome matrix, exact pinned DuckDB runtime identity, runtime/parser distinctions, original-byte execution-input evidence, coverage gaps/residual risks, reproducible offline commands, and unambiguous proceed/stop recommendation; no policy or repository implementation/dependency changes; `git diff --check`.

## Orchestrator gates

- Each task starts with a <=12-step micro-TODO and stops for orchestrator approval before execution. Scratch outputs are scoped only to that task's `/tmp/opencode/` path.
- If exact release source/API cannot be verified without network, stop at 17.2.1 and report the blocker; do not fetch or substitute latest docs.
- A failed or incomplete recursive coverage result is evidence to stop the candidate, not a reason to relax the approved SQL policy.
- Never run shared Cargo/DuckDB builds concurrently; prefer offline cache and existing repository build artifacts, mindful that prior scratch DuckDB rebuild exhausted `/tmp`.
- No implementation gates or full-suite runs are in scope. At completion, verify only research artifacts and `git diff --check`; do not execute production code changes (none are allowed).
- No remote Git operations. After user approval, planning artifacts are tracked in a separate `docs(planning):` commit; scratch files remain under `/tmp/opencode/` and are not committed.
- Final report must explicitly state that dependency addition, parser/API replacement, pin changes, and production authorization-gate work await separate approval.
