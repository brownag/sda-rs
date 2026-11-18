# Opt-in soilDB integration tests

Run the local soilDB client against `sda-rs` and an existing Iowa GeoPackage:

```sh
SDA_TEST_SPATIALITE=/usr/lib/x86_64-linux-gnu/mod_spatialite.so \
SDA_TEST_GEOPACKAGE=/data/SSURGO/output/iowa_ssurgo.gpkg \
SOILDB_PATH=/home/andrew/workspace/soilDB \
make test-soildb
```

The target is opt-in and is not part of `make test`, `cargo test`, or CI.
It requires Cargo, curl, Python 3, R, and the R packages `data.table`, `DBI`,
`curl`, `jsonlite`, `testthat`, `sf`, and `wk`. Install missing R packages in
your normal user library before running; the harness installs only the local
soilDB source into a temporary library and removes that library on exit.

`SOILDB_PATH`, `SDA_TEST_GEOPACKAGE`, and `SDA_TEST_SPATIALITE` can be
overridden as shown above. `SDA_TEST_PORT` optionally selects a free local TCP
port; by default the harness asks the OS for an ephemeral port. The server is
explicitly bound to `127.0.0.1` via `SDA_BIND_ADDRESS`/`--bind-address`.

The backend is opened with SQLite's `mode=ro` URI. Requests are read-only;
server-created temporary tables are request-local. The harness never copies or
modifies the GeoPackage, and it does not edit the soilDB checkout. It builds
the server, waits for a local readiness response, runs the selected testthat
cases, and stops the server on success, failure, or interruption. Logs are
printed if server startup or soilDB installation fails.

Tests check an `IA001` legend row, spatial lookup of verified mukey `1857266`,
and `get_SDA_muaggatt()` results for that mukey. The checks assert returned
data and keys, not only HTTP status.
