#!/bin/bash
set -e

# List of packages to check
PACKAGES=("pcli" "pindexer" "pclientd")

# Function to check for rocksdb dependency
check_rocksdb() {
  local package=$1
  echo -n "$package: "
  
  if cargo tree -p $package | grep -q "rocksdb"; then
    echo "ERROR"
    return 1
  else
    echo "OK"
    return 0
  fi
}

# Main execution
ERRORS=()
for package in "${PACKAGES[@]}"; do
  if ! check_rocksdb "$package"; then
    ERRORS+=("$package")
  fi
done

echo "Checking packages for rocksdb dependencies"
if [ ${#ERRORS[@]} -gt 0 ]; then
  echo "Found ${#ERRORS[@]} package(s) with rocksdb dependency:"
  for package in "${ERRORS[@]}"; do
    echo $package
  done
  exit 1
else
  exit 0
fi
