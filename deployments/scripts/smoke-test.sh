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

export RUST_LOG="pcli=info,penumbra=info"

# Duration that the network will be left running before script exits.
TESTNET_RUNTIME="${TESTNET_RUNTIME:-5m}"
# Duration that the network will run before integration tests are run.
TESTNET_BOOTTIME="${TESTNET_BOOTTIME:-10s}"

echo "Generating testnet config..."
cargo run --release --bin pd -- testnet generate

echo "Starting Tendermint..."
tendermint start --home $HOME/.penumbra/testnet_data/node0/tendermint &
tendermint_pid="$!"

echo "Starting pd..."
cargo run --release --bin pd -- start --home $HOME/.penumbra/testnet_data/node0/pd &

echo "Running integration tests against network"
PENUMBRA_NODE_HOSTNAME="127.0.0.1" \
    PCLI_UNLEASH_DANGER="yes" \
    cargo test --release --features sct-divergence-check --package pcli -- --ignored --test-threads 1 --nocapture

echo "Waiting another $TESTNET_RUNTIME while network runs..."
sleep "$TESTNET_RUNTIME"
# `kill -0` checks existence of pid, i.e. whether the process is still running.
# It doesn't inspect errors, but the only reason the process would be stopped
# is if it failed, so it's good enough for our needs.
if kill -0 "$tendermint_pid"; then
    kill -9 "$tendermint_pid"
    echo "SUCCESS! Smoke test complete. Ran for $TESTNET_RUNTIME, found no errors."
else
    >&2 echo "ERROR: smoke test compose process exited early"
    exit 1
fi
exit 0
