# Execution Plan 14 — Evidence and Investigation for Secure DuckDB Integration

## Goal & Scope

Produce a broad, reproducible evidence package that determines whether and how an optional DuckDB backend can be integrated with reasonable safeguards. Investigate the current old pin, DuckDB security controls and bypasses, the private GeoPackage loader, the six SQL Server geography methods, and representative performance/resource behavior. Base every verdict on cited sources or bounded repeatable probes; a negative or bounded inconclusive finding is a valid outcome only after the prescribed investigation is run. End with a concrete integration architecture, residual-risk register, and explicit go/no-go criteria. This plan is evidence-first: runtime request selection and production enablement remain a separate approval.

**In scope**
- Current authoritative research on DuckDB releases, Rust-client/engine/Spatial compatibility, security guidance and advisories; no dependency or version change.
- Static audit and direct pinned-client probes of G39 controls, including attempts that bypass the executor classifier.
- Test and trust-boundary review of the private Rust-side GeoPackage loader, with synthetic data only.
- Primary-source semantic review and pinned synthetic probes for all six geography methods, with particular attention to `.STArea()` axis order/reference/tolerance/guards and `.STAsText()` dimensional and text behavior.
- A reproducible internal benchmark and resource observation using synthetic/captured deterministic workloads, reported with scope limits.
- A final secure-integration design and risk/decision matrix. Any proposed code fix is separately described and requires explicit user approval before implementation.

**Out of scope**
- Changing `Cargo.toml`, `Cargo.lock`, DuckDB/Spatial pins, or adding dependencies; downloads or network access from tests/probes; use of real user/Iowa/soilDB data.
- Runtime backend selection, routes, request-path exposure, production enablement, numeric production caps, OS sandbox implementation, CI/release work, or claiming production readiness.
- Silently treating planar/spherical/approximate operations as SQL Server geography-compatible.
- Implementing fixes discovered during investigation without a separate, explicit user approval.

**Verbatim user constraints**
- “all of these need evidence and proper investigation, without stopping on arbitrary non-blockers”
- “the next plan has to be a wide, multiwave evidence/investigation plan incorporating as much research as is needed to scope out the above goals and overall goal of secure duckdb backend integration with reasonable safeguards”

## Current facts and key hazards

- Current branch is `feature/duckdb-feature1`, HEAD `f5b8ab1`; `planning/GOALS.md` has an uncommitted user-requested update and must be preserved.
- The repo is a Rust project; see `AGENTS.md`, `CONTRIBUTING.md`, ADRs 0001–0007, and `planning/STATE_13.md`.
- `src/executor/duckdb.rs` is 929 lines; `src/transpiler/spatial.rs` is 1,575 lines; `tests/duckdb_geography_contract.rs` is 472 lines. Workers must use grep anchors, not whole-file reads.
- Exact current pin: Rust crate `duckdb = "=1.20000.2"` with bundled DuckDB engine v2.0.0; the Spatial artifact is v2.0.0 linux_amd64 with SHA-256 `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`. Existing artifact is local at `/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension`.
- Official DuckDB announced 2.0.1 on 2026-11-20 and explicitly lists security fixes affecting `enable_external_access=false`, `CREATE SECRET`, and remote path traversal. This makes pin assessment urgent, but does not authorize an upgrade. Research must establish applicability to the bundled 2.0.0 pin and identify client/extension compatibility questions.
- Direct prior probe found `ST_Read` read an outside file despite `allowed_paths`; do not treat that setting as a security boundary. Current G40 uses Rust-side SQLite read-only loading instead.
- G39 has selected denials but a string-based classifier and finite coverage. Distinguish what direct engine configuration enforces from what only the classifier rejects; do not declare exhaustiveness from finite tests.
- The loader is private qualification code, not wired to deployment setup or HTTP. Passing its tests does not establish end-to-end lifecycle or production trust provisioning.
- Geography methods remain rejected. Existing tests are observations on the pinned build, not SQL Server oracle results. `.STArea()` axis/reference/tolerance/guard evidence is especially incomplete.
- G42's 20% latency target and >10% regression reporting criterion are objectives, not pass/fail gates. Resource observations are useful even if precise production caps cannot yet be recommended.
- Existing source/data and tests must remain local and synthetic. Shared Cargo suites and DuckDB artifact probes run serially.

## Decisions

- **2026-11-22 — Evidence bar:** An investigation is complete when it has reproducible positive, negative, or bounded inconclusive evidence. Lack of prior documentation alone is not a blocker.
- **2026-11-22 — Research/probe authority:** Current official-source research and bounded offline synthetic probes are authorized. No network/download in test runs, no dependency or pin change, and no real user/Iowa data.
- **2026-11-22 — Fixes:** Evidence-first plan may identify narrowly scoped fixes, but every implementation change must be presented to the user and receive separate explicit approval before execution.
- **2026-11-22 — Pin assessment:** Research current release and compatibility information first. Any alternate-version local build or repository pin/dependency change requires separate approval.
- **2026-11-22 — Parallelism:** At most two workers. Shared tests and artifact probes run serially.
- **2026-11-22 — Security target:** Deliver a defensible integration architecture, safeguards, residual risks, and go/no-go gates. Request-path enablement, OS sandboxing, and final numeric production limits remain out of scope pending separate approval.
- **2026-11-22 — Tracking:** Track Plan 14 artifacts in Git and commit planning files separately from any implementation.
- **2026-11-22 — ADR alignment:** Preserve accepted ADR 0007: SQLite remains default; DuckDB is optional; no request selector, no silent fallback, and explicit geography rejection absent evidence. This plan does not supersede the ADR.

## Dependency graph and wave table

| Wave | Tasks | Purpose / dependency |
|---|---|---|
| 1 | 14.1.1, 14.1.2 | Establish current pin/release/security context and audit current G39/G40 state; independent reports. |
| 2 | 14.2.1, 14.2.2 | Run direct pinned-client security probes and independently revalidate the private GPKG loader; both use synthetic/offline inputs. The loader task does not depend on the security probe. |
| 3 | 14.3.1, 14.3.2 | Complete authoritative security-control research and refresh SQL Server geography contract; security follow-up depends on security probes; geography source research proceeds independently of upgrade compatibility. |
| 4 | 14.4.1, 14.4.2 | Probe all six geography candidates on the pinned artifact and test G39 defense mechanisms/gaps; geography depends on 14.3.2, security on 14.3.1. |
| 5 | 14.5.1 | Measure representative DuckDB workload/resource behavior; depends on baseline plus G39/G40 characterization, not on eliminating every residual risk. |
| 6 | 14.6.1 | Synthesize secure backend integration design, residual risks, upgrade recommendation and explicit decision gates from all prior evidence. |

Task files are confined to their named report or scratch-output paths. Wave tasks have disjoint output files. At most two workers run concurrently; no shared test suite or shared-artifact probe runs concurrently.

## Task specifications

### Task 14.1.1 — Assess DuckDB pin, release status, and security-relevant upgrade impact
- **Repo:** `/home/andrew/workspace/sda-rs`; **Depends on:** none.
- **Files in scope — must edit:** `planning/reports/duckdb-pin-and-upgrade-assessment.md`.
- **Can create:** cited research notes in that report only.
- **Do not touch:** manifests/lockfile, source, deployment artifacts, other reports, or network-dependent test setup.
- **Spec:** Research official DuckDB release calendar, 2.0.0/2.0.1 release notes, security documentation/advisories, DuckDB Rust crate release mapping and Spatial extension compatibility/provenance. Determine precisely why the repository selected crate `1.20000.2` (DuckDB 1.2.0 line)/engine 2.0.0, what is actually old or unsupported, and whether the 2.0.1 security items apply to bundled 2.0.0. Provide a source-based comparison matrix for current pin versus a suitable newer candidate across Rust API/client, bundled engine, Spatial artifact, security fixes, loader/result behavior, and geography functions. Mark runtime-dependent axes unverified rather than treating them as blockers. Recommend retain/evaluate/upgrade, without changing the pin or building an alternate version.
- **Grep anchors:** `duckdb` entry in `Cargo.toml` and `Cargo.lock`; “Pin” in `docs/duckdb-deployment.md`; “Reproducibility and extension/security” in `planning/reports/G33-2-duckdb-spatial-followup.md`; official source URLs cited in output.
- **Acceptance:** Report includes dated primary-source citations, exact crate→engine→extension identity, applicability analysis for each relevant security fix, all comparison axes with evidence or explicit unverified status, known compatibility/upgrade risks, and recommendation; `git diff --check`. Research may use web sources; no downloads, alternate-version builds, or changes.

### Task 14.1.2 — Inventory current DuckDB security and GeoPackage claims
- **Repo:** same; **Depends on:** none.
- **Files in scope — must edit:** `planning/reports/duckdb-security-and-loader-evidence.md`.
- **Do not touch:** implementation, tests, policy, other reports.
- **Spec:** Map each current policy claim to the exact implementation path and existing test; label as engine-enforced, classifier-enforced, loader-enforced, tested observation, or unverified. Include known `allowed_paths`/`ST_Read` failure, G39 parser/bypass caveat, G40 private-loader scope, lifecycle/resource gaps, and claims not to make. This is a baseline inventory, not a verdict that missing evidence is a blocker.
- **Grep anchors:** `src/executor/duckdb.rs` symbols `validate_sql_policy`, `open_connection`, `load_geopackage`, `map_value`; `tests/duckdb_security_contract.rs`; `tests/duckdb_result_contract.rs`; `docs/security/duckdb-execution-policy.md`.
- **Acceptance:** Traceability matrix cites implementation symbols and named tests for every current claim; discrepancies/open probe questions are explicit; `git diff --check`.

### Task 14.2.1 — Probe G39 controls directly on the pinned engine
- **Repo:** same; **Depends on:** 14.1.1, 14.1.2.
- **Files in scope — must edit:** `planning/reports/duckdb-security-probe-results.md`.
- **Can create:** disposable probe source/build artifacts only under `/tmp/opencode/sda-rs-plan14-security/`; no committed harness unless separately approved.
- **Do not touch:** repository source/manifests/tests, network, real data, user files, or deployment.
- **Spec:** Using current locked crate/engine offline, exercise direct DuckDB Connection calls that bypass the executor classifier, and run the same adversarial inputs through the classifier. Probe external file/URL functions, secret/settings/config changes, ATTACH/IMPORT/COPY, persistent writes, extension install/load/autoload, allowed-path behavior, compound statements/comments/quoting, and temp-table compatibility. For each, record paired classifier outcome and direct-engine outcome. Avoid destructive or outward network operations; show remote-access denial via external-access configuration/errors without contacting an external endpoint. Do not treat classifier rejection as proof of engine enforcement.
- **Grep anchors:** `Config` construction and policy classifier in `src/executor/duckdb.rs`; current G39 tests; 2.0.1 security fixes in 14.1.1 report.
- **Acceptance:** Scratch runner command: `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-security/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-security/Cargo.toml -- security-matrix`; reproducible command/output and paired result matrix; each threat says engine/config/classifier/unknown. Continue bounded probes when one mechanism fails. Existing regression command: `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; `git diff --check`. Cargo builds/tests are serialized with task 14.2.2.

### Task 14.2.2 — Revalidate private GeoPackage loader evidence
- **Repo:** same; **Depends on:** none (the Wave 1 inventory is useful context, not a gate to run existing loader tests).
- **Files in scope — must edit:** `planning/reports/duckdb-loader-audit.md`.
- **Can create:** no new fixture files or dependencies.
- **Do not touch:** loader implementation, Cargo files, deployment integration, user/Iowa data.
- **Spec:** Inspect the scoped loader and its tests; run the in-module synthetic tests and result contract. Trace canonical-path capability provenance, symlink/canonicalization assumptions, SQLite read-only flags, metadata lookup/ambiguity, actual WKB validation depth, null/empty/dimensional handling, row/scalar copying, and output serialization. Specifically assess adversarial metadata ambiguity/mismatch, truncated GeoPackageBinary/WKB, dimensional values, and exact row/scalar copying. Existing test coverage is run regardless of gaps. If a case needs a new fixture/test or source change, record the exact proposed change for user approval rather than silently expanding scope or stopping the audit.
- **Grep anchors:** `load_geopackage`, `TrustedGeoPackagePath`/path capability, GPKG metadata queries, geometry decoder; loader test names under `executor::duckdb::tests`.
- **Acceptance:** `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb executor::duckdb::tests`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_result_contract`; report exact outcomes and residual trust assumptions plus coverage for every requested adversarial case (tested or proposed follow-up); `git diff --check`. Serialize Cargo commands with task 14.2.1.

### Task 14.3.1 — Research security-control semantics and mitigation options
- **Repo:** same; **Depends on:** 14.2.1.
- **Files in scope — must edit:** `planning/reports/duckdb-security-probe-results.md`.
- **Do not touch:** code, policy, manifests, deployment.
- **Spec:** Compare observed controls with version-matched official DuckDB security documentation and release/advisory notes. Analyze practical defense-in-depth options for an optional embedded backend: SQL parser/authorizer surface, strict query allowlist, database/extension configuration, least-privilege filesystem permissions, connection/process boundaries, and operational monitoring. Explain attack preconditions, what each control does not mitigate, and which options would need later user approval. Do not assert that a process sandbox is necessary or unnecessary without evidence.
- **Grep anchors:** report from 14.2.1; `docs/security/duckdb-execution-policy.md`; ADR 0007; official DuckDB 2.0 securing/securing-extensions docs and release notes.
- **Acceptance:** Cited options/risk matrix reconciles with direct probe observations; distinguishes current achievable guarantees from future design choices; no implementation recommendation presented as approved; `git diff --check`.

### Task 14.3.2 — Refresh the six-method SQL Server geography contract
- **Repo:** same; **Depends on:** none for source research. Pinned runtime probes do not wait on alternate-version/upgrade conclusions.
- **Files in scope — must edit:** `planning/reports/G37-geography-transformations.md`.
- **Do not touch:** transpiler/tests/contract docs or another report.
- **Spec:** Recheck current authoritative Microsoft Learn contracts and version-matched DuckDB Spatial docs for all six methods. Clarify SQL Server documented versus undocumented behavior, numeric/text tolerance and oracle limits, CRS/SRID/axis, NULL/error, dimensions, empty/invalid, boundary and output-type behavior. Give a method-by-method evidence plan. Explicitly specify asymmetric `.STArea()` axis fixtures and independent references, and `.STAsText()` 2D/Z/M/ZM/empty/type cases. Do not repeat prior assumptions as facts.
- **Grep anchors:** report sections 4–8; `docs/spatial-sql-contract.md`; prior DuckDB probe table; official SQL Server method pages.
- **Acceptance:** All six rows include source citations, evidence classification and concrete probes/acceptance; `.STArea()` and `.STAsText()` open questions enumerated; `git diff --check`. Source research may proceed even if crate/extension compatibility for a future version remains unresolved.

### Task 14.4.1 — Run pinned synthetic geography probes and resolve bounded verdicts
- **Repo:** same; **Depends on:** 14.3.2.
- **Files in scope — must edit:** `planning/reports/G37-geography-transformations.md`.
- **Can create:** scratch code/output only under `/tmp/opencode/sda-rs-plan14-geography/`; use already-authorized artifact after verifying SHA-256.
- **Do not touch:** repository code/manifests, dependencies, user data, network, or artifact provisioning.
- **Spec:** On the exact existing pin and local Spatial artifact, record extension identity and run the planned synthetic probes for all six methods. For `.STArea()`, use asymmetric axis-order cases, multiple latitudes/orientations, independent reference values/provenance, stated tolerance rationale, function/guard availability, CRS and NULL/empty/invalid cases. For `.STAsText()`, test all supported geometry classes, Z/M/ZM dropping, NULL, empty, output parse/type and coordinate preservation. For other four, test relevant boundary/SRID/NULL/empty/invalid behavior needed to substantiate no-go or bounded candidate. No implementation. Continue through the matrix even when one candidate fails; do not equate a DuckDB observation with SQL Server parity.
- **Grep anchors:** 14.3.2 probe matrix; `tests/duckdb_geography_contract.rs` for existing pin-loading pattern; `docs/duckdb-deployment.md` for hash.
- **Acceptance:** Exact command: `SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`; verify and record the artifact SHA-256 before probing. Reproducer commands, raw/normalized results, extension hash/metadata, independent area reference source and tolerance rationale are recorded; each method has a completed evidence verdict (support candidate, no-go, or bounded inconclusive with attempted probes). If a proposed candidate cannot be established, finish the remaining probes and record the evidence; do not stop the task. `git diff --check`.

### Task 14.4.2 — Test G39 follow-up controls and characterize residual bypasses
- **Repo:** same; **Depends on:** 14.3.1.
- **Files in scope — must edit:** `planning/reports/duckdb-security-probe-results.md`.
- **Can create:** disposable probe artifacts only under `/tmp/opencode/sda-rs-plan14-security/`.
- **Do not touch:** source/policy/manifests, network, real data, OS sandbox, or runtime enablement.
- **Spec:** Close the planned direct-API matrix with additional targeted bypass cases suggested by 14.2.1 and official 2.0.1 fixes. Test effects safely and locally, including `CREATE SECRET` denial and settings/statement variants. Classify findings by exploitability/preconditions and practical mitigation; separate 2.0.0 observed behavior from 2.0.1 documented fixes, since no alternate version is being built. This task must not stop merely because parser-backed authorization is absent; produce a concrete evidence-backed verdict and later work options.
- **Grep anchors:** security matrix and proposed cases; `tests/duckdb_security_contract.rs`; release note references in 14.1.1.
- **Acceptance:** Run `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-security/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-security/Cargo.toml -- security-followup`; matrix has raw outcomes and reproducible commands, and states residual unknowns without calling them arbitrary blockers. Regression command: `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; `git diff --check`.

### Task 14.5.1 — Measure DuckDB workload and resource behavior
- **Repo:** same; **Depends on:** 14.2.1, 14.2.2, 14.4.2, 14.1.1.
- **Files in scope — must edit:** `planning/reports/duckdb-qualification-benchmark.md`.
- **Can create:** benchmark source/manifest/build artifacts only under `/tmp/opencode/sda-rs-plan14-benchmark/`.
- **Do not touch:** repository dependency manifests, production defaults, routes, user data, network, baseline report.
- **Spec:** Adapt the existing deterministic SQLite benchmark harness for the current pinned DuckDB executor. Cover scalar/select, captured join, batch/temp-table, result serialization, and separately reported spatial operations; assert outputs before and during timing. Collect exact commands, host/OS, crate/engine/extension identity, warmups/repetitions, latency distributions, throughput where meaningful, serial and bounded concurrent behavior, memory/RSS method and limitations, threads and temp-storage observations. No untrusted SQL or arbitrary files/network. Compare to the SQLite baseline and report the 20% objective and >10% regressions as non-gating observations. Include a per-resource matrix for memory, threads, concurrency, temp storage, input/row/value/output bounds, and deadline; for each give measured observation, evidence limitation, candidate nonbinding control, or reason no recommendation is justified. Missing calibration is a reported limitation, not a stop condition; do not set caps.
- **Grep anchors:** `planning/reports/duckdb-sqlite-benchmark-baseline.md`; `tests/duckdb_pipeline_contract.rs`; `tests/duckdb_execution_contract.rs`; `planning/STATE_13.md` G38–G41 evidence.
- **Acceptance:** Run offline scratch runner in both modes: `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-benchmark/target cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-plan14-benchmark/Cargo.toml -- --mode serial` and `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-benchmark/target cargo run --release --offline --manifest-path /tmp/opencode/sda-rs-plan14-benchmark/Cargo.toml -- --mode bounded-concurrent`; both assert expected outputs before and during measurements. Report comparability limits, per-resource matrix, and candidate-control uncertainty; `git diff --check`.

### Task 14.6.1 — Synthesize secure integration architecture and decision gates
- **Repo:** same; **Depends on:** all prior tasks.
- **Files in scope — must edit:** `planning/reports/duckdb-secure-integration-assessment.md`.
- **Do not touch:** ADRs, policy, implementation, routes/config, deployment or dependency files.
- **Spec:** Synthesize G39–G42 evidence into a practical integration proposal: allowed query surface and parser/authorizer requirements; lifecycle/cancellation; trusted read-only data import; extension pin/loading; resource controls and operational monitoring; threat model, abuse cases, residual risk and upgrade position. Distinguish evidence-backed safeguards from recommendations and unverified assumptions. State whether current in-process design is acceptable for any request exposure, and what evidence/approvals remain for a later enablement plan. Include alternatives and a go/no-go matrix; no arbitrary “all blockers must disappear” criterion, but identify material unresolved risks and compensating safeguards. Do not claim production readiness.
- **Grep anchors:** reports from Waves 1–5; ADR 0007; `docs/security/duckdb-execution-policy.md`; G38 lifecycle report.
- **Acceptance:** Architecture describes concrete trust boundaries and defense-in-depth, links every conclusion to evidence, lists residual risks and approval gates, and answers the overall secure-integration question; `git diff --check`.

## Orchestrator gates

- Each task worker writes a <=12-step micro-TODO and stops. Review against scope and evidence criteria before authorizing execution.
- Any proposed source/test/policy implementation fix is surfaced to the user with rationale, affected files, and validation before code work; user approval is mandatory. Research/report updates already listed in scope are authorized after task micro-plan approval.
- Do not declare a task blocked solely because a source, function, or control is undocumented. Perform its specified bounded research/probes, classify the result, and continue other independent probes.
- Run no shared tests concurrently. At each wave boundary run serially: `cargo fmt --check`; `git diff --check`; `cargo test`; `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`; corresponding exact commands for `duckdb_result_contract`, `duckdb_execution_contract`, and `duckdb_pipeline_contract`; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb executor::duckdb::tests`; and `SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`. Record exact commands/counts in STATE_14.md. Workers serialize targeted Cargo tests if they share the build cache.
- No remote Git operations. Planning commits are separate from any future implementation.
