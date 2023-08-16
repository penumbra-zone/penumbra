# Transaction Signing

Transactions are signed used the [`decaf377-rdsa` construction](../crypto/decaf377-rdas.md). As described briefly in that section, there are two signature domains used in Penumbra: `SpendAuth` signatures and `Binding` signatures.

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

In Penumbra, the effect hash of each transaction is computed using the hash
function `hash(label, input)` as BLAKE2b-512 with personalization `label` on
input `input`. The input of each effect hash is the proto-encoding of the action - in
cases where the effecting data and authorizing data are the same, or the *body*
of the action - in cases where the effecting data and authorizing data are different.

Then the effect hash _for each action_ is computed as:

```
effect_hash = hash(label, proto_encode(proto))
```

where `proto` represents the proto used to represent the effecting data, and
`proto_encode` represents encoding the proto message as a vector of bytes.

### Per-Action Effect Hashes

On a per-action basis, the effect hash is computed using the following labels and
protos representing the effecting data in Penumbra:

| Action | Label  | Proto  |
|---|---|---|
| `Spend` | `b"PAH:spend_body"` | `SpendBody`  |
| `Output` | `b"PAH:output_body"` | `OutputBody` |
| `Ics20Withdrawal`  | `b"PAH:ics20wthdrwl"`  | `Ics20Withdrawal` |
| `Swap`  | `b"PAH:swap_body"`  | `SwapBody` |
| `SwapClaim`  | `b"PAH:swapclaimbdy"`  | `SwapClaimBody` |
| `Delegate`  | `b"PAH:delegate"`  | `Delegate` |
| `Undelegate`  | `b"PAH:undelegate"`  | `Undelegate` |
| `UndelegateClaim`  | `b"PAH:udlgclm_body"`  | `UndelegateClaimBody` |
| `Proposal`  | `b"PAH:proposal"`  | `Proposal` |
| `ProposalSubmit`  | `b"PAH:prop_submit"`  | `ProposalSubmit` |
| `ProposalWithdraw`  | `b"PAH:prop_withdrw"`  | `ProposalWithdraw` |
| `ProposalDepositClaim`  | `b"PAH:prop_dep_clm"`   | `ProposalDepositClaim` |
| `Vote`  | `b"PAH:vote"`  | `Vote` |
| `ValidatorVote`  | `b"PAH:val_vote"`  | `ValidatorVoteBody` |
| `DelegatorVote`  | `b"PAH:del_vote"`  | `DelegatorVoteBody` |
| `DaoDeposit`  | `b"PAH:daodeposit"`  | `DaoDeposit` |
| `DaoOutput`  | `b"PAH:daooutput"`  | `DaoOutput` |
| `DaoSpend`  | `b"PAH:daospend"`  | `DaoSpend` |
| `PositionOpen`  | `b"PAH:pos_open"`  | `PositionOpen` |
| `PositionClose`  | `b"PAH:pos_close"`  | `PositionClose` |
| `PositionWithdraw`  | `b"PAH:pos_withdraw"`  | `PositionWithdraw` |
| `PositionRewardClaim`  | `b"PAH:pos_rewrdclm"`  | `PositionRewardClaim` |

### Transaction Data Field Effect Hashes

We compute the transaction data field effect hashes in the same manner as actions:

| Field | Label  | Proto  |
|---|---|---|
| `TransactionParameters` | `b"PAH:tx_params"` | `TransactionParameters`  |
| `Fee` | `b"PAH:fee"` | `Fee` |
| `Clue`  | `b"PAH:decaffmdclue"`  | `Clue` |
| `MemoCiphertext` | `b"PAH:memo"` | `MemoCiphertext`

For the `DetectionData`, we compute the effect hash where `eh(c_i)` represents the effect hash of the $i^{th}$ clue via:

`effect_hash = hash(b"PAH:detect_data", num clues || eh(c_0) || ... ||  eh(c_i))`

### Transaction Effect Hash

To compute the effect hash of the _entire transaction_, we combine the hashes of the individual fields in the transaction body. First we include the fixed-sized effect hashes of the per-transaction data fields: the transaction parameters `eh(tx_params)`, fee `eh(fee)`, (optional) detection data `eh(detection_data)`, and (optional) memo `eh(memo)` which are derived as described above. Then, we include the number of actions $j$ and the fixed-size effect hash of each action `a_0` through `a_j`. Combining all fields:

```
effect_hash = hash(b"PAH:tx_body", eh(tx_params) || eh(fee) || eh(memo) || eh(detection_data) || j || eh(a_0) || ... || eh(a_j))
```


## `Binding` Signature

The `Binding` signature is computed once on the transaction level.
It is a signature over a hash of the authorizing data of that transaction, called the *auth hash*. The auth hash of each transaction is computed using the BLAKE2b-512 hash function over the proto-encoding of the _entire_ `TransactionBody`.

The `Binding` signature is computed using the `decaf377-rdsa` `Sign` algorithm
where the message to be signed is the *auth hash* as described above, and the
`decaf377-rdsa` domain is `Binding`. The binding signing key is computed using the random blinding factors for each balance commitment.

[1]: https://github.com/zcash/zips/issues/651
