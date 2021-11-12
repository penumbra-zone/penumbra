# Value Commitments

Penumbra's shielded pool can record arbitrary assets.  These assets either
originate on Penumbra itself, or, more commonly, originate on other
IBC-connected chains.  To record arbitrary assets and enforce value balance
between them, we draw on [ideas][multi_asset] originally proposed for Zcash and
adapt them to the Cosmos context.

## Asset types and asset IDs

To be precise, we define: 

- an *amount* to be an untyped quantity of some asset;
- an *asset type* to be an [ADR001]-style denomination trace uniquely identifying a cross-chain asset and its provenance, such as:
  - `denom` (native chain A asset)
  - `transfer/channelToA/denom` (chain B representation of chain A asset)
  - `transfer/channelToB/transfer/channelToA/denom` (chain C representation of chain B representation of chain A asset)
- an *asset ID* to be a fixed-size hash of an asset type;
- a *value* to be a typed quantity, i.e., an amount and an asset id.

Penumbra deviates slightly from ADR001 in the definition of the asset ID. While
ADR001 defines the IBC asset ID as the SHA-256 hash of the denomination trace,
Penumbra hashes to a field element, so that asset IDs can be more easily used
inside of a zk-SNARK circuit.  Specifically, define `from_le_bytes(bytes)` as
the function that interprets its input bytes as an integer in little-endian
order, and `hash(label, input)` as BLAKE2b-512 with personalization `label` on
input `input`.  Then asset IDs are computed as
```
asset_id = from_le_bytes(hash(b"Penumbra_AssetID", asset_type)) mod q
```

Asset IDs are used in internal data structures, but users should be presented
with asset names.  To avoid having to reverse the hash function, the chain
maintains a lookup table of known asset IDs and the corresponding asset types.
This table can be exhaustive, since new assets either moved into the chain via
IBC transfers from a transparent zone, or were created at genesis.

## Value Generators

Each asset ID $\mathsf a$ has an associated *value generator* $V_{\mathsf a} \in
\mathbb G$.  The value generator is computed as $V_{\mathsf a} = H_{\mathbb
G}^{\mathsf v}(\mathsf a)$, where $H_{\mathbb G}^{\mathsf v} : \mathbb F_q
\rightarrow \mathbb G$ is a hash-to-group function constructed by first applying
rate-1 Poseidon hashing with domain separator
`from_le_bytes(b"penumbra.value.generator")` and then the `decaf377` CDH
map-to-group method.

## Homomorphic Commitments

We use the value generator associated to an asset ID to construct homomorphic
commitments to (typed) value.  To do this, we first define the *blinding
generator* $\tilde V$ as
```
V_tilde = decaf377_map_to_group_cdh(from_le_bytes(blake2b(b"decaf377-rdsa-binding")))
```

The commitment to value $(v, \mathsf a)$, i.e., amount $v$ of asset $\mathsf a$,
with blinding factor $\tilde v$, is the Pedersen commitment
$$
\operatorname {Commit}_{\mathsf a}(v, \tilde v) = [v]V_{\mathsf a} + [\tilde v]\tilde V.
$$

These commitments are homomorphic, even for different asset types, say values
$(x, \mathsf a)$ and $(y, \mathsf b)$:
$$
([x]V_{\mathsf a} + [\tilde x]\tilde V) + ([y] V_{\mathsf b} + [\tilde y]\tilde V)
= 
[x]V_{\mathsf a} + [y] V_{\mathsf b} + [\tilde x + \tilde y]\tilde V.
$$
Alternatively, this can be thought of as a commitment to a (sparse) vector
recording the amount of every possible asset type, almost all of whose
coefficients are zero.

## Binding Signatures

Finally, we'd like to be able to prove that a certain value commitment $C$ is a
commitment to $0$.  One way to do this would be to prove knowledge of an opening
to the commitment, i.e., producing $\tilde v$ such that $$C = [\tilde v] \tilde
V = \operatorname{Commit}(0, \tilde v).$$  But this is exactly what it means to
create a Schnorr signature for the verification key $C$, because a Schnorr
signature is a proof of knowledge of the signing key in the context of the
message. 

Therefore, we can prove that a value commitment is a commitment to $0$ by
treating it as a `decaf377-rdsa` verification key and using the corresponding
signing key (the blinding factor) to sign a message.  This also gives a way to
bind value commitments to a particular context (e.g., a transaction), by using
the context as the message to be signed, in order to, e.g., ensure that value
commitments cannot be replayed across transactions.




[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://docs.cosmos.network/master/architecture/adr-001-coin-source-tracing.html
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
