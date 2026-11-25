# `sqlparser` release evidence

**Plan:** 17, task 17.2.1
**Status:** Exact 0.64.0 source inspected, versioned docs retrieved, and scratch `cargo check` passed. Running the probe was blocked by `/tmp` exhaustion.
**Date:** 2026-11-24

## Decision

Candidate `sqlparser` 0.64.0 is verified as a real, buildable exact-version candidate for the public DuckDB dialect and visitor API. This does **not** establish complete policy-relevant traversal, DuckDB 2.0.0 grammar compatibility, or suitability as an authorization gate. Versioned docs were retrieved from docs.rs after the crates.io metadata endpoint returned HTTP 403. The compiled probe could not be run because `/tmp` was full; syntax and recursive traversal remain for the next task to exercise.

## Provenance and source evidence

- Exact crate archive: `https://crates.io/api/v1/crates/sqlparser/0.64.0/download`
- Exact-version visitor docs: `https://docs.rs/sqlparser/0.64.0/sqlparser/ast/trait.Visitor.html`
- Exact-version DuckDB dialect docs: `https://docs.rs/sqlparser/0.64.0/sqlparser/dialect/struct.DuckDbDialect.html`
- SHA-256 of downloaded `sqlparser-0.64.0.crate`: `92e0d273b4f8ddd14a35ed36862acbc9cc96cec578e36ee1881a9d42e0e0f2c8`
- Extracted source and normalized manifest: `/tmp/opencode/sda-rs-plan17-release/src/sqlparser-0.64.0/`
- Manifest identifies package `sqlparser` version `0.64.0`, license `Apache-2.0`, upstream repository `https://github.com/apache/datafusion-sqlparser-rs`, and optional `visitor = ["sqlparser_derive"]` feature. The default features include `std` and `recursive-protection`.
- `src/dialect/mod.rs` publicly re-exports `DuckDbDialect`; `src/dialect/duckdb.rs` defines the public dialect. This proves type availability in the release, not parity with DuckDB's own parser/runtime.
- `src/ast/visitor.rs` defines public `Visitor` and `VisitorMut` traits, pre/post callbacks, and generated `Visit` traversal implementations. The callbacks' ordering docs say pre callbacks precede children and post callbacks follow them. Source-level visitor implementations are the actual traversal contract, but do not prove every policy-relevant field is traversed or that downstream code fails closed on future AST changes.
- `src/parser/mod.rs` exposes `Parser::parse_sql` returning `Result<Vec<Statement>, ParserError>`; callers can enforce exactly one statement by checking vector length. Empty/multiple results are not made safe automatically.
- `src/ast/mod.rs` defines a broad `Statement` enum with dedicated variants including `Query`, `Insert`, `Install`, `Load`, and `Copy`, whose `Copy` payload includes source, target, options, and optional inline payload. Specific fields still require case-by-case audit. “Raw” is not established as a universal parse fallback by this inspection; unsupported SQL may error or map to a normal AST node.
- Crate root documentation says AST stringification normalizes whitespace, capitalization, and removes comments. Therefore generated `Display` output cannot substitute for the original execution bytes.

These are source observations, not a security completeness guarantee. Visitor callbacks alone do not certify traversal of every field, payload, future node, raw representation, or source-sensitive construct.

## Scratch consumer and results

All scratch files and Cargo state were under `/tmp/opencode/sda-rs-plan17-release/`, including manifest, `src/main.rs`, `cargo-home/`, and `target/`. The exact consumer depends on `sqlparser = { version = "=0.64.0", features = ["visitor"] }`; it exercises DuckDB-dialect parsing, one-statement count, statement visitor callback, relation walker, and retains the original SQL string as execution input.

Commands/results:

1. `CARGO_HOME=... CARGO_TARGET_DIR=... cargo check --manifest-path /tmp/opencode/sda-rs-plan17-release/Cargo.toml` — first attempt failed because the probe omitted the `Visit` trait import; compiler gave the direct import correction.
2. Same command after adding `Visit` — **passed** (exact 0.64.0 and visitor feature compiled).
3. `CARGO_HOME=... CARGO_TARGET_DIR=... cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan17-release/Cargo.toml` — **blocked** while producing linked artifacts: rustc reported `No space left on device`; `/tmp` was 100% full (55G used, 0 available). No repeat or cleanup of unrelated `/tmp` contents was attempted. The probe did not run.

Crates.io archive retrieval succeeded. The metadata request `https://crates.io/api/v1/crates/sqlparser/0.64.0` returned HTTP 403; exact-version visitor and dialect pages were then retrieved from docs.rs. No latest-version documentation was substituted. Cargo's isolated check retrieved exact version 0.64.0 and its dependencies from crates.io and produced a successful compile check.

## Guarantees, assumptions, and limitations

- **Observed/tested:** exact crate archive and checksum; normalized package manifest; source declaration/re-export of `DuckDbDialect`; public visitor API and generated traversal machinery; `parse_sql` returns a vector; isolated exact-version consumer passes `cargo check`.
- **Documented guarantee:** exact-version docs.rs `Visitor` docs state pre-visit callbacks occur before children and post-visit callbacks after children; `DuckDbDialect` docs describe it as a dialect for DuckDB. Neither guarantees full field coverage or parity with the DuckDB engine.
- **Unverified:** running the probe; parse behavior across the Plan 17 corpus; per-node and payload recursion completeness; comment/source-span correspondence; future-variant safeguards; DuckDB 2.0.0 grammar/runtime compatibility; policy classification; runtime binding and execution.
- This result does not authorize production use, dependency addition, parser/API replacement, or pin changes. Each requires separate approval. Preserve Plan 15 policy, source-byte identity, statement boundary, batch rollback, DuckDB internal-only boundary, and SQLite default.

## Reproduction and scope

The scratch `Cargo.toml` and `src/main.rs` remain in `/tmp/opencode/sda-rs-plan17-release/`. Reproduce the successful check with:

```sh
CARGO_HOME=/tmp/opencode/sda-rs-plan17-release/cargo-home \
CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan17-release/target \
cargo check --offline --manifest-path /tmp/opencode/sda-rs-plan17-release/Cargo.toml
```

No repository Cargo files, implementation, tests, policy, routes, ADRs, or criteria report were changed. No live SDA/API/service, user/external data, SQL with external I/O, or remote Git was accessed.

## Versioned documentation observations

- `Visitor` is a public trait with callbacks for statements, queries, selects, relations, table factors, expressions, values, identifiers, order-by and group-by nodes. Its docs describe callback ordering; they do not guarantee every security-relevant field is traversed.
- `DuckDbDialect` is a public struct documented as “A Dialect for DuckDB.” This indicates dialect intent only, not DuckDB 2.0.0 parser parity.
