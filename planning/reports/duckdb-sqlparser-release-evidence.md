# `sqlparser` release evidence

**Plan:** 17, task 17.2.1
**Status:** Blocked — no exact release source was available in the local Cargo cache.
**Date:** 2026-11-24

## Decision

No `sqlparser` release was selected or inspected, and no consumer was compiled. The task requires inspection of an exact release's actual source/API and forbids network access. Since the local cache contains no release source/archive, this task stops here for orchestrator direction; latest-version documentation is not substituted for release evidence.

## Offline evidence

Read-only checks from `/home/andrew/workspace/sda-rs`:

```sh
find "$HOME/.cargo/registry/src" -maxdepth 3 -type d -name 'sqlparser-*' -print
find "$HOME/.cargo/registry/cache" -maxdepth 2 -type f -name 'sqlparser-*.crate' -print
```

Both commands returned no paths. `Cargo.lock` contains no `sqlparser` entry. `Cargo.toml` was inspected read-only; it has no `sqlparser` dependency, and no repository Cargo file was changed. Prior evidence in `duckdb-parser-api-alternatives.md` also records that `sqlparser` was not present in the local Cargo source cache at that time, while its 0.64.0 docs were only a shortlist—not exact-release source/API verification.

Because no exact source exists locally, the following required claims remain **unverified**: `DuckDbDialect` availability/feature gates; visitor API and traversal behavior; statement representation and count behavior; relevant AST nodes and payload fields; manifest/features/checksum provenance; and limitations around raw or future syntax. No assumptions are treated as guarantees.

## Commands and scope

- Cache inspection: commands above; complete with no matching source directory or crate archive.
- Lockfile inspection: `rg -n 'sqlparser' Cargo.lock`; no matches.
- Repository dependency context: read `Cargo.toml`; no changes.
- No network access, Cargo resolution/build, probe, or scratch harness was run. `/tmp/opencode/sda-rs-plan17-release/` was not needed or created.
- Repository dependency files, source, tests, policy, routes, ADRs, criteria report, and other reports remain untouched.

To continue, an exact release source must first become available in the approved offline cache, or the orchestrator must provide new direction. Do not fetch from the network or infer release behavior from latest docs.
