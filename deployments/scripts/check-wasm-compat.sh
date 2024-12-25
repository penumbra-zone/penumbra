#!/usr/bin/env bash
# CI script for checking that the Penumbra monorepo does not accidentally
# break compatibility with downstream web APIs, via the WASM crate.
# Historically, this breakage has taken the form of inadvertently introducing
# dependencies on std, e.g. via `mio`.
#
# More broadly, we want to ensure that monorepo crates with the "component"
# feature build without that (default) feature enabled. Testing this on the wasm
# target will help ensure compat.
set -euo pipefail


# Consider checking the web repo's wasm Cargo.toml periodically:
#
#   ❯ rg ^penumbra packages/wasm/crate/Cargo.toml --no-line-number | cut -f1 -d' '  | sort
#
# to make sure at least all of those crates are tracked here.

packages=(
    penumbra-sdk-asset
    penumbra-sdk-community-pool
    penumbra-sdk-compact-block
    penumbra-sdk-auction
    penumbra-sdk-dex
    penumbra-sdk-distributions
    penumbra-sdk-fee
    penumbra-sdk-funding
    penumbra-sdk-governance
    penumbra-sdk-ibc
    penumbra-sdk-keys
    penumbra-sdk-sct
    penumbra-sdk-shielded-pool
    penumbra-sdk-stake
    penumbra-sdk-tct
    penumbra-sdk-transaction
    penumbra-sdk-txhash
    # N.B. we can't include those ones because they rely on `getrandom`,
    # but there's a `js` feature...
    # decaf377-fmd
    # decaf377-frost
    # decaf377-ka
    # penumbra-num
    # penumbra-proof-params
    # penumbra-proto
)

# We intentionally loop over the packages one by one to make error-reporting clearer.
# Ostensibly this would be slow, but in CI with a warm cache it's quick.
for p in "${packages[@]}" ; do
    echo "Checking package for wasm compat: $p ..."
    if ! cargo check --release --target wasm32-unknown-unknown --no-default-features --package "$p" ; then
        >&2 echo "ERROR: package appears not to be wasm-compatible: '$p'"
        exit 1
    fi
done
