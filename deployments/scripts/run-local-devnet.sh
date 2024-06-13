#!/usr/bin/env bash
# Helper script for dev/debugging of relayer functionality. Heavily based
# on the "smoke test" network integration test suite scripts.
# Configures and runs a relayer instance between "preview" (latest main)
# and local devnet on current HEAD.
set -euo pipefail

echo "Starting CometBFT..."
cometbft start --log_level=error --home "${HOME}/.penumbra/testnet_data/node0/cometbft" &
cometbft_pid="$!"

export RUST_LOG="info,[CheckTx]=debug"
echo "Starting pd..."
cargo run --release --bin pd -- start --home "${HOME}/.penumbra/testnet_data/node0/pd" &
pd_pid="$!"

# Ensure processes are cleaned up after script exits, regardless of status.
trap 'kill -9 "$cometbft_pid" "$pd_pid"' EXIT

# ~lock forever on the processes
wait
