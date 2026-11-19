#!/usr/bin/env bash
# Install sda-rs as a systemd service bound to 127.0.0.1:8090.
#
# Dry-run by default: prints the plan and changes nothing.
#   sudo deploy/install.sh                  # show plan
#   sudo deploy/install.sh --apply          # install files, daemon-reload, enable
#   sudo deploy/install.sh --apply --restart  # also (re)start the service
#
# Environment overrides:
#   SDA_BINARY   release binary to install (default: target/release/sda-rs in this repo)
#   SDA_DATA     GeoPackage to install (default: /home/soillite/sda-data/openssurgo.gpkg)
#   SDA_USER     service account (default: sdars)

set -euo pipefail

usage() {
  sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

APPLY=0
RESTART=0
for arg in "$@"; do
  case "$arg" in
    --apply) APPLY=1 ;;
    --restart) RESTART=1 ;;
    -h | --help) usage 0 ;;
    *) echo "unknown option: $arg" >&2; usage 2 ;;
  esac
done

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BINARY_SRC="${SDA_BINARY:-$REPO/target/release/sda-rs}"
DATA_SRC="${SDA_DATA:-/home/soillite/sda-data/openssurgo.gpkg}"
SERVICE_USER="${SDA_USER:-sdars}"

BIN_DST=/usr/local/bin/sda-rs
DATA_DIR=/var/lib/sda-rs
DATA_DST="$DATA_DIR/openssurgo.gpkg"
ENV_DIR=/etc/sda-rs
UNIT_DST=/etc/systemd/system/sda-rs.service

run() {
  if [ "$APPLY" -eq 1 ]; then
    echo "+ $*"
    "$@"
  else
    echo "[plan] $*"
  fi
}

if [ "$APPLY" -eq 1 ] && [ "$(id -u)" -ne 0 ]; then
  echo "--apply needs root (try: sudo $0 --apply)" >&2
  exit 1
fi

[ -f "$BINARY_SRC" ] || { echo "missing binary: $BINARY_SRC (run: cargo build --release)" >&2; exit 1; }
[ -f "$DATA_SRC" ] || { echo "missing GeoPackage: $DATA_SRC" >&2; exit 1; }
[ -f "$REPO/deploy/sda-rs.service" ] || { echo "missing unit file in $REPO/deploy" >&2; exit 1; }

echo "mode: $([ "$APPLY" -eq 1 ] && echo APPLY || echo DRY-RUN)"
echo "binary: $BINARY_SRC -> $BIN_DST"
echo "data:   $DATA_SRC -> $DATA_DST"
echo "unit:   $REPO/deploy/sda-rs.service -> $UNIT_DST"

if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
  run useradd --system --no-create-home --shell /usr/sbin/nologin "$SERVICE_USER"
fi
run install -m 0755 -o root -g root "$BINARY_SRC" "$BIN_DST"
run install -d -m 0755 -o root -g root "$DATA_DIR"
run install -m 0644 -o root -g root "$DATA_SRC" "$DATA_DST"
run install -d -m 0755 -o root -g root "$ENV_DIR"
if [ -f "$UNIT_DST" ]; then
  run cp -p "$UNIT_DST" "$UNIT_DST.bak.$(date -u +%Y%m%dT%H%M%SZ)"
fi
run install -m 0644 -o root -g root "$REPO/deploy/sda-rs.service" "$UNIT_DST"
run systemctl daemon-reload
run systemctl enable sda-rs.service
if [ "$RESTART" -eq 1 ]; then
  run systemctl restart sda-rs.service
fi

if [ "$APPLY" -eq 0 ]; then
  echo "dry-run complete; nothing changed. Re-run with --apply to install."
fi
