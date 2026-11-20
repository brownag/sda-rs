# Execution Plan 11 — Test-Only DuckDB T-SQL Pipeline Spike

## Goal & Scope

### Goal

Demonstrate the G47 data path in a repository-local, test-only slice: accept representative SDA T-SQL, translate it to DuckDB SQL, execute it against deterministic synthetic data, and serialize results through the existing SDA serializer. Keep the DuckDB crate as a development dependency so no DuckDB code is included in normal application binaries. This spike gathers integration evidence; it is not a runtime backend, a security qualification, a performance verdict, or production authorization.

### In scope

- Add the approved pinned DuckDB crate as a Cargo development dependency and update the lockfile.
- Add a deterministic offline contract test that exercises the T-SQL → DuckDB SQL → DuckDB execution → existing `ResultSet`/SDA JSON serializer path.
- Cover the selected non-spatial cases: TOP/bracketed identifiers/ISNULL/IIF, the captured synthetic soilDB muaggatt join, STRING_AGG, and a temp-table batch whose CTAS statement contributes no SDA result set.
- Compare supported SDA JSON output with the existing SQLite transpiler/executor/serializer path on the same representative inputs and synthetic data.
- Keep any IIF lowering, statement/result collection, and DuckDB-to-`ResultValue` adapter test-only. Explicitly reject unsupported or lossy DuckDB value types rather than broadening the response contract.
- Record exact inputs, generated DuckDB SQL, expected/observed outputs, reproduction commands, and remaining gaps in a spike report.
- Preserve existing SQLite code, routes, defaults, public response behavior, and accepted ADRs.

### Out of scope / deferred

- Any DuckDB code or dependency in normal application builds; runtime backend selection; HTTP route/configuration changes; production exposure or enablement.
- Production security controls, resource limits, cancellation, concurrency, or deployment artifact provisioning.
- Spatial queries, DuckDB Spatial loading, CLR geography methods, and spatial semantic changes.
- Broad support for the full SQLite T-SQL surface or unbounded/general DuckDB type mapping.
- Performance benchmarking or a verdict against the approved 20% performance target. Existing DuckDB timings are not comparable end-to-end.
- Changes to SQLite execution, its security boundary, batch rollback, wire format, default database/port, or body limit.
- New dependencies other than the specifically authorized pinned DuckDB development dependency.

### Verbatim user direction and dated decisions

- “the plan is to take TSQL as input translate it to duckdb and return the result”
- **2026-11-20 — G47 boundary:** User chose a test-only/internal path with no public route or runtime backend selector.
- **2026-11-20 — Dependency:** User authorized `duckdb = "=1.20000.2"` and its lockfile update for this spike, as a dev-dependency only; no DuckDB dependency in normal binaries.
- **2026-11-20 — Contract cases:** User approved TOP/bracketed identifiers/ISNULL, IIF, the captured synthetic soilDB join, STRING_AGG, and a temp-table batch that suppresses the CTAS count result; unsupported value types are rejected rather than silently converted.
- **2026-11-20 — Execution:** User chose serial work on the current branch and to carry forward Plan 10’s small-worker / one-escalation limit.
- **2026-11-20 — Regression gate:** User approved `cargo fmt --check`, default `cargo test`, and configured-SpatiaLite `cargo test`, run serially after the targeted offline test.
- **2026-11-20 — Planning artifacts:** User chose to track Plan 11 artifacts; planning-only commits remain separate from implementation commits.
- **2026-11-20 — Plan approval:** User approved the revised four-wave decomposition. This authorizes the plan only; implementation remains gated by each task's micro-plan review.

## Reconnaissance

- Repository/worktree: `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-production-backend-task`, HEAD `a859c41`. Do not switch branches or use another worktree.
- Preserve existing changes: `planning/tasks/10.5.2.todo.md` and `planning/tasks/10.6.1.todo.md` are pre-existing untracked drafts and must remain untouched and unstaged. The G47 `planning/GOALS.md` update and Plan 11 files are approved planning deliverables; stage only these explicitly named paths for the planning-only commit.
- Plans 1–10 exist; Plan 11 is the next available orchestration number. Planning paths are tracked/not ignored. User approved tracking this plan set.
- Baseline on 2026-11-20: `cargo fmt --check` passed; `cargo test` passed (75 unit + 30 integration, 0 failures); `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (75 unit + 30 integration, 0 failures); `git diff --check` passed.
- `Cargo.toml` currently includes `polyglot-sql = "0.14"` and `rusqlite`, but no DuckDB dependency. The exact candidate `duckdb = "=1.20000.2"` was built offline in the Plan 10 scratch evaluation.
- `src/transpiler/mod.rs::transpile_with_spatial` runs the current project passes and generates SQLite; `src/transpiler/normalize.rs::generate_sqlite` handles SQLite normalization. Do not change either in this test-only spike.
- `src/result.rs` defines `ResultSet` using `rusqlite::types::Value`; `src/serializer.rs::serialize` produces SDA JSON. `src/executor/mod.rs` exposes the SQLite executor.
- Reference tests: `tests/sql_batch_contract.rs` covers the chosen scalar/IIF/STRING_AGG/temp-table SQLite route behavior; `tests/serializer_contract.rs` covers exact SDA JSON+COLUMNNAME+METADATA serialization. Captured join input is `tests/fixtures/soildb/get-sda-muaggatt.sql`.
- The Plan 10 scratch report found DuckDB's direct dialect output leaves IIF unresolved; DuckDB then errors at execution. Its generic batch result collection emitted a CTAS count result that SQLite response shaping suppresses. Temporal/interval/struct-like value mapping was rejected as lossy or unsupported.
- Accepted ADRs 0001–0007 remain in force. In particular, preserve SQLite as default (ADR 0001/0007), request-local state (0002/0007), all-or-nothing batch semantics (0004/0007), and result numbering by non-empty set (0005). The test-only slice does not supersede or implement production versions of these decisions.
- No production Cargo code, routes, executor, or transpiler files may be changed. Synthetic data and test-authored SQL only; no network, live service, user data, or production files.

## Dependency Graph and Waves

Work is serial in the current worktree because each phase extends the same test-only pipeline file. Each wave completes and passes its focused gate before the next phase starts; no parallel workers or worktrees.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 11.1.1 | Add the dev-only DuckDB dependency and prove the offline crate/engine path with a minimal in-memory smoke test. |
| 2 | 11.2.1 | Lock down raw T-SQL-to-DuckDB dialect output and record translation behavior for selected constructs. |
| 3 | 11.3.1 | Execute the directly supported query subset, map results, serialize SDA JSON, and compare against SQLite. |
| 4 | 11.4.1 | Handle the observed IIF and CTAS result-shaping gaps test-only; reject unsupported values and record spike evidence. |

Waves are strictly ordered 1 → 2 → 3 → 4. The targeted DuckDB contract test runs offline in each wave. At each wave boundary the orchestrator runs `cargo fmt --check`, `git diff --check`, default `cargo test`, and configured-SpatiaLite `cargo test` serially. This plan does not authorize runtime backend wiring or production enablement.

## Global Hazards and Rules

- Every task worker creates a scoped micro-TODO and stops for orchestrator approval before implementation changes.
- Keep DuckDB in `[dev-dependencies]`; do not expose a route, runtime selector, or DuckDB dependency in ordinary builds.
- Keep test adapter logic under the integration test. Do not change `src/`, existing integration tests, runtime modules, security policies, accepted ADRs, or the HTTP response contract.
- Do not represent an IIF workaround in test code as a production transpiler fix. Record the raw dialect gap and the test-only workaround in the report.
- Suppress non-row-producing CTAS counts only in the test adapter; do not alter the existing serializer or claim a production execution contract.
- Map only values that preserve existing serializer behavior. Reject unsupported/lossy types explicitly; do not stringify them silently.
- Use deterministic synthetic schemas and the committed captured SQL fixture only. No production data, user-supplied SQL, live SDA endpoint, or network access.
- Do not benchmark or claim performance, production security, resource limits, or spatial compatibility.
- Run no shared Cargo suites concurrently. The worker runs the targeted test only; the orchestrator runs full gates serially.
- Preserve pre-existing staged/unstaged/untracked work. Stage explicit Plan 11 paths only; no broad staging, destructive Git commands, or remote actions.
- Follow `AGENTS.md`, `CONTRIBUTING.md`, accepted ADRs, and `git-workflow-standards`. Keep plan/state commits separate from the task implementation commit; no attribution trailers.
- Worker policy: serial; the required small-worker model is `opencode-go/deepseek-v4.1-flash`; allow one escalation maximum to `openai/gpt-6-luna`; record usage in `STATE_11.md`.

## Task Specifications

### Task 11.1.1 — Add the pinned DuckDB test dependency and engine smoke test

- **ID:** 11.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** G47 and the approved dependency/scope decisions above.
- **Files in scope — must edit:** `Cargo.toml`, `Cargo.lock`.
- **Files in scope — can create:** `tests/duckdb_pipeline_contract.rs`; `planning/tasks/11.1.1.todo.md`.
- **Do not touch:** `src/`; existing tests/fixtures; runtime configuration/routes/executor/transpiler/serializer; security/deployment/CI; accepted ADRs; unrelated planning files, including the two pre-existing untracked Task 10 drafts.
- **Spec:** Add only `duckdb = "=1.20000.2"` under `[dev-dependencies]`. Create a minimal in-memory DuckDB connection and execute a scalar query from a test. This is an engine/API smoke test only; do not add T-SQL translation, result conversion, or SDA serialization in this task.
- **Grep/recon anchors:** `Cargo.toml` dependency sections; `tests/sql_batch_contract.rs` test conventions; Plan 10 scratch manifest at `/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/Cargo.toml`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` passes with an exact scalar result assertion; `cargo build --offline` succeeds without the DuckDB dev-dependency entering the normal build. `git diff --check` passes. No runtime source changes.

### Task 11.2.1 — Test raw T-SQL to DuckDB dialect translation

- **ID:** 11.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 11.1.1.
- **Files in scope — must edit:** `tests/duckdb_pipeline_contract.rs`.
- **Files in scope — can create:** `planning/tasks/11.2.1.todo.md`.
- **Do not touch:** `Cargo.toml`/`Cargo.lock`; all `src/`; other tests/fixtures; runtime code; reports; unrelated planning files.
- **Spec:** Extend the test file with probe-authored T-SQL dialect cases covering TOP, bracketed identifiers, ISNULL, IIF, and STRING_AGG. Capture/assert generated DuckDB SQL for supported translations and explicitly assert that raw dialect output leaves IIF unresolved. Do not add the IIF workaround yet; this task establishes the translation baseline for the execution phase.
- **Grep/recon anchors:** `polyglot_sql::Dialect::get(DialectType::TSQL).transpile(..., DialectType::DuckDB)`; `tests/soildb_fixture_transpile.rs`; Plan 10 probe `/tmp/opencode/sda-rs-duckdb-evaluation/tsql-pipeline/probe.md`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` passes exact generated-SQL assertions, including the unresolved IIF case. No production transpiler changes; `git diff --check` passes.

### Task 11.3.1 — Execute supported DuckDB queries and serialize SDA JSON

- **ID:** 11.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 11.2.1.
- **Files in scope — must edit:** `tests/duckdb_pipeline_contract.rs`.
- **Files in scope — can create:** `planning/tasks/11.3.1.todo.md`.
- **Do not touch:** `Cargo.toml`/`Cargo.lock`; all `src/`; existing tests/fixtures; runtime code; security/deployment/CI; unrelated planning files.
- **Spec:** Add deterministic in-memory DuckDB schemas for the sample table and captured `legend`/`mapunit`/`muaggatt` join. Execute directly supported translated cases (TOP/brackets/ISNULL without IIF, captured join, STRING_AGG), convert only supported DuckDB values to the existing `ResultSet` value representation, and serialize using `sda_rs::serializer::serialize`. Run equivalent inputs and synthetic data through the existing SQLite transpiler/executor/serializer and assert exact SDA JSON equality. Keep helper/adaptation code in the test file.
- **Grep/recon anchors:** `src/result.rs::ResultSet`; `src/executor/mod.rs::execute`; `src/transpiler/mod.rs::transpile`; `src/serializer.rs::serialize`; `tests/serializer_contract.rs`; `tests/fixtures/soildb/get-sda-muaggatt.sql`.
- **Acceptance:** Offline targeted test executes the captured synthetic soilDB join on both DuckDB and SQLite and asserts exact SDA JSON equality; do the same for the other supported cases. Column ordering and NULL/text representation are checked. No IIF or CTAS workaround in this phase. No runtime source changes; `git diff --check` passes.

### Task 11.4.1 — Test-only gap adapters and spike report

- **ID:** 11.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 11.3.1.
- **Files in scope — must edit:** `tests/duckdb_pipeline_contract.rs`.
- **Files in scope — can create:** `planning/reports/duckdb-test-only-spike.md`; `planning/tasks/11.4.1.todo.md`.
- **Do not touch:** `Cargo.toml`/`Cargo.lock`; all `src/`; existing tests/fixtures; runtime code; security/deployment/CI; accepted ADRs; unrelated planning files.
- **Spec:** Add an explicitly test-only IIF lowering and test the result against SQLite, while retaining an assertion/report of the raw dialect gap. Add the temp-table batch, record the raw DuckDB CTAS result (`{"Table":[["2"]],"Table1":[["2"],["3"]]}`), suppress the non-row-producing CTAS count only in test response collection, and assert adapted JSON (`{"Table":[["2"],["3"]]}`) matches SQLite. Add a focused unsupported-type rejection assertion. Write the report with exact input SQL, generated SQL, raw and adapted output, commands, environment, support boundaries, and explicit statement that workarounds are not production fixes. No performance or production security claims.
- **Grep/recon anchors:** `tests/duckdb_pipeline_contract.rs`; `tests/sql_batch_contract.rs`; `src/serializer.rs` result-set numbering behavior; Plan 10 report `planning/reports/duckdb-tsql-pipeline-evaluation.md`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract` passes all selected assertions. The report distinguishes raw dialect/engine behavior from test-only adaptation and records remaining typed-value gaps. `git diff --check` passes; no runtime source changes.

## Final Acceptance Gate

Each wave ends with these serial orchestrator gates:

1. `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract`
2. `cargo fmt --check`
3. `git diff --check`
4. `cargo test`
5. `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`

All gates must pass before marking each wave's task done. Completion establishes only the test-only experimental path, not production qualification.
