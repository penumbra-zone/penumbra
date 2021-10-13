use ark_ff::PrimeField;
use once_cell::sync::Lazy;

use crate::{addresses::PaymentAddress, ka, keys, poseidon_hash::hash_5, Fq, Value};

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
    pub fn new(dest: &PaymentAddress, value: Value, note_blinding: Fq) -> Self {
        Note {
            diversifier: *dest.diversifier(),
            value: value,
            note_blinding: note_blinding,
        }
    }
}

// Note commitment.
#[derive(Clone, Copy)]
pub struct Commitment(pub Fq);

impl Commitment {
    pub fn new(
        dest: &PaymentAddress,
        value: &Value,
        note_blinding: &Fq,
    ) -> Result<Self, ka::Error> {
        let commit = hash_5(
            &NOTECOMMIT_DOMAIN_SEP,
            (
                *note_blinding,
                value.amount.into(),
                value.asset_id.0,
                dest.diversified_generator().compress_to_field(),
                *dest.tk_s(),
            ),
        );

        Ok(Self(commit))
    }
}
