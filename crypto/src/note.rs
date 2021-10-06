use ark_ff::PrimeField;
use once_cell::sync::Lazy;

use crate::addresses::PaymentAddress;
use crate::keys;
use crate::poseidon_hash::hash_5;
use crate::value::Value;
use crate::Fq;

// TODO: Should have a `leadByte` as in Sapling and Orchard note plaintexts?
// Do we need that in addition to the tx version?

/// A plaintext Penumbra note.
pub struct Note {
    // Diversifier. 11 bytes.
    pub diversifier: keys::Diversifier,

    // Value (32-byte asset ID plus 32-byte amount). 64 bytes.
    pub value: Value,

    // Commitment trapdoor. 32 bytes.
    pub note_blinding: Fq,
}

/// The domain separator used to generate note commitments.
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

impl Note {
    pub fn new(dest: &PaymentAddress, value: &Value, note_blinding: &Fq) -> Self {
        Note {
            diversifier: dest.diversifier,
            value: *value,
            note_blinding: *note_blinding,
        }
    }
}

// Note commitment `cm`.
pub struct NoteCommitment(pub Fq);

impl NoteCommitment {
    pub fn new(dest: &PaymentAddress, value: &Value, note_blinding: &Fq) -> Self {
        let commit = hash_5(
            &NOTECOMMIT_DOMAIN_SEP,
            (
                *note_blinding,
                value.amount.into(),
                value.asset_id.0,
                dest.g_d.compress_to_field(),
                dest.transmission_key.0.compress_to_field(),
            ),
        );

        NoteCommitment(commit)
    }
}
