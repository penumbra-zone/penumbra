
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

#### Global Justification

1.1. This action destroys the value of the note spent, and is reflected in the balance. We do not create value, because of 3.

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
