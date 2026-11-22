# DuckDB G39 direct security probe results

**Task:** Plan 14, task 14.2.1
**Date:** 2026-11-22
**Pin:** Rust `duckdb` 1.20000.2 (`Cargo.lock`); bundled engine v2.0.0 per `duckdb-pin-and-upgrade-assessment.md`.
**Setup:** In-memory direct `Connection`, `enable_autoload_extension(false)`, `enable_external_access(false)`. Paired executor results use public `sda_rs::executor::duckdb::execute`, which includes its validation/transpilation path; these are classifier/executor-path outcomes, not isolated calls to private `validate_statement`.

## Reproduction

Disposable source and build outputs are under `/tmp/opencode/sda-rs-plan14-security/`; no repository source/manifests/tests were changed. Commands were run offline, with no endpoint contacted:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/opencode/sda-rs-plan14-security/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-security/Cargo.toml -- security-matrix
CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract
```

The first command compiled the scratch runner and its locked local dependencies successfully, then ran the matrix. It printed crate `1.20000.2`, engine identity v2.0.0 (from repository pin assessment), and both configuration flags. The second command ran 4 tests: **4 passed, 0 failed**, in 0.25s. No dependencies were downloaded. The remote URL input used `example.invalid` and did not contact it.

## Paired observations

| Probe | Executor/classifier path | Direct configured connection | Attribution and limits |
|---|---|---|---|
| `SELECT * FROM read_csv('/etc/passwd')` | Rejected: `Query is not read-only` | Permission Error: file system operations are disabled by configuration | Classifier/executor rejects; direct outcome demonstrates config-level local-file denial for this function/path. |
| `SELECT * FROM read_parquet('https://example.invalid/data.parquet')` | Rejected: `Query is not read-only` | Catalog Error: `read_parquet` absent; it exists in parquet extension | Remote access was **not reached or tested** because extension was unavailable. No endpoint contacted. |
| `CREATE SECRET plan14_probe (TYPE memory, PROVIDER config)` | Rejected: `Query is not read-only` | Permission Error accessing `/home/andrew/.duckdb/stored_secrets`; filesystem operations disabled | Direct attempt failed before secret creation due configured filesystem denial. Does not establish universal `CREATE SECRET` denial or resolve the 2.0.0/2.0.1 release-note issue under other storage paths. |
| `SET enable_external_access = true` | Rejected: `Query is not read-only` | Invalid Input Error: cannot enable external access while database is running | Classifier rejects; engine also refused changing this setting in this active connection. |
| `SET autoload_known_extensions = true` | Rejected: `Query is not read-only` | `OK` | Demonstrates setting is mutable directly, despite initial autoload-disabled config. Does not show a successful extension load or file/network access. |
| `ATTACH ':memory:' AS probe` | Rejected: `Query is not read-only` | `OK` | Classifier-only restriction for this harmless in-memory attach. |
| `IMPORT DATABASE '/tmp/opencode/sda-rs-plan14-security/no-such-db'` | Rejected: `Query is not read-only` | Permission Error accessing scratch `schema.sql`; filesystem operations disabled | Config blocked file access before import. No file was imported. |
| `COPY (SELECT 1) TO '/tmp/opencode/sda-rs-plan14-security/copy.csv'` | Rejected: `Query is not read-only` | Permission Error: filesystem operations disabled | Config blocked persistent file output. No output file created. |
| `CREATE TABLE durable (id INTEGER)` | Rejected: `Query is not read-only` | `OK` | Direct connection permits persistent-schema DDL in its in-memory database; classifier prevents this executor-path statement. |
| `INSTALL spatial` | Rejected: `Query is not read-only` | Permission Error accessing extension directory under `/home/andrew/.duckdb/extensions/v2.0.0/linux_amd64` | Config blocked extension installation at filesystem access; no download attempted. |
| `LOAD spatial` | Rejected: `Query is not read-only` | Permission Error: loading external extensions disabled through configuration | Engine/config denial observed. |
| `SELECT 1; CREATE TABLE durable (id INTEGER)` | Rejected: `Query is not read-only` | `OK` | Direct `execute_batch` accepts both statements. This direct probe used a disposable in-memory connection. |
| `SELECT 1 -- comment` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts the harmless commented statement. |
| `SELECT 1 AS "quoted"` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts quoted identifier syntax. |
| `SELECT 'safe'` | Rejected: `Query is not read-only` | `OK` | Classifier/executor-path restriction; direct engine accepts quoted string syntax. |
| Temp workflow: create temp table, insert, select, drop | Batch accepted; 4 result sets | All four statements returned `OK` on one direct connection | Supported compatibility path confirmed when statements share a connection/batch. Individual statement probes used fresh connections and therefore could not observe prior temp-table state; their isolated “table missing” outcomes are not workflow failures. |

## Evidence interpretation

The direct observations distinguish selective engine/config controls from executor-path restrictions. Filesystem operations were denied for local read, import, copy, secret-storage, and extension installation attempts; external extension loading was denied. In contrast, direct `SET autoload_known_extensions`, in-memory `ATTACH`, persistent in-memory DDL, compound statements, comments, and quoting succeeded. The executor rejected these non-SELECT operations/syntax forms. This is evidence that classifier enforcement is material and that the configured engine is not a complete substitute for it.

The remote URL case is inconclusive: `read_parquet` was unavailable because the parquet extension was not installed, so `enable_external_access=false` was not isolated for remote access. No attempt was made to install/load it or contact an endpoint. The direct `CREATE SECRET` attempt failed on the configured secrets-directory filesystem access; this bounded result does not establish the full applicability or exploitability of the documented v2.0.1 fix. Likewise, the single successful autoload-setting change did not test downstream extension autoload or external I/O.

These finite probes do not prove exhaustive SQL/API security, general filesystem containment, or request-path safety. DuckDB remains qualification-only and is not exposed through HTTP; no production-readiness conclusion follows.

## Regression and diff checks

`CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_security_contract`: **4 passed, 0 failed**.
`git diff --check`: passed with no output.

## Task 14.3.1 — Version-matched security semantics and options

Sources below are DuckDB 2.0 documentation for the observed v2.0.0 engine, plus the official 2.0.1 release announcement dated 2026-11-20. The 2.0.1 announcement describes fixes, not proof that a particular exploit is reachable through this Rust client or configuration.

### Reconciliation with official semantics

DuckDB's [2.0 securing guide](https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/overview.html) explicitly warns that SQL runs with the process user's privileges and compares untrusted SQL to untrusted shell/Python code. It calls capability settings defense-in-depth, not a substitute for sandboxing. It documents `enable_external_access=false` as preventing external-source reads and file `ATTACH`/`COPY`; the observed local `read_csv`, `COPY`, `IMPORT`, and extension filesystem errors are consistent with those claims. The missing `read_parquet` extension means the URL probe says nothing about remote denial. The 2.0 docs also say allow-listed paths/directories remain accessible even with external access disabled and may be written; this is not a read-only filesystem boundary. Do not rely on `allowed_paths` as containment.

The guide documents `autoload_known_extensions=false` and `autoinstall_known_extensions=false`, but the direct probe changed the former back to true successfully. Thus the startup/config state in the executor is not immutable against arbitrary direct SQL on this observed engine. It does not show that a later extension actually loaded or accessed files/network. Official [extension security docs](https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/securing_extensions.html) explain extension code runs with parent-process privileges; signatures establish publisher category, not harmlessness. Autoload/autoinstall controls, signed-extension policy, and banning custom repositories reduce extension routes but do not neutralize already loaded extension code.

The 2.0.1 [official release announcement](https://duckdb.org/2026/11/20/announcing-duckdb-201.html#security-fixes) specifically says 2.0.0 allowed `CREATE SECRET` despite `enable_external_access=false`, and 2.0.1 refuses it; it separately fixes remote path traversal under that configuration. The direct 2.0.0 `CREATE SECRET` probe failed at access to `~/.duckdb/stored_secrets` because filesystem access was disabled. This is consistent with the documented gap: this particular attempt was blocked by its storage path, not by a universal `CREATE SECRET` prohibition. No remote path traversal was probed. Treat both fixes as applicable risks to the 2.0.0 qualification baseline, but exploitability under this application’s narrow executor is unestablished. The 2.0.1 announcement also reports stricter secret scope matching; this task's probes do not assess secret scope.

The direct successes for in-memory `ATTACH`, persistent in-memory DDL, multi-statement batches, comments, and quoted syntax show why the current SQL classifier is a separate control. They do not contradict external-access documentation, which denies file attachment and external IO, not in-memory operations or arbitrary SQL grammar. The official guide's prepared-statement advice applies when the application controls query structure and binds untrusted values; it explicitly does not make user-supplied SQL safe. No DuckDB SQL authorizer API or equivalent complete policy hook is established by these sources. The current string classifier is neither a parser nor an authorizer, as documented in `docs/security/duckdb-execution-policy.md`.

### Defense-in-depth options and risk matrix

| Option / status | Threat addressed and preconditions | Does not mitigate / evidence and approval |
|---|---|---|
| Strict query allowlist / parser-backed AST gate — future design | Reject statement classes, functions, relations, and syntax outside an intentionally small query grammar before execution. Relevant because direct connection accepts DDL, compound SQL, comments, quoting, and mutable settings while executor rejects selected forms. | Parser coverage, DuckDB grammar/version compatibility, function/table-function allowlisting, and bypass resistance are unproven. A string allowlist can be bypass-prone; finite tests do not prove completeness. Selecting or implementing a parser/authorizer and broadening accepted SQL require separate approval. |
| Connection configuration — partially observed | Disable external access and extension autoload/autoinstall before queries; use restrictive extension/repository settings; lock configuration after trusted initialization where compatible. Helps constrain file/URL operations, extension acquisition, and setting changes. | Does not make arbitrary SQL safe; current direct probe re-enabled autoload. `CREATE SECRET` and remote-path issues in 2.0.0 show version-specific gaps. `allowed_paths` are readable/writable exceptions, not a sandbox. Exact configuration order and lock behavior need pinned-runtime validation; any policy/config change requires approval. |
| Least-privilege filesystem/process identity — not assessed by probe | Run under a dedicated non-root identity with only required read access to input and deployment-provisioned extension artifacts, no credentials/secrets, and no writable sensitive locations. Limits impact if an engine/extension operation escapes DB-level controls. | Does not prevent CPU/memory exhaustion, all data disclosure within granted access, or network access absent OS/network controls. Actual deployment permissions and path needs are not assessed; deployment changes require approval. |
| Connection/process boundary — future option, no necessity verdict | A fresh connection per request limits database state sharing (current ADR contract); separate process/container/VM or OS sandbox can reduce blast radius if hostile SQL or native extension code executes. Official DuckDB guidance recommends additional sandboxing for untrusted SQL and lists isolated container, VM/process, OS sandbox and network isolation. | Fresh connection alone shares process privileges and resource envelope. Process isolation alone does not bound resource use unless constrained; boundary design and operational cost are not qualified. The source guidance does not prove a particular sandbox is necessary or sufficient here. Choose only after threat model and explicit approval. |
| Operational monitoring and resource controls — future work | Observe query failures, denied capabilities, resource consumption, extension identity, and suspicious behavior; set application deadlines and calibrated memory/thread/temp-storage/concurrency/output bounds. Addresses detection and denial-of-service risk, especially with request-controlled query input. | Monitoring detects rather than prevents; logs cannot replace authorization/isolation and may expose sensitive SQL. No DuckDB production limits or monitoring instrumentation are evidenced here; thresholds require benchmark evidence and approval. |
| Version update — separate qualification decision | Evaluate 2.0.1 to obtain documented `CREATE SECRET` and remote path traversal fixes (and secret-scope correction). | Does not fix arbitrary untrusted SQL or prove client/extension compatibility. No upgrade or alternate-version probe is authorized here; exact Rust client/engine/Spatial tuple remains unqualified. |

### Current guarantees and decision boundary

Evidence supports only selected configuration-level denials on the pinned 2.0.0 connection and selected classifier rejections on the executor path. It does not establish comprehensive file/network denial, complete SQL authorization, resilience to malicious extensions, production filesystem isolation, or request-path safety. Remote URL access remains untested; secret creation was blocked by storage access in this case. Continue to treat the executor as qualification-only. The matrix describes options, not approved implementation or a conclusion for/against process sandboxing. SQLite remains default under accepted ADR 0007; no request selector or silent fallback is introduced.

**Official sources (accessed 2026-11-22):**
- DuckDB 2.0, “Securing DuckDB”: <https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/overview.html>
- DuckDB 2.0, “Securing Extensions”: <https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/securing_extensions.html>
- DuckDB team, “Announcing DuckDB 2.0.1 LTS,” security fixes: <https://duckdb.org/2026/11/20/announcing-duckdb-201.html#security-fixes>
