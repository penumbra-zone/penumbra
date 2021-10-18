use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use crate::{
    addresses::PaymentAddress, ka, memo::MemoCiphertext, note, proofs::transparent::OutputProof,
    value, Fq, Fr, Note, Value,
};

pub const OVK_WRAPPED_LEN_BYTES: usize = 80;
pub const NOTE_ENCRYPTION_BYTES: usize = 80;

pub struct Output {
    pub body: Body,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES],
}

pub struct Body {
    pub value_commitment: value::Commitment,
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; NOTE_ENCRYPTION_BYTES],
    pub proof: OutputProof,
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        v_blinding: Fr,
        dest: &PaymentAddress,
    ) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let value_commitment = value.commit(v_blinding);

        let note_blinding = Fq::rand(rng);

        let note = Note::new(dest, value, note_blinding);
        let note_commitment = note.commit();
        // TODO: Encrypt note here and add to a field in the Body struct (later).
        // TEMP
        let encrypted_note = [0u8; NOTE_ENCRYPTION_BYTES];

        let esk = ka::Secret::new(rng);
        let ephemeral_key = esk.diversified_public(note.dest.diversified_generator());

        let proof = OutputProof {};

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
            encrypted_note,
            proof,
        }
    }
}
