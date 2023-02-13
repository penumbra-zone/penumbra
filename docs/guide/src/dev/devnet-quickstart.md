# Devnet Quickstart

This page describes a quickstart method for running `pd`+`tendermint` to test
changes during development.

To start, you'll need to [install Tendermint `v0.34`](../pd/build.md#installing-tendermint).

## Generating configs

To generate a clean set of configs, run

```shell
pd testnet generate
```

This will write configs to `~/.penumbra/testnet_data/`.

## Running `pd`

You'll probably want to set `RUST_LOG`.  Here's one suggestion:

```shell
export RUST_LOG="warn,pd=debug,penumbra=debug,jmt=info"
```

To run `pd`, run

```shell
pd start  --home ~/.penumbra/testnet_data/node0/pd
```

This will start but won't do anything yet, because Tendermint isn't running.

## Running `tendermint`

To run Tendermint, run

```shell
tendermint --home ~/.penumbra/testnet_data/node0/tendermint/ start
```

in another terminal window.

## Running `pcli`

To interact with the chain, first do

```shell
pcli view reset
```

and then pass the `-n` flag to any commands you run to point `pcli` at your local node, e.g.,

```shell
pcli -n 127.0.0.1 view balance
```

By default, `pd testnet generate` uses the latest snapshot of the Discord's
faucet channel, so if you posted your address there more than a week or two ago,
you should already have an allocation in your local devnet.

If not, reset the state as below, and edit the `genesis.json` to add your address.

## Resetting and restarting

After making changes, you may want to reset and restart the devnet:

```shell
pd testnet unsafe-reset-all
```

You'll probably also want to reset your wallet state:

```shell
pcli view reset
```

At this point you're ready to generate new configs, and restart both `pd` and
`tendermint`.  The order they're started in doesn't particularly matter for
correctness, because `tendermint` will retry connecting to the ABCI server until
it succeeds.

## Optional: running smoke-tests

Once you have a working devnet running, you should be able to run the [smoke tests](https://en.wikipedia.org/wiki/Smoke_testing_(software)) successfully. This can be useful if you are looking to contribute to Penumbra, or if you need to check that your setup is correct.

To run the smoke tests:

1. Make sure you have a devnet running (see previous steps)
2. Run integration tests:
```shell
PENUMBRA_NODE_HOSTNAME=127.0.0.1 PCLI_UNLEASH_DANGER=yes cargo test --package pcli -- --ignored --test-threads 1
```
