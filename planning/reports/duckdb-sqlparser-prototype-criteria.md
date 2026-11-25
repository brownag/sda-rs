# `sqlparser` DuckDB prototype criteria

**Plan:** 17, task 17.1.1
**Status:** Finite research corpus and coverage criteria; not an authorization implementation or approval.

## Decision boundary

This corpus freezes the Plan 15 policy as written in `duckdb-parser-policy.md`; it does not broaden the accepted language to match a parser. `sqlparser` with `DuckDbDialect` is a candidate to evaluate, not evidence of DuckDB 2.0.0 grammar parity or complete visitor traversal. Each expected outcome below is a policy decision: **conditional** means allow only if every stated condition is proven; **narrow allow** means only the exact constrained form; **deny** means reject. Parser acceptance alone never changes the expected policy decision.

For every case, record parse outcome, top-level statement count, classified AST nodes/payloads and child coverage, policy decision, and—where runtime compatibility is specified—runtime outcome separately. An unclassified or unvisited syntax-bearing field, visitor failure, unsupported/raw representation, or unreviewed future node fails closed.

## Case corpus

Case IDs cite the corresponding construct row(s) of the Plan 15 policy matrix by row label.

| ID | Input/shape to exercise | Expected policy outcome and criterion |
|---|---|---|
| Q01 | `SELECT 1` | **Conditional allow** (One top-level `SELECT`): only with complete recursive query/expression classification and source-byte identity. |
| Q02 | SELECT with literal, identifier/column, alias, arithmetic, comparison, boolean predicate, cast, ordering, grouping, and limit, tested individually and nested | **Conditional allow** (Literal…ordering/grouping/limit): enumerate each admitted variant and all children; any unsupported form denies. |
| Q03 | Nested SELECT in projection, predicate, and relation/subquery positions | **Conditional allow** (Nested SELECT…): recursively cover every depth and payload; an unclassified descendant denies. |
| Q04 | CTE, including nested CTE query and references | **Conditional allow** (Nested SELECT, CTE…): inspect CTE names, query payloads, and nested children; no root-only approval. |
| Q05 | Set operation with query on both sides and nested operands | **Conditional allow** (…set operations): classify both branches and operation/modifiers recursively. |
| Q06 | Join with both relations, join constraint/condition, and nested relation/query | **Conditional allow** (…joins; relations): recursively classify relation/table-factor and condition children. |
| Q07 | Plain unqualified ordinary table identifier; separately qualified/catalog/external relation and dynamic relation form | Plain identifier **conditional** only if proven to resolve inside request in-memory DB or tracked temp set. Qualified/catalog/external and unresolved/dynamic forms **deny** (Ordinary table references). Resolution semantics are an explicit contingent check; no resolution allowance is inferred from parsing. |
| Q08 | `SELECT abs(-2)` and exact-name variants; unknown, dynamically named, qualified/unresolved function | Exact reviewed scalar `abs`: **allow by exact name**, subject to complete argument and representation checks. Other names/forms **deny** (Scalar functions) unless separately reviewed; `abs` is the only initial seed. |
| Q09 | `range`, `generate_series`, `read_csv`, `read_parquet`, and other table-function/source shapes | **Deny** (Table functions; File/network/external source or sink), including external-capable functions. No table function is admitted by default; `range`/`generate_series` are only candidates for later explicit review. |
| T01 | `CREATE TEMP TABLE fresh_name AS SELECT ...` | **Narrow allow** only for fresh, unqualified, validated identifier, recursively allowed SELECT body, and request/batch-local registration. Other options/definitions deny (CREATE TEMP…). |
| T02 | `INSERT INTO tracked_temp VALUES (...)` using only the tested VALUES form; separately test untracked target, query source, unsupported VALUES shape, and nested expressions | **Narrow allow** only for tracked target and fully classified payload; current runtime compatibility is evidenced only for the tested VALUES form. Other source/forms remain **deny or conditional-unproven**, never presumed compatible (INSERT…). |
| T03 | `SELECT` from tracked temp, and same query from untracked temp | Tracked relation **conditional allow** under all SELECT rules; untracked relation **deny** (SELECT from tracked temp). |
| T04 | `DROP TABLE tracked_temp`; separately cascade, qualified name, untracked name, non-table object | **Narrow allow** only for tracked unqualified table; remove state only after successful operation. All listed alternatives **deny** (DROP TABLE tracked_temp). |
| D01 | Persistent CREATE/ALTER/DROP, UPDATE, DELETE, MERGE, schema/database/object creation | **Deny** (Other DDL/DML), irrespective of parser or engine acceptance. |
| B01 | Empty input, one valid statement, two statements in one SQL string, trailing/embedded extra statement | Exactly one parsed top-level statement is required per SQL string. Zero, multiple, incomplete, or ambiguous parse **deny** (Multiple top-level expressions; Parse error). |
| B02 | Vector batch of separately validated SQL strings, including temp lifecycle then deliberate later failure | Each element is independently subject to B01 and classified as its exact bytes. **Preserve** transaction-wide rollback on any classification/execution failure; temp state is shared only within that batch/request and cannot leak to another request. This batch boundary does not permit compound SQL in one element. |
| S01 | `SET`, `RESET`, `PRAGMA`, transaction control, `ATTACH`, `DETACH`, `IMPORT`, `COPY`, `INSTALL`, `LOAD`, secrets, command statements | **Deny** (configuration/control and external-capability statement row), including harmless-looking/in-memory forms. |
| E01 | File, network, path-bearing source/sink, external table/source or command | **Deny** (File/network/external source or sink); engine flags are not authorization. |
| C01 | Comments, quoted identifiers/strings, and dialect extensions, including parser-normalized generation cases | **Deny initially** unless exact source/parser handling and correspondence are separately qualified. Generated/reformatted SQL is never a substitute for classifying execution bytes (Comments…). |
| U01 | `Raw`, unsupported dialect-specific syntax, unknown/future AST variant, unknown function, unhandled statement/payload/child | **Deny** (Raw/unknown row). Demonstrate fail-closed behavior; no wildcard allow or silent traversal omission. |
| P01 | Parse error, incomplete parse, visitor/classifier failure, ambiguous node-to-source correspondence | **Deny** (Parse error…); no string-heuristic, generated-SQL, or execution fallback. |

## Visitor and payload coverage checklist

For each parseable case above, the prototype must establish and record:

- [ ] Exactly-one top-level statement count for the individual SQL string; no confusion with the separate vector batch.
- [ ] Statement root and every policy-relevant statement payload, including CTAS query, INSERT target/source/VALUES, and DROP target/options.
- [ ] Query and SELECT children at every depth, including CTEs, both set-operation sides, projections, filters, grouping/ordering/limit and nested query-bearing fields.
- [ ] Join operands, constraints, relations/table factors, aliases/qualifiers and any source-bearing or dynamic fields.
- [ ] Expression children, including scalar function names/qualification/arguments and any dynamic or unresolved function representation.
- [ ] Table-function and external-source forms are detected and denied, not overlooked as ordinary relations.
- [ ] Temp-table names are checked against fresh/tracked state; create/register and successful-drop/remove transitions are explicit and batch-local.
- [ ] Raw strings, unsupported syntax, parse/visitor failures, and unhandled/new nodes fail closed. Audit the exact release's public visitor and source; callbacks alone do not prove every security-relevant field is traversed.
- [ ] The classified input and designated execution input are the identical original SQL bytes. Do not execute generated or reformatted SQL; ambiguous correspondence denies.
- [ ] Batch failure retains transaction-wide rollback; no case weakens the executor's per-string boundary or state isolation.

## Runtime-compatibility contingencies

- **Ordinary relations (Q07):** policy allow remains contingent on robust proof that plain identifiers resolve only in the request's in-memory database or tracked temporary tables. Prior parser/runtime samples do not establish those resolution semantics.
- **INSERT VALUES (T02):** only the tested VALUES form is compatibility-evidenced. Other VALUES shapes and query sources require explicit runtime evidence and policy review; they are not implicitly allowed.
- **Parser versus engine:** record syntax parse, policy classification, DuckDB binding/runtime, and execution result as separate facts. A parser pass is not runtime compatibility; engine acceptance is not policy authorization. Existing reports show bounded sampled support and known dialect/runtime gaps, not comprehensive grammar parity.

## Traceability and evidence basis

The case labels in parentheses map one-to-one to all 17 construct rows in `planning/reports/duckdb-parser-policy.md`; B02 additionally maps to its batch/source-correspondence rules. The required criteria from `duckdb-parser-api-alternatives.md` map to B01, Q03–Q09, T01–T04, U01/P01, the visitor checklist, and exact-byte rule. Recursive payload/coverage limits are informed by `duckdb-ast-coverage.md` and `duckdb-ast-guard-feasibility.md`. Bounded parser/runtime examples and compatibility distinctions are informed by `duckdb-parser-feasibility.md` and `duckdb-tsql-pipeline-evaluation.md`.

This report defines acceptance criteria only. It does not establish visitor completeness, authorize production parsing/gating, or authorize dependency addition, parser/API replacement, or any pin change; each requires separate explicit approval.
