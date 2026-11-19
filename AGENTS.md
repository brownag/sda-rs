# AGENTS.md - sda-rs

## Quick Start
* **Install dependencies**: `cargo fetch`
* **Build**: `cargo build`
* **Test all**: `cargo test`
* **Test target**: `cargo test --test sda_mvp` or `cargo test <name>`
* **Lint/format**: `cargo fmt --check && cargo clippy --all-targets`
* **Run**: `cargo run -- --backend-url <path/to/db> --port 8080`

## Find Things
`sda-rs` is a Rust HTTP server emulating the USDA-NRCS Soil Data Access endpoint. It accepts T-SQL-style queries, transpiles them through `polyglot-sql` to SQLite/SpatiaLite, and returns SDA-shaped JSON.

* `src/main.rs` -- binary entry point; configuration and server startup
* `src/lib.rs` -- `app()` router, `AppState`, middleware
* `src/config.rs` -- CLI and environment configuration
* `src/routes.rs`, `src/request.rs` -- HTTP handlers and request parsing
* `src/transpiler/` -- SQL parsing, validation, and rewrite passes
* `src/executor/` -- SQLite query execution
* `src/spatial.rs`, `src/gpkg_geometry.rs` -- spatial capability and GeoPackage geometry handling
* `src/serializer.rs`, `src/result.rs` -- SDA response shaping
* `tests/` -- integration and contract tests; `tests/fixtures/` contains query fixtures
* `scripts/` -- fixture capture and test helpers
* `planning/` -- goals, execution plans, tasks, and architecture specification
* `docs/` -- decisions, security policy, and SQL/API contracts

## Workflows
* **Planned work**: `planning/GOALS.md` -> `planning/EXECUTION_PLAN_N.md` -> task files in `planning/tasks/`; see plan/state files for workflow details.
* **Adding a transpiler pass**: add it under `src/transpiler/`, wire it into `mod.rs`, and cover it with unit and integration tests.
* **Adding a route**: register it in `app()` in `src/lib.rs`; preserve the body-limit and CORS layers.

## Infrastructure
* **Default bind**: `0.0.0.0:8080` (`SDA_PORT` / `--port`)
* **Default database**: `sda.gpkg` (`SDA_BACKEND_URL` / `--backend-url`)
* **Optional environment**: `SDA_SPATIAL_EXTENSION` (SpatiaLite extension path), `SDA_GEOM_COLUMN` (geometry column override)
* **Request body limit**: 2 MiB (`src/lib.rs`)
* **Log level**: `RUST_LOG` (defaults to `info`)
* **Build output**: `target/` (git-ignored)

## References
* `planning/sda-rs-scoping.md` -- architecture, transpilation, spatial rewrites, and wire protocol
* `planning/GOALS.md` and `planning/EXECUTION_PLAN_*.md` -- planning artifacts
* `docs/decisions/` -- architecture decision records
* `docs/security/sqlite-execution-policy.md` -- SQLite execution policy
* `docs/spatial-sql-contract.md` -- spatial SQL contract

## Operational Rules
* Run `cargo test` before reporting a change as done; keep SQL transpilation changes covered by tests.
* Ask before adding dependencies, changing the wire protocol/response shape, or changing default port, database path, or body-size limit.
* Spatial functions require `SDA_SPATIAL_EXTENSION`; non-spatial queries must still work without it.
* Failed statement batches roll back as a whole. Do not weaken rollback or geometry metadata validation to make tests pass.
* `mukey` TVF rewrites require geometry metadata from `mupolygon` in `gpkg_geometry_columns`, or an `SDA_GEOM_COLUMN` override.
* SDA JSON + COLUMNNAME + METADATA responses place column metadata in the second result array.
* Never commit `target/` or `.gpkg`/`.sqlite` database files.
