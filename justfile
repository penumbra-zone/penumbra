smoke:
    # resetting network state
    cargo run --release --bin pd -- testnet unsafe-reset-all || true
    ./deployments/scripts/smoke-test.sh

go:
    # stop services
    systemctl --user stop penumbra cometbft

    # building source
    cargo build --release --bin pd --bin pcli

    # create network
    cargo run --bin pd --release -- testnet unsafe-reset-all
    cargo run --bin pd --release -- testnet generate

    # start services
    systemctl --user daemon-reload
    systemctl --user restart cometbft
    RUST_LOG=warn cargo run --bin pd --release -- start
