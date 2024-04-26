smoke:
    # resetting network state
    cargo run --release --bin pd -- testnet unsafe-reset-all || true
    ./deployments/scripts/smoke-test.sh
