# Execution Plan 2 — Review Cleanup (small and disconnected defects)

## Goal & Scope

### Goal

Clear the small, independent, and dead-code defects found in the 2026-11-17 review of `master...feature/sqlite-mvp`, so that the following orchestrations (G1 remainder, G16–G24) can run as large dependency-ordered plans without interruption. Every change is behavior-preserving except the explicitly listed serializer key and XML changes. No new features. The cleanup goal is GOALS.md `[C1]`.

### In scope

- Confirmed dead code: duplicate `varchar` arm; `PhantomData` in `RequestVisitor`.
- Conditional dead code (decision D2: delete): `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, `SpatialCapability.geometry` field, `geometry_metadata()`, and the response-extension seam for `SpatialCapability`.
- Duplicated logic: the `":memory:"` branch and open/initialize/require-spatial sequence in the executor; the STGeomFromText arity check; the mukey column and SRID fallbacks, bundled once with a non-optional SRID.
- Dead branch: `let _ = function;` and the unreachable `"geometry::STGeomFromText"` check.
- Error classification: transpiler validation failures return `Rejected`, not `Generate`.
- Security guardrail: deny `Expression::Pragma` and `Expression::Attach`.
- Observability: install a tracing subscriber so `TraceLayer` output is emitted.
- Serializer: validate XML element names (G21, the XML part only), and apply decision D4 to result-set key numbering.

### Out of scope / deferred

- Batch rollback (G16, D3 decided: roll back all). Next orchestration.
- `#temp` name isolation (G17). Next orchestration.
- DECLARE substitution and re-DECLARE rejection (G18). Next orchestration.
- Scoped spatial metadata failure (G19). Next orchestration.
- Mukey wrong-layer fallback (G20). Next orchestration.
- AST-level STGeomFromText rewrite (G22). Next orchestration.
- Live SpatiaLite un-skip (G23). Next orchestration.
- In-memory DB test rule (G24, D1: shared-cache). Next orchestration.
- Splitting `src/transpiler/spatial.rs` by reason to change (G25). Backlog.
- Dependency list vs INITIAL_PROMPT (G26). Backlog.
- Rewriting commit subjects (G27). Pruned.

### Verbatim constraints

- “All non-null values in JSON rows must be serialized as JSON Strings (`"123"`, `"45.6"`, `"Sacramento"`), and SQL `NULL` as JSON `null`.” (from `planning/INITIAL_PROMPT.md:49`; unchanged by this plan)
- “Multiple result sets map to top-level object keys: `"Table"`, `"Table1"`, `"Table2"`, etc.” (from `planning/EXECUTION_PLAN_1.md:34`; amended by D4 below)
- “If a query returns 0 rows across all result sets, return `{}` with HTTP 200 OK” (unchanged)
- User-supplied planning sources and `planning/GOALS.md` must remain unchanged by implementation workers.

## Reconnaissance log

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/sqlite-mvp`, clean working tree (only `target/` ignored). Baseline from STATE_1: `cargo fmt --check` passed; `cargo test` passed (33 unit + 3 integration, 0 failures, with `SDA_TEST_SPATIALITE` set to `/usr/lib/x86_64-linux-gnu/mod_spatialite.so`).
- `Cargo.toml` has no tracing subscriber crate; `tower-http` has `trace` enabled. Task 2.1.2 adds `tracing-subscriber` and authorizes `Cargo.toml` and `Cargo.lock` changes for that purpose only.
- `src/transpiler/mod.rs:13-17` `TranspilerError` has `Parse`, `Generate`, `Rejected(String)`; `Rejected` is available.
- No `docs/decisions/`, `AGENTS.md`, or `CONTRIBUTING.md` exists. Standards source is the planning documents.
- File sizes (`wc -l`): `src/serializer.rs` 269, `src/request.rs` 90, `src/main.rs` 10, `src/executor/sqlite.rs` 298, `src/executor/mod.rs` 6, `src/spatial.rs` 282, `src/routes.rs` 309, `src/transpiler/spatial.rs` 385, `src/transpiler/security.rs` 45. Enter via grep anchors below; whole-file reads are not needed.
- Review anchors (verified against the current tree on 2026-11-17):
  - `src/serializer.rs:148` and `:150` duplicate `varchar` arm; `:46` result-set key; `:169-185` XML element emission.
  - `src/request.rs:15` `RequestVisitor(PhantomData<()>)`; `:33` construction.
  - `src/executor/sqlite.rs:11-48` `SqliteExecutor`; `:103` `execute_spatial`; `:109` and `:167` `":memory:"`; `:130` `execute_query_blocking`; `:175-180` `execute_blocking`; `:182-196` `execute_spatial_blocking`; `:199-234` execution helpers; `:286` test using `execute_blocking`; `:296` test referencing `geometry`.
  - `src/executor/mod.rs:4-5` re-exports.
  - `src/spatial.rs:15` `geometry` field; `:25-27` `geometry_metadata()`; `:165-177` derived `geometry`; test references at `:190`, `:207`, `:264`.
  - `src/routes.rs:83` executor call; `:96-99` error mapping (not changed here); `:108` response-extension insertion; `:279` and `:284` test reads.
  - `src/transpiler/spatial.rs:13-45` `normalize_static_calls` (kept; deferred to G22); `:55-70` mukey column selection; `:71` `mukey_srid` `Option`; `:85`, `:88` `Generate` mapping for rewrite errors; `:108-109`, `:125-126`, `:133-134` `unwrap_or` repeats; `:216` `let _ = function;`; `:218` scalar rejection; `:229` unreachable `"geometry::STGeomFromText"` check; `:233` and `:260` arity check.
  - `src/transpiler/security.rs:8-30` denylist.
  - `src/main.rs:1-10` no subscriber.
- Test-only callers of `execute_blocking`: `src/executor/sqlite.rs:286` only (verified by repository search).
- Repository search for `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, `geometry_metadata(`, `RequestVisitor`, `PhantomData`: no references outside the definitions and listed call sites.
- No local reference implementation exists for the cleanup. Pattern-match existing neighbors in each file.

## Dated decisions

- 2026-11-17 — D1 (in-memory DB rule): use shared-cache in-memory DBs. Applies to G24, not to this plan. Recorded so the next plan inherits it.
- 2026-11-17 — D2 (conditional dead code): delete `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, and the `SpatialCapability.geometry` field with `geometry_metadata()`. The crate is treated as unpublished.
- 2026-11-17 — D3 (batch rule): a failed multi-statement batch rolls back all writes. Applies to G16, not this plan.
- 2026-11-17 — D4 (result keys): the first non-empty result set is `Table`; subsequent non-empty sets are `Table1`, `Table2`, etc. Empty result sets are skipped when numbering. Task 2.1.1 implements this.
- 2026-11-17 — Track planning artifacts; commit this plan separately with `docs(planning):`.
- 2026-11-17 — Authorize `tracing-subscriber` (task 2.1.2 only) with `Cargo.toml` and `Cargo.lock` changes limited to that addition.
- 2026-11-17 — Serializer XML: element names are validated; invalid names are rejected with a structured error. Do not sanitize names silently.
- 2026-11-17 — Wave 2 tasks have disjoint file scopes: 2.2.1 owns the executor, spatial, and routes files; 2.2.2 owns the transpiler spatial file. They run in parallel.

## DAG and wave table

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 2.1.1, 2.1.2 | Serializer cleanup and XML validation/keys (2.1.1); security denylist, tracing, and request visitor (2.1.2). File scopes are disjoint. |
| 2 | 2.2.1, 2.2.2 | Executor/spatial/routes dead code and seam removal (2.2.1); transpiler spatial cleanup and error classification (2.2.2). File scopes are disjoint. |

Dependencies: `{2.1.1, 2.1.2} -> 2.2.1` is not required because the file scopes are disjoint, but wave 2 starts only after wave 1 passes its wave-boundary verification. `2.2.1` and `2.2.2` do not depend on each other.

## Global hazards and execution rules

- Confirm `tracing-subscriber` version compatibility with the pinned `tower-http` 0.6 and `tracing` before adding. Use the version resolved by `cargo add` against the existing lock file; do not upgrade unrelated crates.
- Do not change executor lifecycle semantics: one request-local connection per request, blocking work in `spawn_blocking`.
- Do not change `TranspilerError` variants. Only change which variant the existing code returns.
- Preserve existing public names used by tests, except those explicitly deleted by D2. If a test referenced a deleted item, move the test to the surviving public path, and keep its assertion.
- Serializer key numbering (D4) changes output only for responses that begin with an empty result set. Add a test for that case.
- Per-task tests only in workers. Orchestrator runs `cargo fmt --check` and full `cargo test` serially at each wave boundary.
- Worktree isolation is required for the two tasks in each wave. No remote Git actions.
- `planning/GOALS.md` and planning sources are out of scope for implementation workers.

## Task specifications

### Task 2.1.1 — Serializer cleanup, XML element validation, and result-set keys

- **Repo:** `/home/andrew/workspace/sda-rs`
- **Files in scope — must edit:** `src/serializer.rs`; `src/routes.rs` (scope amendment, 2026-11-17: only the single `serialize` call site near `src/routes.rs:101`, mapping a serializer error to HTTP 400 with the error message; no other routes change). Amendment reason: `serialize` is infallible and `routes.rs` is its only caller, so the 400 requirement in spec item 2 is otherwise unmet. Task 2.2.1 still owns the rest of `routes.rs` in wave 2.
- **Can create:** none.
- **Depends on:** none (wave 1).
- **Spec:**
  1. Delete the duplicate `Some(Value::Text(_)) => ("varchar", varchar_provider())` arm at `src/serializer.rs:148` (or `:150`; keep the `_` arm). Output must not change.
  2. XML element names: before emitting a column name as an element, validate it as an XML name (non-empty; first character a letter or `_`; remaining characters letters, digits, `_`, `-`, `.`). If any column name is invalid, return a structured serializer error that the route maps to 400, with a message naming the column. Do not sanitize. Add tests for: a name with a space, a leading digit, an empty name, and a valid name.
  3. Result-set keys (decision D4): number only non-empty result sets. The first non-empty set is `Table`; then `Table1`, `Table2`, and so on. Replace the current absolute-index keying at `:46`. Add a test where the first set is empty and the second has rows; expected key is `Table`.
  4. Do not change JSON value encoding (string/null rule).
- **Do not touch:** `src/routes.rs`, `src/executor/*`, `src/spatial.rs`, `src/transpiler/*`, `planning/*` (except this task's `planning/tasks/2.1.1.todo.md` micro-plan).
- **Grep anchors:** `varchar_provider`, `fn metadata`, `Table{`, `NewDataSet`, `fn xml`.
- **Acceptance:** `cargo test serializer::` passes; `cargo fmt --check` passes; `git diff --check` passes. Route-level error mapping for the new serializer error is covered by the existing routes test pattern only if already present; do not add route tests here.

### Task 2.1.2 — Security denylist, tracing subscriber, and request visitor

- **Repo:** same.
- **Files in scope — must edit:** `src/transpiler/security.rs`, `src/main.rs`, `src/request.rs`, `Cargo.toml`, `Cargo.lock` (only to add `tracing-subscriber` and its resolved dependencies).
- **Can create:** none.
- **Depends on:** none (wave 1).
- **Spec:**
  1. In `src/transpiler/security.rs:8-30`, deny `Expression::Pragma` and `Expression::Attach`. Use the same error shape as the existing denied constructs. Add a test for `ATTACH DATABASE` and for `PRAGMA` each producing a rejection.
  2. In `src/main.rs`, install a `tracing_subscriber` fmt subscriber at startup with an env-filter defaulting to `info` (`RUST_LOG` respected). Add `tracing-subscriber` to `Cargo.toml` with the features needed for fmt and env-filter only.
  3. In `src/request.rs:15`, change `struct RequestVisitor(PhantomData<()>);` to `struct RequestVisitor;`. Update the construction at `:33` and drop the `PhantomData` import if unused.
- **Do not touch:** `src/lib.rs`, `src/routes.rs`, `src/serializer.rs`, `src/executor/*`, `src/spatial.rs`, `src/transpiler/mod.rs`, `src/transpiler/spatial.rs`, `planning/*` (except this task's micro-plan).
- **Grep anchors:** `Expression::Pragma`, `Expression::Attach`, `fn main`, `RequestVisitor`, `PhantomData`.
- **Acceptance:** `cargo test transpiler::security`; `cargo test request::`; `cargo build` (confirms the subscriber compiles; the build is the only non-test command allowed); `cargo fmt --check`; `git diff --check`.

### Task 2.2.1 — Executor, spatial, and routes dead code

- **Repo:** same.
- **Files in scope — must edit:** `src/executor/sqlite.rs`, `src/executor/mod.rs`, `src/spatial.rs`, `src/routes.rs`.
- **Can create:** none.
- **Depends on:** wave 1 complete (orchestrator verification).
- **Spec:**
  1. Delete `SqliteExecutor` and its impl (`src/executor/sqlite.rs:11-48`) and its re-export at `src/executor/mod.rs:5`.
  2. Delete `execute_spatial_blocking` (`:182-196`) and its re-export at `src/executor/mod.rs:4`.
  3. Delete `execute_blocking` (`:175-180`). Move the test at `:286` to call `execute` directly, keeping its assertions. Remove the re-export at `src/executor/mod.rs:4`.
  4. Remove duplication: the `":memory:"` branch at `:109` and `:167` becomes one helper. `execute_spatial` (`:103`) and `execute_query_blocking` (`:130`) share one open-initialize-require-spatial sequence. Behavior and error precedence must not change; in particular, keep the second `requires_spatial` check at `:147` (it orders errors differently and is not redundant).
  5. Remove the `geometry` field from `SpatialCapability` (`src/spatial.rs:15`), the `geometry_metadata()` accessor (`:25-27`), and its derived assignment (`:165-177`). Keep `geometry_metadata_for_table` and `geometries`. Update tests at `src/spatial.rs:190`, `:207`, `:264`, `src/executor/sqlite.rs:296`, and `src/routes.rs:284` to use `geometries` or `geometry_metadata_for_table`, keeping their assertions.
  6. Remove the response-extension insertion of `SpatialCapability` at `src/routes.rs:108` and the test read at `:279`. Replace the test read with an assertion on the response body or status, or delete the test if it only checked the extension.
- **Do not touch:** `src/transpiler/*`, `src/serializer.rs`, `src/request.rs`, `src/main.rs`, `Cargo.toml`, `planning/*` (except this task's micro-plan).
- **Grep anchors:** `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, `":memory:"`, `geometry_metadata(`, `geometries.first()`, `SpatialCapability`.
- **Acceptance:** `cargo test executor::`; `cargo test spatial::`; `cargo test routes::`; `cargo test --test sda_mvp`; `cargo fmt --check`; `git diff --check`. Repository search confirms zero remaining references to removed names outside `planning/`.

### Task 2.2.2 — Transpiler spatial cleanup and error classification

- **Repo:** same.
- **Files in scope — must edit:** `src/transpiler/spatial.rs`.
- **Can create:** none.
- **Depends on:** wave 1 complete (orchestrator verification).
- **Spec:**
  1. Remove `let _ = function;` (`:216`) and the unreachable `"geometry::STGeomFromText"` check (`:229`). Confirm by reading `normalize_static_calls` (`:13-45`) that the check cannot match, and say so in the micro-plan. Do not change `normalize_static_calls`; that is G22.
  2. Extract the duplicated STGeomFromText arity check at `:233` and `:260` into one helper.
  3. Bundle the mukey column and SRID (`:66-71`, `:108-109`, `:125-126`, `:133-134`) into one small struct, for example `MukeyTarget { column: String, srid: u32 }`, built once. Make SRID non-optional. Remove the `unwrap_or` repeats. Behavior must not change in this task: the existing fallbacks stay as they are; G20 changes fallback behavior later.
  4. Rewrite-failure classification (`:85`, `:88`): validation failures (non-literal or missing WKT, arity errors) return `TranspilerError::Rejected`. Genuine SQL generation failures remain `Generate`. Add or update a test asserting the variant for one validation failure.
- **Do not touch:** `src/transpiler/mod.rs`, `src/transpiler/normalize.rs`, `src/transpiler/security.rs`, `src/routes.rs`, `src/spatial.rs`, `tests/sda_mvp.rs` except to update a variant assertion if one exists there (then include it in the micro-plan's file list and note it), `planning/*` (except this task's micro-plan).
- **Grep anchors:** `let _ = function`, `"geometry::STGeomFromText"`, `STGeomFromText requires`, `unwrap_or(geometry_column)`, `unwrap_or(4326)`, `TranspilerError::Generate`, `mukey_srid`.
- **Acceptance:** `cargo test transpiler::spatial`; `cargo test --test sda_mvp` (the AOI and mukey fixtures must still pass unchanged apart from any variant assertion); `cargo fmt --check`; `git diff --check`.

## Wave-boundary verification (orchestrator)

- After wave 1: `cargo fmt --check`, full `cargo test` serially, and `cargo build`. Expected baseline before wave 2: all wave-1 targeted tests green and the full suite at least 33 unit + 3 integration, plus the new tests.
- After wave 2: the same, plus `rg` confirms zero references to `SqliteExecutor`, `execute_spatial_blocking`, `execute_blocking`, `geometry_metadata(` (outside `geometry_metadata_for_table`), and the removed `PhantomData` use outside `planning/`.
- Live SpatiaLite: run with `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so` at each wave boundary, since the extension is installed locally.
