#!/bin/bash
set -e

# List of packages to check
PACKAGES=("pcli")

# Function to check for rocksdb dependency
check_rocksdb() {
  local package=$1
  echo "Checking $package for rocksdb dependency..."
  
  if cargo tree -p $package | grep -q "rocksdb"; then
    echo "ERROR: $package depends on rocksdb!"
    return 1
  else
    echo "OK: $package does not depend on rocksdb"
    return 0
  fi
}

# Main execution
for package in "${PACKAGES[@]}"; do
  check_rocksdb "$package" || exit 1
done

echo "All packages passed the check!"
exit 0