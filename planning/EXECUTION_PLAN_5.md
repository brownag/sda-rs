# Execution Plan 5 — SQLite Execution Safeguards (G32)

## Goal & Scope

### Goal

Make SQLite execution safeguards for submitted SQL, trusted optional extension loading, Rust-registered functions, and resource consumption explicit and tested. The persistent GeoPackage is read-only to submitted SQL, while request-local `#temp` table workflows remain supported. Preserve optional SpatiaLite and the accepted request-local connection, batch transaction, and G1 query behavior.

### In scope

- Add a SQLite authorizer as defense in depth: allow reads from persistent data; reject persistent writes/schema changes; retain only the temp-table operations required by G1.
- Preserve optional extension loading only from configured `SDA_SPATIAL_EXTENSION` via the trusted Rust setup path; submitted SQL cannot load extensions.
- Add a test-only Rust UDF fixture to verify a conservative registration policy; do not add a production SQL function.
- Enforce and test the agreed initial resource caps:
  - Preserve the existing 2 MiB request-body limit and set a 2 MiB SQLite SQL-text ceiling.
  - At most 100,000 returned rows per request.
  - At most 64 MiB serialized response output.
  - Interrupt SQLite execution after 60 seconds.
  - SQLite string/BLOB length at most 64 MiB.
- Enable only the existing `rusqlite` features needed for hooks, limits, and the test-only function fixture; do not add or upgrade crates.

### Out of scope / deferred

- Broader spatial semantics (G29–G31), terrio evaluation (G33), native spatial acceleration (G28), new SDA/CLR functions, new production UDFs, other backends, and wire-protocol changes.
- Changing the existing 2 MiB HTTP body limit or default configuration values.
- Rewriting the SQL transpiler security denylist; the SQLite authorizer is additional connection-level defense, not a replacement for supported-query validation.
- Any remote Git operation.

### Verbatim constraints and selected decisions

- User confirmed the candidate goal set and approved refreshing `planning/GOALS.md`, then handing G32 to plan-wave.
- “Read-only persistent DB”: submitted SQL cannot mutate persistent GeoPackage data, while request-local `#temp` workflows remain supported.
- “Configured-only load”: load SpatiaLite only from configured `SDA_SPATIAL_EXTENSION` during trusted setup; query-controlled extension loading is forbidden.
- User selected a test-only fixture UDF, not a new production function.
- User accepted the proposed caps above as initial values, with later revision possible if representative workloads show them too tight.
- User approved enabling the existing `rusqlite` `hooks`, `limits`, and `functions` features if needed, without changing its version or adding crates.
- User approved at most two concurrent workers, isolated worktrees for concurrent tasks, and `openai/gpt-6-luna` for workers/escalation.
- User approved serial configured full-suite checks at each wave boundary: `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`.
- User chose to track planning artifacts and commit them separately from implementation.

## Reconnaissance log

- Repository: `/home/andrew/workspace/sda-rs`; branch `feature/sqlite-mvp`; baseline HEAD `46beb3c`. `planning/GOALS.md` has the intentional Magic Wand refresh; preserve it and include it in the planning-only commit after plan approval. Existing Plan 3/4 worktrees are retained; do not clean them up.
- Plan 3 completed G16–G20, G22, and G24. Plan 4 completed G1/G6/G7/G23. Their evidence is in `planning/STATE_3.md` and `planning/STATE_4.md`.
- Green configured baseline: 52 unit tests + 11 integration tests passed with `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; the live SpatiaLite test executed. `cargo fmt --check` and `git diff --check` passed.
- Current code opens a request-local SQLite connection in `src/executor/sqlite.rs::open_initialized`; it initializes optional SpatiaLite in `src/spatial.rs::initialize`, then prepares and executes a transaction in `execute_on`. `execute_on` captures all returned rows before serialization.
- `src/lib.rs` applies the existing 2 MiB `RequestBodyLimitLayer`. `src/transpiler/security.rs::validate` rejects selected destructive/admin SQL, but connection-level authorization is not currently installed.
- SpatiaLite loading already uses `rusqlite::LoadExtensionGuard` only around trusted `connection.load_extension` in `src/spatial.rs`; preserve and test this lifecycle.
- `rusqlite` is 0.32.1. Its existing `hooks`, `limits`, and `functions` features expose the authorizer/progress handler, SQLite limits API, and test UDF registration; do not alter the crate version.
- Accepted ADRs 0001–0006 remain applicable. This plan does not supersede them: preserve SQLite/GeoPackage MVP, request-local blocking connections, optional SpatiaLite, batch rollback, SDA result numbering, and shared-cache in-memory integration fixtures.
- Relevant sizes: `src/executor/sqlite.rs` 309 lines; `src/spatial.rs` 333; `src/config.rs` 63; `tests/sda_mvp.rs` 249. Integration tests use Axum `Router::oneshot` and unique shared-cache memory URIs; no fixed TCP ports are needed.
- No new repo-wide baseline failures observed.

## Dated decisions

- 2026-11-18 — Set G32 as the active goal after G1 acceptance; retain broader spatial work behind its recorded prerequisites.
- 2026-11-18 — Submitted SQL is read-only against persistent GeoPackage data; preserve request-local temp-table DDL and writes required by G1.
- 2026-11-18 — SpatiaLite remains optional and loads only from configured trusted setup; submitted SQL cannot load extensions.
- 2026-11-18 — Use a test-only Rust UDF fixture to validate conservative function flags; add no production UDF.
- 2026-11-18 — Initial caps: 2 MiB request/SQL text, 100,000 rows per request, 64 MiB serialized response, 60-second SQLite execution, and 64 MiB SQLite string/BLOB length.
- 2026-11-18 — Existing rusqlite features `hooks`, `limits`, and `functions` may be enabled if required; no new dependency or version changes.
- 2026-11-18 — Maximum two workers, isolated worktrees for concurrent tasks, and `openai/gpt-6-luna` worker/escalation model.
- 2026-11-18 — Run configured full suites serially at each wave boundary; track planning artifacts and commit them separately.
- 2026-11-18 — Accepted ADRs 0001–0006 are preserved; no supersession is authorized.

## DAG and wave table

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 5.1.1 | Install SQLite authorizer protection for persistent data while preserving G1 temp-table behavior. |
| 2 | 5.2.1, 5.2.2 | Independently prove trusted extension lifecycle and conservative test-UDF flags. |
| 3 | 5.3.1, 5.3.2 | Enforce SQLite execution/row limits and bounded serialized output. |

Dependencies: `5.1.1 -> 5.2.1`, `5.1.1 -> 5.2.2`, `5.1.1 -> 5.3.1`, and `5.1.1 -> 5.3.2`. Wave 2 tasks have disjoint test-file scopes. Wave 3 tasks have disjoint implementation/test-file scopes. Full suites run serially at every wave boundary.

## Global hazards and execution rules

- Do not edit unrelated goal/status history or remove planning 3/4 worktrees.
- Keep request-local connection ownership, `spawn_blocking`, transaction rollback on any failed batch, optional spatial behavior, and exact SDA response structure unchanged.
- Install the authorizer only after trusted connection and SpatiaLite initialization. Account for the executor-owned transaction while denying submitted persistent writes and transaction-control SQL.
- Permit only the temp schema operations needed for `#temp` create/populate/read workflows; test that persistent tables remain unchanged.
- Preserve the transpiler security checks; connection authorization is defense in depth.
- Resource limits must fail with the existing structured HTTP error shape and must stop memory/CPU growth at the specified bound rather than detect excess only after building an unbounded response.
- Use test-specific low thresholds through internal helpers for boundary tests; do not wait 60 seconds or allocate 64 MiB in routine unit tests.
- Workers run only task-targeted tests. At wave boundaries, the orchestrator runs format, diff, and configured full suites serially.
- Use worktree isolation for concurrent tasks (maximum two workers); no remote Git actions. Planning-only commits remain separate from implementation commits.

## Task specifications

### Task 5.1.1 — Document and enforce the SQLite authorizer policy

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `Cargo.toml`, `src/executor/sqlite.rs`.
- **Can create:** `docs/security/sqlite-execution-policy.md`, `tests/security_contract.rs`.
- **Depends on:** none.
- **Spec:** Write the G32 policy document covering the authorizer allow/deny rules, trusted extension lifecycle, UDF flags, and agreed resource caps. For future Rust UDFs, require `SQLITE_UTF8` plus only justified flags: use `SQLITE_DIRECTONLY` unless schema invocation is explicitly required; use `DETERMINISTIC` only for pure stable results; use `INNOCUOUS` only after security review. Enable the existing rusqlite `hooks` feature and install an authorizer on each request-local connection after trusted spatial initialization and before submitted statements execute. Allow persistent SELECT/READ; permit only request-local temp-table create/drop and DML/read operations required for `#temp` workflows; allow the executor-owned transaction action; allow other SQL functions except query-controlled `load_extension`; deny all other authorizer actions by default. Preserve the transpiler checks and verify user transaction-control SQL remains rejected. Add router-level tests proving SELECT and `#temp` create/populate/read still work and attempted persistent writes fail without changing seeded rows.
- **Do not touch:** `planning/GOALS.md`, spatial initialization implementation, serializer, request decoder, other integration test files, or dependency versions.
- **Grep anchors:** `src/executor/sqlite.rs` `open_initialized`, `execute_on`; `src/transpiler/security.rs` `validate`; `tests/request_contract.rs` `Router::oneshot` fixture pattern.
- **Acceptance:** `cargo test --test security_contract`; `cargo fmt --check`; `git diff --check`.

### Task 5.2.1 — Prove trusted optional extension loading

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** none.
- **Can create:** `tests/extension_security_contract.rs`.
- **Depends on:** 5.1.1.
- **Spec:** Add in-process router acceptance for configured SpatiaLite setup and no-extension ordinary SQLite behavior. Attempt query-controlled `load_extension` and assert it is unavailable/rejected after trusted setup. Assert configured extension loading and a real spatial operation succeed; do not modify production lifecycle unless an in-scope test exposes a blocker, in which case stop and report.
- **Do not touch:** `planning/GOALS.md`, production files, `tests/sda_mvp.rs`, other contract tests, or Cargo manifests.
- **Grep anchors:** `src/spatial.rs` `initialize`, `LoadExtensionGuard`; `tests/aoi_contract.rs` configured router fixture.
- **Acceptance:** `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test --test extension_security_contract`; `cargo fmt --check`; `git diff --check`.

### Task 5.2.2 — Test conservative Rust UDF flags

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `Cargo.toml`.
- **Can create:** `tests/udf_flags_contract.rs`.
- **Depends on:** 5.1.1.
- **Spec:** Enable rusqlite's existing `functions` feature and add a test-only fixture UDF with the minimum flags required for direct invocation. Prove direct SQL invocation works and schema-mediated invocation (such as through a view or trigger) is denied using `SQLITE_DIRECTONLY`; do not register any production UDF or claim the fixture establishes broader function semantics.
- **Do not touch:** `planning/GOALS.md`, production source, other contract tests, or dependency versions.
- **Grep anchors:** rusqlite 0.32.1 `functions::FunctionFlags` and `Connection::create_scalar_function`; existing direct SQLite fixture patterns in `src/executor/sqlite.rs`.
- **Acceptance:** `cargo test --test udf_flags_contract`; `cargo fmt --check`; `git diff --check`.

### Task 5.3.1 — Enforce SQLite execution and row limits

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `Cargo.toml`, `src/executor/sqlite.rs`.
- **Can create:** `tests/sqlite_limits_contract.rs`.
- **Depends on:** 5.1.1.
- **Spec:** Enable rusqlite's existing `limits` feature if required. Apply a 2 MiB SQL-text limit, 64 MiB SQLite string/BLOB limit, 60-second progress-handler deadline, and 100,000 total returned-row request limit. Bound row collection before it exceeds the limit. Preserve batch rollback and request-local temp tables. Make limits testable through internal lower-limit helpers and test both boundary and exceeded cases without waiting 60 seconds or large allocations.
- **Do not touch:** `planning/GOALS.md`, `src/lib.rs` request body limit, serializer, routes, other contract tests, or dependency versions.
- **Grep anchors:** `src/executor/sqlite.rs` `open_initialized`, `execute_query_blocking`, `execute_on`; rusqlite 0.32.1 `limits::Limit`, hooks `progress_handler`.
- **Acceptance:** `cargo test --test sqlite_limits_contract`; `cargo fmt --check`; `git diff --check`.

### Task 5.3.2 — Bound serialized response output

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/serializer.rs`.
- **Can create:** `tests/serializer_limits_contract.rs`.
- **Depends on:** 5.1.1.
- **Spec:** Enforce a maximum serialized response size of 64 MiB while generating output, not only after allocating the full unbounded string. Return a serialization error that routes through the existing structured error response. Keep all existing JSON/XML/wire-shape behavior unchanged below the limit. Add boundary tests using a lower test-only cap.
- **Do not touch:** `planning/GOALS.md`, executor/resource limit implementation, routes, other contract tests, or Cargo manifests.
- **Grep anchors:** `src/serializer.rs` `serialize`, `json_results`, `xml_results`; `src/routes.rs` serialization error mapping; `tests/serializer_contract.rs` exact contract fixture.
- **Acceptance:** `cargo test --test serializer_limits_contract`; `cargo fmt --check`; `git diff --check`.

## Wave-boundary verification (orchestrator)

- After each wave, run `cargo fmt --check`, `git diff --check`, and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially.
- Inspect failures for preservation of G1 acceptance, temp tables, transaction rollback, and the configured live SpatiaLite path.
- Final G32 evidence must demonstrate persistent writes rejected without side effects, temp workflows accepted, trusted extension loading only, test UDF schema restrictions, and all resource thresholds enforced at their boundary.
