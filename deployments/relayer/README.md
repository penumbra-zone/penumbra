# relayer config

Stores config generation scripts for use with the [relayer](https://github.com/cosmos/relayer),
for IBC functionality. Prior to mainnet, we use `relayer` to synchronize actions
from testnet-preview to testnet.

## Running it

```
./generate-configs preview
./generate-configs testnet
./configure-relayer penumbra-testnet penumbra-preview penumbra_path
./configure-relayer penumbra-preview babylon-testnet babylon

./run-relayer
```

## Further reading
The config format for the JSON files are adapted from the [example-configs](https://github.com/cosmos/relayer/tree/main/docs/example-configs)
in the relayer repo. Our configs will get out of date very quickly: the preview chain id changes
on every merge into main, for instance. Short-term, that's fine: we want to exercise IBC
in our CI deployments, and dynamically generating the configs is good enough. Longer term, we'll want
to upload our configs to the [chain-registry repo](https://github.com/cosmos/chain-registry).
Full documentation on the underlying steps used by the relayer can be found in the
[cross-chain docs](https://github.com/cosmos/relayer/blob/main/docs/create-path-across-chain.md).
