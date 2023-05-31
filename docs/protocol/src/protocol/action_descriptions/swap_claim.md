# SwapClaim Descriptions

Each swap claim contains a SwapClaimBody and a zk-SNARK swap claim proof.

## SwapClaim zk-SNARK Statements

The swap claim proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Swap plaintext corresponding to the swap being claimed. This consists of:
  * Trading pair, which consists of two asset IDs $ID_1, ID_2 \isin \mathbb F_q$
  * Fee value which consists of an amount $v_f$ interpreted as an $\mathbb F_q$ and
 an asset ID $ID_{v_f} \isin \mathbb F_q$
  * Input amount $\Delta_{1i}$ of the first asset interpreted as an $\mathbb F_q$
  * Input amount $\Delta_{2i}$ of the second asset interpreted as an $\mathbb F_q$
  * `Rseed`, interpreted as an $\mathbb F_q$
  * Diversified basepoint $B_d \isin \mathbb G$ corresponding to the claim address
  * Transmission key $pk_d \isin \mathbb G$ corresponding to the claim address
  * Clue key $\mathsf{ck_d} \isin \mathbb F_q$ corresponding to the claim address
* Swap commitment $scm \isin \mathbb F_q$
* Merkle proof of inclusion for the swap commitment, consisting of a position `pos` and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)
* Nullifier deriving key $nk \isin \mathbb F_q$
* Output amount $\Lambda_{1i}$ of the first asset interpreted as an $\mathbb F_q$
* Output amount $\Lambda_{2i}$ of the second asset interpreted as an $\mathbb F_q$
* Note blinding factor $rcm_1 \isin \mathbb F_q$ used to blind the first output note commitment
* Note blinding factor $rcm_2 \isin \mathbb F_q$ used to blind the second output note commitment

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Nullifier $nf$ corresponding to the swap
* Fee to claim the outputs which consists of an amount $v_f$ interpreted as an $\mathbb F_q$ and
 an asset ID $G_{v_f} \isin \mathbb G$
* The batch swap output data, which consists of:
  * trading pair, which consists of two asset IDs $ID_{pi1}, ID_{pi2} \isin \mathbb F_q$
  * 128-bit fixed point values (represented in circuit as four 64-bit (Boolean constraint) limbs) for the batched inputs $\Delta_1, \Delta_2$, outputs $\Lambda_1, \Lambda_2$, and the unfilled quantities $U_1, U_2$
  * block height $h \isin \mathbb F_q$
  * starting height of the epoch $h_e \isin \mathbb F_q$
* Note commitment of the first output note $cm_1 \isin \mathbb F_q$
* Note commitment of the second output note $cm_2 \isin \mathbb F_q$

### Swap Commitment Integrity

The zk-SNARK certifies that the witnessed swap commitment $scm$ was derived as:

$scm_{inner} = hash_4(ds, (ID_1, ID_2, \Delta_1, \Delta_2))$

$scm = hash_7(ds, (rseed, v_f, G_{v_f}, B_d, pk_d, \mathsf{ck_d}, scm_{inner})$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.swap")) mod q`

### Merkle auth path verification

The zk-SNARK certifies that the witnessed Merkle authentication path is a valid Merkle path of the swap commitment to the provided public anchor.

### Nullifier Integrity

The zk-SNARK certifies that the nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q`

as described in [Nullifiers](../notes/nullifiers.md).

### Fee Consistency Check

The zk-SNARK certifies that the public claim fee is equal to the value
witnessed as part of the swap plaintext.

### Height Consistency Check

The zk-SNARK certifies that the swap commitment's height is equal to the height
of the batch swap output data (the clearing price height).

We compute the intra-epoch block height $h_b$ from the position $pos$ of the swap
commitment and check the following identity:

$h = h_e + h_b$

where $h, h_e$ are provided on the batch swap output data as a public input.

### Trading Pair Consistency Check

The zk-SNARK certifies that the trading pair included in the swap plaintext corresponds
to the trading pair included on the batch swap output data, i.e.:

$ID_1 = ID_{pi1}$

$ID_2 = ID_{pi2}$

### Output amounts integrity

The zk-SNARK certifies that the claimed output amounts $\Lambda_{1i}, \Lambda_{2i}$
were computed correctly following the pro-rata output calculation performed
using the correct batch swap output data.

### Output Note Commitment Integrity

The zk-SNARK certifies that the note commitments $cm_1$ and $cm_2$ were derived as:

$cm_1 = hash_5(ds, (rcm_1, \Lambda_{1i}, ID_1, B_d, pk_d)$

$cm_2 = hash_5(ds, (rcm_2, \Lambda_{2i}, ID_2, B_d, pk_d)$

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`
