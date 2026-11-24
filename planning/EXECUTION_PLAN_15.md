# Execution Plan 15 — Parser-Backed DuckDB SQL Authorization

## Goal & Scope

Establish whether the existing `polyglot-sql` parser can support a fail-closed SQL authorization gate for the internal DuckDB executor, specify a reviewed allowlist, implement the gate only if the existing dependency can parse the required forms, and adversarially qualify its behavior on the pinned DuckDB build. Reassess security readiness without exposing DuckDB to HTTP requests or claiming exhaustive security from finite tests.

**In scope**
- Evaluate `polyglot-sql` parsing and AST support against the DuckDB executor's required SELECT and temporary-table workflows.
- Specify an explicit allowlist for statement, expression, relation, scalar-function, and table-function forms; reject unreviewed functions and external-I/O sources.
- Implement parser-backed validation using existing dependencies only, if feasibility is established and the approved forms are representable.
- Test accepted and rejected SQL against the pinned DuckDB executor and direct-engine observations; update security policy and readiness evidence.

**Out of scope**
- Adding or changing dependencies, Cargo/DuckDB/Spatial pins, or downloading/building an alternate parser or engine without separate explicit user approval.
- Exposing DuckDB to HTTP/request-controlled SQL, adding a route/backend selector, changing SQLite defaults, or production enablement.
- OS sandboxing, production resource caps, broad SQL compatibility expansion, or asserting complete security based solely on parser use or finite tests.
- Changing wire/response behavior or enabling unreviewed SQL functions/table functions.

**Verbatim user constraints and decisions**
- User asked whether parser use could make the interim DuckDB gate more secure, then requested a prioritized series of goals and approved this plan for G50.
- Existing parser first: test `polyglot-sql`; any parser dependency is a separate approval decision.
- Function policy: allow selected reviewed functions/table functions; reject unknown functions and external-I/O sources.
- Execute tasks serially; target tests per wave and run full suites at the end.
- Planning artifacts are tracked. Use current branch `feature/duckdb-feature1`.

## Reconnaissance

- Repo: `/home/andrew/workspace/sda-rs`; branch `feature/duckdb-feature1`, HEAD `9c68514`; clean at plan intake. Existing worktrees are retained and must not be modified.
- Next orchestration number is 15. Planning artifacts are tracked by user decision.
- `src/executor/duckdb.rs` (944 lines) contains private `validate_statement`; treat it as oversized and use grep anchors. It currently admits SELECT by prefix plus textual denials and narrowly recognizes temp-table operations.
- `src/transpiler/` already uses `polyglot-sql` 0.14, including T-SQL parsing and AST traversal (`src/transpiler/mod.rs`, `security.rs`, `spatial.rs`). Feasibility must establish actual AST/API behavior rather than infer that T-SQL parsing is suitable for DuckDB SQL.
- DuckDB is optional and pinned to crate `=1.20000.2`, bundled engine 2.0.0. DuckDB contracts require `--features duckdb`; no dependency changes are authorized here.
- Reference tests: `tests/duckdb_security_contract.rs`, `tests/duckdb_execution_contract.rs`, and `tests/duckdb_pipeline_contract.rs`. Existing report `planning/reports/duckdb-security-probe-results.md` records direct-engine acceptance of settings changes, `ATTACH`, DDL, compound statements, comments, and quoting versus executor-path rejection.
- `docs/security/duckdb-execution-policy.md` states parser-backed complete classification and bypass resistance as a blocker. `planning/reports/duckdb-secure-integration-assessment.md` recommends no request exposure of the current executor.
- Accepted ADR 0007 preserves SQLite default and bars silent fallback; no ADR change or request selector is in scope.
- Baseline already passed during the preceding review-fix work: `cargo fmt --check`; default `cargo test` (75 unit + 30 integration); configured-SpatiaLite `cargo test` (75 + 30); `CARGO_NET_OFFLINE=true cargo test --offline --all-features` (79 unit + all integration targets, including DuckDB contracts); focused DuckDB security (5), execution (4), and result (4) targets. Re-run final gates serially after implementation.
- No network access, user/Iowa data, artifact provisioning, or parallel Cargo suite execution.

## Decisions

- **2026-11-23 — Goal ordering:** G50 parser feasibility precedes G51 policy, G52 implementation, G53 adversarial qualification, and G54 readiness reassessment.
- **2026-11-23 — Parser:** Evaluate existing `polyglot-sql` first. If a new dependency is required, stop and present evidence/options for separate approval; do not add it in this plan.
- **2026-11-23 — SQL policy:** Ordinary table reads plus specifically reviewed scalar and table functions; reject unknown functions and external-I/O sources. Exact names and AST semantics are decided in G51 from feasibility evidence.
- **2026-11-23 — Runtime boundary:** DuckDB remains internal-only; SQLite remains default. No request selector, HTTP route, production exposure, or readiness claim.
- **2026-11-23 — Execution:** Serial tasks; targeted tests in each implementation/qualification task; full default, SpatiaLite, formatting, and all-features gates at the end.
- **2026-11-23 — Planning artifacts:** Track Plan 15 artifacts separately from implementation commits.
- **2026-11-23 — Worker model:** Use `opencode-go/deepseek-v4.1-flash` for tasks, with at most one escalation to `openai/gpt-6-luna`.
- **2026-11-23 — ADRs:** No accepted ADR is superseded. Preserve ADR 0007 and its SQLite-default/no-silent-fallback contract.

## Dependency graph and wave table

| Wave | Task(s) | Purpose / dependency |
|---|---|---|
| 1 | 15.1.1 | Establish `polyglot-sql` feasibility and concrete grammar/AST gaps; no implementation. |
| 2 | 15.2.1 | Specify an explicit AST authorization policy from the feasibility report. |
| 3 | 15.3.1 | Implement the parser-backed gate and focused regressions if the existing parser meets the policy needs; otherwise stop for separate dependency/user decision. |
| 4 | 15.4.1 | Adversarially qualify parser/runtime behavior and update security policy/readiness evidence. |

All tasks are serial because each task's findings constrain the next. Cargo tests run serially.

## Task specifications

### Task 15.1.1 — Assess existing parser feasibility for DuckDB authorization
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** none.
- **Files in scope — must edit:** `planning/reports/duckdb-parser-feasibility.md`.
- **Can create:** bounded scratch examples under `/tmp/opencode/sda-rs-plan15-parser/`; no repository parser harness unless listed in the report as a proposed follow-up.
- **Do not touch:** source, tests, Cargo files/lockfile, security policy, ADRs, routes/configuration, other plans/reports, or remote resources.
- **Spec:** Map actual executor SQL workflows to the existing `polyglot-sql` API/AST. Probe simple and nested SELECT, CTEs, joins, scalar and table functions, comments/quoted identifiers, multiple statements, temp-table create-as-select/insert/select/drop, and DuckDB-specific syntax appearing in supported contracts. Record parse/generate behavior, AST node shapes, source/normalization concerns, and exact limits. Compare against the pinned DuckDB's accepted syntax using local offline inputs only. Recommend reuse, bounded fallback, or a candidate parser; do not add a dependency or declare security completeness.
- **Grep anchors:** `validate_statement` in `src/executor/duckdb.rs`; parser entry/traversal in `src/transpiler/mod.rs` and `src/transpiler/security.rs`; DuckDB SQL examples in `tests/duckdb_security_contract.rs` and `tests/duckdb_execution_contract.rs`; `planning/reports/duckdb-security-probe-results.md`.
- **Acceptance:** Report gives reproducible offline examples and a matrix mapping each required construct to parse result/AST support/runtime result; lists gaps and a recommendation; `git diff --check`. No Cargo or source changes.

### Task 15.2.1 — Specify the DuckDB AST allowlist
- **Repo:** same; **Depends on:** 15.1.1.
- **Files in scope — must edit:** `planning/reports/duckdb-parser-policy.md`.
- **Do not touch:** implementation, tests, Cargo files, existing security policy, routes/configuration, or ADRs.
- **Spec:** Use the feasibility evidence to specify accepted statement forms and nested AST nodes, including SELECT and the currently tested request-local temp-table workflow. Define reviewed scalar/table-function allowlists, relation rules, rejection of settings/extension control/external-I/O sources, treatment of unknown AST variants, and any syntax that must remain unsupported. Include compatibility impact, policy-versioning concerns, and a fail-closed response when parsing or AST classification is incomplete. If the existing parser cannot represent a necessary distinction, document the exact decision point and stop before implementation pending user approval for a dependency or scope change.
- **Grep anchors:** `planning/reports/duckdb-parser-feasibility.md`; G39 matrix in `planning/reports/duckdb-security-probe-results.md`; existing temp-table and security contracts.
- **Acceptance:** Reviewable policy matrix maps each construct to allow/deny and rationale; all open choices are resolved or explicitly raised; `git diff --check`. Documentation only.

### Task 15.3.1 — Implement parser-backed DuckDB statement validation
- **Repo:** same; **Depends on:** 15.2.1 and a feasibility result showing existing parser suffices.
- **Files in scope — must edit:** `src/executor/duckdb.rs`.
- **Can edit:** `tests/duckdb_security_contract.rs`.
- **Do not touch:** Cargo files/lockfile, dependencies, route/configuration/runtime selection, SQLite executor, response shape, geography behavior, policy docs, or unlisted files.
- **Spec:** Replace string-prefix authorization with the approved `polyglot-sql` parse/AST policy. Default-deny parse errors, unknown AST nodes, multiple statements outside the explicit batch API, unreviewed functions/table functions, settings/extension controls, and external-I/O surfaces. Preserve verified SELECT behavior and the tested temp-table lifecycle only where the AST can reliably identify it. Keep per-statement batch rollback and request-local state unchanged. If a parser dependency or parser API change outside current scope is needed, stop and report; do not approximate with more string heuristics.
- **Grep anchors:** `validate_statement`, `execute_batch`, and `identifier` in `src/executor/duckdb.rs`; policy matrix in `planning/reports/duckdb-parser-policy.md`; security tests in `tests/duckdb_security_contract.rs`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; `cargo fmt --check`; `git diff --check`. Tests assert allowed SELECT/temp workflows and denied AST/runtime adversarial cases; no new dependency or request exposure.

### Task 15.4.1 — Adversarially qualify and reassess readiness
- **Repo:** same; **Depends on:** 15.3.1.
- **Files in scope — must edit:** `docs/security/duckdb-execution-policy.md`; `planning/reports/duckdb-parser-feasibility.md`; `planning/reports/duckdb-secure-integration-assessment.md`.
- **Can edit:** `tests/duckdb_security_contract.rs` only for a concrete missing regression case discovered by qualification; if source change is necessary, stop and report it for a follow-up task.
- **Do not touch:** implementation, Cargo files, routes/configuration, ADRs, deployment, other reports, or user data.
- **Spec:** Exercise the gate against parser/runtime disagreement, nested and compound statements, CTEs, comments, quoted identifiers/literals, statement separators, settings/configuration syntax, extension controls, reviewed/unreviewed scalar and table functions, external I/O functions, and temp-table transitions. Run only local/offline synthetic cases; do not contact network endpoints. Record exact pinned versions, commands, results, accepted query compatibility, and residual limits. Update policy and integration assessment to state what AST checks enforce and reassess go/no-go against remaining version, lifecycle, import, and resource risks. Retain no-exhaustiveness/no-request-exposure limits unless evidence and separate approval warrant otherwise.
- **Grep anchors:** implementation from Task 15.3.1; test matrix in `tests/duckdb_security_contract.rs`; `planning/reports/duckdb-security-probe-results.md`; integration assessment in `planning/reports/duckdb-secure-integration-assessment.md`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; evidence report covers each required case as pass/fail/unknown; updated policy matches implementation; `git diff --check`.

## Orchestrator gates

- Each task worker writes a micro-TODO of at most 12 steps and stops. Review and approve it before execution.
- Workers edit only scoped paths. Any dependency, pin, policy expansion, or route exposure request stops for user approval.
- Run tasks serially; never run shared Cargo tests concurrently.
- At the final gate run serially: `cargo fmt --check`; `git diff --check`; `cargo test`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; and `CARGO_NET_OFFLINE=true cargo test --offline --all-features`.
- No remote Git operations. Keep planning-only commits separate from implementation commits.
