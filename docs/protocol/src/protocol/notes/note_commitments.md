# Note Commitments

We commit to:

* the value $v$ of the note.
* the asset ID $ID$ of the note,
* the diversified payment address $pk_d$,
* the diversified basepoint $B_d$,

The note commitment is generated using rate-5 Poseidon hashing with domain separator $ds$ defined as the `Fq` element constructed using:

`ds = from_le_bytes(BLAKE2b-512(b"penumbra.notecommit")) mod q`

The note commitment is then constructed using the above domain separator and
hashing together the above contents along with the note blinding factor $rcm$:

`note_commitment = hash_5(ds, (rcm, v, ID, B_d, pk_d))`

We commit to the diversified basepoint and payment address instead of the
diversifier itself, as in the circuit `OutputProof` when we verify the integrity of
the derived ephemeral key $epk$, we need $B_d$: 

$epk = [esk] B_d$.

We save a hash-to-group in circuit by committing to the diversified basepoint instead of recomputing $B_d$ from the diversifier. See related discussion
[here](https://github.com/zcash/zcash/issues/2277#issuecomment-367209705) from
ZCash.
