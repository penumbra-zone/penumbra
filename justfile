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

# Run smoke test suite, via process-compose config.
smoke:
    # resetting network state
    cargo run --release --bin pd -- testnet unsafe-reset-all || true
    ./deployments/scripts/smoke-test.sh
