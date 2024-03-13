
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

### Delegator Votes

#### Local Invariants

Privacy note: Value of staked note used for voting is revealed during a delegator vote.

1. Available voting power for a proposal = total delegated stake to active validators when the proposal was created

1.1 Voting power must have been present before the proposal was created.

1.2 You can't vote with a note that was spent prior to proposal start.

1.3 That staked note must correspond to a validator that was in the active set when the
proposal being voted on was created.

2. You can't use the same voting power twice on a proposal.

3. You can't vote on a proposal that is not votable, i.e. it has not been withdrawn or voting has finished.

4. All invariants with regards to spending a note apply to voting with it.

#### Local Justification

1.1 The circuit checks the age of the staked note, and the stateful check verifies that the claimed position matches that of the proposal.

1.2 We check that the note was spent only after the block of the proposal.

1.3 The stateful check for the exchange rate makes sure the validator was active at that time.

2. We maintain a nullifier set for delegator votes and check for duplicate nullifiers in the stateful checks.

3. The stateful check looks up the proposal ID and ensures it is votable.

4. c.f. justification for spend circuit

### Swap

#### Local Invariants

1. The swap binds to the specific trading pair.
2. The swap binds to a specific claim address.

#### Local Justification

1. The swap commitment includes as inputs the trading pair of the two assets.
2. The swap commitment includes as inputs each component of the claim address.

#### Global Justification

1.1 This action consumes the value of the two input notes (one per asset) consumed and the value of the pre-paid fee to be used by the SwapClaim, and is reflected in the balance by subtracting the value from the transaction value balance. We do not create value, because of 3.

### SwapClaim

#### Local Invariants

1. You cannot mint swap outputs to a different address than what was specified during the initial swap.

2. You cannot mint swap outputs to different assets than those specified in the original swap.

3. You cannot mint swap outputs to different amounts than those specified by the batch swap output data.

3.1. You can only claim your contribution to the batch swap outputs.

3.2 You can only claim the outputs using the batch swap output data for the block in which the swap was executed.

4. You can only claim swap outputs for swaps that occurred.

5. You can only claim swap outputs once.

5.1. Each positioned swap has exactly one valid nullifier for it.

5.2. No two swaps can have the same nullifier.

#### Local Justification

1. The Swap commits to the claim address. The SwapClaim circuit enforces that the same address used to derive the swap commitment is that used to derive the note commitments for the two output notes.

2. The SwapClaim circuit checks that the trading pair on the original Swap is the same as that on the batch swap output data.

3.1 The output amounts of each note is checked in-circuit to be only due to that user's contribution of the batch swap output data.

3.2 The circuit checks the block height of the swap matches that of the batch swap output data. The ActionHandler for the SwapClaim checks that the batch swap output data provided by the SwapClaim matches that saved on-chain for that output height and trading pair.

4. The SwapClaim circuit verifies that there is a valid Merkle authentication path to the swap in the global state commitment tree.

5.1. A swap's transmission key binds to the nullifier key, and all components of a positioned swap, along with this key are hashed to derive the nullifier, in circuit.

5.2. In the `ActionHandler` for `check_stateful` we check that the nullifier is unspent.

#### Global Justification

1.1 This action mints the swap's output notes, and is reflected in the balance by adding the value from the transaction value balance. We do not create value, because of 3.

### UndelegateClaim

#### Local Invariants

1. You cannot claim undelegations that have not finishing unbonding.
2. Slashing penalties must be applied when unbonding.

#### Local Justification

1. In the `ActionHandler` for `check_stateful` we check that the undelegations have finished unbonding.
2. The `ConvertCircuit` verifies that the conversion from the unbonding token to the staking token was done using the correct conversion rate calculated from the penalty. We check in the `ActionHandler` for `check_stateful` that the _correct_ penalty was used.

#### Global Justification

1.1. This action consumes the amount of the unbonding tokens and contributes the unbonded amount of the staking tokens to the transaction's value balance. We do not create value, because of 3.
