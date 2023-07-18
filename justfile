smoke:
    cargo run --release --bin pd -- testnet unsafe-reset-all
    ./deployments/scripts/smoke-test.sh
