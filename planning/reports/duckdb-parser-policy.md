# DuckDB parser authorization policy proposal

**Plan:** 15, task 15.2.1  
**Date:** 2026-11-23  
**Status:** Policy design only; no authorization gate is implemented or approved by this report.

## Decision summary

Use the pinned `polyglot-sql` DuckDB dialect only as a candidate syntax classifier. The proposed policy is default-deny: allow a narrow query language plus only the tested request-local temporary-table lifecycle; deny unknown functions, external-I/O sources, unreviewed AST forms, and all other statement classes. Do not authorize based on generated SQL. Parse failure, multiple statements where one is expected, or inability to prove complete classification means reject before execution.

**Implementation is blocked pending an AST-coverage decision.** The crate's public `ExpressionWalk::dfs` offers nodes, but the traversal implementation calls a generated child visitor. Its implementation and field traversal are private to the crate. The report's sampled probes do not prove that every syntax-bearing child of every DuckDB-reachable AST node is exposed, nor that every expression variant is handled by an authorization visitor. Before implementation, either establish a maintainable completeness mechanism (e.g. exhaustive matching plus a verified public child traversal contract) or request approval for a scope/dependency change. Do not claim parser-backed authorization completeness from DFS alone.

## Evidence and constraints

- `planning/reports/duckdb-parser-feasibility.md` reports successful DuckDB-dialect parsing for sampled SELECT, nested SELECT, CTE, join, functions, temp-table workflow, sensitive statements, and compound SQL. It explicitly says exhaustive AST child coverage and stable source-to-AST correspondence were not established.
- The existing `polyglot-sql` 0.14.1 `Expression` enum contains many statement, expression, and dialect-specific variants (including `Raw`, DDL, `Pragma`, `Set`, `Attach`, `Install`, table/query nodes, and many function nodes). It is not sufficient to allow a broad category solely by root variant.
- `ExpressionWalk` delegates to crate-private `ast_children` traversal. Its module documents generated traversal, but is not a public policy hook. The present evidence does not demonstrate a public way to enumerate every field or prove that newly added variants cannot be silently skipped.
- `src/executor/duckdb.rs` executes the supplied SQL string after validation; it wraps a vector of statements in one transaction and separately tracks temporary table names. The current validator accepts a limited temp create/insert/drop pattern and SELECT, but does not parse SQL.
- `tests/duckdb_security_contract.rs` confirms the temp create/insert/select/drop workflow and rejection of persistent writes, file/network and extension access, settings, compound strings, comments, and selected malformed forms. These tests establish current compatibility, not a complete safe language.
- G39 direct probes show DuckDB v2.0.0 can accept settings mutation, in-memory ATTACH, persistent in-memory DDL, and compound batches directly. Configuration restrictions are defense-in-depth, not a substitute for authorization.

## Proposed authorization matrix

| Construct | Policy | Rationale / condition |
|---|---|---|
| One top-level `SELECT` | Allow conditionally | Only after complete recursive classification of its query, expressions, relations, and functions; execute the same input that was classified. |
| Nested SELECT, CTE, joins, set operations | Allow conditionally | Must be explicitly supported and recursively checked at every depth. If any nested node is unclassified, reject. |
| Literal, identifier, column, alias, arithmetic, comparisons, boolean predicates, casts, ordering/grouping/limit | Allow conditionally | Only explicitly enumerated expression/clause variants and their complete children; unsupported syntax remains denied. |
| Scalar functions | Allow by exact reviewed name | Initial candidate list: `abs`; expand only by explicit review against the pinned engine and AST representation. Unknown, dynamically named, or unresolved function forms deny. This conservative seed is intentionally not a claim that other built-ins are unsafe. |
| Table functions | Deny by default | Do not admit table functions in the initial policy. `range`/`generate_series` may be proposed for later explicit review; external-capable functions (including `read_csv` and `read_parquet`) remain denied. |
| Ordinary table references | Allow conditionally | Only plain identifiers resolving within the request's in-memory database or tracked temporary tables; deny qualified/external/catalog sources unless separately reviewed. Current evidence does not establish robust resolution semantics, so relation allow remains contingent. |
| `CREATE TEMP[ORARY] TABLE name AS SELECT ...` | Allow narrowly | Only a fresh, unqualified, validated identifier; SELECT body recursively authorized; register the created name in request-local state. No other create options or table definitions. |
| `INSERT INTO tracked_temp ...` | Allow narrowly | Target must be a tracked temp table. Source/values expressions must be classified; only the tested VALUES form is currently compatibility-evidenced. |
| `SELECT` from tracked temp table | Allow conditionally | Same SELECT rules; relation must be in tracked request-local set or an explicitly trusted built-in source. |
| `DROP TABLE tracked_temp` | Allow narrowly | Only tracked temp target; remove it from state after successful operation. No cascade, schema-qualified name, or other drop object. |
| Other DDL/DML, including persistent CREATE/ALTER/DROP, UPDATE, DELETE, MERGE, schema/database/object creation | Deny | Not part of the supported request workflow; persistent writes are explicitly rejected by contract. |
| Multiple top-level parsed expressions in a per-string validator input | Deny | Parser returns multiple expressions for compound SQL; enforce exactly one. The executor's vector-of-statements batch remains a separate boundary, each item independently classified and batch rollback retained. |
| `SET`, `RESET`, `PRAGMA`, transaction control, `ATTACH`/`DETACH`, `IMPORT`, `COPY`, `INSTALL`, `LOAD`, secrets, commands | Deny | Mutates engine/configuration, accesses external capabilities, or is outside supported query semantics. |
| File/network/external source or sink, including table functions and path-bearing commands | Deny | Engine flags alone do not establish authorization or containment; external-I/O functions are explicitly denied. |
| Comments, quoted identifiers/strings, dialect extensions | Deny initially unless exact parser/source handling is qualified | Existing validator rejects these forms. Parser generation normalizes comments/quoting, so generated output is not a source-preserving authorization substitute. Relax only after round-trip/correspondence qualification. |
| `Raw`, unsupported dialect-specific, unknown/future AST variant, or unknown function | Deny | Fail closed; parser/version changes must not silently widen accepted SQL. |
| Parse error, incomplete parse, ambiguous node-to-source correspondence, visitor failure | Deny | No fallback to string heuristics, generated SQL, or execution. |

The scalar-function seed is intentionally narrow and will reduce compatibility relative to arbitrary DuckDB SQL. Functions already appearing in runtime contracts (such as `range`, spatial functions, and other built-ins) are not implicitly approved; each needs a documented security/compatibility review and explicit category-specific decision. Spatial extension calls require separate trusted extension loading and are outside this initial function allowance.

## Batch and source correspondence rules

The externally supplied batch is a vector of SQL strings, not permission to submit compound SQL in one string. Classify each string separately and require exactly one parsed top-level expression. Preserve transaction-wide rollback on any classification or execution failure. Temporary-table state is shared only across statements in that batch and must not leak to another request/connection.

Classification must apply to the exact SQL bytes later prepared/executed, with no unexamined rewrite between policy check and execution. Parsing with DuckDB dialect and then executing the original input is the candidate model; generating SQL and authorizing the generated output is not acceptable. Since comments and quoting normalize in generation and source correspondence is not fully established, inputs whose mapping cannot be proven unambiguous must be denied. Any preprocessing that removes comments, splits strings, or rewrites syntax would itself need a separately reviewed correspondence guarantee.

## Compatibility, versioning, and blockers

The policy preserves only the tested temp-table lifecycle and a constrained SELECT subset. It will reject presently parser-accepted constructs unless reviewed, including comments, quoted forms, compound SQL, arbitrary functions/table functions, and unclassified DuckDB syntax. This is a deliberate compatibility boundary; expansion requires policy update and regression evidence.

Treat changes to either `polyglot-sql` or DuckDB pins as authorization-policy changes. Re-review the full `Expression` variant set, child traversal semantics, dialect parsing, generated/source correspondence, function and table-function inventory, and representative accepted/rejected runtime contracts. A newly introduced AST variant or function is denied until explicitly reviewed. Record policy revision alongside dependency versions in implementation and qualification records.

### Exact unresolved decision

The sampled feasibility report only exercised `ExpressionWalk::dfs`, while `ast_children` and its generated field-walker are crate-private. The source shows traversal delegation but does not establish that a consumer can exhaustively validate all syntax children or detect a future variant not traversed by that walker. Therefore, a security implementation must not begin until this is resolved by inspecting/verifying the pinned crate's complete generated behavior and identifying an enforceable upgrade guard, or obtaining approval for a different parser/API/scope. No dependency change or implementation is authorized by this policy task.

## Evidence references

- `planning/reports/duckdb-parser-feasibility.md`
- `planning/reports/duckdb-security-probe-results.md` (G39 evidence)
- `src/executor/duckdb.rs` (`execute_batch`, `validate_statement`)
- `tests/duckdb_security_contract.rs`
- `tests/duckdb_execution_contract.rs`
- Pinned crate source: `polyglot-sql-0.14.1/src/expressions.rs`, `traversal.rs`, and `ast_children.rs` (the latter's traversal is private).
