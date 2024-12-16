# pindexer

An indexer that ingests ABCI events emitted by pd and transforms them into useful data.

## Usage

1. Follow the setup instructions in cometindex README
2. `cargo run --bin pindexer -- -s "postgresql://localhost:5432/testnet_raw?sslmode=disable" -d "postgresql://localhost:5432/testnet_compiled?sslmode=disable"`

## pd compatibility

