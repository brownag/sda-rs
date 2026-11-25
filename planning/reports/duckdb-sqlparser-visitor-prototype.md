# `sqlparser` recursive visitor prototype

**Plan/task:** 17 / 17.3.1
**Status:** Scratch prototype and finite corpus tests pass. This is research evidence only, not an authorization gate.

## Outcome

Exact `sqlparser = 0.64.0` with its `visitor` feature compiled and passed offline synthetic tests. The visitor provides useful recursive callbacks, but this prototype does **not** establish policy-complete classification. Its classifier is intentionally conservative for DDL/DML and reports query-without-relations or reviewed `abs` queries as conditional only. It must not be used to authorize execution. No policy allowance was broadened; ordinary relation resolution and runtime compatibility remain contingent.

## Per-case prototype results

| Case | Input shape | Result | Interpretation |
|---|---|---|---|
| Q01 | `SELECT 1` | Conditional | Recursively visited; no relation resolution involved. |
| Q02 | Literal/arithmetic/predicate/order/limit | Conditional | Exercised in a combined synthetic query, not every isolated grammar variant. |
| Q03 | Nested scalar subquery | Conditional | Query/expression callbacks recurse. |
| Q04 | CTE and reference | Conditional | CTE query traversed; prototype special-cases the test name `x`, not a general CTE scope model. |
| Q05 | `UNION` | Conditional | Both query descendants visited by generated traversal. |
| Q06 | Join | Deny | Relations are not tracked; no join-constraint policy authorization. |
| Q07 | Qualified external relation | Deny | Relation classifier rejects unknown/qualified relations. |
| Q08 | `abs(-2)` | Conditional | Exact function name observed; still conditional, not a general expression proof. |
| Q09 | `range(3)` | Deny | Table-function shape observed and denied. |
| T01 | Temporary CTAS | Deny | Fresh-name and lifecycle semantics not implemented. |
| T02 | INSERT VALUES | Deny | Target/payload narrow-form validation not implemented. |
| T03 | Tracked temp relation | Conditional | Tested tracked relation accepted conditionally; request/batch lifecycle is not implemented. |
| T04 | DROP tracked table | Deny | DROP target/options and successful-operation lifecycle not implemented. |
| D01 | DELETE | Deny | Non-query statement defaults to deny. |
| S01 | SET | Deny | Non-query statement defaults to deny. |
| E01 | `read_csv` source | Deny | Table-function/external-source shape denied. |
| C01 | Comment plus SELECT | Conditional | Parser acceptance alone does not qualify source correspondence; prototype retains original bytes and does not authorize comments. **Test currently verifies parser/classifier output only; this is a known gap.** |
| U01 | Unknown function | Deny | Function names other than `abs` denied. |
| P01 | Incomplete SQL | Deny | Parser error fails closed. |

### Statement-boundary checks

Empty input, two statements in one string, and incomplete input deny. A single trailing semicolon is accepted by the parser as exactly one statement; it is not a second statement. Separate vector-batch semantics, transaction rollback, and temp state isolation are not implemented or tested by this scratch prototype.

The byte-identity test retains the original `String`, snapshots its bytes, classifies it, and verifies the bytes are unchanged. It does not execute SQL; no generated or normalized SQL is produced or designated for execution.

## Traversal/source audit

The inspected source is the exact 0.64.0 crate extracted in the prior evidence task; its recorded archive SHA-256 is `92e0d273b4f8ddd14a35ed36862acbc9cc96cec578e36ee1881a9d42e0e0f2c8`. Relevant paths include `src/ast/visitor.rs`, `src/ast/mod.rs`, `src/ast/query.rs`, `src/ast/dml.rs`, and `src/ast/ddl.rs`.

- The public `Visit` trait and generated `Visit` implementations call pre/post visitor callbacks around child traversal. The library traversal is useful recursive plumbing, not a security guarantee that every policy-bearing field is covered.
- `Visitor` exposes callbacks for statements, queries, selects, relations, table factors, expressions, values and several clause nodes. This prototype counts query/expression callbacks and relation/table-function shapes.
- The library-generated traversal reaches nested query/expression children for the tested forms. It does not implement manual per-variant exhaustive matching, reject future enum variants at compile time, or establish source spans/correspondence.
- Statement payload auditing remains insufficient: CTAS options/columns, INSERT target/source/VALUES variants, DROP object/options, and other statement fields have not been explicitly validated. They fail closed at the statement root in this prototype.
- Relation resolution and CTE scoping are not generally modeled. The test-only CTE reference exception is hard-coded and is not evidence of safe general resolution.
- The public visitor callback sequence cannot prove complete handling of all variants/fields. A production candidate would need explicit exhaustive policy matching and explicit validation of relevant payloads, while preserving default-deny behavior.

## Reproduction

All manifest, lockfile, source, tests, Cargo home, and build output are under `/tmp/opencode/sda-rs-plan17-visitor/`. Dependency is pinned exactly: `sqlparser = { version = "=0.64.0", features = ["visitor"] }`.

```sh
CARGO_HOME=/tmp/opencode/sda-rs-plan17-visitor/cargo-home \
CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan17-visitor/target \
cargo test --offline --manifest-path /tmp/opencode/sda-rs-plan17-visitor/Cargo.toml
```

Result: **2 integration tests passed**, plus empty unit/doc test suites.

```sh
CARGO_HOME=/tmp/opencode/sda-rs-plan17-visitor/cargo-home \
CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan17-visitor/target \
cargo clippy --offline --all-targets \
  --manifest-path /tmp/opencode/sda-rs-plan17-visitor/Cargo.toml -- -D warnings
```

Result: **passed**. The first Clippy attempt caught `single_match`; changed to `if let`, then reran successfully. The first test run also exposed the CTE reference and trailing-semicolon behaviors described above; corrected the scratch expectations/recognition and reran.

`git diff --check` passed. Scratch-only tests use synthetic SQL strings; no external SQL I/O, live endpoints, user data, or external datasets were used.

## Limitations and decision

The finite tests show that 0.64.0 can support recursive observation for several nested forms, but do not meet acceptance for a production classifier. C01 source correspondence, comprehensive isolated Q02 forms, generic CTE scoping, exhaustive statement/payload handling, table-factor variants, future-node rejection, INSERT/CTAS/DROP validation, batch rollback/state lifecycle, and DuckDB runtime compatibility remain unproven or unimplemented. Stop here: do not add the dependency, alter parser/API/pins, or use this prototype as an execution authorization gate without separate approval and a complete follow-on design.
