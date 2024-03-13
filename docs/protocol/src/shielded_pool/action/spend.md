# Spend Descriptions

Each spend contains an SpendBody and a zk-SNARK spend proof.

## Invariants

The invariants that the Spend upholds are described below.

#### Local Invariants

1. You must have spend authority over the note to spend

2. You can't spend a positioned note twice.

    2.1. Each positioned note has exactly one valid nullifier for it.

    2.2. No two spends on the ledger, even in the same transaction, can have the same nullifier.

3. You can't spend a note that has not been created, unless the amount of that note is 0.

#### Local Justification

1. We verify the auth_sig using the randomized verification key, which must not be 0, provided on the spend body, even if the amount of the note is 0. A note's transmission key binds the authority.

2. The following checks prevent spending a positioned note twice:

    2.1. A note's transmission key binds to the nullifier key in circuit as described in the [Diversified Address Integrity](#diversified-address-integrity) section, and all components of a positioned note, along with this key are hashed to derive the nullifier in circuit as described below in the [Nullifier Integrity](#nullifier-integrity) section.

    2.2. This is in the `ActionHandler` implementation on `check_stateful`, and an external check about the integrity of each transaction.

3. The circuit verifies for non-zero amounts that there is a valid [Merkle authentication path](#merkle-auth-path-verification) to the note in the global state commitment tree.

#### Global Justification

1.1. This action destroys the value of the note spent, and is reflected in the balance by adding the value to the transaction value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance. We do not create value, because of 3.

## Spend zk-SNARK Statements

The spend proof demonstrates the properties enumerated below for the following private witnesses known by the prover:

* Note amount $v$ (interpreted as an $\mathbb F_q$, constrained to fit in 128 bits) and asset `ID` $\isin \mathbb F_q$
* Note blinding factor $rcm \isin \mathbb F_q$ used to blind the note commitment
* Address associated with the note being spent, consisting of diversified basepoint $B_d \isin \mathbb G$,
transmission key $pk_d \isin \mathbb G$, and clue key $\mathsf{ck_d} \isin \mathbb F_q$
* Note commitment $cm \isin \mathbb F_q$
* Blinding factor $\tilde v \isin \mathbb F_r$ used to blind the balance commitment
* Spend authorization randomizer used for generating the randomized spend authorization key $\alpha \isin \mathbb F_r$
* Spend authorization key $ak \isin \mathbb G$
* Nullifier deriving key $nk \isin \mathbb F_q$
* Merkle proof of inclusion for the note commitment, consisting of a position `pos` constrained to fit in 48 bits, and an authentication path consisting of 72 $\mathbb F_q$ elements (3 siblings each per 24 levels)

And the corresponding public inputs:

* Merkle anchor $\isin \mathbb F_q$ of the state commitment tree
* Balance commitment $cv \isin G$ to the value balance
* Nullifier $nf \isin F_q$ of the note to be spent
* Randomized verification key $rk \isin \mathbb G$

### [Note Commitment Integrity](#note-commitment-integrity)

The zk-SNARK certifies that the note commitment $cm$ was derived as:

$cm = hash_6(ds, (rcm, v, ID, B_d, pk_d, ck_d))$.

using the above witnessed values and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [v] G_v + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator and $G_v$ is an asset-specific generator point derived in-circuit as described in [Value Commitments](../../protocol/value_commitments.md).

### [Nullifier Integrity](#nullifier-integrity)

The zk-SNARK certifies that the revealed nullifier $nf$ was derived as:

$nf = hash_3(ds, (nk, cm, pos))$

using the witnessed values above and where `ds` is a constant domain separator:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q`

as described in [Nullifiers](../notes/nullifiers.md).

### [Diversified address Integrity](#diversified-address-integrity)

The zk-SNARK certifies that the diversified address $pk_d$ associated with the note being spent was derived as:

$pk_d ​= [ivk] B_d$

where $B_d$ is the witnessed diversified basepoint and $ivk$ is the incoming viewing key computed using a rate-2 Poseidon hash from the witnessed $nk$ and $ak$ as:

`ivk = hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r`

as described in [Viewing Keys](../addresses_keys/viewing_keys.md).

### Randomized verification key Integrity

The zk-SNARK certifies that the randomized verification key $rk$ was derived using the witnessed $ak$ and spend auth randomizer $\alpha$ as:

$rk = ak + [\alpha]B_{SpendAuth}$

where $B_{SpendAuth}$ is the conventional `decaf377` basepoint as described in [The Decaf377 Group](../../crypto/decaf377.md).

### [Merkle auth path verification](#merkle-auth-path-verification)

The zk-SNARK certifies that for non-zero values $v \ne 0$, the witnessed Merkle authentication path is a valid Merkle path to the provided public anchor. Only for notes with non-zero values $v \ne 0$, the note is unrooted from the state commitment tree to allow for these "dummy" spends to pass stateless verification. Dummy spends may be added for metadata resistance (e.g. to ensure there are two spends and two outputs in each transaction).

### Diversified Base is not Identity

The zk-SNARK certifies that the diversified basepoint $B_d$ associated with the address on the note is not identity.

### The spend authorization key is not Identity

The zk-SNARK certifies that the spend authorization key $ak$ is not identity.
