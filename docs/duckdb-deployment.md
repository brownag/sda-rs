# DuckDB Spatial deployment provisioning (Plan 13 G41)

This document defines an offline provisioning procedure for the Linux x86_64
qualification pin. It does not enable Spatial in the current executor, qualify
GeoPackage execution, or establish production readiness.

## Pin

| Component | Selected identity |
| --- | --- |
| Rust client | `duckdb = "=1.20000.2"` with the `bundled` feature |
| Bundled DuckDB engine | `v2.0.0` |
| Official extension artifact | DuckDB Spatial `v2.0.0`, platform directory `linux_amd64`, file `spatial.duckdb_extension` |
| Official source | `https://extensions.duckdb.org/v2.0.0/linux_amd64/spatial.duckdb_extension.gz` |
| Expected installed-file SHA-256 | `8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc` |
| Observed extension metadata | `("spatial", "a3dbcd1781", "")`; extension identifier `a3dbcd1781` was observed by the G33-2 scratch probe before lockdown. This is an observed metadata value, not a substitute for file hash verification. |

The selected pin reuses the exact G33-2 fingerprint recorded in
`planning/reports/G33-2-duckdb-spatial-followup.md`. The deployment check
verifies the installed file's SHA-256; it does not query extension metadata or
independently prove compatibility. Keep the client, bundled engine, artifact
version/platform, and hash together when updating this pin.

## Provision offline

Provision during deployment, before starting the service, from a locally
available artifact whose origin is controlled by the deployment process. The
script performs no network access and accepts no URL:

```sh
scripts/provision-duckdb-spatial.sh \
  /deployment-staging/spatial.duckdb_extension \
  /var/lib/sda-rs/duckdb-extensions/v2.0.0/linux_amd64
```

The first argument is the explicit source file; the second is the
deployment-controlled destination directory. The destination directory must
not be writable by request-serving users. The script requires the supplied file,
copies it to a temporary file in the destination, verifies the temporary
file's pinned SHA-256, and only then atomically publishes
`spatial.duckdb_extension`. Missing input, an invalid destination, or a hash
mismatch is a hard failure. It does not install packages, download, or use a
fallback source. Verify the resulting file during deployment as an independent
check:

```sh
sha256sum /var/lib/sda-rs/duckdb-extensions/v2.0.0/linux_amd64/spatial.duckdb_extension
```

The output must match the pin above. Do not start a spatial-capable deployment
if the artifact is absent or verification fails. A deployment without Spatial
may retain non-spatial DuckDB operation; spatial behavior must fail explicitly,
not fall back to SpatiaLite or another backend.

## Runtime boundary

If a separately approved runtime integration loads this artifact in the future,
it must use the deployment-controlled absolute path after verifying the pinned
hash, before serving work. Disable known-extension autoload/autoinstall and
external access, and expose no request-, SQL-, or user-controlled extension
path. Requests must never install, fetch, update, or select an extension.

This is a deployment contract, not an implementation of that initialization
sequence. The current `src/executor/duckdb.rs` opens an in-memory connection,
disables autoload and external access, and does not load Spatial. Provisioning
alone therefore does not enable spatial queries or GeoPackage access.

G39's SQL classifier remains string-based and file/network/extension denial is
not exhaustive. Those security blockers, along with the unimplemented runtime
extension load and GeoPackage path, remain unresolved. No production-readiness
claim is made.
