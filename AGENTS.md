# AGENTS.md - sda-rs

## Project Overview
`sda-rs` is a standalone Rust HTTP server that emulates the USDA-NRCS Soil Data Access `POST /Tabular/post.rest` endpoint. It accepts T-SQL-style queries, transpiles them (via `polyglot-sql`) to SQLite/SpatiaLite, and returns SDA-shaped JSON. Specs and goals live in `planning/`.

## Quick Start
* **Build**: `cargo build`
* **Test All**: `cargo test`
* **Test Target**: `cargo test --test sda_mvp` (integration tests) or `cargo test <name>` (unit tests in `src/`)
* **Lint/Format**: `cargo fmt --check` and `cargo clippy --all-targets`
* **Run**: `cargo run -- --backend-url <path/to/db> --port 8080`
* **Install**: `cargo fetch` (dependencies are declared in `Cargo.toml`; no workspace)

## Find Things
* `src/main.rs` -- binary entry point; reads config, binds listener, serves `app()`
* `src/lib.rs` -- crate root; `app()` builds the router, `AppState`, and middleware
* `src/config.rs` -- CLI/env configuration (`clap`); the only place defaults are defined
* `src/routes.rs` -- HTTP handler for `/Tabular/post.rest` and `/Tab/post.rest`
* `src/request.rs` -- request parsing (JSON and form-encoded bodies)
* `src/transpiler/` -- T-SQL-to-SQLite AST passes (`normalize`, `security`, `spatial`, `temp_tables`, `vars`)
* `src/executor/` -- database execution (`sqlite.rs`)
* `src/spatial.rs` -- SpatiaLite/geometry handling
* `src/serializer.rs`, `src/result.rs` -- response shaping into SDA JSON
* `tests/` -- integration tests against the router (`tower::ServiceExt`, in-memory SQLite)
* `planning/` -- execution plans, goals, task files (`planning/tasks/`), and the architecture spec (`sda-rs-scoping.md`)

## Workflows
* **Planned work**: `planning/GOALS.md` -> `planning/EXECUTION_PLAN_N.md` -> task files in `planning/tasks/` -> `STATE_N.md` tracks progress.
* **Adding a transpiler pass**: add a module under `src/transpiler/`, wire it into `transpiler/mod.rs`, add a unit test next to it and an end-to-end case in `tests/sda_mvp.rs`.
* **Adding a route**: register it in `app()` in `src/lib.rs`; keep the body limit and CORS layers applied.

## Infrastructure
* **Default bind**: `0.0.0.0:8080` (`SDA_PORT` / `--port`)
* **Default database**: `sda.gpkg` (`SDA_BACKEND_URL` / `--backend-url`)
* **Optional env**: `SDA_SPATIAL_EXTENSION` (path to `mod_spatialite`), `SDA_GEOM_COLUMN` (GeoPackage geometry column override)
* **Request body limit**: 2 MiB (`src/lib.rs`)
* **Log level**: `RUST_LOG` (defaults to `info`)
* **Build output**: `target/` (git-ignored)

## Code Style & Patterns
* **Verification**: `cargo fmt --check && cargo clippy --all-targets && cargo test`
* **Errors**: HTTP errors use the `ErrorBody { error }` JSON shape via `routes::error()`; keep that shape for new error responses.
* **Modules**: all modules are declared `pub` in `src/lib.rs`; `main.rs` only wires config to the router.

## Non-Obvious System Behaviors
* Spatial functions are unavailable unless `SDA_SPATIAL_EXTENSION` is set; non-spatial queries must keep working without it.
* Failed statement batches roll back as a whole (see `router_rolls_back_failed_statement_batch`).
* `mukey` TVF rewrites read geometry metadata from `mupolygon` in `gpkg_geometry_columns`, not from the first geometry row. Without that metadata or a `SDA_GEOM_COLUMN` override, the request is rejected.
* SDA response format (JSON + COLUMNNAME + METADATA) places column metadata in the second result array; see the scoping spec before changing serializer output.

## System Boundaries & Operational Rules
### Always
* Run `cargo test` before reporting a change as done.
* Keep SQL transpilation changes covered by a test in `src/transpiler/` or `tests/`.

### Ask First
* Adding dependencies to `Cargo.toml` (especially new `polyglot-sql` or `rusqlite` versions, which change query behavior).
* Changing the wire protocol or response shape.
* Changing default port, default database path, or the body size limit.

### Never
* Commit `target/` or any `.gpkg`/`.sqlite` database files.
* Remove or weaken the rollback and metadata-validation behavior covered by `tests/sda_mvp.rs` to make a test pass.

## Secondary Documentation Index
* `planning/sda-rs-scoping.md` -- Architecture and transpilation spec (AST passes, spatial rewrite catalog, wire protocol).
* `planning/GOALS.md` -- Goal list and dependency graph.
* `planning/EXECUTION_PLAN_*.md` -- Wave-by-wave execution plans.
