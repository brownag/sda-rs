# TODO

## Goal
Address valid inline review findings on PR #1 and prepare concise draft replies without posting to GitHub.

## Tasks

### 1. Verify findings and establish response plan
- [x] Verified and classified all 16 actionable inline findings; each is valid against the reviewed PR head. Addressing specifics in tasks below.
- [x] Commit this tracked plan separately before implementation (`8077ac6`).

### 2. Correctness, execution, and spatial fixes
- [x] Enforce one deadline across statement batches and row iteration; add expiration/interruption coverage.
- [x] Allow temporary indexes while preserving denial of persistent indexes.
- [x] Restore CLI help/version exit behavior and ignore SQL literals/comments in spatial capability detection.
- [x] Supply Web-Mercator CRS metadata in request-local SpatiaLite state without writing to the GeoPackage.
- [x] Align spatial predicate SRIDs and require true intersection; add executed non-origin coverage.
- [x] Compare table-input keys without lossy numeric casts; add regression coverage.
- [x] Reject unsupported column-alias lists for scalar/table WKT TVFs and add tests.
- [x] Preserve SQL comments and delimited identifiers in preprocessing; add regression coverage.
- [x] Make environment-dependent configured spatial tests skip cleanly when SpatiaLite is unset; validate both test modes.
- [x] Make the soilDB harness prove requests route to the local endpoint, resolve Cargo's executable path from JSON build output, and pass the configured integration suite (`60d9b0c`).

### 3. Documentation, validation, and review replies
- [x] Update SDA_Get support and spatial contract documentation to match implemented support (`7edf016`).
- [x] Run focused/full validation: default and SpatiaLite-configured `cargo test`, soilDB integration suite, `cargo fmt --check`, `cargo clippy --all-targets`, and `git diff --check`. Clippy passes with one pre-existing `needless_question_mark` warning in `src/executor/sqlite.rs:138`.
- [x] Commit fixes in focused Conventional Commits: `7484f8e`, `8f12bd2`, `60d9b0c`, `7edf016`.
- [x] Draft a response for each inline finding, referencing the local commit that addresses it, in `planning/PR_1_REVIEW_RESPONSES.md`; no replies were posted.
- [ ] Commit this plan progress separately after response drafts are prepared.

## Notes
- Repository: `/home/andrew/workspace/sda-rs`; branch: `feature/sda-readiness`.
- Planning artifacts are tracked; keep planning commits separate from implementation.
- PR #1 is open against `main`; prepare drafts only, do not post GitHub replies.
- Preserve the soilDB checkout and Iowa GeoPackage as read-only.
- No push, pull, fetch, or other remote writes.

## Discovered Tasks
- [x] `cargo test` without `SDA_TEST_SPATIALITE` also failed in `aoi_contract.rs`, `mukey_contract.rs`, and `sda_get_contract.rs`; all configured spatial contracts now skip cleanly when unset.

## Implementation Commit Map
- `7484f8e fix(sqlite): enforce deadlines and allow temp indexes`
- `8f12bd2 fix(spatial): align CRS and harden SQL rewrites`
- `60d9b0c test(soildb): verify local endpoint integration`
- `7edf016 docs(spatial): document supported SDA_Get functions`
