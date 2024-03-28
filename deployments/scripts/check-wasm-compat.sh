#!/usr/bin/env bash
# CI script for checking that the Penumbra monorepo does not accidentally
# break compatibility with downstream web APIs, via the WASM crate.
# Historically, this breakage has taken the form of inadvertently introducing
# dependencies on std, e.g. via `mio`.
set -euo pipefail


# List of packages taken from web repo's wasm Cargo.toml:
#
#   ❯ rg ^penumbra packages/wasm/crate/Cargo.toml --no-line-number | cut -f1 -d' '  | sort
#
# should be periodically updated in order to keep parity.

packages=(
    penumbra-asset
    penumbra-compact-block
    penumbra-dex
    penumbra-fee
    penumbra-governance
    penumbra-ibc
    penumbra-keys
    penumbra-sct
    penumbra-shielded-pool
    penumbra-stake
    penumbra-tct
    penumbra-transaction

    # Some aren't ready with no default features, due to js/getrandom:
    # penumbra-num
    # penumbra-proof-params
    # penumbra-proto
    # decaf377-fmd
    # decaf377-frost
    # decaf377-ka
)

# We intentionally loop over the packages one by one to make error-reporting clearer.
# Ostensibly this would be slow, but in CI with a warm cache it's quick.
for p in ${packages[@]} ; do
    echo "Checking package for wasm compat: $p ..."
    if ! cargo check --quiet --release --target wasm32-unknown-unknown --no-default-features --package "$p" ; then
        >&2 echo "ERROR: package appears not to be wasm-compatible: '$p'"
        exit 1
    fi
done
