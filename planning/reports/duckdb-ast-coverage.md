# `polyglot-sql` 0.14.1 AST coverage and upgrade guard

**Plan:** 15, task 15.3.1  
**Date:** 2026-11-23  
**Scope:** Inspection of the locally cached, lockfile-pinned crate source and public API. No implementation, tests, dependencies, or policy were changed. This is bounded API evidence, not a security-completeness claim.

## Conclusion

The public API supports an exhaustive downstream match over the `Expression` enum in pinned `polyglot-sql` 0.14.1. The enum is public and has no `#[non_exhaustive]` attribute. A consumer match with no wildcard/default arm is checked by Rust: adding an enum variant makes that consumer fail to compile until the new variant is handled. Variants can therefore be explicitly allowed or denied at the top level and when encountered by a public walker.

That does **not** establish exhaustive authorization-relevant AST classification. Public recursive traversal (`ExpressionWalk::dfs`, `bfs`, and `children`) delegates to `crate::ast_children`, which is a private module. Its functions and `AstNode` trait are `pub(crate)`. A consumer cannot independently inspect or assert the crate's field-to-child mapping. The pinned derive currently generates visitors over declared fields, including `Expression` values nested in supported containers and syntax payload types, and skips fields marked `#[ast(skip)]`; that behavior is implementation evidence from the pinned dependency, not a public traversal contract that a downstream exhaustive enum match verifies. A future field, derive behavior change, or traversal omission can escape the downstream enum-match guard.

An exact dependency pin plus lockfile/locked builds can make dependency changes visible and require review. An exhaustive consumer match is a useful compile-time guard for `Expression` variant additions only. Neither existing mechanism proves complete recursive child visitation or safely guards a future traversal implementation. On this evidence, the public API alone does not resolve Plan 15's blocker for a security authorization gate. Do not treat this report as authorization to implement one or as evidence of security completeness.

## Pinned source and reproducibility

Repository dependency declaration is `polyglot-sql = "0.14"` in `Cargo.toml`; `Cargo.lock` resolves `polyglot-sql` **0.14.1**, checksum `8ea83ace7b8d9fc85b6a1d0bd129250cd29461dd61991791c44a4ea124f29904`. The derive crate is also pinned to 0.14.1, checksum `6d21a35c30dc6637c3fd839bd92e60d7973a65966b3718cc5afa5390853066e3`.

In this workspace, the source directory is:

```text
/home/andrew/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/polyglot-sql-0.14.1/
/home/andrew/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/polyglot-sql-ast-derive-0.14.1/
```

Offline commands to reproduce the version/source lookup and dependency resolution:

```sh
rg -n -A8 '^name = "polyglot-sql"$' Cargo.lock
rg -n -A8 '^name = "polyglot-sql-ast-derive"$' Cargo.lock
find "$HOME/.cargo/registry/src" \
  -path '*/polyglot-sql-0.14.1/src/*' -type f -print | sort
cargo tree --offline -i polyglot-sql
cargo tree --offline -i polyglot-sql-ast-derive
```

The `cargo tree` commands showed the workspace depends on the 0.14.1 parser and its 0.14.1 derive crate. Inspection commands used for the API and traversal:

```sh
rg -n 'pub enum Expression|non_exhaustive' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-0.14.1/src/expressions.rs
rg -n 'ast_children|for_each_child|pub trait ExpressionWalk|fn dfs|fn children' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-0.14.1/src/{lib.rs,traversal.rs,ast_children.rs}
rg -n 'fn visit_fields|fn visit_variant_fields|fn immutable_visit|fn is_skipped' \
  "$HOME"/.cargo/registry/src/*/polyglot-sql-ast-derive-0.14.1/src/lib.rs
```

No compile probe or test harness was added or run. A downstream guard's compile check can be reproduced with a literal, no-wildcard match over `Expression`; `cargo check --locked` (or the project's chosen locked build/check command) then fails if the locked enum gains a variant not covered. That check alone does not exercise the private traversal implementation.

## Public enum and node access

- `polyglot-sql-0.14.1/src/lib.rs:15,29,70-72,96` declares `ast_children` private, exposes `expressions`, and exports `Expression`. `traversal` is public only with the relevant crate features; otherwise it is private.
- `polyglot-sql-0.14.1/src/expressions.rs:95-99` derives `AstNode` and declares `pub enum Expression`. No `#[non_exhaustive]` is present on this enum (the source's other `non_exhaustive` occurrences are unrelated).
- `expressions.rs:115-145`, `205-216`, and `582-659` show query/DML/function and authorization-sensitive variants. Examples include `Select`, set-operation and subquery nodes; `Function`, aggregate and window functions; `Insert`, `Update`, `Delete`, `Copy`; DDL; `Pragma`, `Attach`, `Detach`, `Install`, `Load`, `Set`, and `Raw`. The full enum is very large and continues past line 1000; a concise category list is not a substitute for enumerating every variant in any future policy implementation.
- Representative public payloads expose fields: `Select` at `expressions.rs:3940-4030` has expressions, sources, joins, filters, clauses, `settings`, `format`, `into`, and other optional syntax fields; `TableRef` at `3569-3630` has name, schema/catalog, aliases, sample/hints, dynamic identifier, time-travel and other dialect fields; `Function` at `5461-5497` has qualified and textual names, arguments, error behavior, and flags; `Insert` at `4665-4709` has target, values/query, returning/output/conflict and other fields. Struct fields are public, but there is no public generic reflection API over those fields.
- `Raw` at `expressions.rs:7082-7087` contains an unparsed SQL string. Sensitive statement structs also have nested expression fields (for example `Attach` and `Set` at `11336-11412`).
- Public fields let handwritten code inspect fields on known payload structs. But exhaustive enum matching alone checks the outer variant set; it does not force a consumer to inspect every field of every payload. A handwritten payload pattern without `..` can also be made sensitive to field additions, at substantial maintenance cost. Neither technique checks that a recursive walker visits those fields.

## Child traversal and visibility

- `lib.rs:15` makes `ast_children` private. In `ast_children.rs:20-40`, `ChildPathSegment` and `AstNode` are `pub(crate)`. The child functions at lines 43-72 are likewise `pub(crate)`, including `for_each_child` and `for_each_child_untracked`.
- Public `ExpressionWalk` declares `dfs`, `bfs`, and `children` in `traversal.rs:260-300`. Their implementation delegates to private functions: `DfsIter::next` at `traversal.rs:209-220` calls `for_each_child_untracked`; BFS at `243-251` does likewise; `children` at `362-365` calls `for_each_child`. The public results are flattened expression references (and, for `children`, no field path), not a caller-supplied visitor over every field.
- The private derive implementation is present separately at `polyglot-sql-ast-derive-0.14.1/src/lib.rs`. `node_visitor` generates a match over every declared enum variant (`114-127`). Field handling (`142-201`, `203-285`) skips `#[ast(skip)]` fields and descends into the remaining fields. `immutable_visit` handles `Expression`, `Option`, `Box`, `Vec`, tuples, scalar leaves and nested derived `AstNode` payloads (`288-353`). This is useful evidence that the pinned build intends typed syntax-child traversal. The generated body is not exposed as API to downstream code, and the derive crate's behavior is separately versioned.
- Source comments in `ast_children.rs:1-6` describe this as canonical generated traversal and say expression-containing payload fields are traversed automatically, with `#[ast(skip)]` reserved for derived metadata. That is documentation about the private implementation, not a downstream-verifiable completeness guarantee.
- The walker deliberately follows expression/syntax types, not arbitrary scalar or string content. Thus a `Raw.sql` string is not parsed into children. Comments, source spans, metadata and other non-`Expression` fields are outside the expression-child iterator. Whether each such field matters to a particular authorization rule must be decided from node semantics; DFS alone cannot answer that.
- The derive tests at `ast_children.rs:128-189` compare immutable/mutable traversal for a sample and assert example child paths. They are crate-internal tests, not an external guarantee and do not prove every field/variant or future changes are covered.

## Exhaustive-match and upgrade-guard assessment

1. **Variant-set guard: feasible.** With the current public, non-`non_exhaustive` enum, write a match with an explicit arm for every variant and no `_` arm. Rust exhaustiveness checking requires a newly added variant to be handled on recompilation. Exact pinned version source and `cargo check --locked` make this a practical compile-time change detector for the enum API. The review must still decide the meaning of each arm; “handled” does not mean “safe.”
2. **Payload-field guard: possible but costly and incomplete by itself.** Explicitly destructuring public fields without `..` can make field additions fail compilation at selected payloads. It must be done for every relevant payload type and does not inspect opaque nested traversal behavior. The public API has no stable generic mechanism to assert recursive field coverage.
3. **Traversal guard: not established by downstream compilation.** DFS/children compile against a public API but call crate-private generated traversal. A dependency update that retains those signatures can change traversal behavior without causing a consumer compile error. An exhaustive match over yielded nodes only guards variants that are yielded; it cannot detect a child that the walker omits.
4. **Version/review guard: practical but procedural.** `Cargo.lock` fixes the resolved crate and derive versions for locked builds, and `cargo tree --locked`/`cargo check --locked` can verify resolution/build consistency. The manifest's `"0.14"` requirement is a compatible-version range, not an exact pin; refreshing the lock may select a newer compatible release. A strict manifest pin would constrain that version choice but would not prove traversal coverage. Any parser or derive version change requires an explicit review of source and traversal semantics.
5. **Stronger evidence needed for implementation.** Options would require a separately approved/maintained guard or scope decision, such as controlling dependency upgrades and re-reviewing the complete derive-generated traversal, or using an API that exposes a verifiable visitor. No such code, dependency, or policy change is made or recommended as already approved here.

## Bounded result

The enum offers a real compile-time exhaustiveness hook for top-level variant classification, contrary to a claim that no exhaustive matching is possible. But that hook does not close the recursive child-coverage gap. Because traversal field enumeration is private, and a consumer compile check cannot detect omissions in it, the public API and current repository lock do not provide a fully enforceable AST-coverage/upgrade guard on their own. This resolves the narrow API question while leaving Plan 15's implementation blocker unresolved unless its owner accepts and specifies a separate traversal/version-review guard. No security completeness or parser-backed authorization claim follows.
