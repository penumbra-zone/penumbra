# Delegator Vote Descriptions

Each delegator vote contains an DelegatorVoteBody and a zk-SNARK delegator vote proof.

## Delegator Vote zk-SNARK Statements

The delegator vote proof demonstrates the properties enumerated below for the following private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$) and asset `ID` $\isin \mathbb G$
* Note blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Address associated with the note being spent, consisting of diversified basepoint $B_d \isin \mathbb G$, 
transmission key $pk_d \isin \mathbb G$, and clue key $\mathsf{ck_d} \isin \mathbb F_q$
* Note commitment $cm \isin \mathbb F_q$
* Spend authorization randomizer used for generating the randomized spend authorization key $\alpha \isin \mathbb F_r$
* Spend authorization key $ak \isin \mathbb G$
* Nullifier deriving key $nk \isin \mathbb F_q$
* Merkle proof of inclusion for the note commitment, consisting of a position `pos` and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Balance commitment $cv \isin G$ to the value balance
* Nullifier $nf$ of the note to be spent
* Randomized verification key $rk \isin \mathbb G$
* The start position `start_pos` of the proposal being voted on

### Start Position Verification

The zk-SNARK certifies that the position of the staked note `pos` is less than the position of the proposal being voted on:

`pos < start_pos`

This demonstrates that the staked note used in voting existed prior to the proposal. 

The zk-SNARK also certifies that the commitment index of the start position is zero.

### Note Commitment Integrity

The zk-SNARK certifies that the note commitment $cm$ was derived as:

$cm = hash_5(ds, (rcm, v, ID, B_d, pk_d)$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\tilde v] G_{\tilde v}$

where $G_{\tilde v}$ is a constant generator and $G_v$ is an asset-specific generator point derived as described in [Value Commitments](../../value_commitments.md). For delegator votes, $[\tilde v] = 0$.

### Nullifier Integrity

The zk-SNARK certifies that the revealed nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q`

as described in [Nullifiers](../notes/nullifiers.md).

### Diversified Address Integrity

The zk-SNARK certifies that the diversified address $pk_d$ associated with the note was derived as:

$pk_d ​= [ivk] B_d$

where $B_d$ is the witnessed diversified basepoint and $ivk$ is the incoming viewing key computed using a rate-2 Poseidon hash from the witnessed $nk$ and $ak$ as:

`ivk = hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r`

as described in [Viewing Keys](../addresses_keys/viewing_keys.md).

### Spend Authority

The zk-SNARK certifies that for the randomized verification key $rk$ was derived using the witnessed $ak$ and spend auth randomizer $\alpha$ as:

$rk = ak + [\alpha]B_{SpendAuth}$

where $B_{SpendAuth}$ is the conventional `decaf377` basepoint as described in [The Decaf377 Group](../../crypto/decaf377.md).

### Merkle Verification

The zk-SNARK certifies that the witnessed Merkle authentication path is a valid Merkle path to the provided public anchor.

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ associated with the address on the note is not identity.

### Spend Authorization Key is not Identity

The zk-SNARK certifies that the spend authorization key $ak$ is not identity.

