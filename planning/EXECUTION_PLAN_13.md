# Execution Plan 13 — DuckDB Qualification Through G42

## Goal & Scope

### Goal

Qualify the already-authorized optional DuckDB path through query semantics, execution lifecycle, security/resource-control mechanisms, result compatibility, deployment provisioning, and a final performance/resource assessment. Resolve the stale Plan 7 state record. Preserve SQLite as default; do not add runtime backend selection or claim production readiness. Explore transformations for SQL Server geography methods, implementing only bounded, runtime-enforced domains whose tests establish the required SQL Server contract. End with G42 measurements and candidate resource-limit recommendations; do not make the 20% performance objective a pass/fail gate.

### In scope

- Reconcile Plan 7 reporting with its completed G31/G8/G9 work; leave any unrun Plan 7 final acceptance gate accurately represented.
- Clarify ADR 0007 and the Plan 13 decision record: the 20% performance target is an objective, not a blocker.
- G37: specify the production SQL support/rejection contract and investigate/implement mathematically valid transformations for the six named geography methods in bounded domains where SQL Server semantics can be established.
- G38: build and test a request-local DuckDB execution boundary, including temporary state, batch failure behavior, blocking execution, and cancellation where the pinned API permits.
- G39: implement/test DuckDB-specific security and resource-control mechanisms. Do not invent final numeric resource caps; G42 measures and recommends candidates.
- G40: validate GeoPackage metadata/geometry handling, DuckDB value mapping, and SDA response compatibility.
- G41: make Linux x86_64 DuckDB/Spatial provisioning reproducible, pinned, hash-verified, and independent of request-time network access.
- G42: compare representative workloads with the SQLite baseline; measure/report performance and resource behavior and recommend candidate caps. Missing the 20% performance objective alone does not block qualification.
- Run targeted task checks and serial full-suite gates at every wave boundary.

### Out of scope / deferred

- G43 runtime backend selection, routes/configuration that expose DuckDB to requests, release enablement, or changing SQLite defaults.
- G44 CI integration, G45 operations documentation, hosted trials, and general service rollout.
- General SQL Server geography parity. The six G12 methods may be supported only within a rigorously defined and runtime-enforced domain with compatibility evidence; otherwise preserve explicit rejection. No planar approximation may be described as geography-compatible.
- Final numeric resource-limit approval. G42 may recommend candidates; the user must approve final values before any later runtime-enablement plan.
- Adding a Cargo dependency without separate explicit user approval. Candidate dependencies may be researched and proposed.
- Any network access by submitted requests, request-time extension install/autoload, silent SpatiaLite fallback, OS sandboxing, user-data/Iowa GeoPackage access, or live SDA requests.

### Verbatim user constraints and decisions

- “we should write that all out including thru G41 as the next plan, capped with 42.”
- “performance is not a hard limit or blocker if we dont achieve it. i think these are sufficient”
- “it would be good if we could do the geography methods thru some creative transformation tho”
- “yes” — require SQL Server geography semantics for any method called compatible; reject approximations/planar substitutes.
- “yes” — a bounded, runtime-enforced subset is acceptable when compatibility tests pass; reject outside its proven domain.
- “yes” — measure and recommend DuckDB resource-limit candidates in G42; leave final values and runtime enablement gated pending approval.
- “TRACK” — track Plan 13 artifacts; keep planning-only commits separate.
- Max two workers in isolated worktrees for disjoint scopes; run shared Cargo suites serially.
- Use `opencode-go/deepseek-v4.1-flash`, with at most one escalation to `openai/gpt-6-luna`.
- At wave boundaries run formatting, diff check, default and configured-SpatiaLite full suites, and relevant offline DuckDB-feature tests serially.

## Dated Decisions

- **2026-11-20 — Scope:** Plan 13 covers Plan 7 status reconciliation and G37–G41, ending with G42. G43–G45 are explicitly deferred to a later plan.
- **2026-11-20 — G31:** Implementation and targeted tests are complete. Current branch `STATE_7.md` is stale; reconcile it from the completed Plan 7 records. Do not reimplement G31 or mark Plan 7's final acceptance complete unless its own final gate is evidenced.
- **2026-11-20 — Performance:** Clarify ADR 0007: the approved 20% latency target is an objective and reporting criterion, not a hard pass/fail qualification or release gate. Missing it alone does not block further work.
- **2026-11-20 — Geography:** Investigate transformations for all six G12 methods. A bounded exact subset is acceptable only with enforceable runtime guards and tests supporting SQL Server `geography` semantics; otherwise retain explicit rejection. Never claim planar or approximate results are compatible.
- **2026-11-20 — Dependencies:** Workers may research candidate dependencies but must stop and request separate user approval before adding one.
- **2026-11-20 — Resource limits:** G42 may measure and recommend candidate DuckDB limits. Do not finalize numeric values or enable runtime selection without later explicit approval.
- **2026-11-20 — Backend boundary:** Build/test qualification components without making DuckDB selectable by HTTP requests. Preserve ADRs 0001–0007 except the explicit ADR 0007 performance clarification.
- **2026-11-20 — Execution:** Maximum two workers, isolated worktrees for concurrent tasks; serial shared test suites. Current branch is `feature/duckdb-feature1`; do not switch branches or touch the separate Plan 10 worktree.
- **2026-11-20 — Artifacts:** Track Plan 13 artifacts and keep planning-only commits separate from implementation.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-feature1`, starting HEAD `bbf48f9`. Preserve the current uncommitted `planning/GOALS.md` update; stage only explicitly scoped paths.
- A separate worktree `/tmp/opencode/sda-rs-plan10-merge` on `feature/duckdb-production-backend` has untracked Plan 10 drafts (`planning/tasks/10.1.1.todo.md` and `planning/reports/duckdb-sqlite-benchmark-baseline.md` there). Do not modify, stage, or remove that worktree. The committed benchmark report is available on this branch at `planning/reports/duckdb-sqlite-benchmark-baseline.md`.
- Plan 7 source changes for metadata and configuration match `spatial-readiness`; its latest state records G31/G8/G9 tasks complete and targeted tests passing. The state copy on this branch still has pending waves and no reports. Plan 7's own final acceptance gate remains pending and must not be marked complete without evidence.
- Recent Plan 12 final checks on this code: default and configured-SpatiaLite `cargo test` each passed (75 unit + 30 integration); offline `duckdb_pipeline_contract` and `--all-features` target each passed (12 tests); `cargo fmt --check` and `git diff --check` passed. Only planning goals have changed since those code tests.
- Cargo already has optional bundled `duckdb = "=1.20000.2"` and a `duckdb` feature; `tests/duckdb_pipeline_contract.rs` is feature-gated. No production `src/` module imports DuckDB yet.
- Main anchors: `src/transpiler/spatial.rs` (1,519 lines), `src/executor/sqlite.rs` (591), `src/routes.rs` (330), `src/serializer.rs` (448), `tests/duckdb_pipeline_contract.rs` (481), `docs/spatial-sql-contract.md`, and `docs/security/duckdb-execution-policy.md`. Use grep anchors; do not read large files wholesale.
- Plan 8 records DuckDB 2.0.0 / Rust client 1.20000.2 and an official Spatial artifact hash, plus bounded-read and synthetic operation evidence. These observations are not automatically a production pin or semantic guarantee.
- The previously used Plan 8 extension cache path is absent in this checkout. G37/G41 tests must provision a verified local official artifact into their scoped scratch/deployment path or report the exact availability blocker; do not silently depend on a developer cache.
- No SQL Server runtime oracle is recorded as available. Where direct runtime comparison is impossible, use sourced SQL Server contracts and independent reference calculations; if exact compatibility cannot be established for a bounded domain, preserve rejection.
- Plan 9 explicitly rejected all six geography methods at the SQLite/SpatiaLite seam because safe input bounds were not enforceable there. Plan 13's DuckDB transformations must establish a new, enforceable contract rather than assume those rejections are false.
- The DuckDB security policy identifies write/file/network/extension controls, cancellation, memory/thread/concurrency/temp-storage and input/output limits as blockers until tested. SQLite limits and Plan 8 probe settings are not DuckDB production limits.
- Existing benchmark report: `planning/reports/duckdb-sqlite-benchmark-baseline.md`, deterministic SQLite workload protocol and results. It is a single-host microbenchmark, not HTTP/production-load evidence; peak RSS and concurrency are unmeasured.
- Accepted ADRs 0001–0007 preserve SQLite default, request-local execution, batch failure semantics, optional spatial capability, and verified deployment provisioning. Plan 13 does not authorize G43 selection or supersede those contracts.

## Dependency Graph and Waves

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 13.1.1, 13.1.2 | Reconcile Plan 7 state and clarify the durable performance decision. |
| 2 | 13.2.1 | Research the G37 geography contract and transformation candidates for all six methods. |
| 3 | 13.3.1 | Apply evidence-backed predicate transformations/rejections (`STContains`, `STWithin`). |
| 4 | 13.4.1 | Apply evidence-backed overlay transformations/rejections (`STIntersection`, `STBuffer`). |
| 5 | 13.5.1 | Apply evidence-backed measurement/text transformations/rejections (`STArea`, `STAsText`) and close G37. |
| 6 | 13.6.1 | Implement/test the internal DuckDB execution lifecycle (G38), without route selection. |
| 7 | 13.7.1 | Implement/test DuckDB security-control mechanisms (G39); leave numeric caps for G42. |
| 8 | 13.8.1 | Implement/test GeoPackage and result compatibility (G40). |
| 9 | 13.9.1 | Implement/test reproducible, pinned Linux x86_64 Spatial provisioning (G41). |
| 10 | 13.10.1 | Benchmark and report performance/resource behavior and candidate limits (G42). |

Waves are sequential by dependency and phase. Wave 1 tasks have disjoint files and may run in parallel in isolated worktrees. Waves 3–5 share the spatial transpiler/contract paths and are sequential. G38–G40 share executor/value boundaries and are sequential. At most two workers run concurrently; no shared Cargo suite runs concurrently. If G37 needs a new dependency, stop for separate user approval before proceeding with that dependency.

## Global Hazards and Rules

- Preserve SQLite as default and supported backend; do not add route/configuration selection or make DuckDB reachable from HTTP requests (G43 is out of scope).
- Do not weaken ADR 0001, 0002, 0004, 0005, or 0007 contracts. Only the approved D9 performance clarification is in scope for ADR 0007.
- Geography compatibility is not planar geometry compatibility. Every supported subset needs runtime-enforced guards and compatibility tests; otherwise reject explicitly. Do not assume AST shape proves runtime geometry dimensions, SRID, extent, or validity.
- No dependency addition until separate explicit user approval. If a dependency is necessary, stop and report the precise need and options.
- Treat the DuckDB policy's blockers as real. Do not claim security parity, bounded resources, deployability, or production readiness if an enforcement/API question remains unresolved.
- G39 tests may use injected test limits, but must not declare final production numeric caps. G42 recommends candidate values; final values need later approval before G43.
- G41 provisions only pinned official artifacts and verifies hashes before service use; requests never install/download extensions. No silent SpatiaLite fallback.
- Synthetic/committed fixtures only. Do not access Iowa/soilDB source data or live services for DuckDB qualification.
- No G43 backend selection, G44 CI, G45 operations docs, hosted trials, OS sandbox, or remote Git actions.
- Workers create a scoped micro-TODO and stop for orchestrator approval before implementation. Workers run only named targeted tests; orchestrator runs wave suites serially.
- Preserve current branch and pre-existing changes, including the dirty `planning/GOALS.md` and the separate Plan 10 worktree. No broad staging, destructive Git operations, or remote actions.
- Planning artifacts are tracked and committed separately with `docs(planning):`; implementation commits are focused Conventional Commits without attribution trailers.
- Escalation cap: one escalation total to `openai/gpt-6-luna`; after a repeated same-cause failure twice or acceptance failure after one retry, stop and report.
- Standard orchestration bookkeeping: each task may create `planning/tasks/13.W.ID.todo.md` for its required micro-plan and append its <=15-line result to `planning/STATE_13.md`; these bookkeeping files are the only additions to each task's explicit deliverable file scope.

## Task Specifications

### Task 13.1.1 — Reconcile Plan 7 completion records

- **ID:** 13.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** Plan 7 task reports and implementation commits cited in this plan.
- **Files in scope — must edit:** `planning/STATE_7.md`, `planning/GOALS.md`.
- **Do not touch:** source/tests, other plans/states/reports, Plan 10 worktree, or external data.
- **Spec:** Reconcile the current branch's stale Plan 7 wave/task records using `spatial-readiness` commit `60584ed` and the integrated implementation commits. Record 7.1.1 (G31) and 7.1.2 (G8/G9) as complete with their exact commit/test evidence. Reconcile the remaining Plan 7 wave statuses only where authoritative reports support them; leave Plan 7's orchestrator final acceptance pending unless its full gate is evidenced. Move G31/G8/G9 goals to Completed in `GOALS.md`, explicitly distinguishing completion of those goal outcomes from completion of Plan 7's overall final gate. This is documentation reconciliation, not reimplementation.
- **Grep anchors:** `planning/STATE_7.md` wave board/task reports; `git show 60584ed:planning/STATE_7.md`; current-branch implementation commits `257202a`, `e93eed0` (equivalent integration commits on `spatial-readiness`: `65dabbe`, `08c212a`); `tests/spatial_metadata_contract.rs`, `tests/body_limit_contract.rs`.
- **Acceptance:** Plan 7 state clearly distinguishes completed tasks/waves from any pending final gate; GOALS agrees; every completion statement has commit/test provenance; `git diff --check`.

### Task 13.1.2 — Clarify the non-blocking performance objective in ADR 0007

- **ID:** 13.1.2
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** User's 2026-11-20 D9 clarification recorded in this plan.
- **Files in scope — must edit:** `docs/decisions/0007-duckdb-opt-in-query-backend.md`.
- **Do not touch:** ADR index/title/status, other ADRs, Plan 10 history, or implementation.
- **Spec:** Preserve the 20% comparison target, workload aggregation, and 10% individual-regression reporting rule as an objective. Clarify that missing the objective alone is not a qualification/release blocker. Keep numeric resource-limit approval and security gates separate. Do not weaken other ADR 0007 requirements.
- **Grep anchors:** ADR 0007 “Decision Outcome” performance paragraph; Plan 10 D9; this plan's dated decision.
- **Acceptance:** ADR unambiguously states the performance objective is non-blocking and is consistent with this plan and GOALS D9; all unrelated ADR commitments are unchanged; `git diff --check`.

### Task 13.2.1 — Research G37 geography transformations and contract

- **ID:** 13.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.1.1, 13.1.2.
- **Files in scope — must edit:** `planning/reports/G37-geography-transformations.md`.
- **Can create:** synthetic-only scratch probes and locally cached official extension artifacts under `/tmp/opencode/sda-rs-plan13-geography/`.
- **Do not touch:** production source, Cargo manifests/lock, existing contract/report files, user data, or external repositories.
- **Spec:** For all six methods, document SQL Server `geography` requirements and Plan 8/9 evidence; investigate transformations and candidate libraries that could preserve semantics over a bounded, runtime-enforceable input domain. Cite authoritative sources, define expected inputs/results/NULL/errors/units/SRID behavior, propose adversarial/boundary tests, and distinguish proven facts from hypotheses. Do not claim equivalence from names or a small happy-path probe. A local official DuckDB extension may be provisioned into the scoped scratch directory for synthetic offline probes only; record source, version, and hash. No user data, request-time install, or external network from tests. If a new dependency is needed, report the need and stop for user approval before adding it or relying on it.
- **Grep anchors:** `planning/reports/G12-geography-method-feasibility.md`; `planning/reports/G33-2-duckdb-spatial-followup.md`; `docs/spatial-sql-contract.md`; `docs/security/duckdb-execution-policy.md`; `src/transpiler/spatial.rs::rewrite_node`.
- **Acceptance:** report covers each method, cites primary SQL Server/DuckDB references, defines feasible bounded-domain candidates or explicit no-go, includes reproducible scratch commands/results if probes run, and claims no unverified parity; `git diff --check`.

### Task 13.3.1 — Apply evidence-backed Contains/Within behavior

- **ID:** 13.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.2.1; separate approval first if the report requires a new dependency.
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`, `tests/spatial_methods_contract.rs`, `tests/duckdb_geography_contract.rs` (create).
- **Files in scope — may edit:** `Cargo.toml`, only to register the new `duckdb_geography_contract` target once with `required-features = ["duckdb"]`. Task 13.3.1 owns target registration; later G37 tasks must not alter Cargo target configuration.
- **Do not touch:** `src/routes.rs`, config, runtime selector, other geography methods, SQLite executor/security, or response shape.
- **Spec:** Address only `.STContains(...)` and `.STWithin(...)`. Implement only transformations established by Task 13.2.1 to preserve SQL Server semantics within runtime-enforced bounds; otherwise retain/strengthen explicit rejection. Test accepted and rejected domains, boundaries, SRID mismatch, NULL, invalid/empty inputs, and result type as applicable. No planar substitution.
- **Grep anchors:** `Expression::MethodCall` and unsupported method handling in `src/transpiler/spatial.rs`; current `tests/spatial_methods_contract.rs`; G37 report.
- **Acceptance:** `cargo test --offline --test spatial_methods_contract`; `SDA_DUCKDB_SPATIAL_EXTENSION=<verified-local-extension> CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`; configured SpatiaLite method test if any SQLite behavior is touched; `cargo fmt --check`, `git diff --check`. Tests prove guards reject inputs outside any supported domain. Extension test setup must not download/install at request time.

### Task 13.4.1 — Apply evidence-backed Intersection/Buffer behavior

- **ID:** 13.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.3.1.
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`, `tests/spatial_methods_contract.rs`, `tests/duckdb_geography_contract.rs`.
- **Do not touch:** other goals' executor/security/deployment files, routes/config, or unrelated methods.
- **Spec:** Address only `.STIntersection(...)` and `.STBuffer(...)`. Apply only evidence-backed, runtime-enforced transformations; otherwise explicitly reject them. Cover distance units, output geometry/type, SRID handling, empty/disjoint/NULL/error and boundary cases applicable to the proven domain. Do not lower to planar operations as geography.
- **Grep anchors:** method rewrite/validation helpers in `src/transpiler/spatial.rs`; G37 report; DuckDB feature-gated geography tests.
- **Acceptance:** `cargo test --offline --test spatial_methods_contract`; `SDA_DUCKDB_SPATIAL_EXTENSION=<verified-local-extension> CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`; configured SpatiaLite tests only if SQLite behavior is changed; `cargo fmt --check`, `git diff --check`.

### Task 13.5.1 — Apply evidence-backed Area/AsText behavior and close G37

- **ID:** 13.5.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.4.1.
- **Files in scope — must edit:** `src/transpiler/spatial.rs`, `docs/spatial-sql-contract.md`, `tests/spatial_methods_contract.rs`, `tests/duckdb_geography_contract.rs`.
- **Do not touch:** executor lifecycle/security, deployment, routes/config, or unrelated behavior.
- **Spec:** Address only `.STArea()` and `.STAsText()`. Implement a method only if the transformation and enforced bounds establish SQL Server units, type/text and edge behavior; otherwise retain explicit rejection. Update the method matrix in the spatial contract with supported domains and tested exclusions for all six methods.
- **Grep anchors:** method rewrite and rejection helpers; `docs/spatial-sql-contract.md` six-method section; Task 13.2.1 report.
- **Acceptance:** `cargo test --offline --test spatial_methods_contract`; `SDA_DUCKDB_SPATIAL_EXTENSION=<verified-local-extension> CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`; docs match tested support/rejection for all six methods; `cargo fmt --check`, `git diff --check`.

### Task 13.6.1 — Implement the internal DuckDB execution lifecycle (G38)

- **ID:** 13.6.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.5.1.
- **Files in scope — must edit:** `src/executor/mod.rs`; `src/executor/duckdb.rs` (new).
- **Can create:** `tests/duckdb_execution_contract.rs`; feature-gate target in `Cargo.toml` if required.
- **Do not touch:** `src/routes.rs`, `src/config.rs`, public selector/HTTP response, SQLite executor behavior, security policy values, or deployment.
- **Spec:** Build an internal feature-gated executor boundary using the pinned DuckDB crate. Preserve request-local connection/temp state, blocking-worker execution, whole-batch failure semantics, and cleanup. Probe cancellation/deadline support on the exact client; implement only a demonstrated mechanism and report unresolved limits as blockers. Do not route user requests to this executor.
- **Grep anchors:** `src/executor/sqlite.rs::execute_query_blocking`, `ExecutionLimits`, transaction handling; ADRs 0002/0004/0007; Plan 10 scratch pipeline.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; tests cover isolated request state, successful temp workflow, failed-batch behavior and cleanup; cancellation test or a documented, reproducible blocker; default and full-suite gates remain orchestrator-owned.

### Task 13.7.1 — Implement/test DuckDB security mechanisms (G39)

- **ID:** 13.7.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.6.1.
- **Files in scope — must edit:** `src/executor/duckdb.rs`, `docs/security/duckdb-execution-policy.md`.
- **Can create:** `tests/duckdb_security_contract.rs`; feature-gate target in `Cargo.toml` if required.
- **Do not touch:** routes/config selection, SQLite security policy, OS sandbox, unrelated docs, or final numeric resource defaults.
- **Spec:** Verify and enforce persistent-write denial, the narrowly allowed temp-table workflow, file/network access denial, extension autoload/install/user-load denial, and cleanup using the pinned engine/client. Qualify the internal executor while it remains unreachable from requests; do not add a production request path or route integration. Use injected limits in tests if needed; do not choose final numeric memory/concurrency/row/byte/deadline/temp-storage caps. Mark any unverified setting/API as a blocker, not a guarantee.
- **Grep anchors:** `docs/security/duckdb-execution-policy.md`; DuckDB `Connection` setup in `src/executor/duckdb.rs`; SQLite authorizer/limits in `src/executor/sqlite.rs`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` and `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; each required deny/allow case asserted, with lifecycle contract retained; policy matches tested mechanisms and explicitly leaves unproven controls blocked; fmt/diff checks.

### Task 13.8.1 — Validate DuckDB GeoPackage and result compatibility (G40)

- **ID:** 13.8.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.1.1, 13.5.1, 13.6.1.
- **Files in scope — must edit:** `src/executor/duckdb.rs`.
- **Can create:** `tests/duckdb_result_contract.rs`; feature-gate target in `Cargo.toml` if required.
- **Do not touch:** `src/serializer.rs` public response behavior, routes/config, SQLite result semantics, deployment, or test-only Plan 11 adapters.
- **Spec:** Convert DuckDB outputs to existing `ResultSet` values without lossy silent coercion; explicitly reject unsupported types. Validate GeoPackage geometry metadata/encoding, CRS/SRID, NULL/empty/dimensional cases within the G37 contract, and exact SDA serialization. Keep any DuckDB-to-shared-value adapter in production executor code, not the Plan 11 test adapter.
- **Grep anchors:** `src/result.rs::ResultSet`; `src/serializer.rs::serialize`; `tests/serializer_contract.rs`; Plan 11 test-only value adapter as a comparison only.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_result_contract` and `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_execution_contract`; exact expected columns, values, metadata and serialized response; lifecycle contract remains green; no unapproved response-shape change; fmt/diff checks.

### Task 13.9.1 — Provision pinned DuckDB Spatial reproducibly (G41)

- **ID:** 13.9.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.6.1, 13.7.1, 13.8.1.
- **Files in scope — must edit:** `docs/duckdb-deployment.md`.
- **Can create:** `scripts/provision-duckdb-spatial.sh`, focused script tests/fixtures under `tests/` if needed.
- **Do not touch:** CI (G44), runtime route/selection (G43), request-time networking, existing SQLite deployment, or external user data.
- **Spec:** Select and document the exact Linux x86_64 engine/client and official Spatial artifact/version/source/hash; provision only into a deployment-controlled path; verify hash before load; configure extension loading so no request-time install/network or user-selected extension is possible. Reuse Plan 8 fingerprints only after confirming they match the selected engine/artifact. Keep non-spatial behavior available if Spatial is absent and spatial failure explicit. Script tests must use local/cached artifacts and must not download during task validation.
- **Grep anchors:** D10 in `planning/GOALS.md`; ADR 0007; DuckDB Spatial evidence in `planning/reports/G33-2-duckdb-spatial-followup.md`; executor initialization in `src/executor/duckdb.rs`.
- **Acceptance:** `bash -n scripts/provision-duckdb-spatial.sh`; local/offline provisioning verification checks expected artifact identity, hash mismatch rejection, and absent-artifact behavior; deployment doc gives exact reproducible steps and no request-time network path; `git diff --check`.

### Task 13.10.1 — Measure DuckDB performance/resources and recommend limits (G42)

- **ID:** 13.10.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 13.6.1, 13.7.1, 13.8.1, 13.9.1.
- **Files in scope — must edit:** `planning/reports/duckdb-qualification-benchmark.md`.
- **Can create:** benchmark source and manifest only under `/tmp/opencode/sda-rs-plan13-benchmark/`.
- **Do not touch:** production defaults/limits, routes/selection, user data, network, repository dependency manifests, or benchmark baseline report.
- **Spec:** Follow the scratch-harness pattern and public `sda_rs::executor` API used by the committed SQLite benchmark baseline; use a local path dependency with feature `duckdb` to run both executor paths without repository benchmark-target/manifest edits. Re-run deterministic scalar/select, captured join, batch/temp-table, serialization, and separately reported spatial workloads. Assert outputs before timing and each measured iteration. Record exact commands, hardware/OS, versions, build mode, warmups/repetitions, correctness checks, latency distributions, throughput where meaningful, concurrency and resource observations (peak RSS method/limitations, threads, memory, temp storage). Compare with `planning/reports/duckdb-sqlite-benchmark-baseline.md`; report the 20% aggregate objective and any workload regression over 10%, but neither is a hard gate. Recommend candidate resource limits with evidence/uncertainty; do not finalize numeric caps or enable runtime selection. Record that approval of final caps must be an explicit dated decision in `GOALS.md` and a later G43 plan before runtime selection.
- **Grep anchors:** committed SQLite benchmark report; G38–G41 reports/tests; `docs/security/duckdb-execution-policy.md`; Plan 10 D9 and this plan's clarification.
- **Acceptance:** `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan13-benchmark/target cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-plan13-benchmark/Cargo.toml`; runner asserts workload results before and during measurements and prints/documented raw samples sufficient to reproduce medians/p95; report has matching explicit comparability limits, 20% objective and >10% regression observations (neither a gate), candidate caps and explicit approval handoff; `git diff --check`.

## Final Acceptance Gate

Workers run only their named targeted tests. At every wave boundary, the orchestrator serially runs `cargo fmt --check`, `git diff --check`, default `cargo test`, configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`, and relevant offline DuckDB feature target(s), then records exact outcomes in `STATE_13.md`. G42 ends this plan with a measured report and candidate resource recommendations only; final caps require an explicit dated approval in `GOALS.md` and a later G43 plan. No runtime selection or production-readiness claim is authorized.
