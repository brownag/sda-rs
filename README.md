# sda-rs

`sda-rs` is a standalone server for running Soil Data Access (SDA)-style queries
locally. It emulates the USDA-NRCS SDA `POST /Tabular/post.rest` endpoint while
using SQLite as its execution engine, so you can work with local data without
depending on a remote SDA service or a database server.

The goal is practical compatibility, not a claim of complete SQL Server or SDA
parity. The server accepts supported T-SQL-style queries, translates them to
SQLite, and returns SDA-shaped JSON or XML responses.

## Use your own data

Point the server at an existing SQLite database or OpenSSURGO GeoPackage. No
database import or project-specific schema is required just to run ordinary
queries: the database is the data source.

```bash
cargo run -- --backend-url path/to/your-data.gpkg --port 8080
```

Spatial operations are optional. Plain SQLite queries work without SpatiaLite;
to use spatial queries, provide a SpatiaLite extension:

```bash
cargo run -- \
  --backend-url path/to/your-data.gpkg \
  --spatial-extension /path/to/mod_spatialite.so
```

The executable can also be configured with environment variables: `SDA_BACKEND_URL`,
`SDA_PORT`, `SDA_SPATIAL_EXTENSION`, and `SDA_GEOM_COLUMN`. See `--help` for
available options.

## Query interface

The server accepts JSON or form-encoded `POST` requests at
`/Tabular/post.rest` (`/Tab/post.rest` is an alias). For example:

```bash
curl -X POST http://localhost:8080/Tabular/post.rest \
  -H 'Content-Type: application/json' \
  -d '{"query":"SELECT areaname FROM legend","format":"JSON+COLUMNNAME"}'
```

Supported response formats include `JSON`, `JSON+COLUMNNAME`,
`JSON+COLUMNNAME+METADATA`, and `XML`. The `~` format echoes the query without
executing it. The implementation supports a deliberate subset of SDA/T-SQL
behavior; see [`docs/`](docs/) for the spatial SQL contract, security policy,
and documented support details.

## Development

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for build and test workflows and
[`AGENTS.md`](AGENTS.md) for repository-specific development guidance.

## License

MIT; see [LICENSE](LICENSE).
