# SQLite execution policy (G32)

Each request receives a request-local SQLite connection. Trusted spatial
initialization, including configured extension loading, runs before the SQLite
authorizer is installed. Submitted statements are then prepared and executed
with the authorizer active. SQL submitted by a caller cannot load extensions.

## Authorizer rules

The authorizer allows `SELECT` and `READ` against persistent and request-local
data. It allows creating and dropping temporary tables and INSERT, UPDATE,
DELETE, and READ operations in SQLite's `temp` database for `#temp` workflows.
It allows SQLite transaction actions used by the executor-owned batch
transaction, and SQL functions other than `load_extension`. Function matching
for `load_extension` is case-insensitive.

All other authorizer actions are denied, including persistent writes and schema
changes, PRAGMA, ATTACH/DETACH, savepoints, virtual tables, and unknown actions.
The transpiler's security validation remains in place; in particular, user
transaction-control SQL must still be rejected before execution. The
authorizer is defense in depth, not a replacement for transpiler validation.

## Trusted extensions and future UDFs

Only the configured trusted spatial extension may be loaded during connection
initialization, before authorizer installation. Query-controlled extension
loading is denied. Spatial support remains optional when no extension is
configured.

Future Rust-defined SQL functions must use `SQLITE_UTF8` and only justified
flags. Use `SQLITE_DIRECTONLY` unless invocation from schema objects is
explicitly required. Use `DETERMINISTIC` only when results are pure and stable.
Use `INNOCUOUS` only after security review. No production Rust UDF is introduced
by this policy.

## Agreed resource caps

The execution safeguards plan sets these caps:

| Resource | Cap |
| --- | ---: |
| SQL text | 2 MiB |
| Rows | 100,000 |
| Serialized output | 64 MiB |
| Execution time | 60 seconds |
| SQLite string/BLOB value | 64 MiB |

These are agreed thresholds for the G32 work; enforcement is handled by the
corresponding resource-limit tasks.
