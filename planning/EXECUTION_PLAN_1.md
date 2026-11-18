# Execution Plan 1 — SQLite/GeoPackage Spatial MVP

## Goal & Scope

### Goal

Build `sda-rs` as a Rust HTTP service that implements a tested, SQLite/GeoPackage-focused subset of the USDA-NRCS SDA tabular POST endpoint. The MVP accepts JSON and URL-encoded requests, parses and safely transforms supported T-SQL with `polyglot-sql`, executes it through `rusqlite` while preserving request-local session state, optionally uses SpatiaLite, discovers GeoPackage geometry metadata, and emits SDA-compatible result formats. The priority is an end-to-end spatial AOI query, not full SDA or full T-SQL parity.

### In scope

- Cargo project, CLI/environment configuration, Axum server and tabular POST routes.
- SQLite execution through `rusqlite`, one request-local connection executed via `spawn_blocking`, dynamic result columns, multi-statement execution, and temporary tables.
- T-SQL parsing/code generation using the actual `polyglot-sql` API; security guardrails; MVP normalization (`TOP`, `ISNULL`, `IIF`, `STRING_AGG`, bracketed identifiers).
- Variable folding and the AOI path (`geometry::STGeomFromText`, `.STIntersects`), one WGS84 `SDA_Get_Mukey_from_intersection_with_WktWgs84` UDF.
- Optional SpatiaLite loading; GeoPackage `gpkg_geometry_columns` geometry column/SRID discovery; clear errors when required spatial support is unavailable.
- SDA JSON, `JSON+COLUMNNAME`, `JSON+COLUMNNAME+METADATA`, and XML serialization; deterministic mock SSURGO integration tests.
- The scoping-spec `FORMAT=~` query-echo mode, without expanding deferred macros.
- CLI/environment configuration and HTTP CORS, tracing, and request-size limits, as part of completing the service shell.

### Out of scope / deferred

- PostgreSQL/PostGIS backend and backend parity.
- Full 34-function SDA UDF catalog, table-input macros, and additional spatial UDF families.
- SDA `~DeclareIntTable(...)~` / `~DeclareVarcharTable(...)~` macro expansion and `FORMAT=~` macro expansion; macro echo is not required for the MVP.
- General `CROSS APPLY` / `OUTER APPLY` support.
- Full CLR method catalog (`STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, `STAsText`) except where directly required by the selected AOI query. No general area-unit semantics are promised.
- SDA AOI, interpretation, report, or Shape service modes.
- Live USDA endpoint parity testing; undocumented wire details remain explicit compatibility risks.
- Arbitrary T-SQL, arbitrary DML, or complete SQL Server semantics. Only the safe supported MVP subset is promised.

### Verbatim constraints

- “All non-null values in JSON rows must be serialized as JSON Strings (`"123"`, `"45.6"`, `"Sacramento"`), and SQL `NULL` as JSON `null`.”
- “Multiple result sets map to top-level object keys: `"Table"`, `"Table1"`, `"Table2"`, etc.”
- “If a query returns 0 rows across all result sets, return `{}` with HTTP 200 OK”
- “Row 0 is `["col1", "col2"]`, Row 1+ are data rows.”
- For metadata format: “Row 0: `["col1", "col2"]`”; “Row 1: Metadata strings per column”; “Row 2+: Data rows.”
- GeoPackage geometry names and SRS IDs are dynamic in `gpkg_geometry_columns`, not fixed to `mupolygongeo`.
- The JSON+COLUMNNAME+METADATA metadata header is array row 1 (zero-based), i.e. the second array row.
- Only `planning/` artifacts explicitly requested for this orchestration are to be tracked; keep planning-only commits separate from implementation commits.

## Reconnaissance log

- Repository: `/home/andrew/workspace/sda-rs`, Git repository on `master`, no commits yet. Existing user-supplied planning files are untracked; do not overwrite them.
- Project currently contains only planning documents; no `Cargo.toml`, `src/`, tests, conventions files, ADRs, prior orchestration files, or ignore rules were found in the repository.
- Entry point and test files do not yet exist. Baseline `cargo fmt` / `cargo test` cannot run before project scaffolding; record as not applicable, then establish and run the baseline once a crate exists.
- No local reference implementation exists in this repository. `planning/INITIAL_PROMPT.md` and `planning/sda-rs-scoping.md` are the source requirements; the latter explicitly warns that `polyglot-sql` AST fields and exact wire metadata details need validation against the actual crate/API.
- Integration tests must avoid fixed network ports: use the Axum router in-process with an in-memory database. Spatial tests must be deterministic when SpatiaLite is absent by checking the explicit unavailable-capability behavior; exercise the actual extension path when available.
- No `AGENTS.md`, `CONTRIBUTING.md`, `docs/decisions/README.md`, accepted ADR, `TODO.md`, or prior `STATE_N.md` was present at reconnaissance.
- Git status before planning: `master`, no commits, original source docs and `planning/GOALS.md` untracked. Do not select/switch branches, initialize a repository, or commit until separately authorized by repository workflow; plan says planning artifacts tracked.

## Dated decisions

- 2026-11-17 — MVP prioritizes SQLite/GeoPackage spatial execution; do not make PostgreSQL parity a release gate.
- 2026-11-17 — First spatial slice is AOI variable construction/intersection plus one WGS84 mukey intersection UDF; broad UDF and APPLY support deferred.
- 2026-11-17 — Use `rusqlite`; execute each request on a blocking worker with one connection per request to preserve batch/temp-table session semantics.
- 2026-11-17 — Load SpatiaLite optionally; plain SQLite remains usable, and unsupported spatial operations return a clear error. Discover geometry metadata from GeoPackage tables.
- 2026-11-17 — Acceptance is local and deterministic: `cargo fmt --check`, `cargo test`, and conditional SpatiaLite integration; no live SDA dependency.
- 2026-11-17 — Up to three concurrent workers with worktree isolation for parallel tasks; integration/full suites run serially.
- 2026-11-17 — Use `openai/gpt-6-luna` for all worker tasks and escalation; retain the wave-orchestration retry limits.
- 2026-11-17 — Track orchestration artifacts; keep planning-only commits separate from implementation.
- 2026-11-17 — Sequence tasks 1.2.1 and 1.2.2 because both require `src/lib.rs` module registration; executor/result registration belongs to 1.2.1 and transpiler registration to 1.2.2.

## DAG and wave table

Tasks in each wave have disjoint file scopes. Waves are logical phases and run sequentially.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 1.1.1 | Scaffold crate, dependencies, config, test conventions, and executable server shell. |
| 2 | 1.2.1 | SQLite execution/session result model. |
| 3 | 1.2.2 | Core T-SQL parsing/transpilation and guardrails. |
| 4 | 1.3.1, 1.3.2 | SDA request routes; result serialization formats. |
| 5 | 1.4.1, 1.4.2 | Batch variables/temp-table compatibility; optional spatial capability and GeoPackage metadata discovery. |
| 6 | 1.5.1 | Integrate AOI + selected UDF end to end with seeded mock tables and acceptance suite. |

Dependencies: `1.1.1 -> 1.2.1 -> 1.2.2`; `{1.2.1, 1.2.2} -> {1.3.1, 1.3.2}`; `1.2.2 -> 1.4.1`; `1.2.1 -> 1.4.2`; `{1.3.1, 1.3.2, 1.4.1, 1.4.2} -> 1.5.1`.

## Global hazards and execution rules

- Confirm exact dependency versions, `polyglot-sql` API, and SQLite dialect generation from the actual crate before implementation; do not invent AST variant fields from pseudocode.
- `rusqlite` calls must not block Tokio runtime threads; the task scope requires request execution in `spawn_blocking`.
- SQLite connections and temp tables are request-local. Do not share a connection between concurrent requests.
- Extension loading is unsafe/platform-dependent in some `rusqlite` builds. Keep it isolated, opt-in/explicitly configured, and fail with a structured error; never imply spatial support solely from successful file opening.
- `gpkg_geometry_columns` is optional and can be absent in ordinary SQLite databases. Non-spatial queries must continue to work without it.
- Never execute raw multi-statement SQL as a single prepared statement. Split/parse with the supported parser and execute statements sequentially in the same request connection.
- User SQL is untrusted. Implement an AST-level allowlist/denylist before execution; SQLite's single-statement API is not the security policy.
- User-supplied planning sources and `planning/GOALS.md` are out of scope for implementation workers and must remain unchanged.
- Per-task tests only in workers. Orchestrator verifies changes and runs `cargo fmt --check` and full `cargo test` serially at wave boundaries.
- Worktree isolation is required for concurrent tasks in this single repository. No remote Git actions.

## Task specifications

### Task 1.1.1 — Scaffold crate and runnable service shell

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/lib.rs`, `src/config.rs`, `.gitignore` (only if build outputs need ignoring).
- **Can create:** `src/`, `tests/` only for a minimal crate smoke test, `target/`.
- **Depends on:** None.
- **Spec:** Create the Rust crate with Axum/Tokio/tower-http, `rusqlite`, `polyglot-sql`, serde, and clap dependencies needed by this plan. Establish a library/router-friendly crate shape and CLI/environment configuration for backend URL/path, port, optional spatial extension, and geometry override. Build a minimal server shell that starts without requiring SpatiaLite and applies CORS, tracing, and payload limits. Preserve the explicitly selected `rusqlite` plus per-request blocking execution architecture for dependent work.
- **Do not touch:** `planning/INITIAL_PROMPT.md`, `planning/sda-rs-scoping.md`, `planning/GOALS.md`, task modules owned by later tasks.
- **Grep anchors:** None; crate does not exist yet.
- **Acceptance:** `cargo check`; `cargo test`; `cargo fmt --check`. Add a config unit test for defaults/overrides.

### Task 1.2.1 — SQLite session executor and dynamic result sets

- **Repo:** same.
- **Files in scope — must edit:** `src/executor/mod.rs`, `src/executor/sqlite.rs`, `src/result.rs`, `src/lib.rs` (register executor/result modules only).
- **Can create:** `src/executor/`.
- **Depends on:** 1.1.1.
- **Spec:** Implement the SQLite execution seam using `rusqlite`, dynamic column names/types, sequential statements and multiple result sets on one request-local connection. Expose execution via a blocking boundary usable by Axum; do not couple this task to HTTP types. Preserve SQLite NULL and value representations for serializer conversion. Test in-memory query results, multiple statements, empty result sets, and request-local temp-table visibility within a batch.
- **Do not touch:** `src/main.rs`, `src/config.rs`, route/transpiler/serializer modules, planning documents; do not register the transpiler module.
- **Grep anchors:** `SdaBackend`, `ResultSet` in `planning/INITIAL_PROMPT.md`.
- **Acceptance:** `cargo test executor::`; targeted tests use in-memory SQLite and assert column order, values, nulls, and session continuity.

### Task 1.2.2 — T-SQL parser pipeline and safety guardrails

- **Repo:** same.
- **Files in scope — must edit:** `src/transpiler/mod.rs`, `src/transpiler/normalize.rs`, `src/transpiler/security.rs`, `src/lib.rs` (register transpiler module only).
- **Can create:** `src/transpiler/`.
- **Depends on:** 1.2.1.
- **Spec:** Validate the real `polyglot-sql` Rust API and use it to parse T-SQL and generate SQLite SQL. Establish structured transpilation errors and AST guardrails rejecting administrative/destructive constructs outside the explicitly supported temp-table pattern. Implement the basic compatible subset needed by the plan, including bracketed identifiers, `TOP`, `ISNULL`, `IIF`, and SQLite `STRING_AGG` lowering where required. Do not implement variable folding or spatial/UDF rewriting here.
- **Do not touch:** executor, routes, serializer, CLI/config, planning files; do not alter executor/result module registration.
- **Grep anchors:** `DialectType::TSQL`, `Expression`, `generate`, function normalization in `planning/sda-rs-scoping.md`.
- **Acceptance:** `cargo test transpiler::`; tests parse/generate representative queries and reject `DROP`, `ALTER`, `TRUNCATE`, `GRANT`, `EXEC`, and `WAITFOR`.

### Task 1.3.1 — SDA query request routes

- **Repo:** same.
- **Files in scope — must edit:** `src/routes.rs`, `src/request.rs`.
- **Can create:** none.
- **Depends on:** 1.2.1, 1.2.2.
- **Spec:** Implement `POST /Tabular/post.rest` and `/Tab/post.rest`; parse JSON and `application/x-www-form-urlencoded`; normalize `query`/`QUERY` and `format`/`FORMAT` case-insensitively; invoke transpiler then the SQLite executor and return structured errors. For `FORMAT=~`, return the submitted SQL as `text/plain` without executing it; macro expansion is deferred. Use in-process router tests, not a fixed TCP port. Mount router through the crate API from task 1.1.1.
- **Do not touch:** main/config, executor, transpiler, serializer internals, planning files.
- **Grep anchors:** endpoints and content types in `planning/INITIAL_PROMPT.md` lines 12–20.
- **Acceptance:** `cargo test routes::`; exercise both endpoints with both body encodings, mixed-case keys, missing query, and unsafe-query error.

### Task 1.3.2 — SDA result serialization

- **Repo:** same.
- **Files in scope — must edit:** `src/serializer.rs`.
- **Can create:** `tests/serializer.rs` only if module-local tests are insufficient.
- **Depends on:** 1.2.1.
- **Spec:** Serialize result sets into JSON, JSON+COLUMNNAME, JSON+COLUMNNAME+METADATA, and XML. JSON non-null values are strings and SQL NULL is JSON null; use `Table`, `Table1`, … for result sets; return `{}` for zero rows across all sets. For metadata format put names in row 0, metadata in row 1, then data. Include stable SQLite-to-SQL Server type metadata mapping for MVP types, clearly documenting inferred/provisional mappings rather than claiming unverified exact SDA parity.
- **Do not touch:** route, executor, transpiler, CLI/config, planning files.
- **Grep anchors:** wire-format requirements in `planning/INITIAL_PROMPT.md` lines 48–59; metadata row correction in `planning/sda-rs-scoping.md` lines 228–251.
- **Acceptance:** `cargo test serializer::`; exact JSON/XML structure tests for values, nulls, multiple results, empty results, and metadata row offsets.

### Task 1.4.1 — Batch variable and temp-table rewrites

- **Repo:** same.
- **Files in scope — must edit:** `src/transpiler/mod.rs`, `src/transpiler/vars.rs`, `src/transpiler/temp_tables.rs`.
- **Can create:** module-local tests in these files.
- **Depends on:** 1.2.2.
- **Spec:** Extend the validated AST pipeline to fold declared and assigned variables (`DECLARE`, `SET`, and supported `SELECT @var = expr`) across statements and rewrite `SELECT … INTO #name` plus downstream references into SQLite temp-table SQL. Preserve statement ordering and do not silently pass unsupported assignment forms through.
- **Do not touch:** executor/routes/serializer modules or any files outside this task's explicit list.
- **Grep anchors:** variable/temp pseudocode and caveat in `planning/sda-rs-scoping.md` lines 23–92; `INITIAL_PROMPT.md` lines 26–31.
- **Acceptance:** `cargo test transpiler::vars`; `cargo test transpiler::temp_tables`; test variable substitution and select-into/temp-table join batch.

### Task 1.4.2 — Optional spatial initialization and GeoPackage metadata

- **Repo:** same.
- **Files in scope — must edit:** `src/spatial.rs`.
- **Can create:** module-local tests.
- **Depends on:** 1.2.1.
- **Spec:** Add SQLite spatial capability initialization behind the configured optional SpatiaLite extension; inspect `gpkg_geometry_columns` for table, geometry-column name, and SRS ID; allow plain SQLite without spatial metadata. Return a structured capability error when a spatial query requires absent functions/extension. Keep extension configuration/connection initialization behind a small interface for the executor to call; coordinate interface with orchestrator before changing executor files.
- **Do not touch:** executor files, transpiler, routes, serializer, planning files.
- **Grep anchors:** dynamic GeoPackage metadata requirement in `planning/sda-rs-scoping.md` lines 1–4 and 183–205.
- **Acceptance:** `cargo test spatial::`; fixture tests cover no metadata, discovered metadata, configured extension unavailable, and capability error. If extension exists in environment, run a real function smoke test; report availability.

### Task 1.5.1 — Spatial AOI transpilation and end-to-end acceptance

- **Repo:** same.
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `tests/sda_mvp.rs`; modify shared module registry only with orchestrator authorization.
- **Can create:** integration fixtures in `tests/` scoped to this suite.
- **Depends on:** 1.3.1, 1.3.2, 1.4.1, 1.4.2.
- **Spec:** Implement bottom-up rewriting for `geometry::STGeomFromText`, `.STIntersects`, `@aoi` substitution integration, and `SDA_Get_Mukey_from_intersection_with_WktWgs84` for supported scalar/derived-table contexts. Use discovered GeoPackage geometry column metadata rather than assuming `mupolygongeo`. Add deterministic in-process router tests over seeded mock `legend`, `mapunit`, `component`, `chorizon`, `muaggatt`, and `mupolygon` tables. Cover query paths in `WHERE mukey IN (...)` and `FROM` table-valued form where parser support permits; explicitly report unsupported shape/AST form rather than emitting malformed SQL. Preserve no-extension error behavior.
- **Do not touch:** executor, serializer, routes, config, existing planning sources; module registry without orchestrator authorization.
- **Grep anchors:** spatial rewrite catalog in `planning/INITIAL_PROMPT.md` lines 32–46; AOI and UDF templates in `planning/sda-rs-scoping.md` lines 94–138, 186–205, and benchmark query 14.
- **Acceptance:** `cargo test --test sda_mvp`; then orchestrator runs `cargo fmt --check` and full `cargo test` serially. Spatial integration executes when SpatiaLite is available; absent-extension tests must pass regardless.
