# Prints the list of recipes.
default:
    @just --list

# Creates and runs a local devnet with solo validator. Includes ancillary services
# like metrics, postgres for storing ABCI events, and pindexer for munging those events.
dev:
    ./deployments/scripts/check-nix-shell && \
        ./deployments/scripts/run-local-devnet.sh \
        --config ./deployments/compose/process-compose-postgres.yml \
        --config ./deployments/compose/process-compose-metrics.yml

# Formats the rust files in the project.
fmt:
    cargo fmt --all

# Render livereload environment for editing the Protocol documentation.
protocol-docs:
    # Access local docs at http://127.0.0.1:3002
    cd docs/protocol && \
        mdbook serve -n 127.0.0.1 --port 3002

# Generate code for Rust & Go from proto definitions.
proto:
    ./deployments/scripts/protobuf-codegen

# Run a local prometheus/grafana setup, to scrape a local node.
metrics:
    ./deployments/scripts/check-nix-shell && \
        process-compose --no-server --config ./deployments/compose/process-compose-metrics.yml up --keep-tui

# Rebuild Rust crate documentation
rustdocs:
    ./deployments/scripts/rust-docs

# Run smoke test suite, via process-compose config.
smoke:
    ./deployments/scripts/warn-about-pd-state
    ./deployments/scripts/smoke-test.sh
