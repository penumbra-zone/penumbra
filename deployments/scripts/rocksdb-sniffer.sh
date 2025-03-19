#!/bin/bash
# CI script to check build dependencies, and ensure that `rocksdb` is not required,
# typically via the cnidarium dep. Only `pd` should depend on rocksdb.
set -eou pipefail
# Function to check for rocksdb dependency
check_rocksdb() {
  local package=$1
  echo -n "$package: "
  
  if cargo tree -p "$package" | grep -q "rocksdb"; then
    echo "ERROR"
    return 1
  else
    echo "OK"
    return 0
  fi
}

# Function to display all binaries built by this workspace,
# excluding pd and other bins known to require rocksdb.
function get_workspace_bins() {
 cargo metadata --format-version 1 | jq -r '.packages[] | select(.manifest_path | startswith("'$(pwd)'")) | . as $pkg | .targets[] | select(.kind[] | contains("bin")) | "\($pkg.name)"' \
   | grep -vP '^(pd|summonerd|penumbra-sdk-parameter-setup)$'
}

# List of packages to check
declare -a PACKAGES
readarray -t PACKAGES < <(get_workspace_bins)

# Main execution
echo "Checking packages for rocksdb dependencies"
ERRORS=()
for package in "${PACKAGES[@]}"; do
  if ! check_rocksdb "$package"; then
    ERRORS+=("$package")
  fi
done

if [ ${#ERRORS[@]} -gt 0 ]; then
  echo "Found ${#ERRORS[@]} package(s) with rocksdb dependency:"
  for package in "${ERRORS[@]}"; do
    echo "$package"
  done
  exit 1
else
  exit 0
fi
