# DuckDB parser/API alternatives

**Plan:** 16, research follow-up to the Plan 15 AST-coverage blocker  
**Date:** 2026-11-24  
**Scope:** Research and recommendation only. No dependencies, pins, parser APIs, or code changed.

## Recommendation

For the next decision, prototype **`sqlparser`'s public `DuckDbDialect` plus `ast::Visitor`/`VisitorMut`**, isolated from production and tested against the exact accepted/rejected corpus in `duckdb-parser-policy.md`. It is the strongest documented Rust candidate found that combines a DuckDB dialect with a public recursive visitor. This is a candidate to qualify, not evidence that its grammar matches DuckDB 2.0.0 or that visitor traversal proves every security-relevant field is covered.

If that prototype fails required syntax or cannot establish the policy's recursive coverage guarantee, stop and evaluate a maintained `polyglot-sql` fork/upstream visitor API or a reviewed version-gated traversal contract. Do not substitute PostgreSQL's parser as the DuckDB gate.

## Required criteria derived from Plan 15

The policy is intentionally narrower than DuckDB SQL. A viable parser/API must support:

- Parsing with DuckDB dialect intent, and distinguishing exactly one statement from zero or multiple statements.
- Recursive classification of SELECT, nested SELECT, CTEs, set operations, joins, expressions, relations, scalar calls and table-function/source forms.
- Narrow explicit recognition of `CREATE TEMP TABLE ... AS SELECT`, `INSERT` into a tracked temp table, and `DROP` of one, while denying other DDL/DML.
- Fail-closed handling of unknown syntax/nodes, parse failures, unsupported/raw forms, and unreviewed functions and sources.
- Reliable inspection of policy-relevant children and payloads, with a defensible upgrade/review guard. A public visitor is useful but is not automatically exhaustive; visitor coverage must be tested against the pinned AST/source and semantics.
- Classification of the exact SQL bytes later executed. Do not authorize generated/reformatted SQL as a substitute.

Plan 15 permits only the constrained policy in `planning/reports/duckdb-parser-policy.md`: exact reviewed scalar-function names; table functions denied initially; ordinary relations only under strict resolution rules; external IO, configuration/settings, extension control, transaction control, and all other statement classes denied. Comments, quoting, and dialect-specific syntax remain denied unless correspondence is qualified. The vector-of-statements executor batch remains separate, transaction-wide rollback is preserved, and temp state is request/batch-local.

The production executor currently runs a string-based validator then calls DuckDB `execute_batch`; it tracks temp-table names and wraps a vector batch in a transaction (`src/executor/duckdb.rs:596-674`). Separately, the application SQL path parses T-SQL through `polyglot-sql`, rewrites ASTs for SQLite, then executes SQLite. That path is not a drop-in DuckDB authorization parser: generated SQL may change syntax/semantics, including temp-table syntax. DuckDB must parse the original execution input with DuckDB dialect intent.

## Candidate comparison

| Candidate | Public recursive API / coverage guard | DuckDB and required syntax fit | Maintenance, license, cost | Assessment |
|---|---|---|---|---|
| **`sqlparser` (sqlparser-rs), DuckDbDialect + Visitor** | Documented public `Visitor` has pre/post hooks for statements, queries, SELECTs, expressions, relations, table factors and other nodes; AST types are inspectable. Visitor docs promise callbacks before/after children. This is stronger than opaque private child enumeration, but does not itself promise all security-relevant fields are visited or that new fields trigger downstream compile failures. Add a versioned corpus and source review; test unknown/new AST variants and child-bearing fields. | Current docs expose a `DuckDbDialect`; crate docs describe a multi-dialect SQL parser. Still generic parser grammar, not DuckDB's own parser. Exact v2.0.0 coverage for temp DDL, settings, external functions, DuckDB extensions/lambdas and other syntax is unverified. Could reject unsupported forms by default. | Apache-2.0; mature, documented public API and active project/docs (latest docs observed 0.64.0, dated 2026-11-09). Integration is one parser dependency plus optional visitor feature, but adds a grammar/version differential corpus. | **Recommended prototype.** Best balance of visitor API and DuckDB syntax intent. Approval needed before dependency addition or parser/API/pin change. |
| **`polyglot-sql` fork/upstream public visitor** | The current 0.14.1 AST has public fields and exhaustive `Expression` enum matching, but its public `ExpressionWalk` delegates to private `ast_children`; no external guarantee that all syntax children are visited. A fork could expose a public visitor/path API and field-coverage tests; would need maintained and reviewed upstream changes. | Already demonstrated parsing sampled DuckDB and T-SQL forms, including the policy's temp lifecycle and sensitive statements. Existing evidence is bounded; generator normalizes comments/quoting and must not mediate authorization. | Current pin and source are already local; exact license should be confirmed from the crate manifest before any fork. Fork/upstream coordination and derive behavior increase ongoing maintenance cost. | Technically closest syntax baseline; strongest fallback if `sqlparser` grammar mismatch is material. Not fixed merely by writing a downstream visitor over the current walker. |
| **`pg_query` / libpg_query** | Public protobuf node types and `NodeRef` access are documented; exposed helpers include tables/functions and parse trees, but research did not establish a documented exhaustive recursive visitor with an upgrade guard. Would likely require walking generated protobuf oneofs/fields and reviewing generated schema updates. | PostgreSQL grammar, not DuckDB. DuckDB's 2.0 parser is documented as PEG-based and DuckSQL, with PostgreSQL influence but its own dialect. Must expect compatibility gaps, especially DuckDB-specific forms and temp lifecycle. | MIT wrapper plus PostgreSQL license for parser source; builds native PostgreSQL parser source and uses protobuf/build tooling. Considerably higher integration/build and upgrade-review cost. | Not recommended: wrong grammar target and no demonstrated traversal advantage sufficient to offset that mismatch. |

### Evidence quality and limits

**Documented public guarantees observed**

- `sqlparser` 0.64.0 docs.rs documents `Visitor` as a public AST walker, says pre-visit callbacks run before children and post-visit callbacks after children, and provides callbacks for query, select, relation, table factor, expression, statement, value, identifier, order-by and group-by nodes. `DuckDbDialect` is public and explicitly documented as a dialect for DuckDB. The crate page reports Apache-2.0 and latest version/date.
- `pg_query` 18.1.0 docs.rs says it uses PostgreSQL server source, returns the internal PostgreSQL parse tree, and statically links parser source. Docs expose protobuf `Node`/`NodeRef`; license page identifies MIT wrapper and PostgreSQL-licensed server source.
- DuckDB's 2026 parser announcement describes DuckDB 2.0's parser as PEG-based, its own DuckSQL dialect, and PostgreSQL-influenced rather than identical PostgreSQL.

These are API/documentation guarantees only at the documented versions. None promises an authorization-grade completeness proof across all AST fields or parser upgrades.

**Assumptions / unverified**

- No candidate crate was installed or built; `sqlparser` and `pg_query` were not present in the local Cargo source cache. Latest docs are evidence to shortlist, not confirmation of the exact version Cargo would resolve or APIs at a chosen pin.
- No differential corpus was run against the repository's DuckDB 2.0.0 binary. DuckDbDialect name does not prove full DuckDB 2.0 grammar parity.
- A visitor's documented traversal callbacks do not establish that every field with security meaning is represented by a callback, nor that future AST additions cannot be silently ignored by a visitor implementation. The prototype must explicitly audit this boundary and pin/review source changes.
- Licenses and maintenance observations should be rechecked against the exact candidate release and repository before approval.

## Concrete follow-up and approval gate

Authorize a **scratch-only compatibility/visitor feasibility task**, not an implementation: evaluate a specific `sqlparser` release in an isolated temporary harness, without changing repository Cargo files. Feed it the Plan 15 positive/negative corpus plus representative supported T-SQL-translated DuckDB input and temp lifecycle. For every accepted AST, record visitor coverage of nested query/expression/relation/function and all policy-relevant statement payloads; assert exactly-one-statement behavior; verify the original bytes remain the execution input. Compare parser outcomes to the pinned DuckDB v2.0.0 runtime using existing local/synthetic fixtures only. Fail the candidate if any dangerous syntax is opaque/omitted or required syntax is unsupported; do not broaden policy to fit the parser.

After that evidence, ask for explicit approval before any of: adding `sqlparser`, replacing/changing the parser API, changing dependency or engine pins, or implementing/activating the authorization gate. The follow-up should specify the exact crate/version and accepted residual risks; if visitor completeness remains an assumption, the gate remains blocked.

## Sources

Repository evidence:

- `planning/reports/duckdb-parser-policy.md` — policy matrix and source-correspondence constraints.
- `planning/reports/duckdb-parser-feasibility.md` — bounded DuckDB-dialect probes; normalized generation and parser/runtime limits.
- `planning/reports/duckdb-ast-coverage.md` and `planning/reports/duckdb-ast-guard-feasibility.md` — current public/private visitor boundary and failed recursive-coverage guard.
- `planning/reports/duckdb-tsql-pipeline-evaluation.md` — current T-SQL-to-DuckDB translation compatibility observations.
- `src/executor/duckdb.rs:596-674` — current batch, validation, transaction, and execution path.

External documentation accessed 2026-11-24:

- `https://docs.rs/sqlparser/0.64.0/sqlparser/ast/trait.Visitor.html` — public visitor hooks and callback ordering.
- `https://docs.rs/sqlparser/0.64.0/sqlparser/dialect/struct.DuckDbDialect.html` — DuckDB dialect type and feature surface.
- `https://docs.rs/crate/sqlparser/0.64.0` — release metadata, Apache-2.0 license and project link.
- `https://docs.rs/pg_query/18.1.0/pg_query/` — PostgreSQL parser basis, AST API and license description.
- `https://duckdb.org/2026/08/20/duckdb-20-peg-parser` — DuckDB 2.0 parser design and DuckSQL dialect description.
