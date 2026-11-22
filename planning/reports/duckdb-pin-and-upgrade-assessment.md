# DuckDB pin and upgrade assessment

**Research date:** 2026-11-22
**Scope:** repository pin and source-based upgrade assessment only. No alternate build, dependency change, artifact download, or runtime probe was performed.

## Executive assessment

The committed qualification pin is internally identifiable and reproducible: `duckdb`/`libduckdb-sys` Rust crates `1.20000.2` bundle DuckDB engine `v2.0.0`; the separately provisioned Linux AMD64 Spatial artifact is `v2.0.0`, with the recorded SHA-256 below. It is not DuckDB 1.2.0: the unusual Rust crate version uses a distinct release scheme, while the bundled engine is DuckDB 2.0.0. This is a recent pin, not an obsolete engine line, but it is already behind the current patch release.

As of this research date, DuckDB 2.0.1 is the current 2.0 LTS patch release, announced November 20, 2026, with support through at least October 2027. Official release material explicitly fixes the `CREATE SECRET` bypass when `enable_external_access=false` and a remote path traversal issue under that setting. Those defects are in the bundled 2.0.0 engine, and the documented configuration overlaps this project's threat model. Treat both as applicable risk in the current engine; exact exploitability through this application remains unverified, and the SQL classifier may block some statements but is not evidence of engine enforcement. The fix therefore materially increases urgency to evaluate the patch, but does not itself authorize a pin change.

**Recommendation: evaluate an upgrade promptly; retain the present pin only as a qualification baseline until a separately approved, exact client/engine/Spatial combination has passed compatibility and security checks.** Public sources establish a `duckdb-rs` 2.0.0 client line and DuckDB engine 2.0.1, but this research did not establish a released Rust crate that bundles engine 2.0.1 or a matching official Spatial artifact. Do not infer that selecting crate `2.0.0` yields engine `2.0.1`. Runtime loader/result and geography compatibility on any candidate remain unverified.

## Identity and selection rationale

| Component | Current repository identity | Evidence |
|---|---|---|
| Rust wrapper | `duckdb = "=1.20000.2"`, feature `bundled`; Cargo.lock checksum `23f2dabbb1bbca57b05ab74f0b2e286e1ae8297fa3bc437bc489360a6393736c` | `Cargo.toml`, `Cargo.lock`; crates.io version record [1] |
| Native sys crate | `libduckdb-sys 1.20000.2`; Cargo.lock checksum `deb671f9ffb546c30526fb18c43c3fb303c9f45bddf485b250a8e697b31556f0` | `Cargo.lock` |
| Bundled engine | DuckDB `v2.0.0` | Deployment pin table and G33-2 provenance record [2, 3]; crate release tag targets `v2.0-cyanoptera` [4] |
| Spatial artifact | Official `v2.0.0/linux_amd64/spatial.duckdb_extension`; SHA-256 `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`; observed metadata `("spatial", "a3dbcd1781", "")` | Deployment pin table [2], copied from G33-2 fingerprint [3] |

The repository records selection as reuse of the exact G33-2 fingerprint, keeping client, engine, platform-specific extension and hash together. That is a reproducibility rationale, not evidence that 2.0.0 is the newest or safest option. The Rust crate version `1.20000.2` was published November 13, 2026, and its release tag targets the DuckDB 2.0 code line; it is not evidence of an engine 1.2.0 build [4, 5]. No source found in the scoped repository anchors explains a more specific decision criterion than the existing successful pinned build/artifact fingerprint.

## Release and security context

DuckDB's release calendar lists 2.0.0 LTS on October 21 and 2.0.1 LTS on November 20, 2026; the 2.0 LTS support horizon is at least October 2027 [6, 7]. The announcement identifies 2.0.1 as the first patch in the 2.0 LTS line and points to the full tagged release notes [7, 8]. Accordingly, current 2.0.0 is one patch behind current upstream, not an unsupported major line. The crate's own versioning and release cadence are separate: crates.io records `duckdb 1.20000.2` published November 13 [5], while DuckDB's Rust client documentation identifies `2.0.0` as its latest stable Rust client at research time [9].

| 2.0.1 security item | Official description | Applicability to this pin |
|---|---|---|
| `CREATE SECRET` despite `enable_external_access=false` | 2.0.1 now rejects secret creation with a permission error when external access is disabled; upstream fix #28637 [7, 8]. | **Engine defect applies:** engine is 2.0.0, the exact earlier version described. The repository config sets external access false (see G39 policy/config); an attacker must reach a SQL execution path that can issue secret-creation syntax or equivalent. Whether the executor classifier admits any usable form is not established here. Classifier rejection would not repair direct engine/API exposure. |
| Remote path traversal under `enable_external_access=false` | 2.0.1 fixes a remote path traversal issue under that configuration, #28684 [7, 8]. | **Configuration/version overlap; precise exploitability unverified:** current engine is the affected pre-fix release and configuration matches the release description. Upstream summary does not state all prerequisites, vulnerable functions, or whether classifier-reachable syntax is required. Do not claim a demonstrated application exploit or a complete mitigation. |
| Brotli decoder CVE-2026-111497 | 2.0.1 includes fix #27791 [7, 8]. | **Potential engine/library exposure; unverified:** bundled engine predates the stated fix. No repository-specific evidence establishes whether a reachable input path invokes the vulnerable decoder or its exact bundled revision. |

The announcement also states DuckDB 2.0.1 avoids silent memory zeroing on affected Linux kernels by changing bundled jemalloc behavior, and gives `DUCKDB_JE_MALLOC_CONF=muzzy_decay_ms:0` as a workaround [7]. This is relevant to the bundled allocator, but whether this deployment kernel is affected is not assessed. These release notes provide version applicability, not a substitute for direct probes or a full advisory/CVE analysis. In particular, the source describes security fixes—not a claim that all possible external access is prohibited by one setting.

## Current pin versus candidate

The most conservative candidate is the DuckDB 2.0.1 patch engine, preserving the 2.0 LTS major line. The Rust client documentation names client 2.0.0 as latest stable, but the cited sources do **not** establish a released `duckdb-rs` crate which bundles engine 2.0.1. Thus “Rust client 2.0.0 + engine 2.0.1” is a target combination to investigate, not a verified package pair. A crate update alone must not be assumed to carry the security fixes.

| Axis | Current pin | Newer candidate (2.0.1 engine) | Evidence status / risk |
|---|---|---|---|
| Rust API/client | `duckdb` and `libduckdb-sys` 1.20000.2 | duckdb-rs docs say stable client 2.0.0; exact crate/bundled-engine pairing for engine 2.0.1 not established [4, 9] | **Unverified pairing.** Inspect crate release/tag and bundled source identity before choosing. API changes and Rust/MSRV implications need review. |
| Bundled engine | 2.0.0 | 2.0.1 | Candidate engine release is official/current [6–8]. A matching released bundled Rust package was not found in scoped evidence. |
| Spatial | Official 2.0.0 Linux AMD64 artifact, exact recorded hash above | Must obtain/qualify official 2.0.1 Linux AMD64 artifact and hash separately | DuckDB extension docs/releases are versioned with engine; exact candidate binary identity/hash was not established here. Do not load the 2.0.0 artifact into a 2.0.1 engine by assumption [2, 10]. |
| Security | 2.0.0 lacks the fixes described for 2.0.1 | 2.0.1 release notes document fixes for secret creation with external access disabled, remote path traversal under that setting, and Brotli CVE [7, 8] | Strong source basis to evaluate patch. Actual app exploitability and residual defenses unverified. |
| Loader/result behavior | Existing private loader and result contracts were qualified only against current pin in prior work | No candidate runtime evidence | **Unverified:** loader integration, values/types, errors and serialization can change across engine/client patch. Re-run scoped contract suites after separately authorized pin selection. |
| Six geography functions | Existing observations are from the current pin; methods remain rejected absent parity evidence [3, 11] | No candidate runtime or SQL Server oracle evidence | **Unverified:** do not infer parity or enable methods from upgrade. Revalidate exact extension and all six contracts. |

## Compatibility and upgrade risks

- **Crate/engine mismatch risk:** `duckdb-rs` uses its own version scheme. Pin the client only after confirming its bundled engine with authoritative release metadata/source, rather than interpreting the crate version numerically.
- **Extension ABI/version risk:** Spatial is a separately versioned binary. Keep engine, extension release, platform, provenance and SHA-256 coupled; verify exact load identity and offline loading on the candidate.
- **Behavior/API risk:** 2.0 is a feature release with documented breaking changes from 1.x; the current engine is already 2.0.0, so patch update risk is narrower but still requires Rust API, result, loader, and geography regression checks [7, 12].
- **Storage risk:** DuckDB documents that pre-2.0 versions cannot read databases written in 2.0 format [12]. This service's present loader imports from SQLite into an in-memory DuckDB rather than persisting DuckDB databases, but any future persistence or external database sharing must account for format compatibility.
- **Unproven security breadth:** release summaries identify specific fixes, not exhaustive security guarantees. Continue distinguishing engine configuration from SQL-classifier denials and test direct connection APIs in the planned security probes.
- **No geography inference:** the 2.0.0 Spatial artifact's successful synthetic function evaluations and project observations do not establish SQL Server `geography` semantics; current explicit rejections should remain [3, 11].

## Decision

**Evaluate 2.0.1; do not change the pin under this task.** The present pin is recent and internally traceable, but it is behind the current LTS patch and overlaps two explicitly documented security fixes under the exact external-access setting relevant to this project. The evidence supports prioritizing an upgrade evaluation, not declaring a successful upgrade or production readiness. A later approval should first establish the exact duckdb-rs crate → bundled engine mapping and exact 2.0.1 Spatial artifact/hash, then run offline security, loader/result, and geography qualification on that exact tuple. Until then, preserve the existing pin as the reproducible baseline and do not claim that its classifier closes the engine-level gaps.

## Sources

All sources accessed 2026-11-22; official DuckDB/project sources unless identified otherwise.

1. Repository `Cargo.toml` and `Cargo.lock` (local pinned manifests).
2. [DuckDB Spatial deployment provisioning, Pin](../../docs/duckdb-deployment.md#pin).
3. [G33-2 DuckDB Spatial follow-up, Reproducibility and extension/security](G33-2-duckdb-spatial-followup.md#reproducibility-and-extensionsecurity).
4. [duckdb-rs release v1.20000.2](https://github.com/duckdb/duckdb-rs/releases/tag/v1.20000.2), including release target `v2.0-cyanoptera`.
5. [crates.io duckdb 1.20000.2 version record](https://crates.io/api/v1/crates/duckdb/1.20000.2) (published 2026-11-13; checksum and features).
6. [DuckDB Release Calendar](https://duckdb.org/release_calendar.html) (release dates and LTS horizon).
7. [Announcing DuckDB 2.0.1 LTS](https://duckdb.org/2026/11/20/announcing-duckdb-201.html) (2026-11-20).
8. [DuckDB GitHub release v2.0.1](https://github.com/duckdb/duckdb/releases/tag/v2.0.1) (full release notes).
9. [DuckDB 2.0 Rust Client documentation](https://duckdb.org/docs/2.0/clients/rust/overview.html) (latest stable Rust-client version stated as 2.0.0).
10. [DuckDB 2.0 Spatial extension documentation](https://duckdb.org/docs/2.0/core_extensions/spatial/overview.html) and [extension distribution documentation](https://duckdb.org/docs/2.0/extensions/extension_distribution.html).
11. Repository `tests/duckdb_geography_contract.rs` (pinned observations and explicit rejection tests; not an SQL Server oracle).
12. [Announcing DuckDB 2.0.0 LTS](https://duckdb.org/2026/10/21/announcing-duckdb-200.html) and [DuckDB 2.0 migration guide](https://duckdb.org/docs/2.0/operations_manual/migration.html).
