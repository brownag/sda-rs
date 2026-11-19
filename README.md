# sda-rs

A standalone Rust server that emulates the USDA-NRCS Soil Data Access (SDA) `POST /Tabular/post.rest` endpoint. It accepts SDA-style T-SQL queries, transpiles them to SQLite/SpatiaLite, and returns results in SDA's JSON or XML formats.

The backend is an OpenSSURGO GeoPackage or any SQLite database. Spatial functions require the SpatiaLite extension.

## Requirements

- Rust (edition 2021)
- Optional: SpatiaLite extension (for spatial queries)

## Quick start

```bash
make build
make run DB=path/to/openssurgo.gpkg PORT=8080
```

Or without `make`:

```bash
cargo run -- --backend-url path/to/openssurgo.gpkg --port 8080
```

## Configuration

| Option | Environment variable | Default | Purpose |
|---|---|---|---|
| `--backend-url` | `SDA_BACKEND_URL` | `sda.gpkg` | SQLite or GeoPackage database path |
| `--port` | `SDA_PORT` | `8080` | HTTP listen port |
| `--spatial-extension` | `SDA_SPATIAL_EXTENSION` | none | Path to the SpatiaLite library (e.g. `mod_spatialite.so`) |
| `--geometry-column` | `SDA_GEOM_COLUMN` | none | Override the GeoPackage geometry column |

Logging is controlled by `RUST_LOG` and defaults to `info`.

## API

### `POST /Tabular/post.rest`

`/Tab/post.rest` is an alias for the same handler.

Send `query` and an optional `format` as JSON or form-encoded fields.

```bash
curl -X POST http://localhost:8080/Tabular/post.rest \
  -H 'Content-Type: application/json' \
  -d '{"query": "SELECT areaname FROM legend", "format": "JSON+COLUMNNAME"}'
```

| Field | Required | Description |
|---|---|---|
| `query` | yes | T-SQL-style query text. Empty or whitespace-only queries return `400`. |
| `format` | no | Response format. Defaults to `JSON`. Case-insensitive. |

#### Formats

| `format` | Content type |
|---|---|
| `JSON` (default) | `application/json` |
| `JSON+COLUMNNAME` | `application/json` |
| `JSON+COLUMNNAME+METADATA` | `application/json` |
| `XML` | `application/xml` |
| `~` | `text/plain`, echoes the query back without executing it |

Any other value returns `400` with `Unsupported format`.

#### Errors

Errors return a JSON body of the form `{"error": "<message>"}`.

| Status | Cause |
|---|---|
| `400` | Missing or empty `query`, invalid body, unsupported format, or a query that fails preparation |
| `415` | `Content-Type` is neither JSON nor form-encoded |
| `500` | Internal execution error |

Request bodies are limited to 2 MiB.

## Development

```bash
make help     # list targets
make test     # run tests
make lint     # rustfmt check and clippy
make check    # lint + tests
```

Tests use in-memory SQLite databases through the router. Captured soilDB SQL
fixtures are always parsed/transpiled and replayed against deterministic local
tables; the spatial replay runs when `SDA_TEST_SPATIALITE` is configured. These
tests do not install or load soilDB. The Iowa GeoPackage/soilDB client checks
remain opt-in via `make test-soildb`; see [their README](tests/soildb/README.md)
and the fixture provenance/refresh notes in
[`tests/fixtures/soildb/README.md`](tests/fixtures/soildb/README.md).

## Project layout

- `src/main.rs` -- binary entry point
- `src/lib.rs` -- router and shared state
- `src/config.rs` -- configuration
- `src/routes.rs` -- HTTP handler
- `src/request.rs` -- request parsing
- `src/transpiler/` -- T-SQL to SQLite transformation passes
- `src/executor/` -- query execution
- `src/serializer.rs`, `src/result.rs` -- response formatting
- `tests/sda_mvp.rs` -- integration tests
- `planning/` -- goals, execution plans, and the architecture spec (`planning/sda-rs-scoping.md`)

## License

Not specified yet.
