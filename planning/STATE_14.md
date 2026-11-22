# State 14 — Evidence and Investigation for Secure DuckDB Integration

Plan: `planning/EXECUTION_PLAN_14.md`

## Baseline

- Repository: `/home/andrew/workspace/sda-rs`
- Branch: `feature/duckdb-feature1`; baseline HEAD `f5b8ab1`.
- Existing user change: `planning/GOALS.md` was modified as part of this planning intake and must be preserved. At scaffold creation, `EXECUTION_PLAN_14.md`, `PROMPT_TEMPLATE_14.md`, and `STATE_14.md` are new untracked Plan 14 files; commit them separately after user approval and before wave execution.
- No unrelated worktree changes were observed. No remote actions.
- DuckDB pin: Rust `duckdb = "=1.20000.2"`; bundled engine v2.0.0. Authorized Spatial file exists at `/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension`; verified SHA-256 `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`.
- Baseline commands/results: `cargo fmt --check` passed; `cargo test` passed (75 unit + 30 integration); `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed (75 unit + 30 integration); `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` passed (4); corresponding `duckdb_result_contract` passed (4), `duckdb_execution_contract` passed (4), and `duckdb_pipeline_contract` passed (12); `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb executor::duckdb::tests` passed (4); pinned-artifact `duckdb_geography_contract` passed (7) with exact command `SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_geography_contract`.
- Shared baseline suites were ultimately run serially; an initial parallel invocation caused package/build lock waits but no failures.
- Linux x86_64 only. No user/Iowa data or network was used for tests.

## Decisions

- Evidence-first; bounded negative or inconclusive findings are valid after the investigation is actually performed. Missing old evidence alone is not a blocker.
- Official current-source research and offline synthetic probes are authorized; no dependency/pin changes or downloads/network in probes.
- Any source/test/policy implementation fix requires separate explicit user approval.
- Maximum two workers; shared tests and artifact probes run serially.
- End with a defensible secure-integration design and evidence/risk gates; no request-path enablement, OS sandbox implementation, or final numeric limits.
- Plan artifacts are tracked separately from implementation.

## Wave Board

| Wave | Task(s) | Status | Notes |
|---|---|---|---|
| 1 | 14.1.1, 14.1.2 | done | Pin/upgrade assessment and security/loader baseline committed; boundary suites passed (2026-11-22). |
| 2 | 14.2.1, 14.2.2 | done | Direct security probes and loader revalidation complete; targeted offline suites passed (2026-11-22). |
| 3 | 14.3.1, 14.3.2 | done | Security-source synthesis and geography contract refresh committed; serial boundary suites passed (2026-11-22). |
| 4 | 14.4.1, 14.4.2 | done | Geography and security follow-up evidence committed; serial boundary suites passed (2026-11-22). |
| 5 | 14.5.1 | pending | Correctness-checked performance/resource measurement. |
| 6 | 14.6.1 | pending | Overall secure integration assessment. |

## Task Reports

- **14.1.1 — complete.** Added `planning/reports/duckdb-pin-and-upgrade-assessment.md`; researched official release calendar, DuckDB 2.0.0/2.0.1 releases, security fixes, Rust client metadata, and Spatial provenance. Verdict: evaluate 2.0.1 promptly; do not change the pin here. Engine-level security applicability is source-supported; exact candidate crate/engine/Spatial tuple, runtime loader/result behavior, geography parity, and exploitability remain unverified. No alternate build, downloads, or tests run; no scope deviation. `git diff --check` — passed (no output). Worker model: GPT-6 Luna.

<!-- Append concise reports under task IDs. -->

### 14.4.2 — Test G39 follow-up controls and characterize residual bypasses
- **Status:** Complete; bounded offline follow-up probes and regression passed.
- **Files:** `planning/reports/duckdb-security-probe-results.md`; `planning/tasks/14.4.2.todo.md`; `planning/STATE_14.md`.
- **Commands:** Scratch `security-followup` command recorded in report; `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 4 passed, 0 failed; `git diff --check` passed.
- **Evidence verdict:** Direct settings/reset variants succeeded where executor path rejected them; external-access re-enable failed in tested SET/PRAGMA forms. Secret attempts hit configured secrets-directory denial; alternate `IN memory` syntax errored. No remote endpoint or path traversal tested.
- **Limits/deviations:** Bounded DuckDB 2.0.0 observations; no proof of resulting extension/I/O access, universal secret denial, or 2.0.1 runtime behavior. No code/policy/manifests/network/data changes.
- **Worker model:** GPT-6 Luna.

### 14.4.1 — Run pinned synthetic geography probes and resolve bounded verdicts
- **Status:** Complete with bounded inconclusive candidate findings; six-method matrix recorded without claiming untested coverage.
- **Files:** `planning/reports/G37-geography-transformations.md`; `planning/STATE_14.md`.
- **Commands/results:** Artifact SHA-256 verified as `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`; metadata `(spatial, a3dbcd1781)`. Exact prescribed test command first hit `/tmp` space exhaustion; rerun with existing repo target cache (`CARGO_TARGET_DIR=/home/andrew/workspace/sda-rs/target`) passed 7/7. Offline scratch runner command and per-case observations are recorded in G37.
- **Evidence verdict:** Four binary methods remain no-go; asymmetric spheroidal area probes distinguish coordinate order but lack an independent reference/tolerance, so candidate remains inconclusive. Sampled `ST_AsText(ST_Force2D(...))` outputs preserve tested types/coordinates and drop ZM, but full parse-back/oracle coverage is absent; semantic candidate only, not parity. All six remain rejected.
- **Limits/deviations:** No SQL Server oracle or local independent GeographicLib reference. Scratch target build exhausted `/tmp`; removed only task-created target contents and reused existing repository build cache. No code/manifests/tests/network/data changes. `git diff --check` passed.
- **Worker model:** GPT-6 Luna.

### 14.3.1 — Research security-control semantics and mitigation options
- **Status:** Complete; official DuckDB 2.0 documentation and 2.0.1 release notes reconciled with bounded 2.0.0 probe observations.
- **Files:** `planning/tasks/14.3.1.todo.md`; `planning/reports/duckdb-security-probe-results.md`; `planning/STATE_14.md`.
- **Evidence verdict:** External-access observations align with documented controls; direct autoload-setting mutability remains important; 2.0.1 documents `CREATE SECRET` and remote-path fixes relevant to the 2.0.0 baseline. Remote URL and exploitability remain unverified.
- **Options:** Report adds a risk matrix for parser/allowlist, configuration, filesystem permissions, process boundary, monitoring/resources, and upgrade qualification; no sandbox necessity verdict or approved implementation.
- **Commands:** Official source review; `git diff --check` — passed with no output. No probes or tests run, per instruction.
- **Limits/deviations:** No code, policy, manifest, deployment, or probe changes; SQLite remains default. No commit.
- **Worker model:** GPT-6 Luna.

### 14.3.2 — Refresh the six-method SQL Server geography contract
- **Status:** Complete; refreshed contract from Microsoft Learn SQL Server 17 and versioned DuckDB 2.0 Spatial documentation.
- **Files:** `planning/reports/G37-geography-transformations.md`; `planning/STATE_14.md`.
- **Commands/results:** No probes/tests run by scope. `git diff --check` passed.
- **Evidence verdict:** All six methods now distinguish source-documented behavior, prior pinned observations, and unknowns; specified method-by-method probes and acceptance evidence.
- **Limits/deviations:** SQL Server numeric/text oracle unavailable; SQL Server does not document predicate boundary treatment or area numeric tolerance on reviewed pages. Axis-order and `STAsText` dimensional/runtime behavior remain unverified on pin. Rejection semantics preserved; no code/tests/contracts edited; no commit.
- **Worker model:** GPT-6 Luna.

### 14.2.1 — Probe G39 controls directly on the pinned engine
- **Status:** Complete; bounded offline direct-connection matrix and regression passed.
- **Files:** `planning/tasks/14.2.1.todo.md`; `planning/reports/duckdb-security-probe-results.md`; `planning/STATE_14.md`.
- **Commands/results:** `CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-security/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-security/Cargo.toml -- security-matrix` — scratch runner compiled and completed; output recorded in report. Direct config denied local file access, COPY/IMPORT filesystem access, extension loading/install filesystem access; `SET autoload_known_extensions`, in-memory ATTACH, persistent in-memory DDL, compound SQL, comments, and quoted forms succeeded. `CREATE SECRET` failed on secrets-directory filesystem access. Remote parquet URL was inconclusive because parquet extension was absent; no endpoint contacted. Temp workflow succeeded as one same-connection batch. `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract` — 4 passed, 0 failed, 0.25s. `git diff --check` — passed, no output.
- **Evidence verdict:** The executor path rejects tested non-SELECT/comment/quoted cases while direct DuckDB accepts several of them; classifier enforcement is material. Engine/config denied selected filesystem and extension operations, but the bounded cases do not establish exhaustive protection or remote-URL behavior.
- **Limits/deviations:** No network endpoints, user data, implementation, manifest, test, dependency, or deployment changes. Remote access was not reached; `CREATE SECRET` was blocked at configured filesystem access, not shown as a universal engine denial. No commit.
- **Worker model:** GPT-6 Luna.

### 14.2.2 — Revalidate private GeoPackage loader evidence
- **Status:** Complete; scoped loader audit and targeted tests passed.
- **Files:** `planning/reports/duckdb-loader-audit.md`; `planning/STATE_14.md`.
- **Commands:** Serially ran `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb executor::duckdb::tests` (4 passed, 0 failed) and `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_result_contract` (4 passed, 0 failed); `git diff --check` passed.
- **Evidence verdict:** Existing tests substantiate selected synthetic read-only loading, SRID/dimensional/truncation checks, NULL/empty behavior, one-row value copying/JSON serialization, scalar serialization, and rejection of generic unvalidated blobs. The report maps untested adversarial metadata and payload cases to proposed synthetic follow-ups.
- **Limits/deviations:** Private loader has no deployment/request-path trust-provisioning or resource-bound evidence; structural WKB validation is not topology validation. No source, tests, fixtures, dependencies, or deployment changed. `planning/GOALS.md` preserved.
- **Worker model:** GPT-6 Luna.

### 14.1.2 — Inventory current DuckDB security and GeoPackage claims
- **Status:** Complete; baseline evidence inventory only.
- **Files:** `planning/reports/duckdb-security-and-loader-evidence.md`; `planning/STATE_14.md`.
- **Commands:** Source/policy/test anchors inspected; `git diff --check` (see task completion).
- **Evidence verdict:** Matrix maps classifier, engine/config, loader, tested observations, and unverified claims; records prior `allowed_paths`/`ST_Read` failure and named contract tests.
- **Limits:** No runtime probes or tests; finite executor contracts do not prove direct-engine bypass resistance or exhaustive loader validity.
- **Deviations:** None. No implementation, policy, or other report changed; `planning/GOALS.md` preserved.
- **Worker model:** GPT-6 Luna.
