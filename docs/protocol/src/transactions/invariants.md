
# System-Level

#### Invariants

1. Value cannot be created or destroyed after genesis or via IBC. Value cannot be created by the DEX.
1.1. Each action may create or destroy value, but commits to this imbalance, which when summed over a transaction, will not violate this rule.
2. Individual actions are bound to the transaction they belong to.

#### Justification

1.1. We check that the summed balance commitment of a transaction commits to 0.

## Action-Level

### Output

#### Local Invariants

1. The created output note is spendable by the recipient if its nullifier has not been revealed.
1.1 The output note is bound to the recipient.
1.2 The output note can be spent only by the recipient.

#### Local Justification

1.1 The note commitment binds the note to the typed value and the address of the recipient.
1.2 Each note has a unique note commitment if the note blinding factor is unique for duplicate (recipient, typed value) pairs. Duplicate note commitments are allowed on chain since they commit to the same (recipient, typed value) pair.

#### Global Justification

1.1 This action contributes the value of the output note, which is summed as part of the transaction value balance. Value is not created due to system level invariant 1, which ensures that transactions contribute a 0 value balance.

### Spend

#### Local Invariants

1. You must have spend authority over the note to spend
2. You can't spend a positioned note twice.
2.1. Each positioned note has exactly one valid nullifier for it.
2.2. No two spends on the ledger, even in the same transaction, can have the same nullifier.
3. You can't spend a note that has not been created, unless the amount of that note is 0.

#### Local Justification

1. We verify the auth_sig using the randomized verification key, which must not be 0, provided on the spend body, even if the amount of the note is 0. A note's transmission key binds the authority.
2.1. A note's transmission key binds to the nullifier key, and all components of a positioned note, along with this key are hashed to derive the nullifier, in circuit.
2.2. This is in check_stateful, and an external check about the integrity of each transaction.
3. The circuit verifies for non-zero amounts that there is a valid Merkle authentication path to the note in the global state commitment tree.

#### Global Justification

1.1. This action destroys the value of the note spent, and is reflected in the balance by adding the value to the transaction value balance. We do not create value, because of 3.

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

TODO: What happens if we swap the order of asset IDs? There is a canonical ordering OOC for a TradingPair that is not checked in-circuit.

3.1 The output amounts of each note is checked in-circuit to be only due to that user's contribution of the batch swap output data.
3.2 The circuit checks the block height of the swap matches that of the batch swap output data. The ActionHandler for the SwapClaim checks that the batch swap output data provided by the SwapClaim matches that saved on-chain for that output height and trading pair.

4. The SwapClaim circuit verifies that there is a valid Merkle authentication path to the swap in the global state commitment tree.

5.1. A swap's transmission key binds to the nullifier key, and all components of a positioned swap, along with this key are hashed to derive the nullifier, in circuit.

TODO: The binding of the transmission key to the nullifier key is not currently checked in circuit. Issue: https://github.com/penumbra-zone/penumbra/issues/3978

5.2. In the ActionHandler for check_stateful we check that the nullifier is unspent.

#### Global Justification

1.1 This action mints the swap's output notes, and is reflected in the balance by adding the value from the transaction value balance. We do not create value, because of 3.
