# Execution Plan 8 — DuckDB Spatial Follow-up for Geography Methods

## Goal & Scope

### Goal

Revisit the blocked G33-2 DuckDB Spatial evaluation using the cached extension in the correct initialization order, then assess DuckDB’s community `geography` extension as a spherical alternative. Determine whether either offers useful, evidence-backed support for any of G12’s six SQL Server `geography` methods. This is an isolated evaluation and recommendation, not backend adoption.

### In scope

- Load the already-cached Spatial extension by its exact local path while DuckDB external access is still enabled; then configure access limits and disable external access.
- Check the exact DuckDB-version/platform compatibility of the community `geography` extension, then attempt one install/load from DuckDB’s official community extension repository even if the compatibility matrix is blank. Evaluate its S2 spherical type/functions against all six G12 contracts when load succeeds; if it fails, record the precise failure and investigate official DuckDB/maintainer sources for the compatibility status.
- Evaluate DuckDB’s corresponding operations for all six G12 methods—`STContains`, `STWithin`, `STIntersection`, `STBuffer`, `STArea`, and `STAsText`—using synthetic in-memory geometries and the SQL Server `geography` contract as the comparison baseline.
- Record actual DuckDB/Rust result types, SRID handling, units, representative outputs, and NULL/empty/dimensional behavior where safely testable. Distinguish observed results from documentation and do not treat function-name similarity as parity.
- Attempt Iowa GeoPackage feature comparisons only after synthetic probes and a preflight proves the query will use bounded indexed access. Do not use Bubblewrap or an OS sandbox; use read-only data-access procedures, prohibit writes, and compare source fingerprints before/after.
- Write a new, evidence-based follow-up report under `planning/reports/` in this repository. “Research branch only” means commit it only on `docs/backend-spike-research`; it does not mean placing the file outside the repository. Preserve the original Plan 7 blocker report as historical evidence.

### Out of scope / deferred

- Adding DuckDB, Spatial, or community-extension dependencies to this repository; changing production code, backend selection, configuration, wire behavior, or security policy.
- Any extension download except (a) one explicit fallback to DuckDB's official `INSTALL spatial` if loading the exact cached binary fails, and (b) one `INSTALL geography FROM community; LOAD geography;` attempt from the official DuckDB community repository. Research network access is limited to official DuckDB and maintained `paleolimbot/duckdb-geography` sources. Do not use mirrors, forks, or arbitrary extension sources, and do not modify existing cached extensions. Source-building is outside this task and requires a separately approved plan.
- Any Iowa feature-row read without demonstrated indexed/bounded access, a maximum of 100 returned features, read-only access procedures, and source fingerprints before/after. No fallback to a full scan or a `LIMIT` over an unbounded scan. Do not require or use an OS sandbox.
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
- “do away with all this sandboxing crap”
- “Current research branch”
- “Research branch only”
- “Expand Plan 8” to evaluate the community `geography` extension as a candidate.

## Dated Decisions

- **2026-11-19 — Objective:** Reopen G33-2 as a targeted DuckDB Spatial follow-up because it may help with G12; evaluate all six methods against SQL Server `geography` contracts.
- **2026-11-19 — Official Spatial initialization:** For Task 8.1.1, first load the cached Spatial binary by exact path while external access is enabled. If that fails, one explicit `INSTALL spatial; LOAD spatial;` from DuckDB's official extension repository is allowed. Then set the exact `allowed_paths`, disable external access, and verify it is off. No other network access is allowed in Task 8.1.1.
- **2026-11-19 — Community geography expansion:** Add a synthetic-only evaluation of DuckDB's `geography` community extension before any Iowa feature reads. Its documented S2 spherical model is not assumed equivalent to SQL Server's ellipsoidal geography. Check official DuckDB compatibility for DuckDB 2.0.0/Linux AMD64, but a blank matrix does not block the user's authorized one `INSTALL geography FROM community; LOAD geography;` attempt. Record exact binary provenance/hash if it loads; if it fails, preserve the exact error and research official DuckDB/maintainer sources. No source build in this task.
- **2026-11-19 — No OS sandbox:** At the user's direction, do not use Bubblewrap, read-only bind mounts, or other OS-level sandboxing. Retain indexed/bounded access, read-only query procedures, the 100-feature cap, exact DuckDB path allowlisting, and source fingerprints before/after any Iowa comparison.
- **2026-11-19 — Lockdown:** Set DuckDB’s exact `allowed_paths` entry for the Iowa GeoPackage before disabling external access. Because DuckDB documents allow-listed paths as both readable and writable, this setting alone is not a read-only guarantee.
- **2026-11-19 — Data boundary:** Run synthetic in-memory probes first. Iowa feature rows are permitted only after a read-only metadata/RTree preflight demonstrates indexed/bounded access and the orchestrator approves its evidence. Use read-only SQLite/GDAL procedures, prohibit writes, cap results at 100, and fingerprint the source before/after. Do not use an OS sandbox; if any data-access gate fails, stop without reading feature rows.
- **2026-11-19 — Adoption:** Evaluation only. Even a promising result does not authorize production changes or DuckDB adoption; a separate plan and decision would be required.
- **2026-11-19 — Repository branch:** Plan artifacts are committed separately on the current `docs/backend-spike-research` branch. Reports are files under `planning/reports/` in this repository, committed only on that branch; no `main` integration or remote action.
- **2026-11-19 — Workflow:** Run the official Spatial synthetic evaluation and no-row index preflight first; assess community `geography` synthetically and correct the extension cache path; only then consider conditional Iowa feature reads. The orchestrator reviews all required no-row evidence before authorizing the Iowa task.
- **2026-11-19 — Architecture:** Comply with accepted ADR 0001 (SQLite/GeoPackage remains the MVP backend) and ADR 0003 (SpatiaLite stays optional). No ADR is superseded.

## Reconnaissance

- Repository: `/home/andrew/workspace/sda-rs`; current branch `docs/backend-spike-research`, clean at `a04e12c`. `main` points to the same commit. The commit records the G33 note/GOALS updates; do not rewrite those notes.
- Plans 1–7 precede this orchestration. Plan 8 files are the current draft. Preserve the current branch and commit the three Plan 8 artifacts together in a separate `docs(planning):` commit after approval.
- Separate untracked Plan 9 files appeared during this planning session. They are outside this plan's scope; preserve them and do not edit or stage them. The user reaffirmed Plan 8 as the active request.
- The original G33-2 report is `planning/reports/G33-2-duckdb-spike-results.md`. It records DuckDB crate `1.20000.2` / engine `2.0.0`, a cached 104 MiB Spatial extension at `/tmp/opencode/sda-rs-plan7-duckdb/duckdb-home/v2.0.0/linux_amd64/spatial.duckdb_extension`, and the failed order: connection started with `enable_external_access=false`, then `LOAD spatial`.
- The cached official Spatial extension exists. Treat it as a read-only input; verify its hash/version and do not modify it. Put all new probe sources, targets, extension cache, and temporary data in `/tmp/opencode/sda-rs-plan8-duckdb/`. Use Cargo offline mode; stop if required crate artifacts are unavailable rather than fetching. User-authorized extension acquisition is limited to the official Spatial fallback and, if officially compatible, one DuckDB community `geography` install.
- The Iowa GeoPackage is `/data/SSURGO/output/iowa_ssurgo.gpkg` (6,406,705,152 bytes; prior SHA-256 `7b81951e495799aec46eb3fdd1505131cc62374f2d336420706494fae90571a8`). Prior G33-2 read-only plan checks reported full scans for `mupolygon.mukey='1857266'` and `legend.areasymbol='IA001'`; those queries must not be run unless a new preflight proves bounded indexed access.
- DuckDB 2.0 security documentation says `allowed_paths` must be configured before external access is disabled and permits writes as well as reads. The user has removed OS sandboxing; do not treat the allowlist as a read-only control. Use only read-only SQLite/GDAL access procedures and DuckDB read-only settings where applicable, prohibit write statements, and verify the source fingerprint before/after.
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
| 1 | 8.1.1 | Completed: load/cache official Spatial, run six synthetic probes, and perform a no-feature-row Iowa index preflight. |
| 2 | 8.2.1 | Completed with setup blocker: the community install attempt failed before network access because its extension directory was not redirected. |
| 3 | 8.3.1 | Correct the community extension install directory with DuckDB's `extension_directories` setting and make one corrected official install attempt; run synthetic probes if it loads. |
| 4 | 8.4.1 | Only after explicit orchestrator approval of Wave 1–3 evidence, perform bounded Iowa comparisons; otherwise record why feature reads remain blocked. |

Tasks update the same report sequentially; no same-wave file overlap exists. Wave 4 is conditional and must not start feature reads until the orchestrator reviews the indexed/bounded-access evidence and explicitly approves the gate. No OS sandbox is used. No production files are in scope.

## Task Specifications

### Task 8.1.1 — Re-evaluate DuckDB Spatial for G12 geography methods

- **ID:** 8.1.1
- **Status:** Complete; do not rerun. The historical no-row preflight used Bubblewrap before the user removed OS sandboxing for future tasks.
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** None.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md`.
- **Files in scope — can create:** `planning/tasks/8.1.1.todo.md`; scratch project, scripts, logs, targets, and disposable databases only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production source/tests, `Cargo.toml`/`Cargo.lock`, prior Plan 7 reports, user-owned G33 notes, the cached extension, or the Iowa source file. Do not access soilDB. Do not run any network/install command other than the single explicitly allowed DuckDB official Spatial fallback. No remote Git actions.
- **Spec:** Completed historical task: its standalone Rust probe loaded cached Spatial before lockdown, evaluated all six methods synthetically, and inspected Iowa metadata/RTree IDs only (no feature-table rows). It set the exact Iowa allowlist before external access was disabled and recorded outputs/docs in the report. It does not authorize any future Iowa feature reads. Its historical sandbox evidence is not a requirement for future tasks; follow the user's no-OS-sandbox direction.
- **Grep/recon anchors:** `planning/reports/G33-2-duckdb-spike-results.md` (prior runtime blocker and pinned version); `planning/reports/G12-geography-method-feasibility.md` (six-method contract and citations); DuckDB 2.0 Spatial functions reference (`ST_Contains`, `ST_Within`, `ST_Intersection`, `ST_Buffer`, `ST_Area`, `ST_Area_Spheroid`, `ST_AsText`); DuckDB 2.0 Securing DuckDB (`allowed_paths`, `enable_external_access`, ordering and write permissions); GDAL GeoPackage driver (RTree indexes); `planning/PROMPT_TEMPLATE_7.md` for repository task workflow.
- **Acceptance:** The probe source and exact pinned manifest are preserved in scratch. An offline Cargo build succeeds. In the synthetic run, cached absolute-path load is attempted first; if it fails, only the authorized DuckDB official-install fallback is used. After successful load the probe configures the allowlist, disables external access, confirms it is false, confirms an attempted re-enable is rejected, and only then runs the six synthetic probes. Report exact versions, extension path/hash/acquisition path, outputs/errors/types, SQL Server contract comparison, and DuckDB documentation. The Iowa preflight reads no feature rows; it records read-only metadata and the exact bounded RTree candidate query and `EXPLAIN` output. It may read at most 101 RTree IDs; no Iowa feature query is part of this task. `git diff --check` passes; no out-of-scope repository or external file changed.
- **Targeted acceptance commands:**
  - Offline build: `CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home cargo build --offline --release --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml`
  - Historical Wave 1 commands and outputs are preserved in the report and already verified; do not rerun.
  - Documentation check: `git diff --check`

## Hazard Notes

- `enable_external_access=false` at connection creation blocks dynamic extension loading. The corrected order is essential. Cargo builds offline. Each task's permitted extension acquisition is narrowly specified and ends before lockdown; Iowa reads always run without network access.
- `allowed_paths` is not a read-only control. Do not use an OS sandbox; future Iowa access, if separately approved, must use read-only procedures, indexed/bounded query evidence, the 100-feature cap, and before/after source fingerprints.
- DuckDB `ST_Read` filter pushdown is driver-dependent. A `LIMIT` does not make an unindexed scan safe; the feature-read gate requires demonstrated indexed/bounded access before any feature query.
- Waves 1–3 must not execute an Iowa feature-table query. Wave 4 requires successful no-row indexed/bounded-access evidence and separate explicit orchestrator approval; failure means no Iowa feature reads.
- `ST_Area_Spheroid` is documented for EPSG:4326 with latitude/longitude axis order and cannot be presumed to implement SQL Server’s SRID-dependent area contract. The remaining DuckDB spatial operations are generally planar unless explicitly documented otherwise.
- The cached binary is version/platform-specific. Do not replace or modify it. If it fails, use only the explicitly approved DuckDB official-repository install fallback and record the downloaded extension's resolved path and hash.
- The prior G33-2 report is a valid historical account of the tested sequence, not a conclusion that every offline load strategy fails. Plan 8 adds a separately traceable follow-up rather than rewriting the earlier report.

### Task 8.2.1 — Evaluate DuckDB community geography extension

- **ID:** 8.2.1
- **Status:** Completed with setup blocker; the attempt failed before network access because `DUCKDB_HOME` does not configure DuckDB's extension directory. Follow-up task 8.3.1 corrects this.
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.1.1. This task is synthetic-only and must not read Iowa data.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append Wave 2 outcome; do not rewrite Wave 1 evidence).
- **Files in scope — can create:** `planning/tasks/8.2.1.todo.md`; standalone probe source/manifest/lockfile, target output, scratch DuckDB home/extension cache, and logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, Cargo manifests, prior reports, user-owned notes, cached extensions, and Iowa source. No feature rows, soilDB, non-official extension source, repository dependency changes, or remote actions.
- **Spec:** Inspect DuckDB's official community-extension compatibility matrix for engine `2.0.0` / Linux AMD64 and record it, but do not treat a blank entry as a stop condition. Build the standalone Rust probe offline using the pinned client and keep the Cargo project, DuckDB extension directory, and logs under scratch. Task 8.2.1's single install attempt is historical and failed before network access because `DUCKDB_HOME` did not configure DuckDB's extension directory; do not rerun it. The corrected one-time attempt belongs to Task 8.3.1, using DuckDB's `extension_directories` setting without Bubblewrap. No Iowa access occurs in this task.
- **Grep/recon anchors:** Wave 1 report, Task 8.1.1 probe patterns; official DuckDB community extension page and compatibility matrix (`https://duckdb.org/community_extensions/extensions/geography`); `https://github.com/paleolimbot/duckdb-geography` README/function reference; G12 report and Microsoft Learn citations; DuckDB 2.0 Securing DuckDB.
- **Acceptance:** Record the official compatibility matrix and the result of exactly one official community install/load attempt. Preserve the exact pinned probe/manifest and commands in scratch. On success, establish extension version/path/hash, verify lockdown, and report each method mapping as observed supported, unsupported, or unverified with SQL/Rust types, S2 spherical model/units, outputs/errors, and SQL Server contract comparison. On failure, record the exact error plus official DuckDB/maintainer compatibility evidence; do not retry with another source or source-build. No Iowa feature rows are read. `git diff --check` passes.
- **Targeted acceptance commands:**
  - Record the official DuckDB compatibility matrix for `2.0.0` / Linux AMD64; make exactly one community install attempt through the official registry, even if the entry is blank.
  - Offline build: `CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan8-duckdb/target DUCKDB_HOME=/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home cargo build --offline --release --manifest-path /tmp/opencode/sda-rs-plan8-duckdb/Cargo.toml`
  - Historical invocation and failure are preserved in the report; do not rerun. The corrected attempt is Task 8.3.1.
  - `git diff --check`

### Task 8.3.1 — Correct community extension directory and retry

- **ID:** 8.3.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.2.1 (the first attempt failed before repository/network access because the extension directory was not redirected).
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append corrected-attempt outcome; preserve prior attempt as historical evidence).
- **Files in scope — can create:** `planning/tasks/8.3.1.todo.md`; standalone probe source/manifest/lockfile, target output, scratch extension directory, and logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, repository Cargo manifests, prior reports, user-owned notes, cached official Spatial binary, and Iowa data. No source build, mirrors/forks, or remote Git actions.
- **Spec:** Use DuckDB 2.0's `extension_directories` configuration option, not `DUCKDB_HOME` or Bubblewrap. Set `extension_directories=['/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home']` before the official install. The prior attempt errored before any repository access, so this task may make exactly one corrected `INSTALL geography FROM community; LOAD geography;` attempt. If install/load fails, preserve exact output and inspect only official DuckDB/maintainer sources; do not retry or source-build. If it succeeds, record version/path/hash, set autoload/autoinstall off and bounded settings, disable external access and verify, then run the six synthetic S2 probes. No Iowa feature rows are in scope.
- **Grep/recon anchors:** Task 8.2.1 report section (misdirected extension-directory error); DuckDB 2.0 extension-install docs for `extension_directories`; official community geography page and maintained function reference; scratch probe in `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Acceptance:** Demonstrate DuckDB's effective `extension_directories` value points only to scratch before installation. Make exactly one corrected official install/load attempt. On success, verify extension provenance and post-load lockdown before synthetic SQL, then append method observations. On failure, capture exact error and source evidence. No Iowa data is read; `git diff --check` passes.
- **Targeted acceptance command:** Offline scratch build followed by one ordinary synthetic probe invocation; first set and verify `extension_directories=['/tmp/opencode/sda-rs-plan8-duckdb/duckdb-home']`. Do not use Bubblewrap or `DUCKDB_HOME`.

### Task 8.4.1 — Conditional bounded Iowa comparison

- **ID:** 8.4.1
- **Repo:** `/home/andrew/workspace/sda-rs`
- **Depends on:** 8.1.1, 8.2.1, 8.3.1, and explicit orchestrator approval of indexed/bounded-access evidence in `STATE_8.md`.
- **Files in scope — must edit:** `planning/reports/G33-2-duckdb-spatial-followup.md` (append the Iowa outcome).
- **Files in scope — can create:** `planning/tasks/8.4.1.todo.md`; bounded logs only under `/tmp/opencode/sda-rs-plan8-duckdb/`.
- **Do not touch:** production files, Cargo manifests, prior reports, user-owned notes, cached extensions, or any source data except the exact Iowa GeoPackage. No soilDB, writes, copies, fallback scans, or remote actions.
- **Spec:** Recheck the exact Iowa path and source fingerprint. Use read-only SQLite URI/query-only mode for metadata and RTree preflight; run no feature-table query until `EXPLAIN` for the exact DuckDB spatial-filter query demonstrates indexed/bounded access. Set the exact DuckDB `allowed_paths` entry before `enable_external_access=false`; use only read-only data-access functions and no SQL that writes. If all gates pass, perform only the approved comparison with at most 100 features and fixed memory/thread/temp/time limits. Capture exact SQL/options, returned count/types, plan/profile, and source SHA-256/size/mtime before and after. If the index/filter path is not demonstrably bounded, or any read-only check fails, read zero feature rows and append the blocker. No OS sandbox is to be used.
- **Grep/recon anchors:** Wave 1–3 report sections and preflight artifacts; DuckDB 2.0 `ST_Read` filter docs; GDAL GeoPackage/RTree docs; original Iowa fingerprint in the Plan 7 report.
- **Acceptance:** Either record an approved, indexed, bounded, read-only comparison of at most 100 features with unchanged source fingerprint, or record the precise failed gate and confirm zero feature rows were read. Make no unsupported performance/parity claim. `git diff --check` passes.
- **Targeted acceptance command:** Only after separate orchestrator gate approval, run the ordinary `duckdb-probe bounded-iowa` command with extensions cached by prior waves and configured through `extension_directories`; do not wrap it in Bubblewrap. Otherwise, no feature-read command may run.
