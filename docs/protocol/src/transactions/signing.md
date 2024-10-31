# Transaction Signing

In a transparent blockchain, a signer can inspect the transaction to be signed to
validate the contents are what the signer expects. However, in a shielded
blockchain, the contents of the transaction are opaque. Ideally, using a private
blockchain would enable a user to sign a transaction while also understanding
what they are signing.

To avoid the blind signing problem, in the Penumbra protocol we allow the user
to review a description of the transaction - the `TransactionPlan` - prior to
signing. The `TransactionPlan` contains a declarative description of all details of the proposed transaction, including a plan of each action in a transparent
form, the fee specified, the chain ID, and so on. From this plan, we authorize the and build the transaction. This has the additional advantage of allowing the signer to authorize the
transaction while the computationally-intensive Zero-Knowledge
Proofs (ZKPs) are optimistically generated as part of the transaction build process.

The signing process first takes a `TransactionPlan` and `SpendKey` and returns
the `AuthorizationData`, essentially a bundle of signatures over
the *effect hash*, which can be computed directly from the plan data. You can
read more about the details of the effect hash computation below.

The building process takes the `TransactionPlan`, generates the proofs, and constructs a fully-
formed `Transaction`. This process is internally partitioned into three steps:
1. Each `Action` is individually built based to its specification in the `TransactionPlan`.
2. The pre-built actions to collectively used to construct a transaction with placeholder dummy signatures, that can be filled in once the
signatures from the `AuthorizationData` are ready[^1]. This intermediate state
of the transaction without the full authorizing data is referred to as the "**Unauthenticated Transaction**".
1. Slot the `AuthorizationData` to replace the placeholder signatures to assemble the final `Transaction`.

The Penumbra protocol was designed to only require the custodian, e.g. the hardware wallet
environment, to do signing, as the generation of ZKPs can be done without access to signing keys, requiring only witness data and viewing keys.

A figure showing how these pieces fit together is shown below:

```
╔════════════════════════╗
║         Authorization  ║
║                        ║
║┌──────────────────────┐║
║│ Spend authorization  │║
║│         key          │║    ┌───────────────────┐
║└──────────────────────┘║    │                   │
║                        ║───▶│ AuthorizationData │──┐
║                        ║    │                   │  │
║┌──────────────────────┐║    └───────────────────┘  │
║│      EffectHash      │║                           │
║└──────────────────────┘║                           │
║                        ║                           │
║                        ║                           │
╚════════════▲═══════════╝                           │
             │                                       │
             │                                       │               ┌───────────┐
 ┌───────────┴───────────┐                           │               │           │
 │                       │                           └─────────┬────▶│Transaction│
 │    TransactionPlan    │                                     |     │           │
 │                       │                                     │     └───────────┘
 └───────────┬───────────┘                                     │
             │                                                 │
             │                                                 │
             │                                                 │
 ╔═══════════▼════════════╗                                    │
 ║                Proving ║                                    │
 ║                        ║                                    │
 ║┌──────────────────────┐║                                    │
 ║│     WitnessData      │║                                    │
 ║└──────────────────────┘║   ┌──────────────────────────────┐ │
 ║                        ║   │                              │ │
 ║                        ╠──▶│ Unauthenticated Transaction  ├─┘
 ║┌──────────────────────┐║   │                              │
 ║│   Full viewing key   │║   └──────────────────────────────┘
 ║└──────────────────────┘║
 ║                        ║
 ║                        ║
 ║                        ║
 ╚════════════════════════╝

```

Transactions are signed used the [`decaf377-rdsa` construction](../crypto/decaf377-rdsa.md). As described briefly in that section, there are two signature domains used in Penumbra: `SpendAuth` signatures and `Binding` signatures.

## `SpendAuth` Signatures

`SpendAuth` signatures are included on each `Spend` and `DelegatorVote` action
(see [Multi-Asset Shielded Pool](../shielded_pool.md) and [Governance](../governance.md)
for more details on `Spend` and `DelegatorVote` actions respectively).

The `SpendAuth` signatures are created using a randomized signing key $rsk$ and the corresponding randomized verification key $rk$ provided on the action. The purpose of the randomization is to prevent linkage of verification keys across actions.

The `SpendAuth` signature is computed using the `decaf377-rdsa` `Sign` algorithm
where the message to be signed is the *effect hash* of the entire transaction
(described below), and the `decaf377-rdsa` domain is `SpendAuth`.

## Effect Hash

The effect hash is computed over the *effecting data* of the transaction, which following
the terminology used in Zcash[^2]:

> "Effecting data" is any data within a transaction that contributes to the effects of applying the transaction to the global state (results in previously-spendable coins or notes becoming spent, creates newly-spendable coins or notes, causes the root of a commitment tree to change, etc.).

The data that is _not_ effecting data is *authorizing data*:

>"Authorizing data" is the rest of the data within a transaction. It does not contribute to the effects of the transaction on global state, but allows those effects to take place. This data can be changed arbitrarily without resulting in a different transaction (but the changes may alter whether the transaction is allowed to be applied or not).

For example, the nullifier on a `Spend` is effecting data, whereas the
proofs or signatures associated with the `Spend` are authorizing data.

In Penumbra, the effect hash of each transaction is computed using the BLAKE2b-512
hash function. The effect hash is derived from the proto-encoding of the action - in
cases where the effecting data and authorizing data are the same, or the *body*
of the action - in cases where the effecting data and authorizing data are different.
Each proto has a unique string associated with it called its *Type URL*,
which is included in the inputs to BLAKE2b-512.
Type URLs are variable length, so a fixed-length field (8 bytes) is first included
in the hash to denote the length of the Type URL field.

Summarizing the above, the effect hash _for each action_ is computed as:

```
effect_hash = BLAKE2b-512(len(type_url) || type_url || proto_encode(proto))
```

where `type_url` is the bytes of the variable-length Type URL, `len(type_url)` is the length of the Type URL encoded as 8
bytes in little-endian byte order, `proto` represents the proto used to represent
the effecting data, and `proto_encode` represents encoding the proto message as
a vector of bytes. In Rust, the Type URL is found by calling [`type_url()` on the protobuf
message](https://docs.rs/prost/latest/prost/trait.Name.html#method.type_url).

All transaction data field effect hashes, such as the `Fee`, `MemoCiphertext`, and `TransactionParameters`, as well as the per-action effect hashes, are computed using this method.

### Transaction Effect Hash

To compute the effect hash of the _entire transaction_, we combine the hashes of the individual fields in the transaction body. First we include the fixed-sized effect hashes of the per-transaction data fields: the transaction parameters `eh(tx_params)`, fee `eh(fee)`, (optional) detection data `eh(detection_data)`, and (optional) memo `eh(memo)` which are derived as described above. Then, we include the number of actions $j$ and the fixed-size effect hash of each action `a_0` through `a_j`. Combining all fields:

```
effect_hash = BLAKE2b-512(len(type_url) || type_url || eh(tx_params) || eh(fee) || eh(memo) || eh(detection_data) || j || eh(a_0) || ... || eh(a_j))
```

where the `type_url` is the variable-length Type URL of the transaction body message, and `len(type_url)` is the length of that string encoded as 8 bytes in little-endian byte order.

Test vectors for the effect hash computation for 100 randomly generated `TransactionPlan`s are available [here](https://github.com/penumbra-zone/penumbra/tree/main/crates/core/transaction/tests/signing_test_vectors). You can also use a tool in that same repository to re-generate those test vectors or generate additional random test vectors via:

```
cargo test -- --ignored --test generate_transaction_signing_test_vectors
```

## `Binding` Signature

The `Binding` signature is computed once on the transaction level.
It is a signature over a hash of the authorizing data of that transaction, called the *auth hash*. The auth hash of each transaction is computed using the BLAKE2b-512 hash function over the proto-encoding of the _entire_ `TransactionBody`.

The `Binding` signature is computed using the `decaf377-rdsa` `Sign` algorithm
where the message to be signed is the *auth hash* as described above, and the
`decaf377-rdsa` domain is `Binding`. The binding signing key is computed using the random blinding factors for each balance commitment.

[^1]: At this final stage we also generate the last signature: the binding signature, which can only be added
once the rest of the transaction is ready since it is computed over the proto-encoded `TransactionBody`.

[^2]: https://github.com/zcash/zips/issues/651
