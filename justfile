go:
    # shutting down services
    sudo systemctl stop cometbft penumbra
    cargo run --bin pd -- testnet unsafe-reset-all
    cargo run --bin pd -- testnet generate
    cargo build -r

    # restarting services
    sudo systemctl restart cometbft penumbra
    sleep 10

    # make a request, so we get log messages
    cargo run --bin pcli -- --home ~/.local/share/pcli-localhost view reset || true
    cargo run --bin pcli -- --home ~/.local/share/pcli-localhost view balance
