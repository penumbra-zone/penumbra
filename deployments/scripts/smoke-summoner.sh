#!/bin/bash
# Run e2e summoner ceremony in CI
set -euo pipefail

export RUST_LOG="summonerd=info,pcli=info"
# This is not a secret, it is a test account seed phrase, used for integration tests like this one only.
export SEED_PHRASE="comfort ten front cycle churn burger oak absent rice ice urge result art couple benefit cabbage frequent obscure hurry trick segment cool job debate"

echo "Building latest version of summonerd from source..."
cargo build --quiet --release --bin summonerd

echo "Generating phase 1 root..."
cargo run --quiet --release --bin summonerd -- generate-phase1 --output phase1.bin

echo "Setting up storage directory..."
mkdir /tmp/summonerd
cargo run --quiet --release --bin pcli -- --home /tmp/summonerd --node https://grpc.testnet-preview.penumbra.zone keys generate
export SUMMONER_ADDRESS=$(PCLI_UNLEASH_DANGER="yes" cargo run --quiet --release --bin pcli -- --home /tmp/summonerd --node https://grpc.testnet-preview.penumbra.zone view address 0 2>&1)
export SUMMONER_FVK=$(PCLI_UNLEASH_DANGER="yes" cargo run --quiet --release --bin pcli -- --home /tmp/summonerd --node https://grpc.testnet-preview.penumbra.zone keys export full-viewing-key 2>&1)
cargo run --quiet --release --bin summonerd -- init --storage-dir /tmp/summonerd --phase1-root phase1.bin

echo "Starting phase 1 run..."
cargo run --quiet --release --bin summonerd -- start --phase 1 --storage-dir /tmp/summonerd --fvk $SUMMONER_FVK --node https://grpc.testnet-preview.penumbra.zone &
phase1_pid="$!"
# If script ends early, ensure phase 1 is halted.
trap 'kill -9 "$phase1_pid"' EXIT

echo "Setting up test accounts..."
# We are returning 0 always here because the backup wallet file does not respect the location of
# the home directory, and so if there is already a backup wallet, we refuse to overwrite it,
# and will exit non-zero. We don't care about the backup wallet for this test, so we ignore the
# exit code.
echo $SEED_PHRASE | cargo run --quiet --release --bin pcli -- --home /tmp/account1 keys import phrase || true
export ACCOUNT1_ADDRESS=$(PCLI_UNLEASH_DANGER="yes" cargo run --quiet --release --bin pcli -- --home /tmp/account1 --node https://grpc.testnet-preview.penumbra.zone view address 0 2>&1)

echo "Phase 1 contributions..."
cargo run --quiet --release --bin pcli -- --node https://grpc.testnet-preview.penumbra.zone --home /tmp/account1 ceremony contribute --coordinator-url http://127.0.0.1:8081 --coordinator-address $SUMMONER_ADDRESS --phase 1 --bid 10penumbra

echo "Stopping phase 1 run..."
if ! kill -0 "$phase1_pid" ; then
    >&2 echo "ERROR: phase 1 exited early"
    kill -9 "$phase1_pid"
    exit 1
else
    echo "Phase 1 complete."
fi

echo "Transitioning..."
cargo run --quiet --release --bin summonerd -- transition --storage-dir /tmp/summonerd

echo "Starting phase 2 run..."
cargo run --quiet --release --bin summonerd -- start --phase 2 --storage-dir /tmp/summonerd --fvk $SUMMONER_FVK --node https://grpc.testnet-preview.penumbra.zone &
phase2_pid="$!"
# If script ends early, ensure phase 2 is halted.
trap 'kill -9 "$phase2_pid"' EXIT

echo "Phase 2 contributions..."
cargo run --quiet --release --bin pcli -- --node https://grpc.testnet-preview.penumbra.zone --home /tmp/account1 ceremony contribute --coordinator-url http://127.0.0.1:8081 --coordinator-address $SUMMONER_ADDRESS --phase 2 --bid 10penumbra

# TODO: Export keys

echo "Stopping phase 2 run..."
if ! kill -0 "$phase2_pid" ; then
    >&2 echo "ERROR: phase 2 exited early"
    kill -9 "$phase2_pid"
    exit 1
else
    echo "Phase 2 complete."
fi

rm -rf /tmp/summonerd
rm -rf /tmp/account1
exit 0
