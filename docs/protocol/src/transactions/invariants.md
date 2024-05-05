
# System-Level

#### Invariants

1. Value cannot be created or destroyed after genesis or via IBC except staking (creates value), and DEX arbitrage execution (destroys value). The DEX cannot create value.

    1.1. Each action may create or destroy value, but commits to this imbalance, which when summed over a transaction, will not violate this rule.

2. Individual actions are bound to the transaction they belong to.

#### Justification

1.1. We check that the summed balance commitment of a transaction commits to 0.

2. The transaction binding signature is computed over the `AuthHash`, calculated from the proto-encoding of the entire `TransactionBody`. A valid binding signature can only be generated with knowledge of the opening of the balance commitments for each action in the transaction.

## Action-Level

* [Output Invariants](../shielded_pool/action/output.md)

* [Spend Invariants](../shielded_pool/action/spend.md)

* [DelegatorVote Invariants](../governance/action/delegator_vote.md)

* [Swap Invariants](../dex/action/swap.md)

* [SwapClaim Invariants](../dex/action/swap_claim.md)

* [UndelegateClaim Invariants](../stake/action/undelegate_claim.md)
