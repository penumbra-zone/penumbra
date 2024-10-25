# Prints the list of recipes.
default:
    @just --list

# Run integration tests for pmonitor tool
test-pmonitor:
  # prebuild cargo binaries required for integration tests
  cargo -q build --package pcli --package pd --package pmonitor
  cargo -q run --release --bin pd -- network unsafe-reset-all
  rm -rf /tmp/pmonitor-integration-test
  cargo nextest run -p pmonitor --run-ignored=ignored-only --test-threads 1
  # cargo test -p pmonitor -- --ignored --test-threads 1 --nocapture

# Creates and runs a local devnet with solo validator. Includes ancillary services
# like metrics, postgres for storing ABCI events, and pindexer for munging those events.
dev:
    ./deployments/scripts/check-nix-shell && \
        ./deployments/scripts/run-local-devnet.sh \
        --config ./deployments/compose/process-compose-postgres.yml \
        --config ./deployments/compose/process-compose-metrics.yml \
        --config ./deployments/compose/process-compose-dev-tooling.yml

# Formats the rust files in the project.
fmt:
    cargo fmt --all

# Runs 'cargo check' on all rust files in the project.
check:
  RUSTFLAGS="-D warnings" cargo check --release --all-targets

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
