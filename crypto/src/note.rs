use ark_ff::PrimeField;
use once_cell::sync::Lazy;

use crate::addresses::PaymentAddress;
use crate::keys;
use crate::poseidon_hash::hash_4;
use crate::value::Value;
use crate::Fq;

// TODO: Should have a `leadByte` as in Sapling and Orchard note plaintexts?
// Do we need that in addition to the tx version?

/// A plaintext Penumbra note.
pub struct Note {
    // Diversifier. 11 bytes.
    pub diversifier: keys::Diversifier,

    // 256 + 256
    pub value: Value,

    // Commitment trapdoor. 256 bits.
    pub note_blinding: Fq,
}

/// The domain separator used to generate note commitment.
static _NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

// Note commitment `cm`.
pub struct NoteCommitment(pub Fq);

impl NoteCommitment {
    // TODO: We are temporarily using the note_blinding randomness as the domain separator
    // until a rate 5 poseidon instance is added. Or see if we don't need the domain separator at all.
    pub fn new(dest: &PaymentAddress, v: &Value, note_blinding: &Fq) -> Self {
        let g_d = dest.diversifier.diversified_generator();

        let commit = hash_4(
            &note_blinding,
            (
                v.amount.into(),
                v.asset_id.0,
                g_d.compress_to_field(),
                dest.transmission_key.0.compress_to_field(),
            ),
        );

        NoteCommitment(commit)
    }
}
