# Output Descriptions

Each output contains an OutputBody and a zk-SNARK output proof.

## [Output Body](#output-body)

The body of an `Output` has four parts:

1. A `NotePayload`, which consists of the note commitment, the `NoteCiphertext`, and an ephemeral public key used to encrypt the note;
2. A balance commitment, which commits to the value balance of the output note;
3. The ovk wrapped key, which enables the _sender_ to later decrypt the `NoteCiphertext` using their `OutgoingViewingKey`;
4. The wrapped memo key, which enables one who can decrypt the `NoteCiphertext` to additionally decrypt the [`MemoCiphertext`](../../transactions/memo.md) on the transaction.

## Invariants

The invariants that the Output upholds are described below.

#### Local Invariants

1. The created output note is spendable by the recipient if its nullifier has not been revealed.

    1.1 The output note is bound to the recipient.

    1.2 The output note can be spent only by the recipient.

2. The output data in the transaction does not reveal the amount, asset type, sender identity, or recipient identity.

    2.1 The output data included in a transaction preserves this property.

    2.2 The [note commitment integrity check](#note-commitment-integrity) and the [balance commitment integrity check](#balance-commitment-integrity) are done in zero-knowledge.

3. The balance contribution of the value of the note is private.

#### Local Justification

1. The created note is spendable only by the recipient if its nullifier has not been revealed since:

    1.1 The note commitment binds the note to the typed value and the address of the recipient.

    1.2 Each note has a unique note commitment if the note blinding factor is unique for duplicate (recipient, typed value) pairs. Duplicate note commitments are allowed on chain since they commit to the same (recipient, typed value, randomness) tuple[^1].

2. The privacy of the note data is enforced via:

    2.1 The output note, which includes the amount, asset, and address of the recipient, is symmetrically encrypted to a key that only the recipient and sender can derive, as specified in [Transaction Cryptography](../../addresses_keys/transaction_crypto.md). The return (sender) address can *optionally* be included in the symmetrically encrypted memo field of the transaction, which can be decrypted by any output in that transaction as specified in [Transaction Cryptography](../../addresses_keys/transaction_crypto.md). The sender or recipient can authorize other parties to view the contents of the note by disclosure of these symmetric keys.

    2.2 The note commitment scheme used for property 1 preserves privacy via the hiding property of the note commitment scheme. The sender demonstrates knowledge of the opening of the [note commitment in zero-knowledge](#note-commitment-integrity).

3. The balance contribution of the value of the note is hidden via the hiding property of the balance commitment scheme. Knowledge of the opening of the [balance commitment is done in zero-knowledge](#balance-commitment-integrity).

#### Global Justification

1.1 This action contributes the value of the output note, which is summed as part of the transaction value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance.

## Note Decryption Checks

Clients using the ephemeral public key $epk$ provided in an output body to decrypt a note payload MUST check:

$epk = [esk] B_d$

This check exists to mitigate a potential attack wherein an attacker attempts to
link together a target's addresses. Given that the attacker knows the address of
the target, an attacker can send a note encrypted using the $B_d$ of a second
address they suspect the target controls. If the target _does_ control both
addresses, then the target will be able to decrypt this note. If the target responds to
the attacker out of band confirming the payment was received, the attacker has
learned the target does control both addresses. To avoid this, client software checks the
_correct_ diversified basepoint is used during note decryption. See
[ZIP 212](https://zips.z.cash/zip-0212) for further details.

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

### [Note Commitment Integrity](#note-commitment-integrity)

The zk-SNARK certifies that the public input note commitment $cm$ was derived as:

$cm = hash_6(ds, (rcm, v, ID, B_d, pk_d, ck_d))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### [Balance Commitment Integrity](#balance-commitment-integrity)

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [-v] G_v + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator and $G_v$ is an asset-specific
generator point derived in-circuit as described in [Assets and
Values](../../assets.md).

The asset-specific generator is derived in-circuit instead of witnessed to avoid a malicious prover witnessing the negation of a targeted asset-specific generator. See [Section 0.3 of the MASP specification for more details of this attack](https://github.com/anoma/masp/blob/main/docs/multi-asset-shielded-pool.pdf).

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ is not identity.

Note that we do _not_ check the integrity of the ephemeral public key $epk$ in the zk-SNARK.
Instead this check should be performed at note decryption time as described above.

[^1]: Duplicate note commitments are allowed such that nodes do not need to
maintain a database of all historical commitments and check for distinctness.
They simply maintain the state commitment tree, adding new state commitments
as they appear. Two notes with the same typed value and address will not have
the same commitment due to the randomness provided by the note blinding factor.
