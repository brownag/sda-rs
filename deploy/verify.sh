#!/usr/bin/env bash
# Read-only post-install checks for sda-rs. Runs as a normal user.
# Exit status is non-zero if any check fails.

set -uo pipefail

PORT="${SDA_PORT:-8090}"
ENDPOINT="http://127.0.0.1:${PORT}/Tabular/post.rest"
FAILS=0

record() {
  local status="$1" name="$2" detail="$3"
  printf '%-4s %-34s %s\n' "$status" "$name" "$detail"
  [ "$status" = FAIL ] && FAILS=$((FAILS + 1))
  return 0
}

if command -v systemctl >/dev/null 2>&1; then
  if systemctl is-active --quiet sda-rs.service; then
    record PASS "service active" "sda-rs.service"
  else
    record FAIL "service active" "sda-rs.service is not active"
  fi
  if systemctl is-enabled --quiet sda-rs.service; then
    record PASS "service enabled" "starts on boot"
  else
    record FAIL "service enabled" "sda-rs.service is not enabled"
  fi
else
  record SKIP "service state" "systemctl not installed"
fi

if command -v ss >/dev/null 2>&1; then
  listen="$(ss -ltn 2>/dev/null | awk 'NR>1 {print $4}')"
  if printf '%s\n' "$listen" | grep -qx "127.0.0.1:${PORT}"; then
    record PASS "loopback listener" "127.0.0.1:${PORT}"
  else
    record FAIL "loopback listener" "no 127.0.0.1:${PORT} listener"
  fi
  if printf '%s\n' "$listen" | grep -E -q "^(0\.0\.0\.0|\*|\[::\]|::):${PORT}$"; then
    record FAIL "no wildcard listener" "port ${PORT} is listening on a wildcard address"
  else
    record PASS "no wildcard listener" "port ${PORT} not on a wildcard address"
  fi
else
  record SKIP "listener checks" "ss not installed"
fi

if command -v curl >/dev/null 2>&1; then
  body="$(curl -s -m 10 -X POST "$ENDPOINT" -H 'Content-Type: application/json' \
    -d '{"query":"SELECT 1"}' -w '\n%{http_code}')"
  code="${body##*$'\n'}"
  payload="${body%$'\n'*}"
  if [ "$code" = 200 ] && [ "$payload" = '{"Table":[["1"]]}' ]; then
    record PASS "SELECT 1 round trip" "HTTP 200, expected body"
  else
    record FAIL "SELECT 1 round trip" "HTTP ${code:-none}, body: ${payload:0:120}"
  fi

  code="$(curl -s -o /dev/null -m 10 -w '%{http_code}' -X POST "$ENDPOINT" \
    -H 'Content-Type: application/json' -d '{"query":"DROP TABLE x"}')"
  if [ "$code" = 400 ]; then
    record PASS "rejects write SQL" "HTTP 400"
  else
    record FAIL "rejects write SQL" "HTTP ${code:-none} (expected 400)"
  fi

  code="$(curl -s -o /dev/null -m 10 -w '%{http_code}' "$ENDPOINT")"
  if [ "$code" = 405 ]; then
    record PASS "GET is not allowed" "HTTP 405"
  else
    record FAIL "GET is not allowed" "HTTP ${code:-none} (expected 405)"
  fi
else
  record SKIP "HTTP checks" "curl not installed"
fi

if [ -e /var/lib/sda-rs/openssurgo.gpkg ]; then
  if [ -w /var/lib/sda-rs/openssurgo.gpkg ]; then
    record WARN "backend writable" "data file is writable by this user"
  else
    record PASS "backend read-only" "data file not writable by this user"
  fi
fi

echo
if [ "$FAILS" -eq 0 ]; then
  echo "all checks passed"
else
  echo "$FAILS check(s) failed"
fi
exit $((FAILS > 0))
