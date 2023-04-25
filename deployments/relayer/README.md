# relayer config

Stores config generation scripts for use with the [relayer](https://github.com/cosmos/relayer),
for IBC functionality. Prior to mainnet, we plan to use `relayer` to synchronize actions
from testnet-preview to testnet. During 2023Q2, we're focusing on relaying between
testnet-preview and a local devnet on the same or similar commit.

## Running a local devnet
To create a path between the public testnet-preview chain and a local devnet:

1. Run `./deployments/scripts/relayer-local-devnet` to bootstrap the local chain.
2. Wait until the message "OK, devnet is up and running" is printed.
3. In another terminal, `cd deployments/relayer` and  `./build-path`.

The `pd` logs visible from the `relayer-local-devnet` are intentionally verbose,
to aid in debugging the creation of clients, connections, and channels. You may
wish to add more tracing statements to your local copy of `pd`.

## Building a path between testnet & preview
Inside this directory, run:

```
./generate-configs preview
./generate-configs testnet
./configure-relayer

./build-path
```
Or, you can use `just` to run it all, soup to nuts. See the path configuration
block in the `./configure-relayer` script for some example paths between chains
that aren't yet known to work.

Given the rapid pace of development, it's possible that proto definitions
are out of sync between testnet & preview, in which case there may be errors.
To debug, consider running a local devnet and linking it with preview.

## Updating proto definitions in relayer
Sometimes the protos between preview & testnet get out of sync. When this happens,
we must submit a PR upstream to the relayer repo. See [example here](https://github.com/cosmos/relayer/pull/1170),
along with instructions on the commands to generate the protos for the golang repo.
Until the protos are back in sync, relaying between Penumbra chains may not work.

## Further reading
The config format for the JSON files are adapted from the [example-configs](https://github.com/cosmos/relayer/tree/main/docs/example-configs)
in the relayer repo. Our configs will get out of date very quickly: the preview chain id changes
on every merge into main, for instance. Short-term, that's fine: we want to exercise IBC
in our CI deployments, and dynamically generating the configs is good enough. Longer term, we'll want
to upload our configs to the [chain-registry repo](https://github.com/cosmos/chain-registry).
Full documentation on the underlying steps used by the relayer can be found in the
[cross-chain docs](https://github.com/cosmos/relayer/blob/main/docs/create-path-across-chain.md).
