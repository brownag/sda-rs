# Execution Plan 10 — DuckDB Production Qualification

## Goal & Scope

### Goal

Evaluate DuckDB's capabilities and performance as an opt-in SDA query backend while retaining SQLite as the default and supported backend; use that evidence to decide production limits and controls before any production enablement. The initial evaluation targets Linux x86_64 and uses a local, isolated scratch prototype with synthetic/committed fixtures only. It may explore engine behavior beyond the proposed production controls, but is not exposed to production or untrusted queries. Production use remains gated on a separately approved security/resource policy, request-local lifecycle, documented SQL contract, performance acceptance, and deployment-time pinned/hash-verified Spatial artifacts.

### In scope

- Measure current SQLite performance on existing committed contract-suite workloads and define a reproducible benchmark protocol.
- Obtain user approval of a numeric DuckDB performance target after reviewing that baseline and before backend implementation.
- Evaluate DuckDB capabilities, performance, and resource behavior in a local-only scratch prototype before selecting production controls or numeric resource limits; report what is demonstrated, what remains unknown, and tradeoffs.
- Define DuckDB's supported/rejected SQL and spatial contract, including all six Plan 9 geography-method rejections unless new evidence proves a runtime-enforceable SQL Server-compatible contract.
- Design the backend/configuration/execution boundary, request lifecycle, isolation, batch failure semantics, and temporary-table workflow.
- After the exploratory evaluation and an explicit follow-up decision, design and test the DuckDB security/resource boundary, typed result/geometry handling, deployment-time artifact provisioning, Linux x86_64 CI, and operator documentation.
- If implementation proceeds, add a new accepted ADR for DuckDB as an opt-in backend; preserve ADRs 0001/0003 and all other current accepted decisions.
- Production implementation/enablement remains explicitly conditional on follow-up approval of resource/security controls and the production boundary. The current authorization permits only the isolated capability evaluation until that follow-up decision.

### Out of scope / deferred

- Replacing SQLite as the default, removing SQLite support, silent fallback between backends, or requiring SpatiaLite for DuckDB.
- General SQL Server geography parity, community geography adoption, or planar substitutes for the six rejected CLR geography instance methods.
- Cross-platform production qualification beyond the first Linux x86_64 target.
- Hosted deployment/client trials; G34 remains deferred until a target and authorization are identified.
- Production request-time network access, extension auto-install, arbitrary extension sources, OS sandboxing, or broadening the existing SQL contract without evidence and explicit approval. The local evaluation must not access external/user data or make live network requests.
- The user separately authorized the pinned Rust DuckDB crate and lockfile update for Task 10.4.1. This does not authorize repository dependency changes for the current scratch-only evaluation or any unrelated dependency.
- Changes to the HTTP wire format, default port/database, request body limit, or unrelated goals.

### Verbatim user direction

- “productionize DuckDB as an opt-in SDA query backend, retaining SQLite”
- “improve performance and avoid requiring SpatiaLite”
- “Preserve the documented query contract and explicit rejections; first target Linux x86_64.”
- “Mirror SQLite’s security boundary; preserve request-local temporary-table workflows.”
- “Benchmark existing contract-suite workloads first; agree on a numeric performance threshold before backend implementation.”
- “Provision pinned, hash-verified official DuckDB Spatial artifacts at deployment; no request-time network or silent SpatiaLite fallback. If Spatial is unavailable, keep non-spatial queries available and fail spatial requests explicitly.”
- “Plan around a pinned Rust `duckdb` crate; dependency addition still needs implementation authorization.”
- “the scope is to see what is possible then we will limit after we evaluate that”

## Dated Decisions

- **2026-11-19 — Backend role:** DuckDB is an opt-in alternative; SQLite remains the default and supported backend.
- **2026-11-19 — Scope/platform:** Preserve the existing documented SQL contract and explicit rejections; qualify Linux x86_64 first. No general SQL Server geography parity is claimed.
- **2026-11-19 — Performance:** Production value is performance. Benchmark existing committed contract workloads against SQLite first; pause for user approval of a numeric target before backend implementation.
- **2026-11-19 — Security/lifecycle:** Mirror SQLite's security boundary and request-local temporary-table workflows. No OS sandbox.
- **2026-11-19 — Extension deployment:** Pin and hash-verify official DuckDB Spatial artifacts during deployment; no request-time networking or automatic install. If Spatial is unavailable, non-spatial work remains available and spatial requests fail explicitly. Never fall back to SpatiaLite on DuckDB.
- **2026-11-19 — Dependency:** Plan around a pinned Rust `duckdb` crate. This is not authorization to add/update the repository dependency.
- **2026-11-19 — Architecture:** Add a new DuckDB ADR. Do not supersede ADR 0001 (SQLite/GeoPackage MVP) or ADR 0003 (optional SpatiaLite).
- **2026-11-19 — Workers:** Serial task execution on `feature/duckdb-production-backend`.
- **2026-11-19 — Worker budget:** Use `opencode-go/deepseek-v4.1-flash` for scoped tasks; allow one escalation to `openai/gpt-6-luna` under the escalation ladder; no fixed dollar cap. Report usage.
- **2026-11-19 — Acceptance:** Targeted tests per task; final serial gates are `cargo fmt --check`, `git diff --check`, default `cargo test`, configured-SpatiaLite full `cargo test`, and Linux x86_64 DuckDB contract/security/deployment-artifact tests. Hosted trials deferred.
- **2026-11-19 — Resources:** Calibrate DuckDB memory, threads/concurrency, timeout, temporary storage, and result caps from workload evidence before implementation; Plan 8's 1 GB/2-thread probe settings are not production limits.
- **2026-11-19 — Planning artifacts:** Track Plan 10 artifacts and keep the planning-only commit separate from implementation; the user approved finalization/commit on `feature/duckdb-production-backend`.
- **2026-11-19 — User-change safety:** Preserve the pre-existing uncommitted `planning/GOALS.md` update from this planning session; include it only in the dedicated planning-artifact commit after review. No broad staging.
- **2026-11-19 — Baseline:** During Plan 10 drafting, default `cargo test` and configured-SpatiaLite full `cargo test` both passed (75 unit + 30 integration tests, 0 failures); `cargo fmt --check` passed.
- **2026-11-19 — Performance target approval:** User approved at the Task 10.2.1 gate: DuckDB median latency must improve the geometric mean across the four non-spatial baseline workloads (scalar/select, captured join, batch/temp-table, serialization) by at least 20%; no individual workload may regress by more than 10%. Spatial is reported separately and excluded from this initial threshold.
- **2026-11-19 — Dependency authorization:** User explicitly authorized adding the pinned Rust `duckdb` crate and updating `Cargo.lock` for Task 10.4.1, subject to the task's security/API feasibility checks. This does not authorize unrelated dependencies or weakening required controls.
- **2026-11-19 — Exploratory scope revision:** User directed that the next scope is to evaluate what DuckDB can do and then choose limits afterward. A local-only scratch prototype may explore capabilities beyond proposed production controls, but may use only synthetic/committed data, no live network or production service, and no untrusted queries. No production backend is enabled; production controls, limits, and enablement require a later explicit decision.

## Reconnaissance

- Repository/worktree: `/tmp/opencode/sda-rs-plan10-merge`, Git branch `feature/duckdb-production-backend`, clean at `7b1ad61` before Plan 10 edits. `main` also points to `7b1ad61`; this branch was created for Plan 10. Do not alter the source worktree or stale worktree registrations.
- Planning artifacts are tracked. Plans 1–9 exist; Plan 10 is the next orchestration number. `planning/` paths are not ignored.
- The default baseline passed during planning: `cargo fmt --check`; `cargo test` (75 unit, 30 integration, 0 failed); configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` (75 unit, 30 integration, 0 failed). Re-run full gates before reporting completion.
- No benchmark harness/dependency currently exists. `Cargo.toml` has no benchmark framework. The current endpoint tests use deterministic in-memory SQLite fixtures; captured workloads are in `tests/fixtures/soildb/`, `tests/soildb/`, and contract tests. Benchmark tasks must not add a dependency without authorization and must avoid real user data/network.
- Relevant structure: `src/executor/sqlite.rs` is 591 lines and owns request-local blocking execution, 60-second deadline, 100,000-row limit, SQL/value limits, authorizer, temporary tables, and rollback behavior. `src/routes.rs` selects current executor and formats errors. `src/config.rs` owns CLI/env configuration. `src/result.rs` aliases rusqlite `Value`; `src/serializer.rs` serializes shared result sets. `src/transpiler/` is backend-specific SQL transformation.
- Existing tests to pattern-match: `tests/security_contract.rs`, `tests/extension_security_contract.rs`, `tests/sql_batch_contract.rs`, `tests/sqlite_limits_contract.rs`, `tests/spatial_metadata_contract.rs`, `tests/spatial_methods_contract.rs`, `tests/serializer_contract.rs`, `tests/soildb_query_fixtures.rs`, and `tests/sda_get_contract.rs`.
- Existing policy/contracts: `docs/security/sqlite-execution-policy.md`; `docs/spatial-sql-contract.md`; `planning/sda-rs-scoping.md`; accepted ADRs 0001–0006. Preserve all. Plan 9 explicitly retains all six G12 instance methods as unsupported absent runtime-enforceable SQL Server-compatible semantics.
- DuckDB evidence: Plan 8 used Rust crate `duckdb = "=1.20000.2"` with engine `v2.0.0`; the official Spatial extension SHA-256 was `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`. This evidence is version/platform-specific and must not be assumed to be the chosen production pin without implementation-time verification. Plan 8's bounded read was not a performance or service-readiness result.
- Shared test resources and Cargo build state mean no concurrent full Cargo suites. Workers are serial; orchestrator runs suite gates at wave boundaries.
- Conventions: root `AGENTS.md`, `CONTRIBUTING.md`, Plans 8/9 and `PROMPT_TEMPLATE_9.md`; no remote Git actions. Keep plan/state commits separate from implementation commits.

## Dependency Graph and Waves

Tasks are serial and run on `feature/duckdb-production-backend`; same-file/shared executor, routing, and configuration edits are never concurrent.

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 10.1.1 | Establish reproducible SQLite contract-workload benchmark baseline and resource observations; no DuckDB implementation. |
| 2 | 10.2.1 | Present baseline and obtain explicit user approval of numeric performance target; stop until approved. |
| 3 | 10.3.1 then 10.3.2 | Specify backend contract/ADR, then threat/resource/deployment architecture. Serial even though docs differ. |
| 4 | 10.4.1 | Evaluate DuckDB capabilities/performance in a local-only scratch prototype; no production backend wiring. |
| 5 | 10.5.1, 10.5.2 | Add DuckDB spatial/result compatibility and deployment-time pinned artifact provisioning, in disjoint files. |
| 6 | 10.6.1 | Integrate contract, adversarial security/resource, and deployment tests/CI/docs; verify final gates. |

Wave 2 is a mandatory human approval gate. If the user does not approve a numeric threshold, do not begin Waves 3–6. Wave 4 is now a local-only capability evaluation, not production backend implementation; after Task 10.4.1, pause for user review and approval of production controls/resource limits before proceeding to Tasks 10.5.1, 10.5.2, or 10.6.1. All tasks run serially; no parallel workers or worktrees are authorized. Same-wave tasks have disjoint file sets, but remain ordered as listed.

## Global Hazards and Rules

- Every worker creates a scoped micro-TODO and stops for orchestrator approval before code or state-changing commands.
- Serial workers only; workers run targeted tests, never shared/global Cargo suites.
- Never silently alter SQL semantics, HTTP response shape, SQLite defaults, or active ADRs.
- DuckDB's extension loading, file access, temporary files, threads, memory, query cancellation, row/byte limits, and batch rollback must be characterized before production enablement. SQLite controls do not automatically transfer to DuckDB APIs.
- Task 10.4.1 may probe engine behavior only with disposable local data/paths; do not access external/user data, make live network requests, or send untrusted queries. It is an evaluation, not a production security certification.
- Extension paths, version, platform, source, and hash must be verified; no request-time network, auto-install, or arbitrary user-directed loading.
- DuckDB spatial-unavailable behavior must not prevent non-spatial queries and must be explicit for spatial requests; no SpatiaLite fallback.
- Do not infer geography parity from similarly named planar operations. Preserve all six G12 explicit instance-method rejections.
- Do not run benchmarks on the Iowa source or external service. Use committed deterministic fixtures and record hardware, build mode, warmup/repetition protocol, timing distribution, peak RSS where available, versions, and exact commands.
- No dependency addition, wire-protocol/default changes, hosted trial, remote Git action, OS sandbox, or unrelated cleanup.
- At each wave boundary, orchestrator runs `cargo fmt --check`, `git diff --check`, and the relevant full suite serially; final gate includes default and configured-SpatiaLite suites plus DuckDB Linux x86_64 tests.
- Worker models: `opencode-go/deepseek-v4.1-flash`; one escalation to `openai/gpt-6-luna` maximum for the orchestration, then stop and report. Record usage in `STATE_10.md`.
- Commit discipline: focused local Conventional Commits only; no attribution trailers; no remote actions. Planning-only changes use a separate `docs(planning):` commit.

## Task Specifications

### Task 10.1.1 — Benchmark SQLite contract workloads

- **ID:** 10.1.1
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** Current baseline recorded above. Task 10.1.1 may run before approval of a threshold; all later tasks are blocked on Task 10.2.1.
- **Files in scope — must edit:** `planning/reports/duckdb-sqlite-benchmark-baseline.md`.
- **Files in scope — can create:** `planning/tasks/10.1.1.todo.md`; benchmark source/scripts and outputs only under `/tmp/opencode/sda-rs-plan10-benchmark/`.
- **Do not touch:** production source, Cargo manifests/lock, external/user data, live SDA endpoints, other plans/reports, and unrelated `GOALS.md` content. No dependency additions or network access.
- **Spec:** Build a reproducible benchmark of representative existing contract-suite workloads on the current SQLite backend, using the repository's committed deterministic fixtures and real request/executor path where practical. Include scalar/select, captured non-spatial join, batch/temp-table, serialization, and configured spatial cases if deterministic fixtures permit. Record exact workload/query inputs, hardware/OS, compiler/build mode, warmups/repetitions, median and tail latency, throughput if meaningful, peak RSS method/limitations, fixture identity, and exact commands. Do not claim DuckDB performance from this baseline. Keep probes/scratch outside the repository and make the result rerunnable without network or external datasets.
- **Grep/recon anchors:** `tests/soildb_query_fixtures.rs`; `tests/sql_batch_contract.rs`; `tests/spatial_metadata_contract.rs`; `tests/serializer_contract.rs`; `tests/sda_get_contract.rs`; `src/executor/sqlite.rs` `execute_query_blocking`, `ExecutionLimits`, `execute_on_with_limits`; `Makefile`.
- **Acceptance:** The report includes deterministic workload definitions and a reproducible command/protocol; all measured SQLite workloads complete correctly and their output is checked against contract expectations; resource observations and limitations are explicit; `git diff --check` passes. No new Cargo dependency or external-data access.
- **Targeted acceptance:** Scratch benchmark run plus its correctness checks; `git diff --check`.

### Task 10.2.1 — Approve numeric performance threshold

- **ID:** 10.2.1
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.1.1 and user review of its report.
- **Files in scope — must edit:** `planning/EXECUTION_PLAN_10.md`, `planning/STATE_10.md`.
- **Files in scope — can create:** `planning/tasks/10.2.1.todo.md`.
- **Do not touch:** source, tests, manifests, other planning/research files, or branch state. No implementation and no commits by the worker.
- **Spec:** Present the baseline's workload-level latency/resource evidence and ask the user to approve a numeric DuckDB performance target (including metric, workload aggregation, minimum improvement, and regressions tolerated). Record the exact approved threshold and rationale in the plan/state. If no approval is received, stop the orchestration at this gate; do not start implementation tasks or infer a threshold.
- **Grep/recon anchors:** Task 10.1.1 report; Goal G36/G42; existing SQLite execution limits; the benchmark protocol in the report.
- **Acceptance:** After the report, pause and ask the user directly. Plan and state contain a user-attributed dated numeric threshold and measurement rule, or clearly show the orchestration blocked with no later tasks authorized. No source diff.

### Task 10.3.1 — Define backend contract and record ADR

- **ID:** 10.3.1
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.2.1 approved target. If the target is not approved, do not execute this or any subsequent task.
- **Files in scope — must edit:** `docs/decisions/README.md`.
- **Files in scope — can create:** `docs/decisions/0007-duckdb-opt-in-query-backend.md`; `planning/tasks/10.3.1.todo.md`.
- **Do not touch:** Existing accepted ADR bodies/status, production source, tests, Cargo files, or HTTP contracts outside documentation.
- **Spec:** Add ADR 0007 for an additive opt-in DuckDB query backend. Define preserved SQLite default/support, Linux x86_64 initial target, documented query/rejection surface, request-local execution/temp tables, failure semantics, no SpatiaLite requirement/fallback, no request-time extension network access, non-spatial availability when Spatial is missing, and explicit spatial errors. Explicitly state that this ADR does not itself authorize dependency changes or claim SQL Server geography parity. Update only the ADR index row; do not supersede ADR 0001 or 0003.
- **Grep/recon anchors:** `docs/decisions/README.md`; ADRs 0001–0006; `planning/GOALS.md` D6–D11; `docs/spatial-sql-contract.md`; `docs/security/sqlite-execution-policy.md`.
- **Acceptance:** New ADR is Accepted, additive, indexed, and consistent with all dated decisions; existing ADR statuses/content remain unchanged; `git diff --check`.

### Task 10.3.2 — Specify DuckDB threat, resource, and deployment controls

- **ID:** 10.3.2
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.2.1 approved target. If the target is not approved, do not execute this or any subsequent task.
- **Files in scope — must edit:** `docs/security/duckdb-execution-policy.md`.
- **Files in scope — can create:** `planning/tasks/10.3.2.todo.md`.
- **Do not touch:** SQLite policy, production source, manifests, CI, or deployment scripts.
- **Spec:** Translate the SQLite security policy to DuckDB-specific controls with documented APIs/evidence and testable deny/allow behavior: persistent writes denied; arbitrary file/network access denied; extension auto-install/user loading denied; only deployment-provisioned, pinned/hash-verified official Spatial artifact; request-local temporary tables retained; deadline/cancellation, memory/thread/concurrency/temp-storage and row/response limits explicitly covered. Resource numeric limits must be calibrated against Task 10.1.1 and remain separate from Plan 8 probe caps. Mark unresolved capability/API questions as blockers, not assumed controls. Do not use an OS sandbox.
- **Grep/recon anchors:** `docs/security/sqlite-execution-policy.md`; DuckDB evidence in `planning/reports/G33-2-duckdb-spatial-followup.md`; `src/executor/sqlite.rs` authorizer, `ExecutionLimits`, progress handler.
- **Acceptance:** Policy maps each threat to DuckDB control and adversarial test; identifies resource values and calibration evidence; no unverified claim of equivalence; `git diff --check`.

### Task 10.4.1 — Evaluate DuckDB capabilities in an isolated prototype

- **ID:** 10.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 10.3.1, 10.3.2 and approved numeric performance target.
- **Files in scope — must edit:** `planning/reports/duckdb-capability-evaluation.md`.
- **Files in scope — can create:** scratch prototype, scripts, and results only under `/tmp/opencode/sda-rs-duckdb-evaluation/`; `planning/tasks/10.4.1.todo.md`.
- **Do not touch:** production source, `Cargo.toml`/`Cargo.lock`, application configuration/routes/executors, deployment/CI, SQLite behavior, wire format, or existing ADRs. No production backend enablement.
- **Spec:** Use the already installed/pinned candidate DuckDB Rust client `duckdb = "=1.20000.2"` in an isolated local scratch prototype to evaluate what the engine/API can do before choosing production limits. Characterize representative non-spatial query, request-local connection/temp-table and batch behavior, failure semantics, result types, observed performance, and resource behavior (time, memory, threads, temp storage) where safely measurable. Probe security-relevant capabilities/settings to understand possible boundaries, including persistent writes, local file access, network/external access, and extension loading, but use only disposable scratch paths and synthetic/committed fixtures; do not access external/user data, make live network requests, or execute user-supplied/untrusted SQL. Any controlled test queries must be authored specifically for the local probe and constrained to disposable scratch resources. Record exact versions, commands, tested behavior, limitations, and what cannot be verified. Do not treat prototype observations as production controls, choose/claim production numeric limits, or claim policy parity. No repository dependency change is in scope.
- **Grep/recon anchors:** `planning/reports/G33-2-duckdb-spatial-followup.md`; `docs/security/duckdb-execution-policy.md`; `planning/reports/duckdb-sqlite-benchmark-baseline.md`; DuckDB crate source and documented settings; `src/executor/sqlite.rs` only for workload/lifecycle comparison, not as behavior to copy.
- **Acceptance:** Scratch evaluation reruns locally without external data/network; report contains reproducible capability probes and measured results/limitations, clearly separates observed behavior from production guarantees, includes the approved four-workload performance comparison where feasible and reports spatial separately; `git diff --check`. No application source or repository manifest changes. After review, stop for a follow-up user decision on production limits/control design before Tasks 10.5.1, 10.5.2, or 10.6.1 are authorized.

### Task 10.5.1 — Add DuckDB spatial and result compatibility

- **ID:** 10.5.1
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.4.1 and 10.3.2.
- **Files in scope — must edit:** `src/spatial.rs`, `src/serializer.rs`.
- **Files in scope — can create:** `tests/duckdb_spatial_contract.rs`; `planning/tasks/10.5.1.todo.md`.
- **Do not touch:** `src/transpiler/spatial.rs` behavior for rejected G12 instance methods; deployment/CI files; existing SQLite wire behavior.
- **Spec:** Implement only spatial operations already included in the documented spatial contract, with DuckDB-specific preparation, GeoPackage geometry metadata/encoding, CRS and value mapping. Preserve exact SDA serializer behavior and support ordinary non-spatial values. Keep all six G12 geography instance methods explicitly rejected unless new primary evidence and runtime-enforceable semantics have first been approved as a separate scope change. Treat missing/unverified Spatial artifact as an explicit spatial capability error while ordinary DuckDB SQL continues.
- **Grep/recon anchors:** `src/spatial.rs` initialization/metadata resolution; `src/gpkg_geometry.rs`; `src/result.rs`; `src/serializer.rs` `json_results` and XML value mapping; `docs/spatial-sql-contract.md`; `tests/spatial_methods_contract.rs`.
- **Acceptance:** Linux x86_64 tests cover geometry values/metadata/CRS/NULL/empty cases, SDA JSON/XML compatibility, supported spatial contract, all six explicit rejections, and absent-extension behavior; SQLite serializer tests remain green; formatting and diff checks pass.

### Task 10.5.2 — Provision pinned DuckDB Spatial deployment artifact

- **ID:** 10.5.2
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.3.1, 10.3.2 and selected crate/engine/platform pin from authorized Task 10.4.1.
- **Files in scope — must edit:** `Dockerfile` or existing deployment definition identified during analysis.
- **Files in scope — can create:** deployment provisioning script/manifest and its artifact verification test; `planning/tasks/10.5.2.todo.md`.
- **Do not touch:** runtime request paths, source data, arbitrary registries, existing unrelated CI, or non-Linux-x86_64 deployment targets.
- **Spec:** Provision only the official DuckDB Spatial extension at image/deployment build time for the exact pinned DuckDB engine and Linux x86_64. Verify artifact hash and compatibility before packaging; record source, version, license/provenance, and update procedure. Runtime must not install/download extensions, and absent/invalid artifacts must not disable non-spatial queries or trigger SpatiaLite fallback. Do not modify cached research artifacts.
- **Grep/recon anchors:** repository deployment files (locate during analysis); Plan 8 exact artifact evidence in `planning/reports/G33-2-duckdb-spatial-followup.md`; `docs/security/duckdb-execution-policy.md`.
- **Acceptance:** Reproducible deployment build/check confirms exact engine/platform/artifact version and SHA-256; negative tests reject a mismatched hash/version; no request-time network/install; non-spatial startup/query works when Spatial is absent; spatial request fails explicitly.

### Task 10.6.1 — Complete Linux x86_64 contract, security, CI, and operations gate

- **ID:** 10.6.1
- **Repo:** `/tmp/opencode/sda-rs-plan10-merge`
- **Depends on:** 10.4.1, 10.5.1 and 10.5.2.
- **Files in scope — must edit:** `.github/workflows/` existing Rust workflow; `CONTRIBUTING.md`.
- **Files in scope — can create:** focused Linux DuckDB contract/security/deployment tests only if not created by dependencies; `docs/duckdb-backend.md`; `planning/tasks/10.6.1.todo.md`.
- **Do not touch:** non-Linux hosted workflows, live service credentials/data, external production deployment, or unrelated docs.
- **Spec:** Wire deterministic Linux x86_64 backend tests and artifact verification into CI without request-time downloads or user data. Document opt-in selection, deployment provisioning, supported/rejected SQL, explicit spatial-unavailable behavior, security/resource controls, benchmark result/threshold, recovery and SQLite rollback path. Run all final suites serially. Hosted deployment/client trials remain deferred.
- **Grep/recon anchors:** `.github/workflows/`; `CONTRIBUTING.md`; `Makefile`; `docs/security/duckdb-execution-policy.md`; `docs/decisions/0007-duckdb-opt-in-query-backend.md`; backend contract/security test files.
- **Acceptance:** Linux x86_64 CI runs deterministic DuckDB contract/security/artifact tests; no external user data or runtime network; documentation matches tested behavior; `cargo fmt --check`, `git diff --check`, `cargo test`, `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`, and all DuckDB Linux x86_64 tests pass serially. Approved numeric target is met or the release gate is reported failed for user decision.

## Final Acceptance Gate

Orchestrator-owned and serial: `cargo fmt --check`; `git diff --check`; `cargo test`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; Linux x86_64 DuckDB contract/security/resource/deployment tests; benchmark comparison against the Task 10.2.1 approved target. Record exact counts, artifact hashes, hardware/runtime versions, resource observations, and any deviations in `STATE_10.md`. If a gate fails twice for the same reason, stop and escalate per the worker policy.
