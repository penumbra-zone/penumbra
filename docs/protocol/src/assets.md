# Assets and Values

Penumbra can record arbitrary assets.  These assets either originate on Penumbra
itself, or, more commonly, originate on other IBC-connected chains.  To record
arbitrary assets and enforce value balance between them, we draw on
[ideas][multi_asset] originally proposed for Zcash and adapt them to the Cosmos
context.

## Asset types and asset IDs

To be precise, we define:

- an *amount* to be an untyped `u128` quantity of some asset;
- an *asset ID* to be an $\mathbb F_q$ element;
- a *value* to be a typed quantity, i.e., an amount and an asset ID.

All asset IDs are currently computed as the hash of a *denomination*, an
[ADR001]-style denomination trace uniquely identifying a cross-chain asset and
its provenance, such as:
- `denom` (native chain A asset)
- `transfer/channelToA/denom` (chain B representation of chain A asset)
- `transfer/channelToB/transfer/channelToA/denom` (chain C representation of chain B representation of chain A asset)

However, Penumbra deviates slightly from ADR001 in the definition of the asset
ID. While ADR001 defines the IBC asset ID as the SHA-256 hash of the
denomination trace, Penumbra hashes to a field element, so that asset IDs can be
more easily used inside of a zk-SNARK circuit.  Specifically, define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order, and `hash(label, input)` as BLAKE2b-512 with
personalization `label` on input `input`.  Then asset IDs are computed as
```
asset_id = from_le_bytes(hash(b"Penumbra_AssetID", asset_type)) mod q
```

In the future, Penumbra may have other asset IDs do not correspond to
denominations, but are computed as hashes of other state data.  By making the
asset ID itself be a hash of extended state data, a note recording value of that
type also binds to that extended data, even though it has the same size as any
other note. Currently, however, all asset IDs are computed as the hashes of
denomination strings.

## Asset Metadata

Penumbra also supports Cosmos-style `Metadata` for assets. The chain maintains
an on-chain lookup table of asset IDs to asset metadata, but the on-chain
metadata is minimal and generally only includes the denomination string.  Client
software is expected to be opinionated about asset metadata, supplying
definitions with symbols, logos, etc. to help users understand the assets they
hold.

## Value Generators

Each asset ID $\mathsf a$ has an associated *value generator* $V_{\mathsf a} \in
\mathbb G$.  The value generator is computed as $V_{\mathsf a} = H_{\mathbb
G}^{\mathsf v}(\mathsf a)$, where $H_{\mathbb G}^{\mathsf v} : \mathbb F_q
\rightarrow \mathbb G$ is a hash-to-group function constructed by first applying
rate-1 Poseidon hashing with domain separator
`from_le_bytes(blake2b(b"penumbra.value.generator"))` and then the `decaf377` CDH
map-to-group method.

## Homomorphic Balance Commitments

We use the value generator associated to an asset ID to construct additively homomorphic
commitments to (typed) value.  To do this, we first define the *blinding
generator* $\widetilde{V}$ as
```
V_tilde = decaf377_encode_to_curve(from_le_bytes(blake2b(b"decaf377-rdsa-binding")))
```

The commitment to value $(v, \mathsf a)$, i.e., amount $v$ of asset $\mathsf a$,
with random blinding factor $\widetilde{v}$, is the Pedersen commitment
$$
\operatorname {Commit}_{\mathsf a}(v, \widetilde{v}) = [v]V_{\mathsf a} + [\widetilde{v}]\widetilde{V}.
$$

These commitments are homomorphic, even for different asset types, say values
$(x, \mathsf a)$ and $(y, \mathsf b)$:
$$
([x]V_{\mathsf a} + [\widetilde{x}]\widetilde{V}) + ([y] V_{\mathsf b} + [\widetilde{y}]\widetilde{V})
=
[x]V_{\mathsf a} + [y] V_{\mathsf b} + [\widetilde{x} + \widetilde{y}]\widetilde{V}.
$$

where $V_{\mathsf b}$ and $V_{\mathsf a}$ are generators of the group $\mathbb G$, and $\widetilde{x}$ and $\widetilde{y}$ are random blinding factors from $\mathbb F_r$.

Alternatively, this can be thought of as a commitment to a (sparse) vector
recording the amount of every possible asset type, almost all of whose
coefficients are zero.

[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
