#!/usr/bin/env bash
set -euo pipefail

readonly expected_sha256="8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc"
readonly artifact_name="spatial.duckdb_extension"

usage() {
  echo "Usage: $0 <local-source-artifact> <deployment-controlled-destination-directory>" >&2
  exit 2
}

[[ $# -eq 2 ]] || usage
source_artifact=$1
destination_dir=$2

if [[ ! -f "$source_artifact" || ! -r "$source_artifact" ]]; then
  echo "error: source artifact is absent or unreadable: $source_artifact" >&2
  exit 1
fi
if [[ ! -d "$destination_dir" || ! -w "$destination_dir" ]]; then
  echo "error: destination must be an existing writable deployment directory: $destination_dir" >&2
  exit 1
fi

temporary_file=$(mktemp "$destination_dir/.${artifact_name}.XXXXXX")
cleanup() {
  rm -f -- "$temporary_file"
}
trap cleanup EXIT

cp -- "$source_artifact" "$temporary_file"
actual_sha256=$(sha256sum -- "$temporary_file")
actual_sha256=${actual_sha256%% *}
if [[ "$actual_sha256" != "$expected_sha256" ]]; then
  echo "error: artifact SHA-256 mismatch (expected $expected_sha256, got $actual_sha256)" >&2
  exit 1
fi

chmod 0444 -- "$temporary_file"
mv -f -- "$temporary_file" "$destination_dir/$artifact_name"
trap - EXIT
echo "Provisioned $destination_dir/$artifact_name (sha256 $actual_sha256)"
