# Output Descriptions

Each output contains an OutputBody and a zk-SNARK output proof.

Clients using the ephemeral public key $epk$ provided in an output body to decrypt a note payload MUST check:

$epk = [esk] B_d$

## Output zk-SNARK Statements

The output proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$) and asset `ID` $\isin \mathbb G$
* Blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Diversified basepoint $B_d \isin \mathbb G$ corresponding to the address
* Transmission key $pk_d \isin \mathbb G$ corresponding to the address 
* Clue key $\mathsf{ck_d} \isin \mathbb F_q$ corresponding to the address
* Blinding factor $\tilde v \isin \mathbb F_r$ used to blind the balance commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Note commitment $cm \isin \mathbb F_q$

### Note Commitment Integrity

The zk-SNARK certifies that the public input note commitment $cm$ was derived as:

$cm = hash_5(ds, (rcm, v, ID, B_d, pk_d)$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\tilde v] G_{\tilde v}$

where $G_{\tilde v}$ is a constant generator and $G_v$ is an asset-specific generator point derived as described in [Value Commitments](../../value_commitments.md).  

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ is not identity. 

Note that we do _not_ check the integrity of the ephemeral public key $epk$ in the zk-SNARK.
Instead this check should be performed at note decryption time as described above.
