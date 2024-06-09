# SwapClaim Descriptions

Each swap claim contains a SwapClaimBody and a zk-SNARK swap claim proof[^1].

## [SwapClaim Body](#swapclaim-body)

The body of a `SwapClaim` has four parts:

1. A revealed `Nullifier`, which nullifies the swap commitment being claimed;
2. Note commitments for each of the two output notes minted by the `SwapClaim`;
3. The prepaid `Fee` being consumed by the `SwapClaim`;
4. The `BatchSwapOutputData` corresponding to the block in which the swap was executed.

## Invariants

The invariants that the SwapClaim upholds are described below.

#### Local Invariants

1. You cannot mint swap outputs to a different address than what was specified during the initial swap.

2. You cannot mint swap outputs to different assets than those specified in the original swap.

3. You cannot mint swap outputs to different amounts than those specified by the batch swap output data.

    3.1. You can only claim your contribution to the batch swap outputs.

    3.2. You can only claim the outputs using the batch swap output data for the block in which the swap was executed.

4. You can only claim swap outputs for swaps that occurred.

5. You can only claim swap outputs once.

    5.1. Each positioned swap has exactly one valid nullifier for it.

    5.2. No two swaps can have the same nullifier.

6. The SwapClaim does not reveal the amounts or asset types of the swap output notes, nor does it reveal the identity of the claimant.

7. The balance contribution of the value of the swap output notes is private.

#### Local Justification

1. The Swap commits to the claim address. The SwapClaim circuit enforces that the same address used to derive the swap commitment is that used to derive the note commitments for the two output notes via the [Swap Commitment Integrity](#swap-commitment-integrity) and [Output Note Commitment Integrity](#output-note-commitment-integrity) checks.

2. The SwapClaim circuit checks that the trading pair on the original Swap is the same as that on the batch swap output data via the [Trading Pair Consistency Check](#trading-pair-consistency-check).

3. You cannot mint outputs to different amounts via:

    3.1. The output amounts of each note is checked in-circuit to be only due to that user's contribution of the batch swap output data via the [Output Amounts Integrity](#output-amounts-integrity) check.

    3.2. The circuit checks the block height of the swap matches that of the batch swap output data via the [Height Consistency Check](#height-consistency-check). The `ActionHandler` for the SwapClaim checks that the batch swap output data provided by the SwapClaim matches that saved on-chain for that output height and trading pair.

4. The SwapClaim circuit verifies that there is a valid [Merkle authentication path](#merkle-auth-path-verification) to the swap in the global state commitment tree.

5. You can only claim swap outputs once via:

    5.1. A swap's transmission key binds to the nullifier key as described in the [Nullifier Key Linking](#nullifier-key-linking) section, and all components of a positioned swap, along with this key, are hashed to derive the nullifier, in circuit as described below in the [Nullifier Integrity](#nullifier-integrity) section.

    5.2. In the `ActionHandler` we check that the nullifier is unspent.

6. The revealed SwapClaim on the nullifier does not reveal the swap commitment, since the [Nullifier Integrity](#nullifier-integrity) check is done in zero-knowledge. The amount and asset type of each output note is hidden via the hiding property of the note commitments, which the claimer demonstrates an opening of via the [Output Note Commitment Integrity](#output-note-commitment-integrity) check. The minted output notes are encrypted.

7. The balance contribution of the two output notes is zero. The only contribution to the balance is the pre-paid SwapClaim fee.

#### Global Justification

1.1. This action mints the swap's output notes, and is reflected in the balance by adding the value from the transaction value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance.

## SwapClaim zk-SNARK Statements

The swap claim proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Swap plaintext corresponding to the swap being claimed. This consists of:
  * Trading pair, which consists of two asset IDs $ID_1, ID_2 \isin \mathbb F_q$
  * Fee value which consists of an amount $v_f$ interpreted as an $\mathbb F_q$ constrained to fit in 128 bits and an asset ID $ID_{v_f} \isin \mathbb F_q$
  * Input amount $\Delta_{1i}$ of the first asset interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
  * Input amount $\Delta_{2i}$ of the second asset interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
  * `Rseed`, interpreted as an $\mathbb F_q$
  * Diversified basepoint $B_d \isin \mathbb G$ corresponding to the claim address
  * Transmission key $pk_d \isin \mathbb G$ corresponding to the claim address
  * Clue key $\mathsf{ck_d} \isin \mathbb F_q$ corresponding to the claim address
* Swap commitment $scm \isin \mathbb F_q$
* Merkle proof of inclusion for the swap commitment, consisting of a position `pos` constrained to fit in 48 bits and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)
* Nullifier deriving key $nk \isin \mathbb F_q$
* Spend verification key $ak \isin \mathbb G$
* Output amount $\Lambda_{1i}$ of the first asset interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
* Output amount $\Lambda_{2i}$ of the second asset interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
* Note blinding factor $rcm_1 \isin \mathbb F_q$ used to blind the first output note commitment
* Note blinding factor $rcm_2 \isin \mathbb F_q$ used to blind the second output note commitment

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Nullifier $nf \isin F_q$ corresponding to the swap
* Fee to claim the outputs which consists of an amount $v_{f,pub}$ interpreted as an $\mathbb F_q$ constrained to fit in 128 bits and an asset ID $ID_{v_{f,pub}} \isin \mathbb F_q$
* The batch swap output data, which consists of:
  * trading pair, which consists of two asset IDs $ID_{pi1}, ID_{pi2} \isin \mathbb F_q$
  * 128-bit fixed point values (represented in circuit as four 64-bit (Boolean constraint) limbs) for the batched inputs $\Delta_1, \Delta_2$, outputs $\Lambda_1, \Lambda_2$, and the unfilled quantities $U_1, U_2$
  * block height $h \isin \mathbb F_q$ constrained to fit in 64 bits
  * starting height of the epoch $h_e \isin \mathbb F_q$ constrained to fit in 64 bits
* Note commitment of the first output note $cm_1 \isin \mathbb F_q$
* Note commitment of the second output note $cm_2 \isin \mathbb F_q$

### [Swap Commitment Integrity](#swap-commitment-integrity)

The zk-SNARK certifies that the witnessed swap commitment $scm$ was derived as:

$scm_{inner} = hash_4(ds, (ID_1, ID_2, \Delta_1, \Delta_2))$

$scm = hash_7(ds, (rseed, v_f, ID_{v_f}, B_d, pk_d, \mathsf{ck_d}, scm_{inner}))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.swap")) mod q`

### [Merkle auth path verification](#merkle-auth-path-verification)

The zk-SNARK certifies that the witnessed Merkle authentication path is a valid Merkle path of the swap commitment to the provided public anchor.

### [Nullifier Integrity](#nullifier-integrity)

The zk-SNARK certifies that the nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q`

as described in [Nullifiers](../../sct/nullifiers.md).

### [Nullifier Key Linking](#nullifier-key-linking)

The zk-SNARK certifies that the diversified transmission key $pk_d$ associated with the
swap being claimed was derived as:

$pk_d ​= [ivk] B_d$

where $B_d$ is the witnessed diversified basepoint and $ivk$ is the incoming viewing key computed using a rate-2 Poseidon hash from the witnessed $nk$ and $ak$ as:

`ivk = hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r`

The zk-SNARK also certifies that:
- $ak \neq 0$
- $pk_d \neq 0$

### [Fee Consistency Check](#fee-consistency-check)

The zk-SNARK certifies that the public claim fee is equal to the fee value
witnessed as part of the swap plaintext:

$v_f = v_{f,pub}$

$ID_{v_f} = ID_{v_{f,pub}}$

### [Height Consistency Check](#height-consistency-check)

We compute $h_{swap}$ and $e_{swap}$ from the position of the swap commitment
in the state commitment tree. We compute $h_{BSOD}$ and $e_{BSOD}$ from the
position prefix where the batch swap occurred, provided on the batch swap output
data (BSOD).

The zk-SNARK certifies that the swap commitment's block height is equal to the block height
of the BSOD:

$h_{BSOD} = h_{swap}$

The zk-SNARK also certifies that the swap commitment's epoch is equal to the epoch
of the BSOD:

$e_{BSOD} = e_{swap}$

### [Trading Pair Consistency Check](#trading-pair-consistency-check)

The zk-SNARK certifies that the trading pair included in the swap plaintext corresponds
to the trading pair included on the batch swap output data, i.e.:

$ID_1 = ID_{pi1}$

$ID_2 = ID_{pi2}$

### [Output Amounts Integrity](#output-amounts-integrity)

The zk-SNARK certifies that the claimed output amounts $\Lambda_{1i}, \Lambda_{2i}$ were computed correctly following the pro-rata output calculation performed using the correct batch swap output data.

$\Lambda_{2i} = (\Delta_{1i} / \Delta_{1}) * \Lambda_{2} + (\Delta_{2i}/ \Delta_{2}) * u_2$

$\Lambda_{1i} = (\Delta_{1i} / \Delta_{1}) * u_1 + (\Delta_{2i} / \Delta_{2}) * \Lambda_{1}$

where $\Delta_{1}, \Delta_{2}$ are the total amounts of assets 1 and 2 that was output from the batch swap, and $u_1$ and $u_2$ are the amounts of assets 1 and 2 that was returned unfilled from the batch swap. These fields are part of the batch swap output data.

### [Output Note Commitment Integrity](#output-note-commitment-integrity)

The zk-SNARK certifies that the note commitments $cm_1$ and $cm_2$ were derived as:

$cm_1 = hash_6(ds, (rcm_1, \Lambda_{1i}, ID_1, B_d, pk_d, ck_d))$

$cm_2 = hash_6(ds, (rcm_2, \Lambda_{2i}, ID_2, B_d, pk_d, ck_d))$

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ associated with the address on the note is not identity.

[^1]: There is also a deprecated and unused field `epoch_duration`.