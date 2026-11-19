#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
soilDB_path="${SOILDB_PATH:-/home/andrew/workspace/soilDB}"
geopackage="${SDA_TEST_GEOPACKAGE:-/data/SSURGO/output/iowa_ssurgo.gpkg}"
spatialite="${SDA_TEST_SPATIALITE:-/usr/lib/x86_64-linux-gnu/mod_spatialite.so}"

fail() {
    printf 'test-soildb: %s\n' "$*" >&2
    exit 1
}

for command in cargo curl python3 R Rscript; do
    command -v "$command" >/dev/null 2>&1 || fail "required command '$command' was not found"
done
[[ -d "$soilDB_path" && -f "$soilDB_path/DESCRIPTION" ]] ||
    fail "SOILDB_PATH must point to a soilDB source checkout: $soilDB_path"
[[ -r "$geopackage" ]] || fail "SDA_TEST_GEOPACKAGE is not readable: $geopackage"
[[ -r "$spatialite" ]] || fail "SDA_TEST_SPATIALITE is not readable: $spatialite"

if ! Rscript --vanilla -e '
required <- c("data.table", "DBI", "curl", "jsonlite", "testthat", "sf", "wk")
for (package in required) {
  if (!requireNamespace(package, quietly = TRUE)) {
    stop("Missing R package: ", package)
  }
  loaded <- tryCatch(requireNamespace(package, quietly = TRUE), error = function(e) {
    message(package, ": ", conditionMessage(e))
    FALSE
  })
  if (!loaded) stop("Missing or unloadable R package: ", package)
}
'; then
    fail "install or repair the R dependency reported above"
fi

if [[ -n "${SDA_TEST_PORT:-}" ]]; then
    [[ "$SDA_TEST_PORT" =~ ^[0-9]+$ ]] || fail "SDA_TEST_PORT must be a numeric TCP port"
    (( SDA_TEST_PORT > 0 && SDA_TEST_PORT <= 65535 )) ||
        fail "SDA_TEST_PORT must be between 1 and 65535"
    port="$SDA_TEST_PORT"
else
    port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
fi

backend_uri="$(python3 - "$geopackage" <<'PY'
from pathlib import Path
from urllib.parse import quote
import sys
path = quote(str(Path(sys.argv[1]).resolve(strict=True)), safe="/")
print("readonly:file:" + path + "?mode=ro")
PY
)" || fail "could not construct a read-only SQLite URI for the GeoPackage"

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/sda-rs-soildb.XXXXXX")"
server_pid=""
cleanup() {
    if [[ -n "$server_pid" ]] && kill -0 "$server_pid" 2>/dev/null; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf -- "$tmpdir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

mkdir -p "$tmpdir/Rlib"
preferred_r_library="${R_LIBS_USER:-$(Rscript --vanilla -e \
    'cat(path.expand(Sys.getenv("R_LIBS_USER", unset = "~/R/%p-library/%v")))' )}"
export R_LIBS_USER="$tmpdir/Rlib"
if [[ -n "$preferred_r_library" && -d "$preferred_r_library" ]]; then
    export R_LIBS="$preferred_r_library${R_LIBS:+:$R_LIBS}"
fi
export SDA_BACKEND_URL="$backend_uri"
export SDA_SPATIAL_EXTENSION="$spatialite"
export SDA_BIND_ADDRESS=127.0.0.1
export SOILDB_SDA_URL="http://127.0.0.1:${port}/Tabular/post.rest"

server_executable="$(
    cd "$repo_root"
    cargo build --message-format=json --bin sda-rs |
        python3 -c '
import json
import sys
for line in sys.stdin:
    try:
        message = json.loads(line)
    except json.JSONDecodeError:
        continue
    target = message.get("target", {})
    executable = message.get("executable")
    if executable and target.get("name") == "sda-rs" and "bin" in target.get("kind", []):
        print(executable)
        break
else:
    raise SystemExit("Cargo did not report the sda-rs binary executable")
'
)" || fail "could not locate the built sda-rs executable from Cargo output"

if ! R CMD INSTALL --library="$R_LIBS_USER" "$soilDB_path" \
    >"$tmpdir/install.log" 2>&1; then
    cat "$tmpdir/install.log" >&2
    fail "could not install local soilDB into the temporary library"
fi

"$server_executable" \
    --backend-url "$SDA_BACKEND_URL" \
    --port "$port" \
    --spatial-extension "$SDA_SPATIAL_EXTENSION" \
    --bind-address 127.0.0.1 \
    >"$tmpdir/server.log" 2>&1 &
server_pid=$!

ready=0
for _ in $(seq 1 50); do
    if curl --silent --show-error --fail --max-time 1 \
        -H 'content-type: application/json' \
        --data-binary '{"query":"SELECT 1 AS ready","format":"JSON+COLUMNNAME"}' \
        "$SOILDB_SDA_URL" >/dev/null 2>&1; then
        ready=1
        break
    fi
    if ! kill -0 "$server_pid" 2>/dev/null; then
        break
    fi
    sleep 0.2
done
if [[ "$ready" -ne 1 ]]; then
    cat "$tmpdir/server.log" >&2
    fail "server did not become ready on 127.0.0.1:$port"
fi

(
    cd "$repo_root"
    Rscript --vanilla tests/soildb/testthat.R
)

kill "$server_pid"
wait "$server_pid" || true
server_pid=""
printf 'test-soildb: integration tests passed; stopped server pid %s\n' "$!"
