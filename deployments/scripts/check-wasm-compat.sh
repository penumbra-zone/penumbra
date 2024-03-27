#!/usr/bin/env bash

set -e

packages=(
  "penumbra-asset"
  "penumbra-tct"
  "penumbra-community-pool"
  "penumbra-compact-block"
  "penumbra-dex"
  "penumbra-distributions"
  "penumbra-fee"
  "penumbra-funding"
  "penumbra-governance"
  "penumbra-ibc"
  "penumbra-sct"
  "penumbra-shielded-pool"
  "penumbra-stake"
  "penumbra-keys"
  "penumbra-transaction"
  "penumbra-txhash"
  # Note: we can't include those ones because they rely on `getrandom`
  # but there's a `js` feature...
  # "penumbra-num"
  # "penumbra-proto"
  # "decaf377-fmd"
  # "decaf377-frost"
  # "decaf377-ka"
  # "penumbra-proof-params"
)

for package in "${packages[@]}"; do
  echo "Building package: $package"
  cargo check --release --target wasm32-unknown-unknown --package "$package" --no-default-features
  if [ $? -ne 0 ]; then
    echo "Compile error encountered while building package $package"
    exit 1
  fi
  echo "Package $package built successfully"
  echo "------------------------"
done
