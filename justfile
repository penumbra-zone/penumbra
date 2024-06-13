# Prints the list of recipes.
default:
    @just --list

# Formats the rust files in the project.
fmt:
    cargo fmt --all

# Generate code for Rust & Go from proto definitions.
proto:
    ./deployments/scripts/protobuf-codegen

# Run a local prometheus/grafana setup, in containers, to scrape a local node. Linux only.
metrics:
    cd ./deployments/compose/ \
        && docker-compose -f metrics.yml up --build --abort-on-container-exit --force-recreate --remove-orphans

# Configures and runs a relayer instance between "preview" (latest main) and local devnet on current HEAD
relayer-local-devnet:
    ./deployments/scripts/relayer-local-devnet

local-devnet-generate:
    cargo run --release --bin pd -- testnet generate --chain-id penumbra-devnet-local

local-devnet-run:
    ./deployments/scripts/run-local-devnet.sh

local-devnet-reset-all:
    cargo run --bin pd --release -- testnet unsafe-reset-all

# Rebuild Rust crate documentation
rustdocs:
    ./deployments/scripts/rust-docs

# Run smoke test suite, via process-compose config.
smoke:
    # resetting network state
    cargo run --release --bin pd -- testnet unsafe-reset-all || true
    ./deployments/scripts/smoke-test.sh
