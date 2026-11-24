# DuckDB AST guard feasibility

**Plan:** 16, task 16.1.1
**Date:** 2026-11-24
**Scope:** Feasibility assessment of a downstream guard against locked `polyglot-sql` 0.14.1. No source, tests, dependency declarations, lockfiles, or policy were changed.

## Recommendation

**Stop before implementation.** A downstream exhaustive `Expression` match is viable for detecting added enum variants, but it cannot verify that recursive traversal covers every syntax-bearing child. The crate's public walker delegates child discovery to private generated code; downstream compilation cannot prove its field traversal, nested payload/container coverage, or detect a silent omission. This leaves the authorization completeness condition unmet.

Do not authorize Plan 16.2.1 on the existing public API evidence alone. The report does establish one useful guard (variant-set exhaustiveness), but not the recursive coverage mechanism needed by the approved policy.

## Coverage assessment

The existing policy in `duckdb-parser-policy.md` requires default-deny classification of every nested query, expression, relation/source, and function form, plus narrow temp-table statements. It explicitly rejects `Raw`, unknown/unreviewed forms, unsafe sources/functions, parse ambiguity, and parse errors.

| Coverage surface | Public API evidence | Guard status |
|---|---|---|
| Root `Expression` variants | Public, non-`#[non_exhaustive]` enum; explicit no-wildcard match is Rust-exhaustive | Feasible for added variants; each arm still needs reviewed policy meaning |
| Children yielded by `ExpressionWalk` | Public `dfs`/`children` call `crate::ast_children` internals | Can inspect what is yielded, but cannot establish no child is omitted |
| Payload fields and nested containers | Public payload structs expose fields, but no generic reflection/visitor over all fields; derive visitor handles nested `Expression`, `Option`, `Box`, `Vec`, tuples, and derived payloads internally | Not comprehensively guarded downstream; explicit destructuring can guard selected field additions only |
| Raw SQL/string fields and skipped metadata | Child walker traverses typed syntax children, not arbitrary strings; `Raw` carries unparsed SQL and derive can skip fields | Requires semantic decisions outside DFS; cannot assume traversal sees content |
| Dependency/derive upgrades | Exact lockfile records versions; locked checks catch resolution changes, not behavior changes under same API | Procedural review only; does not prove visitor completeness |

The pinned derive source currently generates traversal over declared non-skipped fields and supported containers. This is useful implementation evidence, not a public downstream contract: `ast_children`, `AstNode`, and child functions are private/`pub(crate)`.

## Reproducible evidence

Source inspection anchors:

```sh
rg -n 'pub enum Expression|non_exhaustive' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-0.14.1/src/expressions.rs
rg -n 'for_each_child|pub trait ExpressionWalk|fn dfs|fn children' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-0.14.1/src/{ast_children.rs,traversal.rs}
rg -n 'fn visit_fields|fn visit_variant_fields|fn immutable_visit|fn is_skipped' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-ast-derive-0.14.1/src/lib.rs
```

An isolated scratch consumer under `/tmp/opencode/sda-rs-plan16-ast/` used `polyglot-sql = "=0.14.1"` and:

```rust
fn classify(expression: &Expression) -> bool {
    match expression {
        Expression::Vertica(_) => false,
        Expression::BracedWildcard(_) => false,
    }
}
```

Reproduce with:

```sh
CARGO_NET_OFFLINE=true \
  CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan16-ast/target \
  cargo check --offline \
  --manifest-path /tmp/opencode/sda-rs-plan16-ast/Cargo.toml
```

The check failed as intended with E0004, reporting `Literal`, `Boolean`, `Null` and 938 more uncovered variants. This confirms the compile-time enum exhaustiveness hook exists. It does **not** establish a usable complete guard until every variant has an explicit classification arm, and even a complete match would only classify yielded nodes, not prove the private walker yielded all relevant children. Scratch-only compilation also compiled a 0.14.2 derive crate through the parser's non-exact transitive requirement; this does not change repository dependencies and underscores that lock/review boundaries must include the derive crate.

Prior bounded source evidence in `duckdb-ast-coverage.md` documents the public/private boundary, derive behavior, and version/upgrade limits. No authorization completeness claim follows from either report.

## Alternatives for a separate decision

If a demonstrable traversal contract is required, options needing investigation and separate approval include:

1. Use a parser/API that exposes a public, documented visitor over all syntax-bearing fields with an exhaustive upgrade contract.
2. Maintain a fork or upstreamed public visitor API in `polyglot-sql`, including a contract/test suite that checks every payload field and relevant container.
3. Approve a procedural dependency-upgrade boundary that requires full source/derive review and adversarial coverage on every parser/derive update. This is review-based risk control, not an enforceable downstream compile-time completeness guard.

No parser replacement, dependency addition, pin change, or implementation is recommended/authorized here. Orchestrator/user must choose whether one of these residual-risk approaches is acceptable.
