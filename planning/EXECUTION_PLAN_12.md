# Execution Plan 12 — Optional DuckDB Cargo Build Feature

## Goal & Scope

### Goal

Make the already-authorized pinned, bundled DuckDB crate available through an explicit, disabled-by-default Cargo feature, and gate the existing Plan 11 integration test behind that feature. The default build and test suite must remain DuckDB-free, SQLite remains the default runtime backend, and enabling the build feature does not select or enable a DuckDB runtime backend.

### In scope

- Move the pinned `duckdb = "=1.20000.2"` bundled dependency from dev-only use to an optional normal dependency exposed by a `duckdb` Cargo feature.
- Require the `duckdb` feature to build/run `tests/duckdb_pipeline_contract.rs`.
- Verify default and opt-in Cargo dependency/test behavior on Linux x86_64, including `--all-features`.
- Keep Plan 11's IIF lowering, CTAS result suppression, and DuckDB value adapter strictly inside the integration test.
- Update `planning/GOALS.md` to record G48 as active and G49 as its directly dependent test-gating goal.

### Out of scope / deferred

- Runtime backend selection, configuration, routes, executor integration, or DuckDB use for submitted requests.
- Production enablement or any claim that DuckDB is secure, resource-bounded, spatially qualified, deployable, or performant.
- Changes to SQLite defaults, SQL/response contracts, Plan 11 test adapter semantics, DuckDB version, or bundled behavior.
- Non-Linux-x86_64 validation or a cross-platform support promise.
- Production security/resource blockers and spatial/performance/deployment work remain on the G36–G45 qualification path; this plan does not resolve them.

### Verbatim user constraints and decisions

- “agree with recommendations, all features includes duckdb, bundling is intended (for now, we will see if it becomes a problem), linux x64 only”
- DuckDB is optional at compile time; the feature alone must not select or enable a runtime backend.
- SQLite remains the default and supported backend.
- Plan 11 IIF and CTAS adapters are test-only.
- Serial task execution; planning artifacts are tracked separately from implementation.

## Dated Decisions

- **2026-11-20 — Goal:** Continue G48 from `planning/GOALS.md`; directly include G49 test-target gating. Do not expand into runtime selection or production enablement.
- **2026-11-20 — Feature:** Name the disabled-by-default feature `duckdb`; `--all-features` includes it.
- **2026-11-20 — Bundling:** Keep the existing `bundled` crate feature for now; reconsider only if it proves problematic.
- **2026-11-20 — Platform:** Validate the opt-in path on Linux x86_64 only; no broader support promise.
- **2026-11-20 — Execution:** Run the task serially; preserve the current branch and all pre-existing user changes.
- **2026-11-20 — Verification:** Preserve Plan 11's offline targeted DuckDB test and serial default/configured-SpatiaLite full-suite gates.
- **2026-11-20 — Worker policy:** Carry forward Plan 11's small-worker model `opencode-go/deepseek-v4.1-flash`, with at most one escalation to `openai/gpt-6-luna`.
- **2026-11-20 — Planning artifacts:** Track Plan 12 artifacts in Git; keep planning-only commits separate from implementation.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`; branch `main`, HEAD `88b5fb6`; preserve the pre-existing modified `planning/GOALS.md`. Do not switch branches or stage broad paths.
- `Cargo.toml` has `duckdb = { version = "=1.20000.2", features = ["bundled"] }` under `[dev-dependencies]`; `Cargo.lock` already includes the dependency tree.
- `tests/duckdb_pipeline_contract.rs` is the only DuckDB test target and contains the Plan 11 test-only IIF/CTAS/value adapters. No runtime source imports DuckDB.
- Cargo commands from `AGENTS.md`: `cargo build`, `cargo test`, `cargo fmt --check`; Plan 11 adds offline targeted command `CARGO_NET_OFFLINE=true cargo test --offline --test duckdb_pipeline_contract`.
- Current baseline on this checkout: `cargo fmt --check` passed; `cargo test` passed (75 unit + 38 integration); configured SpatiaLite suite passed (75 unit + 38 integration).
- Oversized/important anchors: `Cargo.toml` (20 lines), `tests/duckdb_pipeline_contract.rs` (481 lines), use Cargo target configuration rather than editing its test-only adapter code.
- ADR 0007 preserves SQLite as default and does not itself authorize production dependency changes; D13 in `planning/GOALS.md` authorizes the next optional compile-time path only. DuckDB execution policy blockers remain.

## Dependency Graph and Waves

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 12.1.1 | Establish the optional bundled DuckDB dependency and disabled-by-default feature. |
| 2 | 12.2.1 | Gate the Plan 11 integration-test target and verify default/opt-in test behavior. |

The tasks are serial because both change `Cargo.toml`. They are separated by outcome: the first creates the compile-time dependency boundary; the second makes the test target honor that boundary. After Wave 1, `cargo test` is expected to require DuckDB until Wave 2 adds the test gate; do not run the full suite at that intermediate boundary. Run full Cargo suites serially after Wave 2.

## Global Hazards and Rules

- Do not edit `src/`, routes, runtime configuration, executor, transpiler, serializer, security policies, or production deployment/CI.
- Do not change or relocate test adapter behavior in `tests/duckdb_pipeline_contract.rs`; it remains test-only.
- Preserve `duckdb = "=1.20000.2"` and `features = ["bundled"]`; do not add dependencies.
- Default `cargo build`/`cargo test` must not compile or run the DuckDB integration test. The explicit DuckDB feature and `--all-features` must opt it in.
- Feature-enabled dependency presence is established through Cargo's resolved feature/dependency graph and test compilation; do not require a DuckDB symbol in the normal application binary because no runtime code references it.
- Run only the task's targeted commands in the worker. Orchestrator runs full test suites serially.
- Preserve current branch and pre-existing changes. No remote Git actions. Planning-only artifacts are committed separately.
- Production blockers remain: write/file/network/extension enforcement; cancellation; memory, thread, concurrency, temporary-storage and result/input limits; spatial/result semantics; deployment pin/provenance; comparable performance evidence. Do not represent this build step as resolving any blocker.

## Task Specifications

### Task 12.1.1 — Put DuckDB behind an optional Cargo feature

- **ID:** 12.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** G47 complete; G48/G49 user decisions recorded above.
- **Files in scope — must edit:** `Cargo.toml`.
- **Files in scope — can edit if Cargo resolution requires it:** `Cargo.lock`.
- **Do not touch:** `tests/duckdb_pipeline_contract.rs`, `src/`, other tests/fixtures, runtime configuration/routes/executor/transpiler/serializer, security/deployment/CI, ADRs, or unrelated planning artifacts.
- **Spec:** Move the exact pinned bundled DuckDB crate from `[dev-dependencies]` to `[dependencies]` with `optional = true`; define a disabled-by-default `duckdb` feature that enables that optional dependency. Do not add the test target's `required-features` gate in this task; that is Task 12.2.1. Keep the test source and all test-only adapters unchanged. Do not introduce runtime feature-dependent behavior.
- **Intermediate-state note:** Until Task 12.2.1 gates the test target, default `cargo test` may fail to compile the auto-discovered DuckDB test. Do not run it at the Wave 1 boundary; the default `cargo build` and feature-enabled build are the Wave 1 checks.
- **Grep anchors:** `Cargo.toml` dependency sections; `Cargo.lock` DuckDB package entries; Plan 11 `STATE_11.md` dependency notes.
- **Acceptance:**
  - `cargo fmt --check` passes.
  - `cargo build --locked` passes without the feature.
  - `cargo build --locked --features duckdb` passes on Linux x86_64 with the bundled crate.
  - `cargo tree -e normal` does not include DuckDB; `cargo tree -e features --features duckdb` shows DuckDB and `bundled`.
  - `git diff --check` passes; no runtime source or test adapter changes.

### Task 12.2.1 — Gate the DuckDB integration-test target

- **ID:** 12.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 12.1.1.
- **Files in scope — must edit:** `Cargo.toml`.
- **Files in scope — can edit if Cargo demonstrably requires a resolution update:** `Cargo.lock`.
- **Do not touch:** `tests/duckdb_pipeline_contract.rs` and its test-only adapters; `src/`, other tests/fixtures, runtime configuration/routes/executor/transpiler/serializer, security/deployment/CI, ADRs, or unrelated planning artifacts.
- **Spec:** Declare the existing `duckdb_pipeline_contract` integration target with `required-features = ["duckdb"]`. Preserve its source and the Plan 11 IIF lowering, CTAS result suppression, and value adapter without edits. Ensure the default test suite excludes the DuckDB target and explicit feature/all-features test commands include it.
- **Grep anchors:** `Cargo.toml` feature and integration-test target declarations; `tests/duckdb_pipeline_contract.rs` target name/imports; Plan 11 `STATE_11.md` acceptance record.
- **Acceptance:**
  - `cargo test --locked` passes by default and excludes the DuckDB integration target.
  - `cargo test --test duckdb_pipeline_contract` without the feature is rejected as requiring the `duckdb` feature.
  - `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_pipeline_contract` passes all 12 tests.
  - `CARGO_NET_OFFLINE=true cargo test --offline --all-features --test duckdb_pipeline_contract` passes, confirming `--all-features` includes the test dependency.
  - `cargo fmt --check` and `git diff --check` pass; test source and adapters are unchanged.
- **Orchestrator final gates:** Run default `cargo test` and `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` serially after targeted checks.

## Final Acceptance Gate

On Linux x86_64, run serially: default and feature-enabled build/dependency-graph checks; default test suite; targeted opt-in offline DuckDB test; `cargo fmt --check`; `git diff --check`; configured-SpatiaLite full `cargo test`. Record exact results in `STATE_12.md`. Any inability to build the bundled pinned crate offline is a blocker to report, not permission to alter the pin or bundling choice.
