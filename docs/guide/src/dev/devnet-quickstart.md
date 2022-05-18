# Devnet Quickstart

This page describes a quickstart method for running `pd`+`tendermint` to test
changes during development.

To start, you'll need to [install Tendermint `v0.35`](https://docs.tendermint.com/v0.35/introduction/install.html).

## Generating configs

To generate a clean set of configs, run

```
cargo run --release --bin pd -- generate-testnet
```

This will write configs to `~/.penumbra/testnet_data/`.

## Running `pd`

You'll probably want to set `RUST_LOG`.  Here's one suggestion:

```
export RUST_LOG="warn,pd=debug,penumbra=debug,jmt=info"
```

To run `pd`, run

```
cargo run --release --bin pd -- start  -r ~/.penumbra/testnet_data/node0/pd/rocksdb
```

This will start but won't do anything yet, because Tendermint isn't running.

## Running `tendermint`

To run Tendermint, run

```
tendermint --home ~/.penumbra/testnet_data/node0/tendermint/ start
```

in another terminal window.

## Running `pcli`

To interact with the chain, first do

```
cargo run --release --bin pcli -- wallet reset
```

and then pass the `-n` flag to any commands you run to point `pcli` at your local node, e.g.,

```
cargo run --bin pcli -- -n 127.0.0.1 balance
```

By default, `pd generate-testnet` uses the latest snapshot of the Discord's
faucet channel, so if you posted your address there more than a week or two ago,
you should already have an allocation in your local devnet.

If not, reset the state as below, and edit the `genesis.json` to add your address.

## Resetting and restarting

After making changes, you may want to reset and restart the devnet.  This is a
two-step process, so be sure to do both steps.

First, wipe the `pd` state:

```
rm -rf ~/.penumbra/testnet_data/node0/pd/rocksdb
```

Next, wipe the `tendermint` state:

```
tendermint --home ~/.penumbra/testnet_data/node0/tendermint/ unsafe-reset-all
```

At this point you're ready to restart both `pd` and `tendermint`.  The order
they're started in doesn't particularly matter for correctness, because
`tendermint` will retry connecting to the ABCI server until it succeeds.

You'll probably also want to reset your wallet state:

```
cargo run --release --bin pcli -- wallet reset
```
