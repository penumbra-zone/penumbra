# Output Descriptions

Each output contains an OutputBody and a zk-SNARK output proof.

## Invariants

The invariants that the Output upholds are described below.

#### Local Invariants

1. The created output note is spendable by the recipient if its nullifier has not been revealed.

1.1 The output note is bound to the recipient.

1.2 The output note can be spent only by the recipient.

#### Local Justification

1.1 The note commitment binds the note to the typed value and the address of the recipient.

1.2 Each note has a unique note commitment if the note blinding factor is unique for duplicate (recipient, typed value) pairs. Duplicate note commitments are allowed on chain since they commit to the same (recipient, typed value) pair.

#### Global Justification

1.1 This action contributes the value of the output note, which is summed as part of the transaction value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance.

## Note Decryption Checks

Clients using the ephemeral public key $epk$ provided in an output body to decrypt a note payload MUST check:

$epk = [esk] B_d$

## Output zk-SNARK Statements

The output proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$, constrained to fit in 128 bits) and asset `ID` $\isin \mathbb F_q$
* Blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Diversified basepoint $B_d \isin \mathbb G$ corresponding to the address
* Transmission key $pk_d \isin \mathbb G$ corresponding to the address
* Clue key $\mathsf{ck_d} \isin \mathbb F_q$ corresponding to the address
* Blinding factor $\widetilde{v} \isin \mathbb F_r$ used to blind the balance commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Note commitment $cm \isin \mathbb F_q$

### Note Commitment Integrity

The zk-SNARK certifies that the public input note commitment $cm$ was derived as:

$cm = hash_6(ds, (rcm, v, ID, B_d, pk_d, ck_d))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator and $G_v$ is an asset-specific
generator point derived in-circuit as described in [Assets and
Values](../../assets.md).

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ is not identity.

Note that we do _not_ check the integrity of the ephemeral public key $epk$ in the zk-SNARK.
Instead this check should be performed at note decryption time as described above.
