# Execution Plan 8 — DuckDB Spatial Follow-up for Geography Methods

## Goal & Scope

### Goal

Revisit the blocked G33-2 DuckDB Spatial evaluation using the cached extension in the correct initialization order, and determine whether DuckDB offers useful, evidence-backed support for any of G12’s six SQL Server `geography` methods. This is an isolated evaluation and recommendation, not backend adoption.

### In scope

- Load the already-cached Spatial extension by its exact local path while DuckDB external access is still enabled; then configure access limits and disable external access.
- Evaluate DuckDB’s corresponding operations for all six G12 methods—`STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—using synthetic in-memory geometries and the SQL Server `geography` contract as the comparison baseline.
- Record actual DuckDB/Rust result types, SRID handling, units, representative outputs, and NULL/empty/dimensional behavior where safely testable. Distinguish observed results from documentation and do not treat function-name similarity as parity.
- Attempt Iowa GeoPackage feature comparisons only after synthetic probes, and only if an OS sandbox provides a read-only source mount and a preflight proves the query will use bounded indexed access. Otherwise stop after synthetic probes and read-only metadata checks.
- Write a new, evidence-based follow-up report under `planning/reports/` in this repository. “Research branch only” means commit it only on `docs/backend-spike-research`; it does not mean placing the file outside the repository. Preserve the original Plan 7 blocker report as historical evidence.

### Out of scope / deferred

- Adding DuckDB or Spatial dependencies to this repository; changing production code, backend selection, configuration, wire behavior, or security policy.
- Any extension download except a single explicit fallback to DuckDB's official `INSTALL spatial` if loading the exact cached binary fails. No other network operation or extension repository is allowed; do not modify the cached extension.
- Any Iowa feature-row read without both the OS-level read-only sandbox and demonstrated indexed/bounded access. No fallback to a full scan or a `LIMIT` over an unbounded scan.
- Claiming SQL Server parity without a SQL Server runtime; the report may classify operations as useful candidates, incompatible, or unverified.
- Integrating Plan 8’s report into `main`; execution/report commits target `docs/backend-spike-research` only.

### Verbatim user direction

- “i want duckdb as it may resolve some o these issues”
- “you can have network dependenies, spatial has to get loaded somehow”
- “Synthetic, then bounded Iowa”
- “Indexed reads only”
- “Evaluation only”
- “Compare all six contracts”
- “Exact-path allowlist”
- “Require OS read-only sandbox”
- “Current research branch”
- “Research branch only”

## Dated Decisions

- **2026-11-19 — Objective:** Reopen G33-2 as a targeted DuckDB Spatial follow-up because it may help with G12; evaluate all six methods against SQL Server `geography` contracts.
- **2026-11-19 — Extension initialization:** First load the cached Spatial binary by exact path while external access is enabled. If that fails, one explicit `INSTALL spatial; LOAD spatial;` from DuckDB's official extension repository is allowed. Then set the exact `allowed_paths`, disable external access, and verify it is off. No other network access is allowed.
- **2026-11-19 — Lockdown:** Set DuckDB’s exact `allowed_paths` entry for the Iowa GeoPackage before disabling external access. Because DuckDB documents allow-listed paths as both readable and writable, this setting alone is not a read-only guarantee.
- **2026-11-19 — Data boundary:** Run synthetic in-memory probes first. Iowa feature rows are permitted only inside an OS sandbox with the host filesystem read-only, the source explicitly read-only, scratch writable, and network disabled; and only after a read-only preflight demonstrates indexed/bounded access and the orchestrator approves its evidence. If any condition fails, stop without reading feature rows.
- **2026-11-19 — Adoption:** Evaluation only. Even a promising result does not authorize production changes or DuckDB adoption; a separate plan and decision would be required.
- **2026-11-19 — Repository branch:** Plan artifacts are committed separately on the current `docs/backend-spike-research` branch. Reports are files under `planning/reports/` in this repository, committed only on that branch; no `main` integration or remote action.
- **2026-11-19 — Workflow:** Split synthetic feasibility/preflight from conditional Iowa feature reads into separate sequential waves. The orchestrator reviews the preflight evidence and separately authorizes the second wave before any feature query.
- **2026-11-19 — Architecture:** Comply with accepted ADR 0001 (SQLite/GeoPackage remains the MVP backend) and ADR 0003 (SpatiaLite stays optional). No ADR is superseded.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`; current branch `docs/backend-spike-research`, clean at `a04e12c`. `main` points to the same commit. The commit records the G33 note/GOALS updates; do not rewrite those notes.
- Plans 1–7 precede this orchestration. Plan 8 files are the current draft. Preserve the current branch and commit the three Plan 8 artifacts together in a separate `docs(planning):` commit after approval.
- Separate untracked Plan 9 files appeared during this planning session. They are outside this plan's scope; preserve them and do not edit or stage them. The user reaffirmed Plan 8 as the active request.
- The original G33-2 report is `planning/reports/G33-2-duckdb-spike-results.md`. It records DuckDB crate `1.20000.2` / engine `2.0.0`, a cached 104 MiB Spatial extension at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension`, and the failed order: connection started with `enable_external_access=false`, then `LOAD spatial`.
- The cached extension currently exists. Treat it as a read-only input; verify its hash/version and do not modify it. Put all new probe sources, targets, and temporary data in `/tmp/opencode/sda-rs-plan8-duckdb/`. Use Cargo offline mode; stop if required crate artifacts are unavailable rather than fetching. The user allows one extension acquisition fallback via DuckDB's official repository, not Cargo/network dependency fetching.
- The Iowa GeoPackage is `/data/SSURGO/output/iowa_ssurgo.gpkg` (6,406,705,152 bytes; prior SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`). Prior G33-2 read-only plan checks reported full scans for `mupolygon.mukey='1857266'` and `legend.areasymbol='IA001'`; those queries must not be run unless a new preflight proves bounded indexed access.
- DuckDB 2.0 security documentation says `allowed_paths` must be configured before external access is disabled and permits writes as well as reads. An OS-level read-only sandbox is therefore mandatory for Iowa feature access. `bwrap` is present; whether the required namespaces/sandbox are permitted is unverified. Failure to establish the sandbox is a stop condition, not permission to weaken it.
- DuckDB 2.0 Spatial documentation lists all six corresponding free-function operations. Its generic `ST_Buffer` is planar and uses input coordinate units; `ST_Area_Spheroid` assumes WGS84 and documents latitude/longitude axis order; ordinary `ST_Area` uses CRS coordinate units. Function availability alone does not establish SQL Server geography semantics.
- Existing useful evidence: `planning/reports/G12-geography-method-feasibility.md` contains SQL Server primary citations and SpatiaLite comparisons. Use it as comparison context, not as a claim about DuckDB.
- Entry sizes: `src/spatial.rs` 572 lines; `src/transpiler/spatial.rs` 1,519; `src/config.rs` 226; `src/lib.rs` 40; `tests/spatial_methods_contract.rs` 130. Plan 8 makes no production edits.
- Accepted ADRs 0001–0006 remain in force. Relevant choices: SQLite/GeoPackage remains the service backend, spatial support stays optional, and unsupported capabilities are explicit.
- Green baseline on the current branch: `cargo fmt --check`, `git diff --check`, default `cargo test`, and configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed. Both Rust suites reported 75 unit tests and 30 integration tests with no failures.
- Project conventions: `AGENTS.md`, `CONTRIBUTING.md`, and `planning/PROMPT_TEMPLATE_7.md`; test commands are `cargo test`, configured full `cargo test`, and `cargo fmt --check`. Plan 8 is research-only, so its targeted acceptance commands run the standalone probe rather than changing this crate.

## Dependency Graph and Waves

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 8.1.1 | Load/cache Spatial, lock down, run all synthetic six-method probes, and perform no-feature-row Iowa sandbox/index preflight. |
| 2 | 8.2.1 | Only after explicit orchestrator approval of Wave 1 evidence, perform bounded Iowa comparisons; otherwise record why feature reads remain blocked. |

Both tasks update the same report sequentially; no same-wave file overlap exists. Wave 2 is conditional and must not start feature reads until the orchestrator reviews Wave 1's sandbox and index evidence and explicitly approves the gate. No production files are in scope.

## Task Specifications

### Task 8.1.1 — Re-evaluate DuckDB Spatial for G12 geography methods

- **ID:** 8.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** None.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md`.
- **Files in scope — can create:** `planning/tasks/8.1.1.todo.md`; scratch project, scripts, logs, targets, and disposable databases only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production source/tests, `Cargo.toml`/`Cargo.lock`, prior Plan 7 reports, user-owned G33 notes, the cached extension, or the Iowa source file. Do not access soilDB. Do not run any network/install command other than the single explicitly allowed DuckDB official-extension fallback. No remote Git actions.
- **Spec:** In a standalone Rust probe pinned to the already-tested DuckDB client version, first load the exact cached Spatial binary while external access is enabled; if it fails, use exactly one explicit `INSTALL spatial; LOAD spatial;` from DuckDB's official repository. Disable autoload/autoinstall, set bounded resource settings, and set the exact Iowa `allowed_paths` entry before `SET enable_external_access=false`; verify the setting and that an attempt to re-enable it is rejected. Run synthetic WKT tests for the six DuckDB spatial counterparts, recording engine/extension/PROJ versions, Rust result types, documented vs observed behavior, SRID behavior, units, representative output, and safely testable NULL/empty/Z/M cases against the SQL Server citations in `planning/reports/G12-geography-method-feasibility.md`. Do not claim parity from similar names or a single sample. Without reading feature rows, test whether `bwrap` can expose the host read-only, scratch writable, source explicitly read-only, and network disabled; inspect GeoPackage metadata and RTree schema in read-only mode, and run only a capped RTree-index preflight plus `EXPLAIN` showing virtual-table index use. Preflight may read at most 101 RTree candidate IDs, but no feature-table rows. Preserve exact probe/preflight sources and commands under `/tmp/opencode/sda-rs-plan8-duckdb/`; produce the initial sourced report and wait for orchestrator review. Do not start Wave 2 or read feature rows in this task.
- **Grep/recon anchors:** `planning/reports/G33-2-duckdb-spike-results.md` (prior runtime blocker and pinned version); `planning/reports/G12-geography-method-feasibility.md` (six-method contract and citations); DuckDB 2.0 Spatial functions reference (`ST_Contains`, `ST_Within`, `ST_Intersection`, `ST_Buffer`, `ST_Area`, `ST_Area_Spheroid`, `ST_AsText`); DuckDB 2.0 Securing DuckDB (`allowed_paths`, `enable_external_access`, ordering and write permissions); GDAL GeoPackage driver (RTree indexes); `planning/PROMPT_TEMPLATE_7.md` for repository task workflow.
- **Acceptance:** The probe source and exact pinned manifest are preserved in scratch. An offline Cargo build succeeds. In the synthetic run, cached absolute-path load is attempted first; if it fails, only the authorized DuckDB official-install fallback is used. After successful load the probe configures the allowlist, disables external access, confirms it is false, confirms an attempted re-enable is rejected, and only then runs the six synthetic probes. Report exact versions, extension path/hash/acquisition path, outputs/errors/types, SQL Server contract comparison, and DuckDB documentation. The Iowa preflight reads no feature rows; it records the actual sandbox mount configuration (not merely in-sandbox permission bits), read-only metadata, the exact bounded RTree candidate query and `EXPLAIN` output. It may read at most 101 RTree IDs; no Iowa feature query is part of this task. `git diff --check` passes; no out-of-scope repository or external file changed.
- **Targeted acceptance commands:**
  - Offline build: `CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home cargo build --offline --release --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml`
  - Wave 1 synthetic/security run (root read-only; scratch read-write; network is only used if the exact official `INSTALL spatial` fallback is required; no Iowa feature reads): `bwrap --die-with-parent --ro-bind / / --dev /dev --proc /proc --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe synthetic`
  - Wave 1 metadata/RTree preflight (no feature rows; network disabled): run the preflight script preserved in scratch under the equivalent `bwrap` sandbox shown here, using its exact recorded arguments: `bwrap --die-with-parent --unshare-net --ro-bind / / --dev /dev --proc /proc --ro-bind /data/SSURGO/output/iowa_ssurgo.gpkg /data/SSURGO/output/iowa_ssurgo.gpkg --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb python3 /tmp/opencode/sda-rs-plan8-duckdb/preflight_iowa.py --layer mupolygon --bbox=-93.70,41.50,-93.69,41.51 --candidate-limit 101`. This command may inspect metadata and RTree IDs only, never feature-table rows.
  - Wave 2 Iowa command (only after separate orchestrator gate approval; network disabled, host root read-only, source explicitly read-only, scratch read-write): `bwrap --die-with-parent --unshare-net --ro-bind / / --dev /dev --proc /proc --ro-bind /data/SSURGO/output/iowa_ssurgo.gpkg /data/SSURGO/output/iowa_ssurgo.gpkg --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe bounded-iowa`
  - Documentation check: `git diff --check`

## Hazard Notes

- `enable_external_access=false` at connection creation blocks the very dynamic extension load under evaluation. The corrected order is essential. Cargo builds offline. The one authorized extension-install fallback is the only network operation and ends before lockdown; the Wave 2 Iowa process runs without network access.
- `allowed_paths` is not a read-only control. Iowa access without an OS read-only sandbox is forbidden even if the DuckDB path allowlist is exact.
- DuckDB `ST_Read` filter pushdown is driver-dependent. A `LIMIT` does not make an unindexed scan safe; the feature-read gate requires demonstrated indexed/bounded access before any feature query.
- Wave 1 must not execute a feature-table query. Wave 2 requires both a successful no-row sandbox/index preflight and a separate explicit orchestrator approval; failure means no Iowa feature reads.
- `ST_Area_Spheroid` is documented for EPSG:4326 with latitude/longitude axis order and cannot be presumed to implement SQL Server’s SRID-dependent area contract. The remaining DuckDB spatial operations are generally planar unless explicitly documented otherwise.
- The cached binary is version/platform-specific. Do not replace or modify it. If it fails, use only the explicitly approved DuckDB official-repository install fallback and record the downloaded extension's resolved path and hash.
- The prior G33-2 report is a valid historical account of the tested sequence, not a conclusion that every offline load strategy fails. Plan 8 adds a separately traceable follow-up rather than rewriting the earlier report.

### Task 8.2.1 — Conditional bounded Iowa comparison

- **ID:** 8.2.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.1.1 and explicit orchestrator approval of the Wave 1 sandbox/index preflight in `STATE_8.md`.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append Wave 2 outcome; do not rewrite Wave 1 evidence).
- **Files in scope — can create:** `planning/tasks/8.2.1.todo.md`; bounded logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, Cargo manifests, prior reports, user-owned notes, cached extension, or any source data except the exact Iowa GeoPackage path. No soilDB, writes, copies, fallback scans, or remote actions.
- **Spec:** Before any feature query, recheck approved preconditions inside the sandbox: host root read-only, exact GeoPackage read-only, scratch writable, network disabled, exact source allowlisted before external access is disabled, and locally loadable extension. Repeat the RTree candidate query and `EXPLAIN`; require a demonstrated virtual-table index and no more than 100 candidate feature IDs. Run `EXPLAIN` for the exact DuckDB `ST_Read` spatial-filter query before execution; if the driver/filter path cannot be shown to use the bounded spatial filter, do not query feature rows. If all gates pass, run only that query with a result cap of 100 and fixed memory/thread/temp/time limits. Record exact SQL/options, plan/profile evidence, returned count and types, and source SHA-256/size/mtime before and after. Any failed or ambiguous gate means zero feature-table rows are read and the blocker is appended to the report.
- **Grep/recon anchors:** Task 8.1.1 report/preflight; DuckDB 2.0 `ST_Read` filter documentation; GDAL GeoPackage driver and RTree documentation; original fingerprint in `planning/reports/G33-2-duckdb-spike-results.md`.
- **Acceptance:** Either (a) record a bounded read of at most 100 features under the approved read-only/network-isolated sandbox, with indexed preflight and unchanged source fingerprint, or (b) record which gate failed and confirm zero feature rows were read. Make no unsupported performance/parity claim. `git diff --check` passes.
- **Targeted acceptance command:** Only after orchestrator gate approval: run the exact sandboxed `duckdb-probe bounded-iowa` command above. Otherwise no feature-read command may run; append the gate outcome to the report.
