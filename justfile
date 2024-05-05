# Prints the list of recipes.
default:
    @just --list

# Formats the rust files in the project.
fmt:
    cargo fmt --all

# Generate code for Rust & Go from proto definitions.
proto:
    ./deployments/scripts/protobuf-codegen

# Configures and runs a relayer instance between "preview" (latest main) and local devnet on current HEAD
relayer-local-devnet:
    ./deployments/scripts/relayer-local-devnet

# Rebuild Rust crate documentation
rustdocs:
    ./deployments/scripts/rust-docs

# Spin up a local dev environment of pd and cometbft
dev:
    process-compose up --port 9191 --config ./deployments/compose/process-compose.yml --keep-tui

# Perform a chain upgrade on a local devnet
migration-test:
    ./deployments/scripts/warn-about-pd-state
    ./deployments/scripts/migration-test v0.76.0

# Run the smoke test suite, integration tests for client binaries
smoke:
    ./deployments/scripts/warn-about-pd-state
    ./deployments/scripts/smoke-test.sh

# Add a new local node to an already-running devnet
add-node:
    process-compose up --no-server \
        --config ./deployments/compose/process-compose-bootstrap-local-node.yml --keep-tui
