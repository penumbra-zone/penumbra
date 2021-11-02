# Assets and Amounts

Penumbra's shielded pool can record arbitrary assets.  To be precise, we define:

- an *amount* to be an untyped quantity of some asset;
- an *asset type* to be an [ADR001]-style denomination trace uniquely identifying a cross-chain asset and its provenance, such as:
  - `denom` (native chain A asset)
  - `transfer/channelToA/denom` (chain B representation of chain A asset)
  - `transfer/channelToB/transfer/channelToA/denom` (chain C representation of chain B representation of chain A asset)
- an *asset id* to be a fixed-size hash of an asset type;
- a *value* to be a typed quantity, i.e., an amount and an asset id.

Penumbra deviates slightly from ADR001 in the definition of the asset id. While
ADR001 defines the IBC asset ID as the SHA-256 hash of the denomination trace,
Penumbra hashes to a field element, so that asset IDs can be more easily used
inside of a zk-SNARK circuit.

[ADR001]: https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md
