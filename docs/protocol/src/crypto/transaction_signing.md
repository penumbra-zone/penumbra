# Transaction Signing

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
the terminology used in Zcash[^1]:

> "Effecting data" is any data within a transaction that contributes to the effects of applying the transaction to the global state (results in previously-spendable coins or notes becoming spent, creates newly-spendable coins or notes, causes the root of a commitment tree to change, etc.).

The data that is _not_ effecting data is *authorizing data*:

>"Authorizing data" is the rest of the data within a transaction. It does not contribute to the effects of the transaction on global state, but allows those effects to take place. This data can be changed arbitrarily without resulting in a different transaction (but the changes may alter whether the transaction is allowed to be applied or not).

For example, the nullifier on a `Spend` is effecting data, whereas the
proofs or signatures associated with the `Spend` are authorizing data.

In Penumbra, the effect hash of each transaction is computed using the BLAKE2b-512
hash function. The effect hash is derived from the proto-encoding of the action - in
cases where the effecting data and authorizing data are the same, or the *body*
of the action - in cases where the effecting data and authorizing data are different.
Each proto has a unique string associated with it which we call its *Type URL*,
which is included in the inputs to BLAKE2b-512.
Type URLs are variable length, so a fixed-length field (8 bytes) is first included
in the hash to denote the length of the Type URL field.

Summarizing the above, the effect hash _for each action_ is computed as:

```
effect_hash = BLAKE2b-512(len(type_url) || type_url || proto_encode(proto))
```

where `type_url` is the bytes of the variable-length type URL, `len(type_url)` is the length of the type URL encoded as 8
bytes in little-endian byte order, `proto` represents the proto used to represent
the effecting data, and `proto_encode` represents encoding the proto message as
a vector of bytes.

### Per-Action Effect Hashes

On a per-action basis, the effect hash is computed using the following labels and
protos representing the effecting data in Penumbra:

| Action | Type URL  | Proto  |
|---|---|---|
| `Spend` | `b"/penumbra.core.transaction.v1alpha1.SpendBody"` | `SpendBody`  |
| `Output` | `b"/penumbra.core.transaction.v1alpha1.OutputBody"` | `OutputBody` |
| `Ics20Withdrawal`  | `b"/penumbra.core.ibc.v1alpha1.Ics20Withdrawal"`  | `Ics20Withdrawal` |
| `Swap`  | `b"/penumbra.core.dex.v1alpha1.SwapBody"`  | `SwapBody` |
| `SwapClaim`  | `b"/penumbra.core.dex.v1alpha1.SwapClaimBody"`  | `SwapClaimBody` |
| `Delegate`  | `b"/penumbra.core.stake.v1alpha1.Delegate"`  | `Delegate` |
| `Undelegate`  | `b"/penumbra.core.stake.v1alpha1.Undelegate"`  | `Undelegate` |
| `UndelegateClaim`  | `b"/penumbra.core.stake.v1alpha1.UndelegateClaimBody"`  | `UndelegateClaimBody` |
| `Proposal`  | `b"/penumbra.core.governance.v1alpha1.Proposal"`  | `Proposal` |
| `ProposalSubmit`  | `b"/penumbra.core.governance.v1alpha1.ProposalSubmit"`  | `ProposalSubmit` |
| `ProposalWithdraw`  | `b"/penumbra.core.governance.v1alpha1.ProposalWithdraw"`  | `ProposalWithdraw` |
| `ProposalDepositClaim`  | `b"/penumbra.core.governance.v1alpha1.ProposalDepositClaim"`   | `ProposalDepositClaim` |
| `Vote`  | `b"/penumbra.core.governance.v1alpha1.Vote"`  | `Vote` |
| `ValidatorVote`  | `b"/penumbra.core.governance.v1alpha1.ValidatorVoteBody"`  | `ValidatorVoteBody` |
| `DelegatorVote`  | `b"/penumbra.core.governance.v1alpha1.DelegatorVoteBody"`  | `DelegatorVoteBody` |
| `DaoDeposit`  | `b"/penumbra.core.governance.v1alpha1.DaoDepositt"`  | `DaoDeposit` |
| `DaoOutput`  | `b"/penumbra.core.governance.v1alpha1.DaoOutput"`  | `DaoOutput` |
| `DaoSpend`  | `b"/penumbra.core.governance.v1alpha1.DaoSpend"`  | `DaoSpend` |
| `PositionOpen`  | `b"/penumbra.core.dex.v1alpha1.PositionOpen"`  | `PositionOpen` |
| `PositionClose`  | `b"/penumbra.core.dex.v1alpha1.PositionClose"`  | `PositionClose` |
| `PositionWithdraw`  | `b"/penumbra.core.dex.v1alpha1.PositionWithdraw"`  | `PositionWithdraw` |
| `PositionRewardClaim`  | `b"/penumbra.core.dex.v1alpha1.PositionRewardClaim"`  | `PositionRewardClaim` |

### Transaction Data Field Effect Hashes

We compute the transaction data field effect hashes in the same manner as actions:

| Field | Type URL  | Proto  |
|---|---|---|
| `TransactionParameters` | `b"/penumbra.core.transaction.v1alpha1.TransactionParameters"` | `TransactionParameters`  |
| `Fee` | `b"/penumbra.core.crypto.v1alpha1.Fee"` | `Fee` |
| `MemoCiphertext` | `b"/penumbra.core.crypto.v1alpha1.MemoCiphertext"` | `MemoCiphertext`
| `DetectionData` | `b"/penumbra.core.transaction.v1alpha1.DetectionData"` | `DetectionData`

### Transaction Effect Hash

To compute the effect hash of the _entire transaction_, we combine the hashes of the individual fields in the transaction body. First we include the fixed-sized effect hashes of the per-transaction data fields: the transaction parameters `eh(tx_params)`, fee `eh(fee)`, (optional) detection data `eh(detection_data)`, and (optional) memo `eh(memo)` which are derived as described above. Then, we include the number of actions $j$ and the fixed-size effect hash of each action `a_0` through `a_j`. Combining all fields:

```
effect_hash = BLAKE2b-512(len(type_url) || type_url || eh(tx_params) || eh(fee) || eh(memo) || eh(detection_data) || j || eh(a_0) || ... || eh(a_j))
```

where the `type_url` are the bytes `/penumbra.core.transaction.v1alpha1.TransactionBody`,
and `len(type_url)` is the length of that string encoded as 8 bytes in little-endian byte order.

## `Binding` Signature

The `Binding` signature is computed once on the transaction level.
It is a signature over a hash of the authorizing data of that transaction, called the *auth hash*. The auth hash of each transaction is computed using the BLAKE2b-512 hash function over the proto-encoding of the _entire_ `TransactionBody`.

The `Binding` signature is computed using the `decaf377-rdsa` `Sign` algorithm
where the message to be signed is the *auth hash* as described above, and the
`decaf377-rdsa` domain is `Binding`. The binding signing key is computed using the random blinding factors for each balance commitment.

[1]: https://github.com/zcash/zips/issues/651
