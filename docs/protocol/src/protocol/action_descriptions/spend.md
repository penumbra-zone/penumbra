# Spend Descriptions

Each spend contains an SpendBody and a zk-SNARK spend proof.

## Spend zk-SNARK Statements

The spend proof demonstrates the properties enumerated below for the following private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$) and asset `ID` $\isin \mathbb G$
* Note blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Address associated with the note being spent, consisting of diversified basepoint $B_d \isin \mathbb G$, 
transmission key $pk_d \isin \mathbb G$, and clue key $\mathsf{ck_d} \isin \mathbb F_q$
* Note commitment $cm \isin \mathbb F_q$
* Blinding factor $\tilde v \isin \mathbb F_r$ used to blind the balance commitment
* Spend authorization randomizer used for generating the randomized spend authorization key $\alpha \isin \mathbb F_r$
* Spend authorization key $ak \isin \mathbb G$
* Nullifier deriving key $nk \isin \mathbb F_q$
* Merkle proof of inclusion for the note commitment, consisting of a position `pos` and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Balance commitment $cv \isin G$ to the value balance
* Nullifier $nf$ of the note to be spent
* Randomized verification key $rk \isin \mathbb G$

### Dummy spend

We require each one of the following integrity properties to hold only for notes with non-zero values $v \ne 0$. This is to allow for dummy spends to pass stateless verification. Dummy spends may be added for metadata resistance (e.g. to ensure there are two spends and two outputs in each transaction).

### Note Commitment Integrity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the note commitment $cm$ was derived as:

$cm = hash_5(ds, (rcm, v, ID, B_d, pk_d)$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\tilde v] G_{\tilde v}$

where $G_{\tilde v}$ is a constant generator and $G_v$ is an asset-specific generator point derived as described in [Value Commitments](../../value_commitments.md).  

### Nullifier Integrity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the
revealed nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q
`

as described in [Nullifiers](../notes/nullifiers.md).

### Diversified address Integrity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the diversified address $pk_d$ associated with the note being spent was derived as:

$pk_d ​= [ivk] B_d$

where $B_d$ is the witnessed diversified basepoint and $ivk$ is the incoming viewing key computed using a rate-2 Poseidon hash from the witnessed $nk$ and $ak$ as:

`ivk = hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r`

as described in [Viewing Keys](../addresses_keys/viewing_keys.md).

### Randomized verification key Integrity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the randomized verification key $rk$ was derived using the witnessed $ak$ and spend auth randomizer $\alpha$ as:

$rk = ak + [\alpha]B_{SpendAuth}$

where $B_{SpendAuth}$ is the conventional `decaf377` basepoint as described in [The Decaf377 Group](../../crypto/decaf377.md).

### Merkle auth path verification

The zk-SNARK certifies that for non-zero values[^1] $v \ne 0$, the witnessed Merkle authentication path is a valid Merkle path to the provided public anchor.

### Diversified Base is not Identity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the diversified basepoint $B_d$ associated with the address on the note is not identity.

### The spend authorization key is not Identity

The zk-SNARK certifies that for non-zero values $v \ne 0$, the spend authorization key $ak$ is not identity.

[^1]: Note that [issue 2135](https://github.com/penumbra-zone/penumbra/issues/2135) tracks a bug where dummy spends fail to verify due to the merkle paths.
