#!/bin/bash
# Wrapper script to bottle up logic for running "smoke tests" in CI,
# supporting backgrounding tasks and checking on their status later.
# The execution plan is:
#
#   1. Start the network
#   2. Wait ~10s
#   3. Run integration tests (fail here if non-zero)
#   4. Continue running network ~5m
#
# The goal is to fail fast if an integration test exits, but permit
# a slightly longer runtime for the suite to find more errors.
set -euo pipefail


# Fail fast if testnet dir exists, otherwise `cargo run ...` will block
# for a while, masking the error.
if [[ -d ~/.penumbra/testnet_data ]] ; then
    >&2 echo "ERROR: testnet data directory exists at ~/.penumbra/testnet_data"
    >&2 echo "Not removing this directory automatically; to remove, run: pd testnet unsafe-reset-all"
    exit 1
fi

# Ensure we have cometmock available locally.
if ! hash cometmock > /dev/null 2>&1 ; then
    >&2 echo "ERROR: cometmock not found, please install from https://github.com/informalsystems/CometMock"
    >&2 echo "Make sure to use the 'v0.34.x' branch for now."
    exit 1
fi

export RUST_LOG="pclientd=info,pcli=info,pd=info,penumbra=info"

# Duration that the network will run before integration tests are run.
TESTNET_BOOTTIME="${TESTNET_BOOTTIME:-20}"
# Duration that the network will be left running before script exits.
TESTNET_RUNTIME="${TESTNET_RUNTIME:-10}"

echo "Building pd from latest source..."
cargo build --release --bin pd

echo "Generating testnet config..."
# Export the epoch duration, so it's accessible as env var in test suite.
export EPOCH_DURATION="${EPOCH_DURATION:-100}"
cargo run --quiet --release --bin pd -- testnet generate --epoch-duration "$EPOCH_DURATION" --timeout-commit 500ms

echo "Starting pd..."
cargo run --quiet --release --bin pd -- start --home "${HOME}/.penumbra/testnet_data/node0/pd" &
pd_pid="$!"

sleep 2
echo "Starting CometMock (stand-in for Tendermint/CometBFT)..."
# ❯ cometmock -h
# Usage: <app-addresses> <genesis-file> <cometmock-listen-address> <node-homes> <abci-connection-mode>
cometmock \
    --block-time 50 \
    127.0.0.1:26658 \
    "${HOME}/.penumbra/testnet_data/node0/cometbft/config/genesis.json" \
    tcp://127.0.0.1:26657 \
    "${HOME}/.penumbra/testnet_data/node0/cometbft" \
    socket &
cometmock_pid="$!"

# Ensure processes are cleaned up after script exits, regardless of status.
trap 'kill -9 "$cometmock_pid" "$pd_pid"' EXIT

echo "Waiting $TESTNET_BOOTTIME seconds for network to boot..."
sleep "$TESTNET_BOOTTIME"

export PENUMBRA_NODE_PD_URL="http://127.0.0.1:8080"
export PCLI_UNLEASH_DANGER="yes"
echo "Running pclientd integration tests against network"
cargo test --quiet --release --features sct-divergence-check --package pclientd -- --ignored --test-threads 1 --nocapture

echo "Running pcli integration tests against network"
cargo test --quiet --release --features sct-divergence-check,download-proving-keys --package pcli -- --ignored --test-threads 1 --nocapture

echo "Waiting another $TESTNET_RUNTIME seconds while network runs..."
sleep "$TESTNET_RUNTIME"
# `kill -0` checks existence of pid, i.e. whether the process is still running.
# It doesn't inspect errors, but the only reason the process would be stopped
# is if it failed, so it's good enough for our needs.
if ! kill -0 "$cometmock_pid" || ! kill -0 "$pd_pid" ; then
    >&2 echo "ERROR: smoke test process exited early"
    exit 1
else
    echo "SUCCESS! Smoke test complete. Ran for $TESTNET_RUNTIME, found no errors."
fi
exit 0
