# TODO

## Goal
Address valid inline review findings on PR #1 and prepare concise draft replies without posting to GitHub.

## Tasks

### 1. Verify findings and establish response plan
- [x] Verified and classified all 14 actionable inline findings; each is valid against the reviewed PR head. Addressing specifics in tasks below.
- [ ] Commit this tracked plan separately before implementation.

### 2. Correctness, execution, and spatial fixes
- [ ] Fix deadline enforcement across statement batches and rows; replace the misleading deadline test with an authorized CPU workload and assert interruption/success cases.
- [ ] Allow temporary index creation only for temp tables, and preserve denial of persistent index creation.
- [ ] Fix CLI help/version exit behavior and make spatial capability detection ignore SQL literals/comments.
- [ ] Resolve Web-Mercator CRS metadata for read-only GeoPackages without writing to them; return a structured error if unavailable.
- [ ] Correct spatial predicate SRID alignment and NULL handling; add executed non-origin coverage.
- [ ] Correct table-input key comparison to avoid false matches from numeric casts; add regression coverage.
- [ ] Reject unsupported column-alias lists for scalar/table WKT TVFs and add tests.
- [ ] Fix SQL preprocessing scanners to preserve comments and delimited identifiers.
- [ ] Make the configured spatial security integration test skip cleanly when SpatiaLite is not configured.
- [ ] Make the soilDB test prove the package routes to the local endpoint; resolve Cargo's executable path from JSON build output.

### 3. Documentation, validation, and review replies
- [ ] Update SDA_Get support and spatial contract documentation to match implemented support.
- [ ] Run focused tests, configured full Rust tests, `cargo fmt --check`, `cargo clippy --all-targets`, `git diff --check`, and configured soilDB integration tests where prerequisites permit.
- [ ] Draft a response for each inline finding, including any skipped finding and reason; do not post replies or make other remote changes.
- [ ] Commit verified fixes in focused local commits and update this plan in a separate planning-only commit.

## Notes
- Repository: `/home/andrew/workspace/sda-rs`; branch: `feature/sda-readiness`.
- Planning artifacts are tracked; keep planning commits separate from implementation.
- PR #1 is open against `main`; prepare drafts only, do not post GitHub replies.
- Preserve the soilDB checkout and Iowa GeoPackage as read-only.
- No push, pull, fetch, or other remote writes.
