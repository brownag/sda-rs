# DuckDB parser feasibility: existing `polyglot-sql`

**Task:** Plan 15, task 15.1.1  
**Date:** 2026-11-23  
**Scope:** Offline, synthetic syntax and pinned local dependency/runtime observations. This is feasibility evidence, not an authorization policy or security-completeness claim.

## Summary

The existing dependency is more promising than the current T-SQL usage alone suggests: locked `polyglot-sql` 0.14.1 provides a DuckDB dialect, parses the tested query and temporary-table forms, exposes statement ASTs and traversal, and can generate SQL back in the DuckDB dialect. It also parses several sensitive constructs (`read_csv`, `SET`, `ATTACH`, `DROP`, compound statements) into ASTs rather than silently treating them as opaque text.

This establishes **bounded feasibility for a follow-up policy design**, not sufficiency for an authorization gate. The probes did not establish exhaustive AST child coverage or stable, semantically complete source-to-AST mapping; in particular, the API accepts multiple statements, generator output normalizes comments and quoting, and some dialect/runtime differences remain. Task 15.2 should inspect exact AST variant and child fields and specify explicit default-deny classification before implementation. Do not treat parse success, AST traversal, or DuckDB runtime capability settings as authorization.

## Reproduction

All scratch artifacts and build output were kept in `/tmp/opencode/sda-rs-plan15-parser/`. No network endpoints or user data were used. The parser probe used the exact locked package version in an isolated manifest:

```toml
[dependencies]
polyglot-sql = "=0.14.1"
```

```sh
CARGO_NET_OFFLINE=true \
  CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan15-parser/target \
  cargo run --offline \
  --manifest-path /tmp/opencode/sda-rs-plan15-parser/Cargo.toml
```

The probe ran each SQL example through `Dialect::get(DialectType::DuckDB)` and `DialectType::TSQL`, recorded parse success/error and statement count, walked expressions using `ExpressionWalk::dfs`, and generated each parsed expression in its source dialect. The initial probe completed successfully.

Pinned DuckDB is crate `1.20000.2`, bundled engine v2.0.0. A disposable runner using the same locked crate and flags was attempted under the approved scratch location, but the `/tmp` tmpfs filled during DuckDB's bundled native archive build (`No space left on device`). No source files were changed by that attempt. Runtime comparison was therefore performed with the already-built repository target and the existing pinned contracts:

```sh
CARGO_NET_OFFLINE=true cargo test --offline --features duckdb \
  --test duckdb_security_contract -- --nocapture
```

Result: **5 passed, 0 failed**. The already-built binary confirmed the repository's pinned optional DuckDB feature target was available. In addition, the existing report `planning/reports/duckdb-security-probe-results.md` records earlier direct-engine v2.0.0 observations for comments, quoting, compound SQL, in-memory attach, settings, and temp-table batches. The fresh direct-runtime matrix below is limited to the subset that could be exercised without another engine build.

## Parser and runtime matrix

`AST parsed` means dialect parsing succeeded and yielded the indicated number of top-level expressions. “Generated” is dialect generator output, not proof that generation preserves source text or authorization semantics. For parser-only cases, runtime was not independently executed in this task.

| Construct / synthetic example | DuckDB dialect parse and generation | T-SQL parse and generation | Pinned DuckDB v2.0.0 runtime evidence |
|---|---|---|---|
| Simple SELECT: `SELECT 1` | Parsed, 1 statement; generated `SELECT 1` | Parsed, 1; same | Direct runner: accepted. |
| Scalar function: `SELECT abs(-2)` | Parsed, 1; generated `SELECT ABS(-2)` | Parsed, 1; generated `SELECT ABS(-2)` | Direct runner: accepted. |
| Table function: `SELECT * FROM range(3)` | Parsed, 1; generated `SELECT * FROM RANGE(3)` | Parsed, 1; same | Direct runner: accepted. |
| Nested SELECT | Parsed, 1; generated; traversal visited nested SELECT and `MAX` expression | Parsed, 1; generated | Direct runner: accepted for a nested query over `range(3)`. |
| CTE: `WITH x AS (...) SELECT ...` | Parsed, 1; generated preserving CTE form | Parsed, 1; generated | Direct runner: accepted. |
| Join | Parsed, 1; generated with normalized spacing | Parsed, 1; generated | Parser example parsed; initial direct test had a binder error due to invalid synthetic `VALUES` column names, so runtime syntax evidence is unknown for that exact query. |
| Line comment: `SELECT 1 -- hi` | Parsed, 1; generator rewrote comment to `/* hi */` | Parsed, 1; same | Direct runner: accepted. Earlier G39 direct probe also accepted. |
| Quoted identifier: `SELECT 1 AS "quoted"` | Parsed, 1; generated with `"quoted"` | Parsed, 1; normalized to `[quoted]` | Direct runner: accepted. |
| String literal: `SELECT 'safe'` | Parsed, 1; generated unchanged | Parsed, 1; generated unchanged | Direct runner: accepted. |
| Multiple statements: `SELECT 1; SELECT 2` | Parsed as 2 expressions; generated separately | Parsed as 2; generated separately | Direct runner: accepted by `execute_batch`; earlier G39 probe confirms. Executor validator rejects compound SQL per its single-statement contract. |
| Temp create-as-select | `CREATE TEMP TABLE x AS SELECT 1`: parsed, 1; generated `CREATE TEMPORARY TABLE x AS SELECT 1` | Parsed, 1; generated as `SELECT * INTO #x ...` (not DuckDB-equivalent syntax) | Existing security contract passes temp create/insert/select/drop lifecycle in one executor batch; direct earlier probe accepted lifecycle. |
| Temp insert/select/drop | Each of `INSERT INTO x VALUES (2)`, `SELECT * FROM x`, `DROP TABLE x` parsed as 1 and generated | Each parsed as 1 and generated | Existing tests confirm lifecycle; this task's direct runner verified the whole sequence was accepted on one configured connection. |
| Scalar/list function: `SELECT unnest([1,2])` | Parsed, 1; generated `UNNEST([1, 2])` | Parsed, 1; generated `UNNEST([1,2])` | Direct runner: accepted. |
| External-I/O table function: `read_csv` | Parsed, 1; generated `READ_CSV('/etc/passwd')` | Parsed, 1; generated similarly | Direct runner reached DuckDB and was denied filesystem access by `enable_external_access=false`; it did not read a file. Existing contract rejects it before runtime on executor path. |
| `generate_series` table function | Parsed, 1; generated uppercase function form | Parsed, 1; generated uppercase form | Direct runner: accepted. |
| `SET autoload_known_extensions=true` | Parser probe confirms DuckDB/T-SQL dialects accept this statement; detailed AST capture omitted from compact output | Parsed in same probe | Existing G39 report observed direct v2.0.0 connection accepts setting mutation; executor path rejects it. |
| `ATTACH ':memory:' AS a` | Parser probe confirms DuckDB/T-SQL dialect parsing succeeds | Parsed | Existing G39 report observed direct v2.0.0 accepts harmless in-memory attach; executor path rejects it. |
| Unsupported/malformed SQL | Not exhaustively probed | Not exhaustively probed | No completeness inference. |

The parser probe's compact summary also recorded AST node counts for several examples (DuckDB dialect: simple SELECT 2 nodes; scalar function 4; `range` table function 4; nested SELECT 7; CTE 6; join 7; comment SELECT 3; quoted SELECT 3; literal SELECT 2; two statements 4; temp create 3; temp insert 2; temp select 3; temp drop 1; `read_csv` 4; `unnest` 5; `generate_series` 5). These counts are only a reproducibility aid; counts do not describe safe semantics. `ExpressionWalk::dfs` compiled and traversed tested nodes. Exact node field/member policies remain work for Task 15.2.

## Findings and limits

1. **Use the DuckDB dialect for feasibility/policy work.** `DialectType::DuckDB` exists in 0.14.1 and handled all listed examples. The production transpiler currently uses T-SQL because it targets SQLite; its output can change source semantics (notably temp-table create syntax and identifier quoting). A DuckDB authorization gate must not rely on generated SQLite SQL or assume T-SQL behavior is DuckDB behavior.
2. **Parsing alone does not authorize.** Both dialects parse dangerous table functions and state-changing statements. The AST must be explicitly classified, including table sources, nested expressions, and function calls; parse success is not a safety decision.
3. **Multiple statements are represented, not automatically rejected.** `parse` returns a vector of top-level `Expression`s. The caller must enforce exactly one allowed statement per executor input (while separately retaining the explicitly supported batch API semantics).
4. **Generation is not source preservation.** Comments may be rewritten, whitespace normalized, and quoting changed across dialects. If authorization is based on the AST, execute the original only after its single-statement parse and classification, and qualify parser/source correspondence; do not authorize one representation then execute a materially different unexamined string.
5. **AST support is plausible but not yet proven complete enough.** DFS visits the tested nested nodes, but the probe did not enumerate every `Expression` variant, prove every syntactic child is included in traversal, or test unknown/future variants. A safe implementation needs an explicit statement/expression/relation/function allowlist and fail-closed handling for every unreviewed variant.
6. **DuckDB and parser acceptance are not interchangeable.** The parser accepted generic and DuckDB-specific examples, but this finite matrix is not a grammar differential corpus. Some runtime examples can fail at binding/runtime even when parsed; conversely, parse acceptance does not establish executor support.
7. **Runtime evidence is bounded.** Direct engine probes used synthetic in-memory data and configured external access off. The `read_csv` case proves configured filesystem denial for this path only, not general containment. No URL access, extension installation/loading, real files, or user data were attempted. The failed scratch rebuild is a resource limitation, not a parser/runtime negative result.

## Recommendation

Proceed to Task 15.2 as a **conditional reuse candidate**: define policy against `DialectType::DuckDB` ASTs from the existing pinned crate, not the project's T-SQL-to-SQLite pipeline. Before implementation, inspect and map exact AST variants and child traversal for accepted SELECT, nested queries, relations, functions, and temp lifecycle. Require exactly one parsed statement per validated string, reject unknown/unreviewed nodes and functions, and keep external-I/O table functions denied absent specific review. Confirm the source SQL being executed is the same input classified, and preserve only the separately tested batch lifecycle at the executor layer.

This evidence does **not** yet justify implementing a security gate: if Task 15.2 cannot unambiguously distinguish all relevant DuckDB syntax or ensure nested AST coverage, stop and present the exact gap for a separate parser/dependency or scope decision. No new dependency is recommended from this feasibility probe alone. No claim of exhaustive SQL authorization, production readiness, or request safety follows.

## Exact probe outcomes

- Parser probe: `CARGO_NET_OFFLINE=true ... cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan15-parser/Cargo.toml` — passed for the parser-only version, including both DuckDB and T-SQL cases.
- Fresh DuckDB scratch build: failed during bundled archive assembly because `/tmp` tmpfs had only 366 MiB free; no network was used.
- Existing focused contract: `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract -- --nocapture` — **5 passed, 0 failed**.
- No repository source, tests, manifests, lockfiles, policies, or other reports were modified.
