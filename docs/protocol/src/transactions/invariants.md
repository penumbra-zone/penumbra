
# System-Level

#### Invariants

1. Value cannot be created or destroyed after genesis or via IBC. Value cannot be created by the DEX.

    1.1. Each action may create or destroy value, but commits to this imbalance, which when summed over a transaction, will not violate this rule.

2. Individual actions are bound to the transaction they belong to.

#### Justification

1.1. We check that the summed balance commitment of a transaction commits to 0.

## Action-Level

* [Output Invariants](../shielded_pool/action/output.md)

* [Spend Invariants](../shielded_pool/action/spend.md)

* [DelegatorVote Invariants](../governance/action/delegator_vote.md)

* [Swap Invariants](../dex/action/swap.md)

* [SwapClaim Invariants](../dex/action/swap_claim.md)

* [UndelegateClaim Invariants](../stake/action/undelegate_claim.md)
