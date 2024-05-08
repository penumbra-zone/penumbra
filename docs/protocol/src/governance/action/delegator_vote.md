# DelegatorVote Descriptions

Each delegator vote contains an DelegatorVoteBody, a spend authorization signature, and a zk-SNARK delegator vote proof.

## [DelegatorVoteProof](#delegatorvote-body)

The body of a `DelegatorVote` has three parts:

1. The proposal ID the vote is for;
2. The start position of the proposal in the state commitment tree;
3. The `Vote` on the proposal;
4. The `Value` of the staked note being used to vote;
5. The unbonded amount equivalent to the staked value being used to vote;
6. A revealed `Nullifier`, which nullifies the staked note commitment being used to vote;
7. The randomized verification key, used to verify the spend authorization signature provided on the delegator vote.

## Invariants

The invariants that the DelegatorVote upholds are described below.

#### Local Invariants

1. Available voting power for a proposal = total delegated stake to active validators when the proposal was created

    1.1 Voting power must have been present before the proposal was created.

    1.2 You can't vote with a note that was spent prior to proposal start.

    1.3 That staked note must correspond to a validator that was in the active set when the
proposal being voted on was created.

2. You can't use the same voting power twice on a proposal.

3. You can't vote on a proposal that is not votable, i.e. it has not been withdrawn or voting has finished.

4. Invariants 1-3 with regards to spending a note apply to voting with it.

5. The voting power (amount and asset type of the staked note used for voting), the vote, as well as the proposal being voted on, is revealed during a delegator vote. However, they are anonymous in that they do not reveal the address of the voter.

6. Currently, votes across proposals can be linked if the same staked note is used.

#### Local Justification

1. We check the available voting power for a proposal equals the total delegated stake to active validators when the proposal was created via:

    1.1 The circuit checks the age of the staked note, and the stateful check verifies that the claimed position matches that of the proposal.

    1.2 We check that the note was spent only after the block of the proposal.

    1.3 The stateful check for the exchange rate makes sure the validator was active at that time.

2. We maintain a nullifier set for delegator votes and check for duplicate nullifiers in the stateful checks.

3. The stateful check looks up the proposal ID and ensures it is votable.

4. c.f. justification for spend circuit [here](../../shielded_pool/action/spend.md)

5. A randomized verification key is used to prevent linkability of votes across the same spend authority. The spender demonstrates in zero-knowledge that [this randomized verification key was derived from the spend authorization key given a witnessed spend authorization randomizer](#spend-authority). The spender also demonstrates in zero-knowledge that the [spend authorization key is associated with the address on the note being used for voting](#diversified-address-integrity).

6. The nullifier revealed in the DelegatorVote will be the same if the same staked note is used. Thus, the nullifier can be used to link votes across proposals. Clients
can roll over a staked note that was used for voting for privacy (this is currently done in `Planner::plan_with_spendable_and_votable_notes`).

## DelegatorVote zk-SNARK Statements

The delegator vote proof demonstrates the properties enumerated below for the following private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$ and constrained to fit in 128 bits) and asset `ID` $\isin \mathbb F_q$
* Note blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Address associated with the note being spent, consisting of diversified basepoint $B_d \isin \mathbb G$,
transmission key $pk_d \isin \mathbb G$, and clue key $\mathsf{ck_d} \isin \mathbb F_q$
* Note commitment $cm \isin \mathbb F_q$
* Spend authorization randomizer used for generating the randomized spend authorization key $\alpha \isin \mathbb F_r$
* Spend authorization key $ak \isin \mathbb G$
* Nullifier deriving key $nk \isin \mathbb F_q$
* Merkle proof of inclusion for the note commitment, consisting of a position `pos`, constrained to fit in 48 bits, and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Balance commitment $cv \isin G$ to the value balance
* Nullifier $nf \isin F_q$ of the note to be used for voting
* Randomized verification key $rk \isin \mathbb G$
* The start position `start_pos` of the proposal being voted on, constrained to fit in 48 bits

### Start Position Verification

The zk-SNARK certifies that the position of the staked note `pos` is less than the position of the proposal being voted on:

`pos < start_pos`

This demonstrates that the staked note used in voting existed prior to the proposal.

The zk-SNARK also certifies that the commitment index of the start position is zero.

### Note Commitment Integrity

The zk-SNARK certifies that the note commitment $cm$ was derived as:

$cm = hash_6(ds, (rcm, v, ID, B_d, pk_d, ck_d))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator and $G_v$ is an asset-specific generator point derived in-circuit as described in [Assets and Values](../../assets.md). For delegator votes, $[\widetilde{v}] = 0$.

### Nullifier Integrity

The zk-SNARK certifies that the revealed nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q`

as described in [Nullifiers](../../sct/nullifiers.md).

### Diversified Address Integrity

The zk-SNARK certifies that the diversified transmission key $pk_d$ associated with the note was derived as:

$pk_d ​= [ivk] B_d$

where $B_d$ is the witnessed diversified basepoint and $ivk$ is the incoming viewing key computed using a rate-2 Poseidon hash from the witnessed $nk$ and $ak$ as:

`ivk = hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r`

as described in [Viewing Keys](../../addresses_keys/viewing_keys.md).

### [Spend Authority](#spend-authority)

The zk-SNARK certifies that for the randomized verification key $rk$ was derived using the witnessed $ak$ and spend auth randomizer $\alpha$ as:

$rk = ak + [\alpha]B_{SpendAuth}$

where $B_{SpendAuth}$ is the conventional `decaf377` basepoint as described in [The Decaf377 Group](../../crypto/decaf377.md).

### Merkle Verification

The zk-SNARK certifies that the witnessed Merkle authentication path is a valid Merkle path to the provided public anchor.

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ associated with the address on the note is not identity.

### Spend Authorization Key is not Identity

The zk-SNARK certifies that the spend authorization key $ak$ is not identity.

