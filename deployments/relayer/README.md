# relayer config

Stores config generation scripts for use with the [relayer](https://github.com/cosmos/relayer),
for IBC functionality. Prior to mainnet, we use `relayer` to synchronize actions
from testnet-preview to testnet.

## Running it

```
./generate-configs preview
./generate-configs testnet
./configure-relayer

./run-relayer
```
Or, you can use `just` to run it all, soup to nuts. See the path configuration
block in the `./configure-relayer` script for some example paths between chains
that aren't yet known to work.

## Updating proto definitions in relayer
Sometimes the protos between preview & testnet get out of sync. When this happens,
we must submit a PR upstream to the relayer repo. See [example here](https://github.com/cosmos/relayer/pull/1170),
along with instructions on the commands to generate the protos for the golang repo.
Until the protos are back in sync, relaying between Penumbra chains may not work.

## Running a local devnet
By default the relayer scripts configure a path between testnet and preview.
For debugging, it can be useful to use a path between a local devnet and preview,
potentially even on the same git commit. See the instructions in [GH 2252](https://github.com/penumbra-zone/penumbra/issues/2252)
for details on how to set it up. You may need to edit the chain ids in `./configure-relayer`.

## Further reading
The config format for the JSON files are adapted from the [example-configs](https://github.com/cosmos/relayer/tree/main/docs/example-configs)
in the relayer repo. Our configs will get out of date very quickly: the preview chain id changes
on every merge into main, for instance. Short-term, that's fine: we want to exercise IBC
in our CI deployments, and dynamically generating the configs is good enough. Longer term, we'll want
to upload our configs to the [chain-registry repo](https://github.com/cosmos/chain-registry).
Full documentation on the underlying steps used by the relayer can be found in the
[cross-chain docs](https://github.com/cosmos/relayer/blob/main/docs/create-path-across-chain.md).
