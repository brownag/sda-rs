# Execution Plan 8 — DuckDB Spatial Follow-up for Geography Methods

## Goal & Scope

### Goal

Revisit the blocked G33-2 DuckDB Spatial evaluation using the cached extension in the correct initialization order, then assess DuckDB’s community `geography` extension as a spherical alternative. Determine whether either offers useful, evidence-backed support for any of G12’s six SQL Server `geography` methods. This is an isolated evaluation and recommendation, not backend adoption.

### In scope

- Load the already-cached Spatial extension by its exact local path while DuckDB external access is still enabled; then configure access limits and disable external access.
- Check the exact DuckDB-version/platform compatibility of the community `geography` extension, then attempt one install/load from DuckDB’s official community extension repository even if the compatibility matrix is blank. Evaluate its S2 spherical type/functions against all six G12 contracts when load succeeds; if it fails, record the precise failure and investigate official DuckDB/maintainer sources for the compatibility status.
- Evaluate DuckDB’s corresponding operations for all six G12 methods—`STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—using synthetic in-memory geometries and the SQL Server `geography` contract as the comparison baseline.
- Record actual DuckDB/Rust result types, SRID handling, units, representative outputs, and NULL/empty/dimensional behavior where safely testable. Distinguish observed results from documentation and do not treat function-name similarity as parity.
- Attempt Iowa GeoPackage feature comparisons only after synthetic probes, and only if an OS sandbox provides a read-only source mount and a preflight proves the query will use bounded indexed access. Otherwise stop after synthetic probes and read-only metadata checks.
- Write a new, evidence-based follow-up report under `planning/reports/` in this repository. “Research branch only” means commit it only on `docs/backend-spike-research`; it does not mean placing the file outside the repository. Preserve the original Plan 7 blocker report as historical evidence.

### Out of scope / deferred

- Adding DuckDB, Spatial, or community-extension dependencies to this repository; changing production code, backend selection, configuration, wire behavior, or security policy.
- Any extension download except (a) one explicit fallback to DuckDB's official `INSTALL spatial` if loading the exact cached binary fails, and (b) one `INSTALL geography FROM community; LOAD geography;` attempt from the official DuckDB community repository. Research network access is limited to official DuckDB and maintained `paleolimbot/duckdb-geography` sources. Do not use mirrors, forks, or arbitrary extension sources, and do not modify existing cached extensions. Source-building is outside this task and requires a separately approved plan.
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
- “Expand Plan 8” to evaluate the community `geography` extension as a candidate.

## Dated Decisions

- **2026-11-19 — Objective:** Reopen G33-2 as a targeted DuckDB Spatial follow-up because it may help with G12; evaluate all six methods against SQL Server `geography` contracts.
- **2026-11-19 — Official Spatial initialization:** For Task 8.1.1, first load the cached Spatial binary by exact path while external access is enabled. If that fails, one explicit `INSTALL spatial; LOAD spatial;` from DuckDB's official extension repository is allowed. Then set the exact `allowed_paths`, disable external access, and verify it is off. No other network access is allowed in Task 8.1.1.
- **2026-11-19 — Community geography expansion:** Add a synthetic-only evaluation of DuckDB's `geography` community extension before any Iowa feature reads. Its documented S2 spherical model is not assumed equivalent to SQL Server's ellipsoidal geography. Check official DuckDB compatibility for DuckDB 2.0.0/Linux AMD64, but a blank matrix does not block the user's authorized one `INSTALL geography FROM community; LOAD geography;` attempt. Record exact binary provenance/hash if it loads; if it fails, preserve the exact error and research official DuckDB/maintainer sources. No source build in this task.
- **2026-11-19 — Lockdown:** Set DuckDB’s exact `allowed_paths` entry for the Iowa GeoPackage before disabling external access. Because DuckDB documents allow-listed paths as both readable and writable, this setting alone is not a read-only guarantee.
- **2026-11-19 — Data boundary:** Run synthetic in-memory probes first. Iowa feature rows are permitted only inside an OS sandbox with the host filesystem read-only, the source explicitly read-only, scratch writable, and network disabled; and only after a read-only preflight demonstrates indexed/bounded access and the orchestrator approves its evidence. If any condition fails, stop without reading feature rows.
- **2026-11-19 — Adoption:** Evaluation only. Even a promising result does not authorize production changes or DuckDB adoption; a separate plan and decision would be required.
- **2026-11-19 — Repository branch:** Plan artifacts are committed separately on the current `docs/backend-spike-research` branch. Reports are files under `planning/reports/` in this repository, committed only on that branch; no `main` integration or remote action.
- **2026-11-19 — Workflow:** Run the official Spatial synthetic evaluation and no-row index preflight first; next assess community `geography` synthetically; only then consider conditional Iowa feature reads. The orchestrator reviews all required no-row evidence before authorizing the Iowa task.
- **2026-11-19 — Architecture:** Comply with accepted ADR 0001 (SQLite/GeoPackage remains the MVP backend) and ADR 0003 (SpatiaLite stays optional). No ADR is superseded.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`; current branch `docs/backend-spike-research`, clean at `a04e12c`. `main` points to the same commit. The commit records the G33 note/GOALS updates; do not rewrite those notes.
- Plans 1–7 precede this orchestration. Plan 8 files are the current draft. Preserve the current branch and commit the three Plan 8 artifacts together in a separate `docs(planning):` commit after approval.
- Separate untracked Plan 9 files appeared during this planning session. They are outside this plan's scope; preserve them and do not edit or stage them. The user reaffirmed Plan 8 as the active request.
- The original G33-2 report is `planning/reports/G33-2-duckdb-spike-results.md`. It records DuckDB crate `1.20000.2` / engine `2.0.0`, a cached 104 MiB Spatial extension at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension`, and the failed order: connection started with `enable_external_access=false`, then `LOAD spatial`.
- The cached official Spatial extension exists. Treat it as a read-only input; verify its hash/version and do not modify it. Put all new probe sources, targets, extension cache, and temporary data in `/tmp/opencode/sda-rs-plan8-duckdb/`. Use Cargo offline mode; stop if required crate artifacts are unavailable rather than fetching. User-authorized extension acquisition is limited to the official Spatial fallback and, if officially compatible, one DuckDB community `geography` install.
- The Iowa GeoPackage is `/data/SSURGO/output/iowa_ssurgo.gpkg` (6,406,705,152 bytes; prior SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`). Prior G33-2 read-only plan checks reported full scans for `mupolygon.mukey='1857266'` and `legend.areasymbol='IA001'`; those queries must not be run unless a new preflight proves bounded indexed access.
- DuckDB 2.0 security documentation says `allowed_paths` must be configured before external access is disabled and permits writes as well as reads. An OS-level read-only sandbox is therefore mandatory for Iowa feature access. `bwrap` is present; whether the required namespaces/sandbox are permitted is unverified. Failure to establish the sandbox is a stop condition, not permission to weaken it.
- DuckDB 2.0 Spatial documentation lists all six corresponding free-function operations. Its generic `ST_Buffer` is planar and uses input coordinate units; `ST_Area_Spheroid` assumes WGS84 and documents latitude/longitude axis order; ordinary `ST_Area` uses CRS coordinate units. Function availability alone does not establish SQL Server geography semantics.
- DuckDB's official community-extension catalog describes `geography` as global sphere processing based on Google's S2 library, exposes `GEOGRAPHY` and `s2_*` operations, and documents `INSTALL geography FROM community; LOAD geography;`. At reconnaissance, the catalog compatibility matrix displayed DuckDB 2.0.0 but no Linux AMD64 compatibility entry. This is a compatibility risk, not a stop condition: make the one authorized official install attempt and record the observed result. Source-building remains out of scope.
- Existing useful evidence: `planning/reports/G12-geography-method-feasibility.md` contains SQL Server primary citations and SpatiaLite comparisons. Use it as comparison context, not as a claim about DuckDB.
- Entry sizes: `src/spatial.rs` 572 lines; `src/transpiler/spatial.rs` 1,519; `src/config.rs` 226; `src/lib.rs` 40; `tests/spatial_methods_contract.rs` 130. Plan 8 makes no production edits.
- Accepted ADRs 0001–0006 remain in force. Relevant choices: SQLite/GeoPackage remains the service backend, spatial support stays optional, and unsupported capabilities are explicit.
- Green baseline on the current branch: `cargo fmt --check`, `git diff --check`, default `cargo test`, and configured `SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so cargo test` passed. Both Rust suites reported 75 unit tests and 30 integration tests with no failures.
- Project conventions: `AGENTS.md`, `CONTRIBUTING.md`, and `planning/PROMPT_TEMPLATE_7.md`; test commands are `cargo test`, configured full `cargo test`, and `cargo fmt --check`. Plan 8 is research-only, so its targeted acceptance commands run the standalone probe rather than changing this crate.

## Dependency Graph and Waves

| Wave | Tasks | Purpose |
|---|---|---|
| 1 | 8.1.1 | Load/cache official Spatial, lock down, run all synthetic six-method probes, and perform no-feature-row Iowa sandbox/index preflight. |
| 2 | 8.2.1 | Verify compatibility and, only if listed, load the official community `geography` extension; run synthetic S2 contract probes and append results. |
| 3 | 8.3.1 | Only after explicit orchestrator approval of Wave 1 and 2 evidence, perform bounded Iowa comparisons; otherwise record why feature reads remain blocked. |

Tasks update the same report sequentially; no same-wave file overlap exists. Wave 3 is conditional and must not start feature reads until the orchestrator reviews the sandbox/index evidence and explicitly approves the gate. No production files are in scope.

## Task Specifications

### Task 8.1.1 — Re-evaluate DuckDB Spatial for G12 geography methods

- **ID:** 8.1.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** None.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md`.
- **Files in scope — can create:** `planning/tasks/8.1.1.todo.md`; scratch project, scripts, logs, targets, and disposable databases only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production source/tests, `Cargo.toml`/`Cargo.lock`, prior Plan 7 reports, user-owned G33 notes, the cached extension, or the Iowa source file. Do not access soilDB. Do not run any network/install command other than the single explicitly allowed DuckDB official Spatial fallback. No remote Git actions.
- **Spec:** In a standalone Rust probe pinned to the already-tested DuckDB client version, first load the exact cached Spatial binary while external access is enabled; if it fails, use exactly one explicit `INSTALL spatial; LOAD spatial;` from DuckDB's official repository. Disable autoload/autoinstall, set bounded resource settings, and set the exact Iowa `allowed_paths` entry before `SET enable_external_access=false`; verify the setting and that an attempt to re-enable it is rejected. Run synthetic WKT tests for the six DuckDB spatial counterparts, recording engine/extension/PROJ versions, Rust result types, documented vs observed behavior, SRID behavior, units, representative output, and safely testable NULL/empty/Z/M cases against the SQL Server citations in `planning/reports/G12-geography-method-feasibility.md`. Do not claim parity from similar names or a single sample. Without reading feature rows, test whether `bwrap` can expose the host read-only, scratch writable, source explicitly read-only, and network disabled; inspect GeoPackage metadata and RTree schema in read-only mode, and run only a capped RTree-index preflight plus `EXPLAIN` showing virtual-table index use. Preflight may read at most 101 RTree candidate IDs, but no feature-table rows. Preserve exact probe/preflight sources and commands under `/tmp/opencode/sda-rs-plan8-duckdb/`; produce the initial sourced report and wait for orchestrator review. This task does not evaluate the community extension or start Iowa feature reads.
- **Grep/recon anchors:** `planning/reports/G33-2-duckdb-spike-results.md` (prior runtime blocker and pinned version); `planning/reports/G12-geography-method-feasibility.md` (six-method contract and citations); DuckDB 2.0 Spatial functions reference (`ST_Contains`, `ST_Within`, `ST_Intersection`, `ST_Buffer`, `ST_Area`, `ST_Area_Spheroid`, `ST_AsText`); DuckDB 2.0 Securing DuckDB (`allowed_paths`, `enable_external_access`, ordering and write permissions); GDAL GeoPackage driver (RTree indexes); `planning/PROMPT_TEMPLATE_7.md` for repository task workflow.
- **Acceptance:** The probe source and exact pinned manifest are preserved in scratch. An offline Cargo build succeeds. In the synthetic run, cached absolute-path load is attempted first; if it fails, only the authorized DuckDB official-install fallback is used. After successful load the probe configures the allowlist, disables external access, confirms it is false, confirms an attempted re-enable is rejected, and only then runs the six synthetic probes. Report exact versions, extension path/hash/acquisition path, outputs/errors/types, SQL Server contract comparison, and DuckDB documentation. The Iowa preflight reads no feature rows; it records the actual sandbox mount configuration (not merely in-sandbox permission bits), read-only metadata, the exact bounded RTree candidate query and `EXPLAIN` output. It may read at most 101 RTree IDs; no Iowa feature query is part of this task. `git diff --check` passes; no out-of-scope repository or external file changed.
- **Targeted acceptance commands:**
  - Offline build: `CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home cargo build --offline --release --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml`
  - Wave 1 synthetic/security run (root read-only; scratch read-write; network is only used if the exact official `INSTALL spatial` fallback is required; no Iowa feature reads): `bwrap --die-with-parent --ro-bind / / --dev /dev --proc /proc --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe synthetic`
  - Wave 1 metadata/RTree preflight (no feature rows; network disabled): run the preflight script preserved in scratch under the equivalent `bwrap` sandbox shown here, using its exact recorded arguments: `bwrap --die-with-parent --unshare-net --ro-bind / / --dev /dev --proc /proc --ro-bind /data/SSURGO/output/iowa_ssurgo.gpkg /data/SSURGO/output/iowa_ssurgo.gpkg --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb python3 /tmp/opencode/sda-rs-plan8-duckdb/preflight_iowa.py --layer mupolygon --bbox=-93.70,41.50,-93.69,41.51 --candidate-limit 101`. This command may inspect metadata and RTree IDs only, never feature-table rows.
  - Wave 3 Iowa command (only after separate orchestrator gate approval; network disabled, host root read-only, source explicitly read-only, scratch read-write): `bwrap --die-with-parent --unshare-net --ro-bind / / --dev /dev --proc /proc --ro-bind /data/SSURGO/output/iowa_ssurgo.gpkg /data/SSURGO/output/iowa_ssurgo.gpkg --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe bounded-iowa`
  - Documentation check: `git diff --check`

## Hazard Notes

- `enable_external_access=false` at connection creation blocks dynamic extension loading. The corrected order is essential. Cargo builds offline. Each task's permitted extension acquisition is narrowly specified and ends before lockdown; Iowa reads always run without network access.
- `allowed_paths` is not a read-only control. Iowa access without an OS read-only sandbox is forbidden even if the DuckDB path allowlist is exact.
- DuckDB `ST_Read` filter pushdown is driver-dependent. A `LIMIT` does not make an unindexed scan safe; the feature-read gate requires demonstrated indexed/bounded access before any feature query.
- Waves 1 and 2 must not execute a feature-table query. Wave 3 requires successful no-row sandbox/index evidence and separate explicit orchestrator approval; failure means no Iowa feature reads.
- `ST_Area_Spheroid` is documented for EPSG:4326 with latitude/longitude axis order and cannot be presumed to implement SQL Server’s SRID-dependent area contract. The remaining DuckDB spatial operations are generally planar unless explicitly documented otherwise.
- The cached binary is version/platform-specific. Do not replace or modify it. If it fails, use only the explicitly approved DuckDB official-repository install fallback and record the downloaded extension's resolved path and hash.
- The prior G33-2 report is a valid historical account of the tested sequence, not a conclusion that every offline load strategy fails. Plan 8 adds a separately traceable follow-up rather than rewriting the earlier report.

### Task 8.2.1 — Evaluate DuckDB community geography extension

- **ID:** 8.2.1
- **Status:** Addendum approved by user on 2026-11-19; not yet executed.
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.1.1. This task is synthetic-only and must not read Iowa data.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append Wave 2 outcome; do not rewrite Wave 1 evidence).
- **Files in scope — can create:** `planning/tasks/8.2.1.todo.md`; standalone probe source/manifest/lockfile, target output, scratch DuckDB home/extension cache, and logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, Cargo manifests, prior reports, user-owned notes, cached extensions, and Iowa source. No feature rows, soilDB, non-official extension source, repository dependency changes, or remote actions.
- **Spec:** Inspect DuckDB's official community-extension compatibility matrix for engine `2.0.0` / Linux AMD64 and record it, but do not treat a blank entry as a stop condition. Build the standalone Rust probe offline using the pinned client and keep the Cargo project, DuckDB home, and extension cache under scratch. Run the probe once inside Bubblewrap with host `/` read-only, scratch writable, and `DUCKDB_HOME` pointed at that scratch home; the probe makes exactly one `INSTALL geography FROM community; LOAD geography;` attempt while DuckDB external access is enabled. Record engine/extension versions, resolved path, SHA-256, and exact success/error. If it fails, inspect official DuckDB and maintained extension sources for documented compatibility/release status; do not source-build or use mirrors/forks. If it loads, set autoload/autoinstall off, bounded resource settings, and `enable_external_access=false`; verify lockdown before synthetic SQL. Probe the GEOGRAPHY type and documented `s2_*` equivalents for all six methods: `s2_contains`/reversed containment for `STContains` and `STWithin`, `s2_intersection`, `s2_area`, `s2_astext`, and any directly available buffer equivalent. If no defensible equivalent exists (especially for `STBuffer`), record unsupported rather than substituting a planar function. Capture logical SQL types and Rust values, SRID/input behavior, units/model, representative WKT, antimeridian and boundary cases where applicable, NULL/empty/dimensional cases where safe. Compare observed S2-spherical behavior to SQL Server geography's ellipsoidal contract; do not claim parity from names or docs. No Iowa access occurs in this task.
- **Grep/recon anchors:** Wave 1 report, Task 8.1.1 probe patterns; official DuckDB community extension page and compatibility matrix (`https://duckdb.org/community_extensions/extensions/geography`); `https://github.com/paleolimbot/duckdb-geography` README/function reference; G12 report and Microsoft Learn citations; DuckDB 2.0 Securing DuckDB.
- **Acceptance:** Record the official compatibility matrix and the result of exactly one official community install/load attempt. Preserve the exact pinned probe/manifest and commands in scratch. On success, establish extension version/path/hash, verify lockdown, and report each method mapping as observed supported, unsupported, or unverified with SQL/Rust types, S2 spherical model/units, outputs/errors, and SQL Server contract comparison. On failure, record the exact error plus official DuckDB/maintainer compatibility evidence; do not retry with another source or source-build. No Iowa feature rows are read. `git diff --check` passes.
- **Targeted acceptance commands:**
  - Record the official DuckDB compatibility matrix for `2.0.0` / Linux AMD64; make exactly one community install attempt through the official registry, even if the entry is blank.
  - Offline build: `CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home cargo build --offline --release --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml`
  - Network-authorized one-time install/load and synthetic probe, with host root read-only and scratch writable: `DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home bwrap --die-with-parent --ro-bind / / --dev /dev --proc /proc --bind /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb --chdir /tmp/opencode/sda-rs-plan8-duckdb /tmp/opencode/sda-rs-plan8-duckdb/target/release/duckdb-probe geography-synthetic`
  - `git diff --check`

### Task 8.3.1 — Conditional bounded Iowa comparison

- **ID:** 8.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.1.1, 8.2.1, and explicit orchestrator approval of the Wave 1/2 sandbox and indexed-access evidence in `STATE_8.md`.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append the Iowa outcome).
- **Files in scope — can create:** `planning/tasks/8.3.1.todo.md`; bounded logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, Cargo manifests, prior reports, user-owned notes, cached extensions, or any source data except the exact Iowa GeoPackage. No soilDB, writes, copies, fallback scans, or remote actions.
- **Spec:** Recheck all approved preconditions inside the sandbox: host root read-only, exact GeoPackage read-only, scratch writable, network disabled, exact source allowlisted before external access is disabled, and required locally cached extension(s) loadable without network. Repeat the capped RTree candidate query and `EXPLAIN`; require a demonstrated virtual-table index and no more than 100 candidate IDs. Run `EXPLAIN` for the exact DuckDB spatial-filter query before executing it; if the driver/filter path cannot be shown to use the bounded index, read no feature rows. If all gates pass, perform only the approved comparison with at most 100 features, using only the official Spatial and/or community `GEOGRAPHY` candidates established in prior waves, and fixed memory/thread/temp/time limits. Capture exact SQL/options, returned count/types, plan/profile, and source SHA-256/size/mtime before and after. Any failed or ambiguous gate means zero feature-table rows are read and the blocker is appended to the report.
- **Grep/recon anchors:** Wave 1/2 report sections and preflight artifacts; DuckDB 2.0 `ST_Read` filter docs; GDAL GeoPackage/RTree docs; original Iowa fingerprint in the Plan 7 report.
- **Acceptance:** Either record an approved, indexed, bounded, read-only comparison of at most 100 features with unchanged source fingerprint, or record the precise failed gate and confirm zero feature rows were read. Make no unsupported performance/parity claim. `git diff --check` passes.
- **Targeted acceptance command:** Only after separate orchestrator gate approval: run the exact sandboxed `duckdb-probe bounded-iowa` command, using only extensions already cached by prior waves. Otherwise, no feature-read command may run.
