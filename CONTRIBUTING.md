# Contributing

## Development setup

Install Rust and Cargo, then fetch dependencies and run the test suite:

```bash
make install
make check
```

Use `make help` to list available targets. Run the server with
`make run DB=path/to/data.gpkg PORT=8080`, or use Cargo directly.

## Tests

`make test` runs the automated suite, which uses in-memory SQLite databases and
deterministic local fixtures. Some spatial tests require SpatiaLite. The
soilDB/Iowa client checks are opt-in via `make test-soildb`; see
[`tests/soildb/README.md`](tests/soildb/README.md) and
[`tests/fixtures/soildb/README.md`](tests/fixtures/soildb/README.md).

## Project guidance

See [`AGENTS.md`](AGENTS.md) for the code map, test expectations, architectural
constraints, and repository-specific operational rules. Before changing SQL
transpilation, response behavior, or spatial support, consult the relevant
contracts and decisions in [`docs/`](docs/) and the architecture specification
in [`planning/sda-rs-scoping.md`](planning/sda-rs-scoping.md).

## Changes

Keep changes focused and add or update tests for behavior changes. Run
`make check` before submitting. Ask before adding dependencies or changing
public response behavior, defaults, or request limits, as described in
[`AGENTS.md`](AGENTS.md).
