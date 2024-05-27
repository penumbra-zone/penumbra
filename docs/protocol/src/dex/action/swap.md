# Swap Descriptions

Each swap contains a SwapBody and a zk-SNARK swap proof.

## [Swap Body](#swap-body)

The body of an `Swap` has five parts:

1. A `SwapPayload`, which consists of the swap commitment and the `SwapCiphertext`,
2. A fee balance commitment, which commits to the value balance of the pre-paid fee;
3. The `TradingPair` which is the canonical representation of the two asset IDs involved in the trade,
4. `delta_1_i`, the amount of the _first_ asset ID being traded,
5. `delta_2_i`, the amount of the _second_ asset ID being traded.

The `SwapCiphertext` is 272 bytes in length, and is encrypted symmetrically using the
payload key derived from the `OutgoingViewingKey` and the swap commitment as
described [here](../../addresses_keys/transaction_crypto.md#per-action-swap-key).

The corresponding plaintext, `SwapPlaintext` consists of:

* the `TradingPair` which as above is the canonical representation of the two asset IDs involved in the trade,
* `delta_1_i`, the amount of the _first_ asset ID being traded,
* `delta_2_i`, the amount of the _second_ asset ID being traded,
* the value of the pre-paid claim fee used to claim the outputs of the swap,
* the address used to claim the outputs of the swap,
* the swap `Rseed`, which is used to derive the `Rseed` for the two output notes.

The output notes for the `Swap` can be computed given the `SwapPlaintext` and the
`BatchSwapOutputData` from that block. The `Rseed` for each output note are computed from
the `SwapPlaintext` using rate-1 Poseidon hashing with domain separators $ds_1$ and $ds_2$ defined as the `Fq` element constructed using:

`ds_1 = from_le_bytes(BLAKE2b-512(b"penumbra.swapclaim.output1.blinding")) mod q`

`ds_2 = from_le_bytes(BLAKE2b-512(b"penumbra.swapclaim.output2.blinding")) mod q`

The rseed for each note is then constructed using the above domain separator and
hashing together the swap rseed $rseed$:

`rseed_1 = hash_1(ds_1, (rseed))`

`rseed_2 = hash_1(ds_2, (rseed))`

## Invariants

The invariants that the Swap upholds are described below.

#### Local Invariants

1. The swap binds to the specific trading pair.

2. The swap binds to a specific claim address.

3. The swap does not reveal the swapper identity. The swap *does* reveal the assets being swapped, as well as the amounts of those assets.

    3.1 The swap data included in a transaction preserves this property.

    3.2 The integrity checks as described above in invariant 1 are done privately.

4. The swap does not reveal the pre-paid claim fee.

#### Local Justification

1. The swap commitment includes as inputs the trading pair of the two assets, demonstrated in circuit by the [Swap Commitment Integrity](#swap-commitment-integrity) check.

2. The swap commitment includes as inputs each component of the claim address, demonstrated in circuit by the [Swap Commitment Integrity](#swap-commitment-integrity) check.

3. The privacy property of the swap is preserved via:

    3.1 The swap, which includes the claim address of the swapper, is symmetrically encrypted to a key that only the swapper can derive, as specified in [Transaction Cryptography](../../addresses_keys/transaction_crypto.md). The swapper can authorize other parties to view the contents of the swap by disclosure of this symmetric key.

    3.2 The swapper demonstrates knowledge of the [opening of the swap commitment in zero-knowledge](#swap-commitment-integrity).

4. The fee contribution is hidden via the hiding property of the balance commitment scheme. Knowledge of the opening of the [fee commitment is done in zero-knowledge](#fee-commitment-integrity).

#### Global Justification

1.1 This action consumes the value of the two input notes (one per asset) consumed and the value of the pre-paid fee to be used by the SwapClaim, and is reflected in the balance by subtracting the value from the transaction value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance.

## Swap zk-SNARK Statements

The swap proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Swap plaintext which consists of:
  * Trading pair, which consists of two asset IDs  $ID_1, ID_2 \isin \mathbb F_q$
  * Fee value which consists of an amount $v_f$ interpreted as an $\mathbb F_q$ constrained to fit in 128 bits, and an asset ID $ID_{v_f} \isin \mathbb F_q$
  * Input amount of the first asset $v_1$ interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
  * Input amount of the second asset $v_2$ interpreted as an $\mathbb F_q$ constrained to fit in 128 bits
  * `Rseed`, interpreted as an $\mathbb F_q$
  * Diversified basepoint $B_d \isin \mathbb G$ corresponding to the claim address
  * Transmission key $pk_d \isin \mathbb G$ corresponding to the claim address
  * Clue key $\mathsf{ck_d} \isin \mathbb F_q$ corresponding to the claim address
* Fee blinding factor $\widetilde{v_f} \isin \mathbb F_r$ used to blind the fee commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Fee commitment $cv_f \isin G$ to the value of the fee
* Swap commitment $scm \isin \mathbb F_q$

### [Swap Commitment Integrity](#swap-commitment-integrity)

The zk-SNARK certifies that the public input swap commitment $scm$ was derived as:

$scm_{inner} = hash_4(ds, (ID_1, ID_2, v_1, v_2))$

$scm = hash_7(ds, (rseed, v_f, ID_{v_f}, B_d, pk_d, \mathsf{ck_d}, scm_{inner}))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.swap")) mod q`

### [Fee Commitment Integrity](#fee-commitment-integrity)

The zk-SNARK certifies that the public input fee commitment $cv_f$ was derived from the witnessed values as:

$cv_f = [-v_f] G_{v_f} + [\widetilde{v_f}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator and $G_{v_f}$ is an asset-specific generator point derived in-circuit from the asset ID $ID_{v_f}$ as described in [Assets and Values](../../assets.md).

### [Balance Commitment Integrity](#balance-commitment-integrity)

The zk-SNARK certifies that the total public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [-v_1] G_1 + [-v_2] G_2 + cv_f$

where the first two terms are from the input amounts and assets, with the corresponding asset-specific generators $G_1, G_2$ derived in-circuit as described in [Balance Commitments](../../assets.md), and $cv_f$ is the fee commitment.

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ associated with the address on the note is not identity.