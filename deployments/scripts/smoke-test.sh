#!/usr/bin/env bash
# Run smoke test suite, via process-compose config.
set -euo pipefail


# Fail fast if network dir exists, otherwise `cargo run ...` will block
# for a while, masking the error.
#
# If any network data is present, we shouldn't reuse it: the smoke tests assume
# a fresh devnet has been created specifically for the test run. In the future
# we should make this a temp dir so it can always run regardless of pre-existing state.
repo_root="$(git rev-parse --show-toplevel)"
"${repo_root}/deployments/scripts/warn-about-pd-state"

# Check for dependencies. All of these will be installed automatically
# as part of the nix env.
if ! hash cometbft > /dev/null 2>&1 ; then
    >&2 echo "ERROR: cometbft not found in PATH"
    >&2 echo "See install guide: https://guide.penumbra.zone/main/pd/build.html"
    exit 1
fi

if ! hash process-compose > /dev/null 2>&1 ; then
    >&2 echo "ERROR: process-compose not found in PATH"
    >&2 echo "Install it via https://github.com/F1bonacc1/process-compose/"
    exit 1
fi

if ! hash grpcurl > /dev/null 2>&1 ; then
    >&2 echo "ERROR: grpcurl not found in PATH"
    >&2 echo "Install it via https://github.com/fullstorydev/grpcurl/"
    exit 1
fi

>&2 echo "Building all test targets before running smoke tests..."
# We want a warm cache before the tests run
cargo build --release --bins

smoke_test_dir="${repo_root:?}/deployments/.smoke-test-state"
rm -rf "$smoke_test_dir"
mkdir -p "$smoke_test_dir"

# Reuse existing dev-env script
"${repo_root}/deployments/scripts/run-local-devnet.sh" \
    --config ./deployments/compose/process-compose-metrics.yml \
    --config ./deployments/compose/process-compose-dev-tooling.yml \
    --config ./deployments/compose/process-compose-postgres.yml \
    --detached

# Wait a bit for network to start.
sleep 10

# Ensure that process-compose environment gets cleaned up, even if tests error.
trap 'process-compose down --port 8888' EXIT

# Wait for the network to be fully ready by checking block height.
# We need at least one block to be produced before the chain state is queryable.
>&2 echo "Waiting for network to produce blocks..."
max_attempts=120
attempt=0
while true; do
    # Query the latest block height via the tendermint RPC
    height_response=$(curl -s http://127.0.0.1:26657/status 2>&1) || true
    # Extract the block height from the JSON response
    height=$(echo "$height_response" | grep -o '"latest_block_height":"[0-9]*"' | grep -o '[0-9]*' | head -1) || true

    if [ -n "$height" ] && [ "$height" -gt 0 ] 2>/dev/null; then
        >&2 echo "  Block height: $height"
        break
    fi

    attempt=$((attempt + 1))
    if [ $attempt -ge $max_attempts ]; then
        >&2 echo "ERROR: Network did not produce blocks within timeout"
        >&2 echo "Status response: $height_response"
        exit 1
    fi
    >&2 echo "  Waiting for blocks (attempt $attempt/$max_attempts)..."
    sleep 1
done
>&2 echo "Network is producing blocks."
# Wait for a few more blocks to ensure state is fully committed
sleep 10

# Register test assets in the compliance registry before running tests.
# The compliance system requires all assets to be registered (regulated or unregulated)
# before they can be transferred. Note: the staking token (penumbra) is auto-registered
# as unregulated at genesis, so we only need to register other test assets here.
>&2 echo "Registering test assets in compliance registry..."
# Initialize a test wallet for registration transactions
pcli_test_home="${smoke_test_dir}/pcli-test"
mkdir -p "$pcli_test_home"
echo "comfort ten front cycle churn burger oak absent rice ice urge result art couple benefit cabbage frequent obscure hurry trick segment cool job debate" | \
    cargo run --release --bin pcli -- --home "$pcli_test_home" init --grpc-url "http://127.0.0.1:8080" soft-kms import-phrase

# Register gm as unregulated (penumbra is auto-registered at genesis)
cargo run --release --bin pcli -- --home "$pcli_test_home" tx compliance register-asset gm --unregulated
>&2 echo "Asset registration complete."

# Run the integration tests. Using `just` targets so that the exact
# invocations are easily reusable on the CLI in dev loops.
just integration-pclientd
just integration-pcli
# The pd tests come later, as they need work to have been performed for metrics to be emitted.
just integration-pd
# Finally, pindexer tests, to make assertions about emitted events.
just integration-pindexer
