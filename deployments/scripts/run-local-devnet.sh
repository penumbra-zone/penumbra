#!/usr/bin/env bash
# Dev tooling to spin up a localhost devnet for Penumbra.
set -euo pipefail


repo_root="$(git rev-parse --show-toplevel)"
# The process-compose file already respects local state and will reuse it.
# "${repo_root}/deployments/scripts/warn-about-pd-state"

>&2 echo "Building binaries from latest code..."
cargo build --release --bin pd
# Also make sure to invoke via `cargo run` so that the process-compose
# spin-up doesn't block on more building/linking.
cargo --quiet run --release --bin pd -- --help > /dev/null

# Generate network from latest code, only if network does not already exist.
if [[ -d ~/.penumbra/network_data ]] ; then
    >&2 echo "network data exists locally, reusing it"
else
    # Use allocations CSV file that includes:
    # - Dev wallet address for manual usage
    # - test_keys::ADDRESS_0 for pclientd/pcli integration tests
    # Use single-validator config so the network can produce blocks with just one node.
    # The default generates 2 validators, requiring 2/3+ voting power (i.e., both nodes).
    cargo run --release --bin pd -- network generate \
        --chain-id penumbra-local-devnet \
        --unbonding-delay 302400 \
        --epoch-duration 302400 \
        --proposal-voting-blocks 50 \
        --gas-price-simple 0 \
        --timeout-commit 500ms \
        --allocations-input-file "${repo_root}/deployments/compose/devnet-allocations.csv" \
        --validators-input-file "${repo_root}/testnets/validators-single.json"
    # opt in to cometbft abci indexing to postgres
    postgresql_db_url="postgresql://penumbra:penumbra@localhost:5432/penumbra_cometbft?sslmode=disable"
    sed -i -e "s#^indexer.*#indexer = \"psql\"\\npsql-conn = \"$postgresql_db_url\"#" ~/.penumbra/network_data/node0/cometbft/config/config.toml
fi

# Check for interactive terminal session, enable TUI if yes.
if [[ -t 1 ]] ; then
    use_tui="true"
else
    use_tui="false"
fi

# Set unique API port for controlling running services.
export PC_PORT_NUM="8888"

# Run the core fullnode config, plus any additional params passed via `$@`.
process-compose up \
    --ordered-shutdown \
    --tui="$use_tui" \
    --config "${repo_root}/deployments/compose/process-compose.yml" \
    "$@"
