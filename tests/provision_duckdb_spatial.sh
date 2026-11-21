#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
script="$repo_root/scripts/provision-duckdb-spatial.sh"
expected_sha256="8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc"
temporary_root=$(mktemp -d)
trap 'rm -rf -- "$temporary_root"' EXIT

source_file="$temporary_root/source.duckdb_extension"
destination="$temporary_root/deployment"
mkdir "$destination"

# This local fixture exercises success mechanics without requiring or fetching
# the large official binary. Supply the expected digest to the test fixture.
printf 'synthetic fixture\n' > "$source_file"
fixture_sha256=$(sha256sum "$source_file")
fixture_sha256=${fixture_sha256%% *}
sed "s/$expected_sha256/$fixture_sha256/" "$script" > "$temporary_root/provision-test.sh"
chmod +x "$temporary_root/provision-test.sh"

"$temporary_root/provision-test.sh" "$source_file" "$destination"
cmp "$source_file" "$destination/spatial.duckdb_extension"
[[ $(stat -c '%a' "$destination/spatial.duckdb_extension") == 444 ]]

printf 'different synthetic bytes\n' > "$temporary_root/wrong.duckdb_extension"
if "$temporary_root/provision-test.sh" "$temporary_root/wrong.duckdb_extension" "$destination"; then
  echo "expected mismatched artifact to fail" >&2
  exit 1
fi
[[ ! -e "$destination/.spatial.duckdb_extension."* ]]

if "$temporary_root/provision-test.sh" "$temporary_root/missing.duckdb_extension" "$destination"; then
  echo "expected absent artifact to fail" >&2
  exit 1
fi
[[ -f "$destination/spatial.duckdb_extension" ]]
echo "provisioning checks passed: identity/hash success, mismatch rejection, absent-source rejection"
