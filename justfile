# Prints the list of recipes.
default:
    @just --list

# Creates and runs a local devnet with solo validator. Includes ancillary services
# like metrics, postgres for storing ABCI events, and pindexer for munging those events.
dev:
    ./deployments/scripts/check-nix-shell && \
        ./deployments/scripts/run-local-devnet.sh \
        --keep-project \
        --config ./deployments/compose/process-compose-postgres.yml \
        --config ./deployments/compose/process-compose-metrics.yml \
        --config ./deployments/compose/process-compose-dev-tooling.yml

# Formats the rust files in the project.
fmt:
    cargo fmt --all

# warms the rust cache by building all targets
build:
    cargo build --release --all-features --all-targets

# Runs 'cargo check' on all rust files in the project.
check:
  # check, failing on warnings
  RUSTFLAGS="-D warnings" cargo check --release --all-targets --all-features --target-dir=target/check
  # fmt dry-run, failing on any suggestions
  cargo fmt --all -- --check

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

# Run rust unit tests, via cargo-nextest
test:
  cargo nextest run --release

# Run integration tests against the testnet, for validating HTTPS support
integration-testnet:
  cargo nextest run --release --features integration-testnet -E 'test(/_testnet$/)'

# Run integration tests for pmonitor tool
integration-pmonitor:
  ./deployments/scripts/warn-about-pd-state
  rm -rf /tmp/pmonitor-integration-test
  # Prebuild binaries, so they're available inside the tests without blocking.
  cargo build --release --bin pcli --bin pd --bin pmonitor
  cargo -q run --release --bin pd -- --help > /dev/null
  cargo nextest run --release -p pmonitor --features network-integration --no-capture --no-fail-fast

# Run smoke test suite, via process-compose config.
smoke:
  ./deployments/scripts/check-nix-shell
  ./deployments/scripts/warn-about-pd-state
  ./deployments/scripts/smoke-test.sh

# Run integration tests for pclientd. Assumes specific dev env is already running.
integration-pclientd:
  cargo test --release --features sct-divergence-check --package pclientd -- \
    --ignored --test-threads 1 --nocapture

# Run integration tests for pcli. Assumes specific dev env is already running.
integration-pcli:
  cargo test --release --features sct-divergence-check,download-proving-keys --package pcli -- \
    --ignored --test-threads 1 --nocapture

# Run integration tests for pindexer. Assumes specific dev env is already running.
integration-pindexer:
  cargo nextest run --release -p pindexer --features network-integration

# Run integration tests for pd. Assumes specific dev env is already running.
integration-pd:
  cargo test --release --package pd -- --ignored --test-threads 1 --nocapture

# Build the container image locally
container:
  podman build -t ghcr.io/penumbra-zone/penumbra -f ./deployments/containerfiles/Dockerfile .
